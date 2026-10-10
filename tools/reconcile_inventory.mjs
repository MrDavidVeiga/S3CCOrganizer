import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';

const TYPE_NAMES = new Map([
  [0x034AEECB, 'CASP'], [0x015A1849, 'GEOM'], [0x01D0E75D, 'VPXY'],
  [0x736884F1, 'NMAP'], [0x319E4F1D, 'OBJD'], [0x423A2E7C, 'OBJK'],
  [0x0333406C, 'STBL'], [0x062C8204, 'XML'], [0x033A1435, 'ITUN'],
  [0x073FAA07, 'CLIP'], [0x00B2D882, 'S3SA'],
]);
const disabled = ['.disabled', '.disable', '.bak', '.old', '.backup', '.inactive'];
const isPackage = name => /\.(package|dbc|sims3pack)$/i.test(name);
const rel = (root, file) => path.relative(root, file).replaceAll(path.sep, '/') || '.';
const now = () => new Date().toISOString();

async function hash(file) {
  const data = await fs.readFile(file);
  return { sha256: crypto.createHash('sha256').update(data).digest('hex'), data };
}

function status(name) {
  const low = name.toLowerCase();
  if (disabled.some(suffix => low.endsWith(suffix))) return 'disabled_or_backup';
  if (low.endsWith('.package')) return 'active_package';
  if (low.endsWith('.dbc')) return 'archive_package';
  if (low.endsWith('.sims3pack')) return 'sims3pack';
  return 'auxiliary_or_unknown';
}

function parseDbpf(data) {
  const empty = { is_dbpf: false, dbpf_error: null, major: null, minor: null, resource_count: 0, resources: [], resource_types: [], resource_classes: [], instances: [], payload_evidence: [] };
  try {
    if (data.length < 96 || data.readUInt32LE(0) !== 0x46504244) return empty;
    const major = data.readUInt32LE(4), minor = data.readUInt32LE(8);
    const result = { ...empty, is_dbpf: true, major, minor };
    if (major !== 2) return result;
    // DBPF header offsets mirror src-tauri/src/dbpf.rs: count=36,
    // index_length=44, index_version=60, index_position=64.
    const count = data.readUInt32LE(36), length = data.readUInt32LE(44), version = data.readUInt32LE(60), position = data.readUInt32LE(64);
    result.resource_count = count;
    if (!count || version !== 3 || !position || position + length > data.length) return result;
    let cursor = position;
    const indexType = data.readUInt32LE(cursor); cursor += 4;
    const common = Array(8).fill(0);
    for (let i = 0; i < 8; i++) if (indexType & (1 << i)) { common[i] = data.readUInt32LE(cursor); cursor += 4; }
    const resources = [];
    for (let n = 0; n < count; n++) {
      const record = common.slice();
      for (let i = 0; i < 8; i++) if (!(indexType & (1 << i))) { record[i] = data.readUInt32LE(cursor); cursor += 4; }
      const [typeId, group, hi, lo, offset, rawSize, memSize, comp] = record;
      const instance = (BigInt(hi) << 32n) | BigInt(lo);
      resources.push({ type_id: `0x${typeId.toString(16).padStart(8, '0').toUpperCase()}`, type_name: TYPE_NAMES.get(typeId) ?? `0x${typeId.toString(16).padStart(8, '0').toUpperCase()}`, group: `0x${group.toString(16).padStart(8, '0').toUpperCase()}`, instance: `0x${instance.toString(16).padStart(16, '0').toUpperCase()}`, offset, file_size: rawSize & 0x7fffffff, memory_size: memSize, compressed: comp & 0xffff, file_size_high_bit: !!(rawSize & 0x80000000) });
    }
    result.resources = resources;
    result.resource_types = [...new Set(resources.map(r => r.type_name))].sort();
    result.resource_classes = [...new Set(resources.map(r => r.type_id))].sort();
    result.instances = [...new Set(resources.map(r => r.instance))].sort();
    result.payload_evidence = [];
    for (const resource of resources) {
      if (!resource.file_size || resource.file_size > 2_000_000) continue;
      const raw = data.subarray(resource.offset, Math.min(data.length, resource.offset + Math.min(resource.file_size, 256_000)));
      const ascii = raw.toString('utf8').match(/[A-Za-zÀ-ÿ][A-Za-zÀ-ÿ0-9_ .:/\\'\-]{3,}/g) ?? [];
      for (const token of ascii) if (!result.payload_evidence.includes(token.trim())) result.payload_evidence.push(token.trim().slice(0, 160));
      if (result.payload_evidence.length >= 48) break;
    }
    result.payload_evidence = result.payload_evidence.slice(0, 48);
    return result;
  } catch (error) { return { ...empty, dbpf_error: `${error.name}: ${error.message}` }; }
}

function evidence(file, root, dbpf) {
  const relative = rel(root, file).toLowerCase();
  const values = [`path:${path.posix.dirname(relative) || '.'}`];
  const tokens = [['nraas', 'creator_or_mod_token:nraas'], ['simpanel', 'mod_token:simpanel'], ['banking', 'mod_token:banking'], ['baking', 'mod_token:baking'], ['flash fm', 'radio_token:flash_fm'], ['rabbit', 'rabbit_hole_token'], ['poselist', 'pose_list_token'], ['posepack', 'pose_pack_token'], ['pose pack', 'pose_pack_token'], ['fix', 'fix_token'], ['store', 'store_token'], ['aurora skies', 'store_set:aurora_skies'], ['tiny prodigies', 'store_set:tiny_prodigies']];
  for (const [token, label] of tokens) if (relative.includes(token)) values.push(label);
  for (const type of dbpf.resource_types ?? []) values.push(`resource_type:${type}`);
  if (dbpf.is_dbpf) values.push('dbpf:index_parsed');
  if (dbpf.dbpf_error) values.push('dbpf:parse_error');
  return [...new Set(values)].sort();
}

async function walk(root) {
  const files = [], folders = [], errors = [];
  async function visit(current) {
    let entries;
    try { entries = await fs.readdir(current, { withFileTypes: true }); } catch (error) { errors.push(`${current}: ${error.name}: ${error.message}`); return; }
    if (current !== root) folders.push({ absolute_path: current, relative_path: rel(root, current), depth: path.relative(root, current).split(path.sep).length, file_count: 0, subfolder_count: 0, empty: entries.length === 0, hidden: path.basename(current).startsWith('.') });
    for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
      const full = path.join(current, entry.name);
      if (entry.isDirectory()) await visit(full);
      else if (entry.isFile()) {
        try {
          const stat = await fs.stat(full), hashed = await hash(full);
          const dbpf = isPackage(entry.name) ? parseDbpf(hashed.data) : { is_dbpf: false, dbpf_error: null, major: null, minor: null, resource_count: 0, resources: [], resource_types: [], resource_classes: [], instances: [], payload_evidence: [] };
          files.push({ absolute_path: full, relative_path: rel(root, full), extension: path.extname(entry.name).toLowerCase(), name: entry.name, size: stat.size, sha256: hashed.sha256, type: 'file', status: status(entry.name), hidden: entry.name.startsWith('.'), readable: true, dbpf, classification_evidence: evidence(full, root, dbpf) });
        } catch (error) { errors.push(`${full}: ${error.name}: ${error.message}`); }
      }
    }
  }
  await visit(root);
  const fileCounts = new Map(), dirCounts = new Map();
  for (const item of files) { const parent = path.posix.dirname(item.relative_path); fileCounts.set(parent, (fileCounts.get(parent) ?? 0) + 1); }
  for (const item of folders) { const parent = path.posix.dirname(item.relative_path); dirCounts.set(parent, (dirCounts.get(parent) ?? 0) + 1); }
  for (const folder of folders) { folder.file_count = fileCounts.get(folder.relative_path) ?? 0; folder.subfolder_count = dirCounts.get(folder.relative_path) ?? 0; folder.empty = folder.file_count === 0 && folder.subfolder_count === 0; }
  return { generated_at_utc: now(), root, files, folders, errors, summary: { file_count: files.length, folder_count: folders.length, empty_folder_count: folders.filter(f => f.empty).length, bytes: files.reduce((sum, f) => sum + f.size, 0), sha256_counts: Object.fromEntries([...files.reduce((map, f) => map.set(f.sha256, (map.get(f.sha256) ?? 0) + 1), new Map())]), status_counts: Object.fromEntries([...files.reduce((map, f) => map.set(f.status, (map.get(f.status) ?? 0) + 1), new Map())]), resource_type_counts: Object.fromEntries([...files.flatMap(f => f.dbpf.resource_types ?? []).reduce((map, type) => map.set(type, (map.get(type) ?? 0) + 1), new Map())]) } };
}

function csvCell(value) { return `"${String(value ?? '').replaceAll('"', '""')}"`; }
async function writeCsv(file, inventory) {
  const fields = ['absolute_path', 'relative_path', 'extension', 'name', 'size', 'sha256', 'status', 'hidden', 'readable', 'resource_count', 'resource_types', 'instances', 'classification_evidence'];
  const lines = [fields.join(',')];
  for (const item of inventory.files) lines.push(fields.map(field => csvCell(field === 'resource_count' ? item.dbpf.resource_count : field === 'resource_types' ? item.dbpf.resource_types.join(';') : field === 'instances' ? item.dbpf.instances.join(';') : field === 'classification_evidence' ? item.classification_evidence.join(';') : item[field])).join(','));
  await fs.writeFile(file, lines.join('\n') + '\n', 'utf8');
}
async function writeTree(file, inventory) {
  const lines = [`ROOT ${inventory.root}`];
  for (const folder of inventory.folders.sort((a, b) => a.relative_path.localeCompare(b.relative_path))) lines.push(`${'  '.repeat(folder.relative_path.split('/').length)}${path.basename(folder.relative_path)}/${folder.empty ? ' [EMPTY]' : ''}`);
  for (const item of inventory.files.sort((a, b) => a.relative_path.localeCompare(b.relative_path))) lines.push(`${'  '.repeat(path.posix.dirname(item.relative_path) === '.' ? 1 : path.posix.dirname(item.relative_path).split('/').length + 1)}${path.basename(item.relative_path)}`);
  await fs.writeFile(file, lines.join('\n') + '\n', 'utf8');
}
async function relationships(file, manager, model) {
  const left = new Map(), right = new Map();
  for (const item of manager.files) left.set(item.sha256, [...(left.get(item.sha256) ?? []), item]);
  for (const item of model.files) right.set(item.sha256, [...(right.get(item.sha256) ?? []), item]);
  const rows = ['sha256,exact_hash_match,manager_before,model_reference,manager_status,model_status,relation'];
  for (const digest of [...new Set([...left.keys(), ...right.keys()])].sort()) for (const old of left.get(digest) ?? [null]) for (const fresh of right.get(digest) ?? [null]) rows.push([digest, !!old && !!fresh, old?.relative_path, fresh?.relative_path, old?.status, fresh?.status, old && fresh ? 'exact_sha256' : old ? 'manager_only' : 'model_only'].map(csvCell).join(','));
  await fs.writeFile(file, rows.join('\n') + '\n', 'utf8');
}

const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => { if (value.startsWith('--')) pairs.push([value.slice(2), all[index + 1]]); return pairs; }, []));
if (!args.model || !args.manager || !args.out) throw new Error('usage: node reconcile_inventory.mjs --model PATH --manager PATH --out PATH');
await fs.mkdir(args.out, { recursive: true });
const model = await walk(path.resolve(args.model)), manager = await walk(path.resolve(args.manager));
await fs.writeFile(path.join(args.out, 'INVENTARIO_MODELO_INICIAL.json'), JSON.stringify(model, null, 2), 'utf8');
await fs.writeFile(path.join(args.out, 'INVENTARIO_MANAGER_ANTES.json'), JSON.stringify(manager, null, 2), 'utf8');
await writeCsv(path.join(args.out, 'INVENTARIO_MODELO_INICIAL.csv'), model); await writeCsv(path.join(args.out, 'INVENTARIO_MANAGER_ANTES.csv'), manager);
await writeTree(path.join(args.out, 'ARVORE_MODELO_INICIAL.txt'), model); await writeTree(path.join(args.out, 'ARVORE_MANAGER_ANTES.txt'), manager);
await relationships(path.join(args.out, 'RELACIONAMENTO_SHA256_INICIAL.csv'), manager, model);
console.log(JSON.stringify({ model: model.summary, manager: manager.summary, out: args.out }, null, 2));
