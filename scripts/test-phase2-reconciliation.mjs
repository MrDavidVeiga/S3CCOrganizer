import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve('reports/reconciliation-20261010-phase2');
const scanner = fs.readFileSync(path.resolve('src-tauri/src/scanner.rs'), 'utf8');
const catalog = fs.readFileSync(path.resolve('src-tauri/src/catalog.rs'), 'utf8');
assert.match(scanner, /authoritative_nraas_identifier/);
assert.match(scanner, /Battery\.NRaasLogo2/);
assert.match(scanner, /Related scripted packages in named source folder/);
assert.match(scanner, /Scripts.*Jogabilidade.*NRaas/);
assert.doesNotMatch(scanner, /\&\["nraas"\]/);
assert.match(catalog, /Criança a Idoso/);
assert.match(catalog, /Facial Hair.*Body Hair/s);
if (!fs.existsSync(root)) {
  console.log(JSON.stringify({ ok: true, skipped: 'private phase 2 corpus is not part of the public checkout' }, null, 2));
  process.exit(0);
}
const read = name => fs.readFileSync(path.join(root, name), 'utf8');
const json = name => JSON.parse(read(name));
const dataLines = name => read(name).trim().split(/\r?\n/).slice(1);

for (const name of [
  'RELATORIO_FASE_2_IMPLEMENTACAO.md', 'MATRIZ_REGRAS_FINAIS.md',
  'DESTINOS_CANONICOS_CANDIDATOS.csv', 'IDADES_A_RECLASSIFICAR.csv',
  'EXCECOES_65_ANALISADAS.csv', 'EP_FIXES_E_CORRECOES_VERIFICADOS.md',
  'PLANO_FASE_2.json', 'MOVIMENTACOES_FASE_2.json',
  'INVENTARIO_FASE_2_ANTES.json', 'INVENTARIO_FASE_2_DEPOIS.json',
  'REGRESSOES_FASE_2.md',
]) assert.ok(fs.existsSync(path.join(root, name)), `missing phase 2 artifact: ${name}`);

assert.equal(dataLines('DESTINOS_CANONICOS_CANDIDATOS.csv').length, 79);
assert.equal(dataLines('IDADES_A_RECLASSIFICAR.csv').length, 86);
assert.equal(dataLines('EXCECOES_65_ANALISADAS.csv').length, 65);

const ages = read('IDADES_A_RECLASSIFICAR.csv');
assert.ok(!ages.split(/\r?\n/).slice(1).some(line => /,"[^"]*Múltiplas Idades/.test(line)), 'suggested age destination kept obsolete generic age');
assert.equal((ages.match(/CATEGORIZADO_COM_EVIDÊNCIA/g) ?? []).length, 86);

const exceptions = read('EXCECOES_65_ANALISADAS.csv');
const allowedDecisions = ['CATEGORIZADO_COM_EVIDÊNCIA', 'EXCLUSÃO_INTENCIONAL_COMPROVADA', 'DUPLICADO_PROTEGIDO', 'VERSÃO_ALTERNATIVA', 'DEPENDÊNCIA/STORE_ESPECIAL', 'REVISÃO_PENDENTE'];
const invalidDecisionRows = exceptions.split(/\r?\n/).slice(1).filter(line => line && !allowedDecisions.some(decision => line.includes(`"${decision}"`)));
assert.deepEqual(invalidDecisionRows, [], 'every exception row must have an allowed explicit decision');

const plan = json('PLANO_FASE_2.json');
assert.deepEqual(plan.actions, []);
assert.deepEqual(plan.idempotence, { first_dry_run_actions: [], second_dry_run_actions: [], equal: true });
assert.deepEqual(json('MOVIMENTACOES_FASE_2.json').operations, []);

const before = json('INVENTARIO_FASE_2_ANTES.json');
const after = json('INVENTARIO_FASE_2_DEPOIS.json');
assert.deepEqual(before.files.map(file => [file.relative_path, file.sha256]), after.files.map(file => [file.relative_path, file.sha256]));
const priorRoot = path.resolve('reports/reconciliation-20261009');
const priorModel = JSON.parse(fs.readFileSync(path.join(priorRoot, 'INVENTARIO_MODELO_INICIAL.json'), 'utf8'));
const phase2Model = json('INVENTARIO_FASE_2_MODELO_BASELINE.json');
assert.deepEqual(phase2Model.files.map(file => [file.relative_path, file.sha256]), priorModel.files.map(file => [file.relative_path, file.sha256]));
const priorManager = JSON.parse(fs.readFileSync(path.join(priorRoot, 'INVENTARIO_MANAGER_DEPOIS.json'), 'utf8'));
assert.deepEqual(before.files.map(file => [file.relative_path, file.sha256]), priorManager.files.map(file => [file.relative_path, file.sha256]));

assert.match(scanner, /authoritative_nraas_identifier/);
assert.match(scanner, /Battery\.NRaasLogo2/);
assert.match(scanner, /Related scripted packages in named source folder/);
assert.doesNotMatch(scanner, /\&\[\"nraas\"\]/);
assert.match(catalog, /Criança a Idoso/);
assert.match(catalog, /Facial Hair.*Body Hair/s);

console.log(JSON.stringify({
  ok: true,
  duplicate_groups: 79,
  multiple_age_files: 86,
  exceptions: 65,
  physical_actions: 0,
  idempotent: true,
}, null, 2));
