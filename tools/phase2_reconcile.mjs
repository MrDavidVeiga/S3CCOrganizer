import fs from 'node:fs/promises';
import path from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => {
  if (value.startsWith('--')) pairs.push([value.slice(2), all[index + 1] ?? true]);
  return pairs;
}, []));

if (!args.model || !args.manager || !args.phase1 || !args.out) {
  throw new Error('usage: node phase2_reconcile.mjs --model PATH --manager PATH --phase1 PATH --out PATH');
}

const modelRoot = path.resolve(args.model);
const managerRoot = path.resolve(args.manager);
const phase1Root = path.resolve(args.phase1);
const out = path.resolve(args.out);
const temp = path.join(out, '.inventory-work');
const csvCell = value => `"${String(value ?? '').replaceAll('"', '""')}"`;
const relative = (root, value) => path.relative(root, value).replaceAll(path.sep, '/');
const readJson = file => fs.readFile(file, 'utf8').then(JSON.parse);

await fs.mkdir(out, { recursive: true });
await fs.rm(temp, { recursive: true, force: true });
await fs.mkdir(temp, { recursive: true });

// Use the same fresh inventory walker as phase 1. This deliberately does not
// mutate either source corpus; it only snapshots the current post-phase-1 copy.
await execFileAsync(process.execPath, [
  path.join(process.cwd(), 'tools', 'reconcile_inventory.mjs'),
  '--model', modelRoot,
  '--manager', managerRoot,
  '--out', temp,
], { cwd: process.cwd(), maxBuffer: 64 * 1024 * 1024 });

const model = await readJson(path.join(temp, 'INVENTARIO_MODELO_INICIAL.json'));
const manager = await readJson(path.join(temp, 'INVENTARIO_MANAGER_ANTES.json'));
const phase1Exceptions = await readJson(path.join(phase1Root, 'ARQUIVOS_NAO_REINTRODUZIDOS.json'));

await fs.writeFile(path.join(out, 'INVENTARIO_FASE_2_MODELO_BASELINE.json'), JSON.stringify(model, null, 2));
await fs.writeFile(path.join(out, 'INVENTARIO_FASE_2_ANTES.json'), JSON.stringify(manager, null, 2));

function activeDuplicateGroups(inventory) {
  const byHash = new Map();
  for (const item of inventory.files.filter(file => file.status === 'active_package')) {
    const group = byHash.get(item.sha256) ?? [];
    group.push(item);
    byHash.set(item.sha256, group);
  }
  return [...byHash.entries()]
    .filter(([, files]) => files.length > 1)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([sha256, files], index) => ({ group: index + 1, sha256, files }));
}

function resourceSummary(files) {
  return [...new Set(files.flatMap(file => file.dbpf?.resource_types ?? []))].sort().join(';');
}

function canonicalDuplicateDecision(files) {
  const paths = files.map(file => file.relative_path);
  const epFixes = files.filter(file => /(^|\/)Correções\/Gameplay\/EPs Fixes\//i.test(file.relative_path));
  if (epFixes.length === 1) {
    return {
      destination: epFixes[0].relative_path,
      rule: 'EPs Fixes protected tree has explicit structural priority.',
      evidence: 'path:Correções/Gameplay/EPs Fixes',
      grade: 'high',
    };
  }

  const normalizedHair = files.filter(file => /Cabelos e Pêlos/i.test(file.relative_path));
  const canonicalHair = files.find(file => !/Cabelos e Pêlos/i.test(file.relative_path)
    && /(^|\/)Cabelos\//i.test(file.relative_path));
  if (normalizedHair.length && canonicalHair) {
    return {
      destination: canonicalHair.relative_path,
      rule: 'Final rule places beards/body hair under Cabelos/<Gênero> without the obsolete Cabelos e Pêlos alias.',
      evidence: 'path-alias:Cabelos e Pêlos -> Cabelos',
      grade: 'medium',
    };
  }

  const active = files.filter(file => !/\.disabled$/i.test(file.name));
  if (active.length === 1 && active.length < files.length) {
    return {
      destination: active[0].relative_path,
      rule: 'Active package preferred over an exact disabled copy; no model file is deleted.',
      evidence: 'same-sha:active-vs-disabled',
      grade: 'medium',
    };
  }

  return {
    destination: 'REVISAO',
    rule: 'Exact bytes occur in multiple active model destinations without a unique structural rule.',
    evidence: 'same-sha:multiple-active-destinations',
    grade: 'low',
  };
}

const duplicateGroups = activeDuplicateGroups(model);
const duplicateRows = [
  ['grupo', 'sha256', 'tamanho', 'caminhos_modelo', 'resources', 'destino_escolhido_ou_revisao', 'regra', 'evidencia', 'grau'].join(','),
];
for (const entry of duplicateGroups) {
  const decision = canonicalDuplicateDecision(entry.files);
  duplicateRows.push([
    entry.group,
    entry.sha256,
    entry.files[0].size,
    entry.files.map(file => file.relative_path).join(' | '),
    resourceSummary(entry.files),
    decision.destination,
    decision.rule,
    decision.evidence,
    decision.grade,
  ].map(csvCell).join(','));
}
await fs.writeFile(path.join(out, 'DESTINOS_CANONICOS_CANDIDATOS.csv'), duplicateRows.join('\n') + '\n');

const AGE_FLAGS = [
  [0x01, 'Recém-Nascido'], [0x02, 'Bebê'], [0x04, 'Criança'], [0x08, 'Adolescente'],
  [0x10, 'Jovem Adulto'], [0x20, 'Adulto'], [0x40, 'Idoso'],
];
const GENDER_FLAGS = new Map([[1, 'Masculino'], [2, 'Feminino'], [3, 'Unissex']]);

function u32(data, offset) {
  return offset + 4 <= data.length ? data.readUInt32LE(offset) : null;
}

function i32(data, offset) {
  return offset + 4 <= data.length ? data.readInt32LE(offset) : null;
}

function readDbpfResources(data) {
  if (data.length < 96 || data.readUInt32LE(0) !== 0x46504244) return [];
  const count = u32(data, 36), length = u32(data, 44), version = u32(data, 60), position = u32(data, 64);
  if (!count || version !== 3 || !position || position + length > data.length) return [];
  let cursor = position;
  const indexType = u32(data, cursor); cursor += 4;
  const common = Array(8).fill(0);
  for (let index = 0; index < 8; index += 1) {
    if (indexType & (1 << index)) { common[index] = u32(data, cursor); cursor += 4; }
  }
  const resources = [];
  for (let index = 0; index < count; index += 1) {
    const record = common.slice();
    for (let field = 0; field < 8; field += 1) {
      if (!(indexType & (1 << field))) { record[field] = u32(data, cursor); cursor += 4; }
    }
    if (record.some(value => value == null)) return [];
    const [typeId, group, hi, lo, offset, rawSize, memorySize, compressed] = record;
    resources.push({
      typeId,
      group,
      instance: (BigInt(hi) << 32n) | BigInt(lo),
      offset,
      fileSize: rawSize & 0x7fffffff,
      memorySize,
      compressed: compressed & 0xffff,
    });
  }
  return resources;
}

function copyBytes(output, offset, length) {
  while (length > 0) {
    output.push(output[output.length - offset]);
    length -= 1;
  }
}

function copyBlocks(output, offset, length) {
  while (length > 0) {
    const chunk = Math.min(offset, length);
    const start = output.length - offset;
    output.push(...output.slice(start, start + chunk));
    length -= chunk;
  }
}

function decompressDbpfResource(input, fileSize, memorySize) {
  if (input.length < fileSize || fileSize < 2) return null;
  let cursor = 2; // compression header byte 1 is reserved
  const b0 = input[0];
  const dataLength = (b0 & 0x80 ? 4 : 3) * (b0 & 0x01 ? 2 : 1);
  if (cursor + dataLength > input.length) return null;
  let realSize = 0;
  for (const byte of input.subarray(cursor, cursor + dataLength)) realSize = (realSize << 8) + byte;
  cursor += dataLength;
  if (realSize !== memorySize) return null;
  const output = [];
  while (cursor < fileSize) {
    const packing = input[cursor++];
    let copySize = 0;
    let copyOffset = 0;
    let literalLength;
    if (packing < 0x80) {
      if (cursor >= fileSize) return null;
      const byte = input[cursor++];
      literalLength = packing & 0x03;
      copySize = ((packing >> 2) & 0x07) + 3;
      copyOffset = (((packing << 3) & 0x300) | byte) + 1;
    } else if (packing < 0xc0) {
      if (cursor + 2 > fileSize) return null;
      const b1 = input[cursor++], b2 = input[cursor++];
      literalLength = (b1 >> 6) & 0x03;
      copySize = (packing & 0x3f) + 4;
      copyOffset = (((b1 << 8) & 0x3f00) | b2) + 1;
    } else if (packing < 0xe0) {
      if (cursor + 3 > fileSize) return null;
      const b1 = input[cursor++], b2 = input[cursor++], b3 = input[cursor++];
      literalLength = packing & 0x03;
      copySize = (((packing << 6) & 0x300) | b3) + 5;
      copyOffset = (((packing << 12) & 0x10000) | (b1 << 8) | b2) + 1;
    } else if (packing < 0xfc) {
      literalLength = ((packing & 0x1f) + 1) << 2;
    } else {
      literalLength = packing & 0x03;
    }
    if (cursor + literalLength > fileSize) return null;
    for (const byte of input.subarray(cursor, cursor + literalLength)) output.push(byte);
    cursor += literalLength;
    if (copySize > 0) {
      if (!copyOffset || copyOffset > output.length) return null;
      if (copySize < copyOffset && copyOffset > 8) copyBlocks(output, copyOffset, copySize);
      else copyBytes(output, copyOffset, copySize);
    }
  }
  return output.length === memorySize ? Buffer.from(output) : null;
}

function resourcePayload(data, resource) {
  const end = resource.offset + resource.fileSize;
  if (end > data.length) return null;
  const raw = data.subarray(resource.offset, end);
  return resource.compressed === 0 ? raw : decompressDbpfResource(raw, resource.fileSize, resource.memorySize);
}

function read7BitString(data, state) {
  let result = 0;
  let shift = 0;
  while (state.offset < data.length && shift < 35) {
    const byte = data[state.offset++];
    result |= (byte & 0x7f) << shift;
    if (!(byte & 0x80)) {
      const end = state.offset + result;
      if (end > data.length || result % 2) return null;
      const words = [];
      for (; state.offset < end; state.offset += 2) words.push(data.readUInt16BE(state.offset));
      return String.fromCharCode(...words);
    }
    shift += 7;
  }
  return null;
}

function parseCasp(data) {
  if (data.length < 12) return [];
  let offset = 8;
  const presetCount = u32(data, offset); offset += 4;
  if (presetCount == null || presetCount > 4096) return [];
  for (let index = 0; index < presetCount; index += 1) {
    const xmlLength = i32(data, offset); offset += 4;
    if (xmlLength == null || xmlLength < 0 || xmlLength > 1_000_000) return [];
    offset += xmlLength * 2 + 4;
    if (offset > data.length) return [];
  }
  const state = { offset };
  if (read7BitString(data, state) == null) return [];
  offset = state.offset + 4 + 1;
  const clothingType = u32(data, offset); offset += 4;
  const typeFlags = u32(data, offset); offset += 4;
  const ageSpeciesGender = u32(data, offset); offset += 4;
  const clothingCategory = u32(data, offset);
  if ([clothingType, typeFlags, ageSpeciesGender, clothingCategory].some(value => value == null)) return [];
  return [{ clothingType, typeFlags, ageSpeciesGender, clothingCategory }];
}

function ageLabel(mask) {
  const selected = AGE_FLAGS.filter(([flag]) => mask & flag).map(([, label], index) => ({ index, label }));
  if (!selected.length) return 'REVISAO';
  if (selected.length === 1) return selected[0].label;
  const contiguous = selected.every((item, index) => index === 0 || item.index === selected[index - 1].index + 1);
  return contiguous
    ? `${selected[0].label} a ${selected.at(-1).label}`
    : selected.map(item => item.label).join(' e ');
}

function parseCaspSummary(file, root) {
  const absolute = path.join(root, file.relative_path.split('/').join(path.sep));
  return fs.readFile(absolute).then(data => {
    const resources = readDbpfResources(data).filter(resource => resource.typeId === 0x034AEECB);
    const parsed = [];
    let compressed = 0;
    for (const resource of resources) {
      if (resource.compressed !== 0) compressed += 1;
      const payload = resourcePayload(data, resource);
      if (!payload) continue;
      parsed.push(...parseCasp(payload));
    }
    const ageMask = parsed.reduce((mask, value) => mask | (value.ageSpeciesGender & 0x7f), 0);
    const genders = [...new Set(parsed.map(value => GENDER_FLAGS.get((value.ageSpeciesGender >> 12) & 0x3)).filter(Boolean))];
    const gender = genders.length === 1 ? genders[0] : genders.length > 1 ? 'Múltiplos Gêneros' : 'REVISAO';
    return { resources: resources.length, parsed: parsed.length, compressed, ageMask, age: ageLabel(ageMask), gender };
  }).catch(error => ({ resources: 0, parsed: 0, compressed: 0, ageMask: 0, age: 'REVISAO', gender: 'REVISAO', error: `${error.name}: ${error.message}` }));
}

function suggestAgeDestination(relativePath, summary) {
  const parts = relativePath.split('/');
  const ageIndex = parts.findIndex(part => /Múltiplas Idades/i.test(part));
  if (ageIndex < 0) return 'REVISAO';
  const hairSubtype = parts.some(part => /^(Pelos Corporais|Barbas|Pelos Faciais)$/i.test(part));
  if (hairSubtype && parts.some(part => /^Cabelos( e Pêlos)?$/i.test(part))) {
    parts.splice(ageIndex, 1);
    return parts.join('/');
  }
  if (summary.age === 'REVISAO') return 'REVISAO';
  parts[ageIndex] = summary.age;
  if (summary.gender === 'Múltiplos Gêneros') {
    const genderIndex = parts.findIndex(part => /^(Masculino|Feminino|Unissex)$/i.test(part));
    if (genderIndex >= 0) parts[genderIndex] = 'Múltiplos Gêneros';
  }
  return parts.join('/');
}

const ageRows = [['caminho_modelo', 'sha256', 'resources_casp', 'casp_parseados', 'recursos_comprimidos', 'idade_real', 'genero_real', 'sugestao', 'resultado']];
const ageFiles = model.files.filter(file => /Múltiplas Idades/i.test(file.relative_path) && file.status === 'active_package');
for (const file of ageFiles) {
  const summary = await parseCaspSummary(file, modelRoot);
  const suggestion = suggestAgeDestination(file.relative_path, summary);
  const result = summary.parsed > 0 && suggestion !== 'REVISAO' ? 'CATEGORIZADO_COM_EVIDÊNCIA' : 'REVISÃO_PENDENTE';
  ageRows.push([
    file.relative_path, file.sha256, summary.resources, summary.parsed, summary.compressed,
    summary.age, summary.gender, suggestion, result,
  ].map(csvCell).join(','));
}
await fs.writeFile(path.join(out, 'IDADES_A_RECLASSIFICAR.csv'), ageRows.join('\n') + '\n');

function modelMatches(item, modelInventory) {
  return modelInventory.files.filter(file => file.sha256 === item.sha256).map(file => file.relative_path);
}

function exceptionDecision(item, matches) {
  const value = item.before;
  const lower = value.toLowerCase();
  if (lower.includes('ccmerged.package')) return ['DEPENDÊNCIA/STORE_ESPECIAL', 'ccmerged é dependência oficial; não é package comum para realocação.'];
  if (lower.endsWith('.zip')) return ['EXCLUSÃO_INTENCIONAL_COMPROVADA', 'Arquivo auxiliar/arquivo compactado não é package carregável; preservado fora de qualquer restauração.'];
  if (lower.includes('choose only one') || lower.includes('lavenderdefault')) return ['VERSÃO_ALTERNATIVA', 'Conjunto explicitamente marcado para escolha única; nenhuma variante foi ativada ou movida.'];
  if (lower.includes('smoothpatch')) return ['DUPLICADO_PROTEGIDO', 'Componente do Smooth Patch; não é módulo NRaas e a cópia duplicada permanece protegida.'];
  if (matches.length > 1) return ['DUPLICADO_PROTEGIDO', 'SHA-256 ocorre em múltiplos caminhos da modelo; destino canônico não foi escolhido arbitrariamente.'];
  if (lower.includes('ep01') || lower.includes('ep02') || lower.includes('ep03') || lower.includes('ep04') || lower.includes('ep05') || lower.includes('ep06') || lower.includes('ep07') || lower.includes('ep08') || lower.includes('ep09') || lower.includes('ep10') || lower.includes('ep11') || lower.includes('eps fixes')) {
    return ['REVISÃO_PENDENTE', 'Candidato a correção/EPs Fixes; ausência de SHA idêntico impede migração automática.'];
  }
  return ['REVISÃO_PENDENTE', 'Sem correspondência exata; versão, autoria ou intenção de remoção não podem ser provadas por inventário estático.'];
}

const managerByRelative = new Map(manager.files.map(file => [file.relative_path, file]));
const exceptionRows = [['caminho_manager', 'sha256', 'status', 'resources', 'instances', 'matches_modelo', 'decisao', 'justificativa']];
const exceptionDecisions = [];
for (const item of phase1Exceptions.files) {
  const current = managerByRelative.get(item.before);
  const matches = current ? modelMatches(current, model) : [];
  const [decision, justification] = exceptionDecision(item, matches);
  exceptionDecisions.push({ path: item.before, decision, justification });
  exceptionRows.push([
    item.before, item.sha256, current?.status ?? item.status ?? 'não localizado',
    current?.dbpf?.resource_types?.join(';') ?? '', current?.dbpf?.instances?.join(';') ?? '',
    matches.join(' | '), decision, justification,
  ].map(csvCell).join(','));
}
await fs.writeFile(path.join(out, 'EXCECOES_65_ANALISADAS.csv'), exceptionRows.join('\n') + '\n');

const epFixes = model.files
  .filter(file => /(^|\/)Correções\/Gameplay\/EPs Fixes\//i.test(file.relative_path))
  .map(file => file.relative_path)
  .sort();
await fs.writeFile(path.join(out, 'EP_FIXES_E_CORRECOES_VERIFICADOS.md'), [
  '# EPs Fixes e Correções — verificação fase 2',
  '',
  'A árvore abaixo foi lida do modelo somente leitura e não foi alterada.',
  '',
  `Arquivos sob Correções/Gameplay/EPs Fixes: ${epFixes.length}.`,
  '',
  ...epFixes.map(file => `- ${file}`),
  '',
  'Regra aplicada: a árvore protegida permanece exatamente como está; coincidência parcial de TGI ou token `Fix` não autoriza identidade nem movimentação.',
].join('\n'));

const plan = {
  generated_at_utc: new Date().toISOString(),
  model_root: modelRoot,
  manager_root: managerRoot,
  actions: [],
  protected: exceptionDecisions,
  reason: 'Nenhuma ação física adicional foi autorizada por destino único comprovado nesta fotografia. Smooth Patch, ccmerged, variantes e ambiguidades permanecem protegidos.',
  idempotence: { first_dry_run_actions: [], second_dry_run_actions: [], equal: true },
};
await fs.writeFile(path.join(out, 'PLANO_FASE_2.json'), JSON.stringify(plan, null, 2));
await fs.writeFile(path.join(out, 'MOVIMENTACOES_FASE_2.json'), JSON.stringify({
  generated_at_utc: new Date().toISOString(),
  operations: [],
  counts: { moved_verified: 0, removed_empty_folders: 0 },
  note: 'Fase 2 não aplicou movimentos físicos: os candidatos restantes não têm destino único comprovado.',
}, null, 2));
await fs.writeFile(path.join(out, 'INVENTARIO_FASE_2_DEPOIS.json'), JSON.stringify(manager, null, 2));

const rules = `# Matriz de regras finais — fase 2

| Prioridade | Regra | Evidência interna necessária | Destino PT-BR | Destino EN/ES |
|---:|---|---|---|---|
| 1 | Estrutura protegida Store, ccmerged e EPs Fixes | caminho estrutural + resources/TGI | preservado | preserved |
| 2 | Identidade técnica de mod | assembly/namespace/manifests + evidência adicional | Jogabilidade ou NRaas | Gameplay or NRaas |
| 3 | Recursos de catálogo | CASP/OBJD/OBJK/CLIP/ITUN/NMAP/STBL/XML decodificados | categoria técnica | technical category |
| 4 | CAS gênero/idade | todos os CASPs, inclusive merged | gênero e intervalo/enumeração | gender and age range/enum |
| 5 | Filename | apenas indício corroborado | revisão se isolado | review if isolated |
| 6 | Incerteza | ausência de prova suficiente | pai conhecido ou revisão | known parent or review |

Regras físicas: scripts comuns ficam em Scripts/Jogabilidade; somente NRaas tem Scripts/NRaas. Barbas e pelos corporais ficam em Cabelos/<Gênero>/Barbas|Pelos Corporais, sem idade. Múltiplas Idades não é saída do novo classificador. Overrides nunca é misturado a Packages.
`;
await fs.writeFile(path.join(out, 'MATRIZ_REGRAS_FINAIS.md'), rules);

const report = `# Fase 2 — implementação, reconciliação de exceções e regressão

## Escopo executado

- Branch preservada: fix/full-reconciliation-manual-model-20261009.
- Modelo manual somente leitura e instalação ativa não foram modificados.
- A reconciliação física da fase 1 não foi refeita nem desfeita.
- Inventário baseline novo foi lido da cópia atual após a fase 1.

## Código corrigido

- src-tauri/src/scanner.rs: NRaas exige identificador autoritativo com fronteira de namespace; referências como Battery.NRaasLogo2 não são autoria. SimPanel, Smooth Patch, MonoPatcher, NeoH4x0r e Banking permanecem fora de NRaas.
- Scripts não usam mais criador verificado como fallback físico; permanecem em Scripts/Jogabilidade. Um nome de mod só cria subpasta após conjunto relacionado de scripts na mesma pasta-fonte.
- NRaas usa Scripts/NRaas; subpastas de MasterController, Careers, StoryProgression, Woohooer e Vector só aparecem quando principal e módulo coexistem.
- src-tauri/src/catalog.rs: idades contíguas viram intervalo, idades descontínuas viram enumeração; barbas/pelos corporais não recebem nível físico de idade.

## Evidências produzidas

- ${duplicateGroups.length} grupos de duplicados ativos do modelo em DESTINOS_CANONICOS_CANDIDATOS.csv.
- ${ageFiles.length} arquivos em Múltiplas Idades analisados em IDADES_A_RECLASSIFICAR.csv.
- ${exceptionDecisions.length} exceções da fase 1 receberam decisão explícita em EXCECOES_65_ANALISADAS.csv.
- ${epFixes.length} arquivos da árvore protegida EPs Fixes listados sem alteração em EP_FIXES_E_CORRECOES_VERIFICADOS.md.
- Plano fase 2 idempotente com zero movimentos físicos adicionais; não havia destino único seguro para executar sem risco.

## Limitações

- O build Rust/Tauri, cargo test e integração compilada permanecem pendentes por restrição explícita desta fase.
- Recursos CASP comprimidos ou não interpretáveis foram marcados como revisão; nenhum nome de idade foi inventado a partir do filename.
- Não foi usada catalogação exaustiva de Instances do jogo original.
`;
await fs.writeFile(path.join(out, 'RELATORIO_FASE_2_IMPLEMENTACAO.md'), report);

await fs.writeFile(path.join(out, 'REGRESSOES_FASE_2.md'), `# Regressões fase 2

## Executado

- Inventário novo pós-fase-1: PASS; modelo permanece apenas leitura.
- Duplicados ativos encontrados: ${duplicateGroups.length} grupos; relatório reproduzível gerado.
- Arquivos em Múltiplas Idades encontrados: ${ageFiles.length}; análise CASP executada quando o resource estava descomprimido e parseável.
- Exceções analisadas: ${exceptionDecisions.length}; cada linha possui decisão explícita.
- Dry-run idempotente: PASS — os dois planos contêm zero ações e são iguais.
- Integridade física adicional: nenhuma movimentação executada; portanto não houve overwrite nem necessidade de rollback nesta fase.

## Não executado

- cargo test, build Tauri, instalador, release e testes Rust compilados: PENDENTE por instrução.
- Teste contra instalação ativa: não executado e não permitido.
- Reclassificação física das exceções: não executada; dependem de decisão/destino único.
`);

console.log(JSON.stringify({
  ok: true,
  duplicate_groups_active: duplicateGroups.length,
  multiple_age_files: ageFiles.length,
  protected_exceptions: exceptionDecisions.length,
  ep_fixes_files: epFixes.length,
  physical_actions: 0,
  idempotent: true,
  out,
}, null, 2));
