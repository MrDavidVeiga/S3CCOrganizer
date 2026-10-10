import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';

const args = process.argv.slice(2).reduce((out, value, i, all) => { if (value.startsWith('--')) out[value.slice(2)] = all[i + 1] ?? true; return out; }, {});
if (!args.model || !args.manager || !args.inventory || !args.out) throw new Error('usage: node reconcile_apply.mjs --model PATH --manager PATH --inventory PATH --out PATH [--apply --backup PATH]');
const modelRoot = path.resolve(args.model), managerRoot = path.resolve(args.manager), inventoryRoot = path.resolve(args.inventory), out = path.resolve(args.out);
const read = async name => JSON.parse(await fs.readFile(path.join(inventoryRoot, name), 'utf8'));
const model = await read('INVENTARIO_MODELO_INICIAL.json'), before = await read('INVENTARIO_MANAGER_ANTES.json');
const byHash = new Map();
for (const item of model.files) byHash.set(item.sha256, [...(byHash.get(item.sha256) ?? []), item]);
const csvCell = value => `"${String(value ?? '').replaceAll('"', '""')}"`;
const rel = file => path.relative(managerRoot, file).replaceAll(path.sep, '/');
const sha = async file => crypto.createHash('sha256').update(await fs.readFile(file)).digest('hex');
const entries = [];
const notReintroduced = [];
for (const item of before.files) {
  const matches = byHash.get(item.sha256) ?? [];
  const record = { before: item.relative_path, sha256: item.sha256, size: item.size, status: item.status, model_matches: matches.map(x => x.relative_path), decision: 'protected_for_review', destination: null, reason: '' };
  if (matches.length === 1 && item.status === 'active_package') {
    record.destination = matches[0].relative_path;
    record.decision = 'safe_exact_hash_move';
    record.reason = 'Unique SHA-256 match in the read-only manual model; destination is copied as a relative path under Packages.';
    entries.push(record);
  } else if (matches.length === 0) {
    record.reason = item.status === 'disabled_or_backup' ? 'No exact model match and inactive/backup state preserved; never restore automatically.' : 'No exact SHA-256 match in the model; may be intentional removal, replacement, or unresolved version.';
    notReintroduced.push(record);
  } else if (matches.length > 1) {
    record.reason = 'The same bytes occur in multiple model paths; destination choice is ambiguous and no move is inferred.';
    notReintroduced.push(record);
  } else {
    record.reason = 'Non-package/auxiliary item or inactive item is not moved by the package reconciliation pass.';
    notReintroduced.push(record);
  }
}

const plan = [];
for (const item of entries) {
  const source = path.join(managerRoot, item.before.split('/').join(path.sep));
  const destination = path.join(managerRoot, item.destination.split('/').join(path.sep));
  let destinationExists = false;
  try { await fs.access(destination); destinationExists = true; } catch {}
  plan.push({ ...item, source_absolute: source, destination_absolute: destination, destination_exists: destinationExists, action: destinationExists ? 'blocked_destination_exists' : 'move' });
}
await fs.mkdir(out, { recursive: true });
const mapLines = ['before,model_reference,after,sha256,decision,reason'];
for (const item of [...plan, ...notReintroduced]) mapLines.push([item.before, item.model_matches?.length === 1 ? item.model_matches[0] : item.model_matches?.join(' | '), item.action === 'move' ? item.destination : item.before, item.sha256, item.decision, item.reason].map(csvCell).join(','));
await fs.writeFile(path.join(out, 'MAPA_ANTES_MODELO_DEPOIS.csv'), mapLines.join('\n') + '\n');
await fs.writeFile(path.join(out, 'ARQUIVOS_NAO_REINTRODUZIDOS.json'), JSON.stringify({ generated_at_utc: new Date().toISOString(), files: notReintroduced }, null, 2));
await fs.writeFile(path.join(out, 'RECONCILIACAO_PLANO.json'), JSON.stringify({ generated_at_utc: new Date().toISOString(), model_root: modelRoot, manager_root: managerRoot, safe_exact_hash_moves: plan, protected: notReintroduced, counts: { before_files: before.files.length, safe_exact_hash_moves: plan.filter(x => x.action === 'move').length, destination_collisions: plan.filter(x => x.action !== 'move').length, protected: notReintroduced.length } }, null, 2));

let backup = null;
if (args.apply) {
  if (!args.backup) throw new Error('--backup is required with --apply');
  backup = path.resolve(args.backup);
  try { await fs.access(backup); throw new Error(`backup destination already exists: ${backup}`); } catch (error) { if (error.message.startsWith('backup destination')) throw error; }
  await fs.cp(managerRoot, backup, { recursive: true, errorOnExist: true, force: false });
  const operations = [];
  for (const item of plan) {
    if (item.action !== 'move') continue;
    const currentHash = await sha(item.source_absolute);
    if (currentHash !== item.sha256) throw new Error(`source changed after inventory: ${item.source_absolute}`);
    try { await fs.access(item.destination_absolute); throw new Error(`destination appeared: ${item.destination_absolute}`); } catch (error) { if (error.message.startsWith('destination appeared')) throw error; }
    await fs.mkdir(path.dirname(item.destination_absolute), { recursive: true });
    await fs.rename(item.source_absolute, item.destination_absolute);
    const movedHash = await sha(item.destination_absolute);
    if (movedHash !== item.sha256) throw new Error(`destination hash mismatch after move: ${item.destination_absolute}`);
    operations.push({ before: item.before, after: item.destination, sha256: item.sha256, size: item.size, status: 'moved_verified' });
  }
  const removedFolders = [];
  const dirs = [];
  async function collect(dir) { for (const entry of await fs.readdir(dir, { withFileTypes: true })) { const full = path.join(dir, entry.name); if (entry.isDirectory()) { await collect(full); dirs.push(full); } } }
  await collect(managerRoot);
  for (const dir of dirs.sort((a, b) => b.length - a.length)) {
    if ((await fs.readdir(dir)).length === 0) { await fs.rmdir(dir); removedFolders.push(rel(dir)); }
  }
  await fs.writeFile(path.join(out, 'MOVIMENTACOES_APLICADAS.json'), JSON.stringify({ generated_at_utc: new Date().toISOString(), backup_root: backup, model_root: modelRoot, manager_root: managerRoot, operations, counts: { moved_verified: operations.length, removed_empty_folders: removedFolders.length } }, null, 2));
  await fs.writeFile(path.join(out, 'PASTAS_LEGADAS_REMOVIDAS.json'), JSON.stringify({ generated_at_utc: new Date().toISOString(), folders: removedFolders }, null, 2));
  const leftovers = [];
  for (const item of notReintroduced) { const full = path.join(managerRoot, item.before.split('/').join(path.sep)); try { await fs.access(full); leftovers.push(item.before); } catch {} }
  await fs.writeFile(path.join(out, 'PASTAS_COM_SOBRAS.json'), JSON.stringify({ generated_at_utc: new Date().toISOString(), files_left_for_review: leftovers, note: 'Protected files remain because they lack a unique exact model match or are inactive/auxiliary.' }, null, 2));
  console.log(JSON.stringify({ mode: 'APPLIED', backup, moved_verified: operations.length, removed_empty_folders: removedFolders.length, protected: leftovers.length }, null, 2));
} else {
  console.log(JSON.stringify({ mode: 'SIMULATION', safe_exact_hash_moves: plan.filter(x => x.action === 'move').length, destination_collisions: plan.filter(x => x.action !== 'move').length, protected: notReintroduced.length, out }, null, 2));
}
