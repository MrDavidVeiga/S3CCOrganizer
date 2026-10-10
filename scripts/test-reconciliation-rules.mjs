import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const reports = path.join(repo, 'reports', 'reconciliation-20261009');
const source = await fs.readFile(path.join(repo, 'src-tauri', 'src', 'scanner.rs'), 'utf8');
if (!(await fs.stat(reports).catch(() => null))) {
  assert.match(source, /nraas_family_destination/);
  assert.match(source, /is_explicit_non_nraas_integration/);
  assert.match(source, /UnknownLevelRemoved/);
  console.log(JSON.stringify({ ok: true, skipped: 'private phase 1 corpus is not part of the public checkout' }, null, 2));
  process.exit(0);
}
const read = async name => JSON.parse(await fs.readFile(path.join(reports, name), 'utf8'));
const model = await read('INVENTARIO_MODELO_INICIAL.json');
const modelAfter = await read(path.join('after-scan', 'INVENTARIO_MODELO_INICIAL.json'));
const after = await read('INVENTARIO_MANAGER_DEPOIS.json');
const plan = await read('RECONCILIACAO_PLANO.json');
const moved = await read('MOVIMENTACOES_APLICADAS.json');
const excluded = await read('ARQUIVOS_NAO_REINTRODUZIDOS.json');
const conflicts = await read('DUPLICADOS_E_CONFLITOS.json');
const modelPaths = new Set(model.files.map(item => item.relative_path));
const movedPaths = new Set(moved.operations.map(item => item.after));
const protectedPaths = new Set(excluded.files.map(item => item.before));
const hasModelPath = token => [...modelPaths].some(item => item.toLowerCase().includes(token.toLowerCase()));

for (const token of [
  'Scripts/UI/Battery.Simpanel_1.03.package',
  'Scripts/NRaas/MasterController/NRaas_MasterController.package',
  'Scripts/Gameplay/Radio and TV/AE_GTA_VC_RadioMod_Flash_FM.package',
  'Construção/Rabbit Holes/',
  'Poses e Animações/',
  'Sliders/Rosto/Nariz/OneEuroMuttTip Width.package',
  'Correções/Gameplay/EPs Fixes/EP01/',
  'Correções/Gameplay/EPs Fixes/EP11/',
]) assert.ok(hasModelPath(token), `model fixture missing: ${token}`);

assert.equal(moved.counts.moved_verified, 97);
assert.equal(plan.counts.protected, 65);
assert.ok([...movedPaths].every(item => !/\/CAS\//i.test(`/${item}/`)));
assert.ok([...movedPaths].every(item => !/(Unknown|Desconhecido|Desconocido)/i.test(item)));
assert.ok(protectedPaths.has('Sem Categoria/ccmerged.package'), 'ccmerged must remain protected');
assert.ok(protectedPaths.has('Scripts.zip'), 'auxiliary archive must remain protected');
assert.equal(model.files.length, modelAfter.files.length);
assert.deepEqual(model.files.map(item => [item.relative_path, item.sha256]), modelAfter.files.map(item => [item.relative_path, item.sha256]));
assert.ok(conflicts.same_tgi_different_package_sha256_candidates.length >= 1);
assert.match(source, /nraas_family_destination/);
assert.match(source, /is_explicit_non_nraas_integration/);
assert.match(source, /UnknownLevelRemoved/);
console.log(JSON.stringify({ ok: true, moved: moved.counts.moved_verified, protected: plan.counts.protected, model_unchanged: true, tgi_conflict_candidates: conflicts.same_tgi_different_package_sha256_candidates.length }, null, 2));
