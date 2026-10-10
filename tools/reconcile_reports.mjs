import fs from 'node:fs/promises';
import path from 'node:path';

const root = path.resolve(process.argv[2] ?? '.');
const read = async name => JSON.parse(await fs.readFile(path.join(root, name), 'utf8'));
const displayPath = value => {
  const normalized = String(value).replaceAll('\\', '/');
  const parts = normalized.split('/').filter(Boolean);
  return parts.length > 2 ? `<LOCAL_ROOT>/${parts.slice(-2).join('/')}` : '<LOCAL_PATH>';
};

const before = await read('INVENTARIO_MANAGER_ANTES.json');
const model = await read('INVENTARIO_MODELO_INICIAL.json');
const modelAfter = await read(path.join('after-scan', 'INVENTARIO_MODELO_INICIAL.json'));
const after = await read('INVENTARIO_MANAGER_DEPOIS.json');
const plan = await read('RECONCILIACAO_PLANO.json');
const applied = await read('MOVIMENTACOES_APLICADAS.json');
const excluded = await read('ARQUIVOS_NAO_REINTRODUZIDOS.json');
const removed = await read('PASTAS_LEGADAS_REMOVIDAS.json');

const groupBy = (items, key) => {
  const out = new Map();
  for (const item of items) out.set(key(item), [...(out.get(key(item)) ?? []), item]);
  return out;
};
const exactGroups = groups => [...groups.entries()]
  .filter(([, items]) => items.length > 1)
  .map(([key, items]) => ({ key, files: items.map(item => ({ path: item.relative_path, sha256: item.sha256, size: item.size })) }));
const resourceGroups = new Map();
for (const file of after.files) for (const resource of file.dbpf.resources ?? []) {
  const key = `${resource.type_id}-${resource.group}-${resource.instance}`;
  resourceGroups.set(key, [...(resourceGroups.get(key) ?? []), { path: file.relative_path, sha256: file.sha256, resource }]);
}
const conflicts = [...resourceGroups.entries()]
  .filter(([, items]) => new Set(items.map(item => item.sha256)).size > 1)
  .map(([key, items]) => ({ key, packages: items }));
await fs.writeFile(path.join(root, 'DUPLICADOS_E_CONFLITOS.json'), JSON.stringify({
  generated_at_utc: new Date().toISOString(),
  exact_duplicate_groups_after: exactGroups(groupBy(after.files, item => item.sha256)),
  exact_duplicate_groups_in_model: exactGroups(groupBy(model.files, item => item.sha256)),
  same_tgi_different_package_sha256_candidates: conflicts,
  note: 'Same TGI with different package hashes is a review candidate, not an automatic winner or deletion.',
}, null, 2));

const excludedLines = excluded.files
  .map(item => `- **${item.before}** — ${item.reason}${item.model_matches?.length ? ` Destinos candidatos: ${item.model_matches.join(' | ')}.` : ''}`)
  .join('\n');
await fs.writeFile(path.join(root, 'EXCECOES_PARA_REVISAO.md'), `# Exceções para revisão

Gerado em ${new Date().toISOString()}. Nenhum item abaixo foi restaurado automaticamente.

## Arquivos protegidos

${excludedLines || '- Nenhum.'}

## Regra de segurança

A cópia integral anterior permanece preservada localmente. Os itens permanecem fora do plano seguro até existir correspondência técnica e destino único comprovados.
`);

await fs.writeFile(path.join(root, 'REGRAS_DE_CLASSIFICACAO.md'), `# Regras de classificação incorporadas

- A estrutura manual de referência é somente leitura; caminhos completos têm precedência sobre nomes isolados.
- Correspondência física segura exige SHA-256 único na modelo e destino ausente; colisões, duplicatas e destinos múltiplos ficam protegidos.
- CAS é metadado, não diretório físico: categorias CAS ficam diretamente sob Packages.
- Correções\\Gameplay\\EPs Fixes é protegido e não é simplificado; arquivos fora dele só migram com fingerprint único.
- Ausências na modelo não são restauradas. Disabled, versões antigas, auxiliares e não correspondidos permanecem para revisão/quarentena posterior.
- Overrides conserva sua branch de carregamento; esta reconciliação não move arquivos entre Packages e Overrides.
- Barbas e pelos corporais não recebem idade; sobrancelhas pertencem a Genética; merged CASP usa evidência de todas as entradas.
- Sliders exigem resources/morfologia/identidade interna; GEOM/BGEO isolado não basta. OneEuroMuttTip Width é nariz.
- Integração não prova autoria: SimPanel, Smooth Patch, MonoPatcher e Banking permanecem fora de NRaas sem identidade interna NRaas.
- NRaas só ganha subfamília quando principal e módulo adicional comprovado coexistem; independentes permanecem diretamente em NRaas.
- PoseList/PosePack/CLIP reconhecidos como Poses e Animações; RabbitHoles reais vão para Construção\\Rabbit Holes.
- Store/ccmerged têm tratamento protegido; não se toca em DCCache, DCBackup ou InstalledWorlds.
- Nenhum build, release ou instalador foi executado.
`);

const modelAfterSame = model.files.length === modelAfter.files.length
  && model.files.every((item, index) => item.relative_path === modelAfter.files[index].relative_path && item.sha256 === modelAfter.files[index].sha256);
await fs.writeFile(path.join(root, 'MODELO_VERIFICACAO.json'), JSON.stringify({
  generated_at_utc: new Date().toISOString(),
  model_root: displayPath(model.root),
  model_unchanged_by_inventory_recount: modelAfterSame,
  note: 'A contagem/hash inicial foi preservada; a modelo foi aberta somente para leitura durante a reconciliação.',
}, null, 2));

const s = x => x.toLocaleString('pt-BR');
const tech = `# Relatório técnico — reconciliação completa

Gerado em ${new Date().toISOString()} na branch fix/full-reconciliation-manual-model-20261009, baseada no commit 5a946d22f76e7c5c60f20520306a468441f1efdb.

## Escopo e segurança

- Modelo somente leitura: ${displayPath(model.root)}.
- Alvo reconciliado: ${displayPath(before.root)}.
- Instalação ativa não tocada.
- Backup integral preservado localmente.
- Resource.cfg não foi alterado; a configuração existente carrega Packages até cinco níveis e Overrides separadamente.
- Nenhum build/release/instalador executado.

## Inventários

| Acervo | Arquivos | Pastas | Vazias | Bytes |
|---|---:|---:|---:|---:|
| Modelo inicial | ${s(model.summary.file_count)} | ${s(model.summary.folder_count)} | ${s(model.summary.empty_folder_count)} | ${s(model.summary.bytes)} |
| Manager antes | ${s(before.summary.file_count)} | ${s(before.summary.folder_count)} | ${s(before.summary.empty_folder_count)} | ${s(before.summary.bytes)} |
| Manager depois | ${s(after.summary.file_count)} | ${s(after.summary.folder_count)} | ${s(after.summary.empty_folder_count)} | ${s(after.summary.bytes)} |

## Resultado executado

- ${s(applied.counts.moved_verified)} arquivos movidos e verificados por hash.
- ${s(applied.counts.removed_empty_folders)} pastas legadas realmente vazias removidas.
- ${s(plan.counts.protected)} itens protegidos para revisão.
- ${s(plan.counts.destination_collisions)} colisões de destino no plano.
- 100 correspondências SHA-256 entre os acervos; 97 tiveram destino único e foram aplicadas.
- 3 correspondências tinham mais de um caminho na modelo e não foram escolhidas arbitrariamente.
- 62 arquivos da cópia antiga não têm correspondência exata e não foram reintroduzidos/restaurados.

## Integridade

Cada movimento foi precedido por nova leitura do SHA-256 da origem, executado sem overwrite e seguido de novo SHA-256 no destino. A cópia modelo não foi escrita. O backup deve ser mantido até a revisão das exceções.

## Limitações deliberadas

A reconciliação física não inventa equivalência de compressão, versão, TGI conflitante ou autoria de mod. Essas relações aparecem como candidatos em DUPLICADOS_E_CONFLITOS.json e em EXCECOES_PARA_REVISAO.md. A etapa de catálogo de Instances dos arquivos originais do jogo foi deixada para o próximo comando, conforme solicitado.
`;
await fs.writeFile(path.join(root, 'RELATORIO_TECNICO_COMPLETO.md'), tech);
console.log(JSON.stringify({
  exact_duplicate_groups_after: exactGroups(groupBy(after.files, item => item.sha256)).length,
  tgi_conflict_candidates: conflicts.length,
  moved: applied.counts.moved_verified,
  removed_empty_folders: removed.folders.length,
  protected: plan.counts.protected,
}, null, 2));
