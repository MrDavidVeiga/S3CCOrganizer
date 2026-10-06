
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

const PREFS_KEY = "s3cc-organizer-preferences-v1";
const UI_KEY = "s3cc-manager-catalog-ui-v1";
const PAGE_SIZE = 250;

const TEXT = {
  en: {
    title:"CC Catalog", intro:"Create and maintain the master CC catalog used by CC Links.",
    create:"Create Master", open:"Open Master", save:"Save Changes", validate:"Validate", auto:"Auto save",
    noMaster:"No master selected.", entries:"Entries", withUrl:"With URL", withoutUrl:"Without URL",
    broken:"Broken Links", duplicates:"Duplicates", withType:"With Type", compactReady:"Compact Ready",
    sources:"CC Sources & Ignored Folders", included:"CC Sources", ignored:"Ignored", add:"Add", scan:"Scan Sources",
    search:"Search Creator, File Name, URL, Tumblr, Type, Resource Type, Instance or TGI…",
    allTypes:"All Types", allResources:"All Resource Types", allStatuses:"All Statuses", modified:"Modified", fresh:"New", errors:"Errors",
    creator:"Creator / Converter", fileName:"File Name", url:"URL", tumblr:"Tumblr Handle", type:"Type",
    resourceType:"Resource Type", instance:"Instance", tgi:"TGI", selectEntry:"Select a catalog entry.",
    saved:"Saved", error:"Error", revert:"Revert changes", markBroken:"Link broken", partnership:"Partnership / Exception",
    openLink:"Open link", compatible:"S3CC Packer compatible ✓", compactCompatible:"Compact CC Links compatible ✓",
    importMissing:"Import Missing CC Links", compare:"Compare Masters", exportIssues:"Export Missing / Issues",
    queue:"Missing CC Links Queue", comparison:"Master Comparison", createNew:"Create new entry", applyMatch:"Apply to match",
    ignore:"Ignore", resolved:"Resolved", usedBy:"Used By", noRows:"No catalog entries match the current filters.",
    sourceEmpty:"No folders configured.", importTitle:"Open Master Catalog", openCatalog:"Open Catalog", cancel:"Cancel",
    schema:"Schema", rows:"Rows", validRows:"Valid", problemRows:"With problems", recognized:"Recognized columns",
    typeDetected:"Type detected", resourceDetected:"Resource Type detected", saveBlocked:"Fix catalog errors before saving.",
    savedOk:"Master catalog saved.", createdOk:"Master catalog created.", openedOk:"Master catalog opened.",
    technical:"Technical identity", suggestions:"Suggestions", noPreview:"No thumbnail available.", clickPreview:"Click thumbnail to reveal the package.",
    addBelow:"Add row below", remove:"Remove row", confirmDelete:"Remove this saved catalog entry?",
    sortAsc:"A → Z", sortDesc:"Z → A", prev:"Previous", next:"Next", applySuggestion:"Apply", comparisonNone:"No differences found.",
    missingUrl:"Without URL", invalidUrl:"Invalid URL"
  },
  pt: {
    title:"Catálogo de CC", intro:"Crie e mantenha a planilha-mestra de CC usada pelo CC Links.",
    create:"Criar Planilha-Mestra", open:"Abrir Master", save:"Salvar Alterações", validate:"Validar", auto:"Salvar automaticamente",
    noMaster:"Nenhuma master selecionada.", entries:"Entradas", withUrl:"Com URL", withoutUrl:"Sem URL",
    broken:"Links Quebrados", duplicates:"Duplicatas", withType:"Com Tipo", compactReady:"Prontas p/ Compacto",
    sources:"Fontes de CC e Pastas Ignoradas", included:"Fontes de CC", ignored:"Ignoradas", add:"Adicionar", scan:"Escanear Fontes",
    search:"Buscar Criador, Nome do Arquivo, URL, Tumblr, Tipo, Resource Type, Instance ou TGI…",
    allTypes:"Todos os Tipos", allResources:"Todos os Resource Types", allStatuses:"Todos os Status", modified:"Alterado", fresh:"Novo", errors:"Erros",
    creator:"Criador / Conversor", fileName:"Nome do Arquivo", url:"URL", tumblr:"Usuário do Tumblr", type:"Tipo",
    resourceType:"Tipo de Recurso", instance:"Instance", tgi:"TGI", selectEntry:"Selecione uma entrada do catálogo.",
    saved:"Salvo", error:"Erro", revert:"Reverter alterações", markBroken:"Link quebrado", partnership:"Parceria / Exceção",
    openLink:"Abrir link", compatible:"S3CC Packer compatível ✓", compactCompatible:"Compact CC Links compatível ✓",
    importMissing:"Importar Missing CC Links", compare:"Comparar Masters", exportIssues:"Exportar Ausentes / Problemas",
    queue:"Fila de Missing CC Links", comparison:"Comparação de Masters", createNew:"Criar nova entrada", applyMatch:"Associar à existente",
    ignore:"Ignorar", resolved:"Resolvido", usedBy:"Usado por", noRows:"Nenhuma entrada corresponde aos filtros atuais.",
    sourceEmpty:"Nenhuma pasta configurada.", importTitle:"Abrir Planilha-Mestra", openCatalog:"Abrir Catálogo", cancel:"Cancelar",
    schema:"Schema", rows:"Linhas", validRows:"Válidas", problemRows:"Com problemas", recognized:"Colunas reconhecidas",
    typeDetected:"Tipo detectado", resourceDetected:"Resource Type detectado", saveBlocked:"Corrija os erros do catálogo antes de salvar.",
    savedOk:"Planilha-mestra salva.", createdOk:"Planilha-mestra criada.", openedOk:"Planilha-mestra aberta.",
    technical:"Identidade técnica", suggestions:"Sugestões", noPreview:"Thumbnail não disponível.", clickPreview:"Clique na thumbnail para abrir a localização do package.",
    addBelow:"Adicionar linha abaixo", remove:"Remover linha", confirmDelete:"Remover esta entrada já salva do catálogo?",
    sortAsc:"A → Z", sortDesc:"Z → A", prev:"Anterior", next:"Próxima", applySuggestion:"Aplicar", comparisonNone:"Nenhuma diferença encontrada.",
    missingUrl:"Sem URL", invalidUrl:"URL inválida"
  },
  es: {
    title:"Catálogo de CC", intro:"Crea y mantén la hoja maestra de CC usada por CC Links.",
    create:"Crear Hoja Maestra", open:"Abrir Master", save:"Guardar Cambios", validate:"Validar", auto:"Guardar automáticamente",
    noMaster:"Ninguna master seleccionada.", entries:"Entradas", withUrl:"Con URL", withoutUrl:"Sin URL",
    broken:"Enlaces Rotos", duplicates:"Duplicados", withType:"Con Tipo", compactReady:"Listas p/ Compacto",
    sources:"Fuentes de CC y Carpetas Ignoradas", included:"Fuentes de CC", ignored:"Ignoradas", add:"Añadir", scan:"Escanear Fuentes",
    search:"Buscar Creador, Nombre del Archivo, URL, Tumblr, Tipo, Resource Type, Instance o TGI…",
    allTypes:"Todos los Tipos", allResources:"Todos los Resource Types", allStatuses:"Todos los Estados", modified:"Modificado", fresh:"Nuevo", errors:"Errores",
    creator:"Creador / Conversor", fileName:"Nombre del Archivo", url:"URL", tumblr:"Usuario de Tumblr", type:"Tipo",
    resourceType:"Tipo de Recurso", instance:"Instance", tgi:"TGI", selectEntry:"Selecciona una entrada del catálogo.",
    saved:"Guardado", error:"Error", revert:"Revertir cambios", markBroken:"Enlace roto", partnership:"Colaboración / Excepción",
    openLink:"Abrir enlace", compatible:"S3CC Packer compatible ✓", compactCompatible:"Compact CC Links compatible ✓",
    importMissing:"Importar Missing CC Links", compare:"Comparar Masters", exportIssues:"Exportar Faltantes / Problemas",
    queue:"Cola de Missing CC Links", comparison:"Comparación de Masters", createNew:"Crear nueva entrada", applyMatch:"Asociar a existente",
    ignore:"Ignorar", resolved:"Resuelto", usedBy:"Usado por", noRows:"Ninguna entrada coincide con los filtros actuales.",
    sourceEmpty:"Ninguna carpeta configurada.", importTitle:"Abrir Hoja Maestra", openCatalog:"Abrir Catálogo", cancel:"Cancelar",
    schema:"Schema", rows:"Filas", validRows:"Válidas", problemRows:"Con problemas", recognized:"Columnas reconocidas",
    typeDetected:"Tipo detectado", resourceDetected:"Resource Type detectado", saveBlocked:"Corrige los errores del catálogo antes de guardar.",
    savedOk:"Hoja maestra guardada.", createdOk:"Hoja maestra creada.", openedOk:"Hoja maestra abierta.",
    technical:"Identidad técnica", suggestions:"Sugerencias", noPreview:"Thumbnail no disponible.", clickPreview:"Haz clic en la thumbnail para abrir la ubicación del package.",
    addBelow:"Añadir fila debajo", remove:"Eliminar fila", confirmDelete:"¿Eliminar esta entrada ya guardada del catálogo?",
    sortAsc:"A → Z", sortDesc:"Z → A", prev:"Anterior", next:"Siguiente", applySuggestion:"Aplicar", comparisonNone:"No se encontraron diferencias.",
    missingUrl:"Sin URL", invalidUrl:"URL inválida"
  }
};

const el = id => document.querySelector(id);
const clone = value => JSON.parse(JSON.stringify(value));
let seq = 0;
const makeKey = () => "catalog-row-" + Date.now() + "-" + (seq++);

let stored = {};
try { stored = JSON.parse(localStorage.getItem(UI_KEY) || "{}"); } catch {}

const state = {
  language:["en","pt","es"].includes(document.documentElement.lang) ? document.documentElement.lang : "en",
  path:"", entries:[], original:new Map(), rowState:new Map(), selectedKey:"",
  validation:null, pendingDocument:null, saving:false, notice:"", error:"",
  search:stored.search || "", typeFilter:stored.typeFilter || "", resourceFilter:stored.resourceFilter || "",
  statusFilter:stored.statusFilter || "", sort:stored.sort || "creatorConverter", direction:stored.direction === "desc" ? "desc" : "asc",
  page:0, autosave:false, backupLimit:5, sources:[], ignored:[], workspace:null,
  missing:null, comparison:null, preview:{}, previewLoading:new Set(), autosaveTimer:null
};

const tr = key => (TEXT[state.language] && TEXT[state.language][key]) || TEXT.en[key] || key;

function currentRoot() {
  try { return JSON.parse(localStorage.getItem(PREFS_KEY) || "{}").folder || ""; } catch { return ""; }
}

function persistUi() {
  localStorage.setItem(UI_KEY, JSON.stringify({
    search:state.search, typeFilter:state.typeFilter, resourceFilter:state.resourceFilter,
    statusFilter:state.statusFilter, sort:state.sort, direction:state.direction
  }));
}

function officialEntry(entry) {
  return {
    entryId:entry.entryId || "", creatorConverter:entry.creatorConverter || "", fileName:entry.fileName || "",
    url:entry.url || "", tumblrHandle:entry.tumblrHandle || "", type:entry.type || "",
    resourceType:entry.resourceType || "", instance:entry.instance || "", tgi:entry.tgi || "",
    linkBroken:!!entry.linkBroken, partnershipException:!!entry.partnershipException,
    packagePath:entry.packagePath || "", modifiedAt:entry.modifiedAt || ""
  };
}

function attachKeys(entries, isNew) {
  return (entries || []).map(entry => {
    const row = Object.assign({}, entry, {_key:makeKey()});
    state.rowState.set(row._key, isNew ? "new" : "saved");
    if (!isNew) state.original.set(row._key, clone(officialEntry(row)));
    return row;
  });
}

function setStatus(message, error) {
  const node = el("#catalog-state");
  if (!node) return;
  node.textContent = message || "";
  node.className = "scan-state" + (error ? " error" : message ? " success" : "");
}

function normalizeName(value) {
  return String(value || "").toLocaleLowerCase().replace(/\.(package|sims3pack)$/i, "").replace(/[^a-z0-9]/g, "");
}

function duplicateKeys() {
  const out = new Set();
  ["fileName","url","instance","tgi"].forEach(field => {
    const map = new Map();
    state.entries.forEach(entry => {
      let value = String(entry[field] || "").trim().toLocaleLowerCase();
      if (field === "fileName") value = normalizeName(value);
      if (field === "url") value = value.replace(/\/$/, "");
      if (!value) return;
      if (!map.has(value)) map.set(value, []);
      map.get(value).push(entry._key);
    });
    map.forEach(keys => { if (keys.length > 1) keys.forEach(key => out.add(key)); });
  });
  return out;
}

function errorKeys() {
  const out = new Set();
  (state.validation && state.validation.issues || []).forEach(issue => {
    if (issue.severity !== "error" || issue.rowIndex == null) return;
    const entry = state.entries[issue.rowIndex];
    if (entry) out.add(entry._key);
  });
  return out;
}

function rowVisualState(entry) {
  if (errorKeys().has(entry._key)) return "error";
  return state.rowState.get(entry._key) || "saved";
}

function catalogUrlValid(value) {
  const text = String(value || "").trim().toLocaleLowerCase();
  return (text.startsWith("http://") || text.startsWith("https://") || text.startsWith("www."))
    && text.includes(".")
    && !/\s/.test(text);
}

function filteredEntries() {
  const query = state.search.trim().toLocaleLowerCase();
  const dup = duplicateKeys();
  const errors = errorKeys();
  let items = state.entries.filter(entry => {
    if (state.typeFilter && entry.type !== state.typeFilter) return false;
    if (state.resourceFilter && entry.resourceType !== state.resourceFilter) return false;
    if (state.statusFilter === "broken" && !entry.linkBroken) return false;
    if (state.statusFilter === "duplicate" && !dup.has(entry._key)) return false;
    if (state.statusFilter === "modified" && state.rowState.get(entry._key) !== "modified") return false;
    if (state.statusFilter === "new" && state.rowState.get(entry._key) !== "new") return false;
    if (state.statusFilter === "error" && !errors.has(entry._key)) return false;
    if (state.statusFilter === "with_url" && !String(entry.url || "").trim()) return false;
    if (state.statusFilter === "missing_url" && String(entry.url || "").trim()) return false;
    if (state.statusFilter === "invalid_url" && (!String(entry.url || "").trim() || catalogUrlValid(entry.url))) return false;
    if (!query) return true;
    return [entry.creatorConverter,entry.fileName,entry.url,entry.tumblrHandle,entry.type,entry.resourceType,entry.instance,entry.tgi]
      .filter(Boolean).join(" ").toLocaleLowerCase().includes(query);
  });
  items.sort((a,b) => {
    let av, bv;
    if (state.sort === "status") {
      const rank = entry => ({error:3,new:2,modified:1,saved:0}[rowVisualState(entry)] || 0);
      av = rank(a); bv = rank(b);
    } else {
      av = String(a[state.sort] || "").toLocaleLowerCase();
      bv = String(b[state.sort] || "").toLocaleLowerCase();
    }
    const order = av < bv ? -1 : av > bv ? 1 : 0;
    return state.direction === "desc" ? -order : order;
  });
  return items;
}

let validateTimer = null;
function markModified(entry) {
  if (state.rowState.get(entry._key) !== "new") state.rowState.set(entry._key, "modified");
  clearTimeout(validateTimer);
  validateTimer = setTimeout(() => void validateCatalog(), 180);
  if (state.autosave && state.path) {
    clearTimeout(state.autosaveTimer);
    state.autosaveTimer = setTimeout(() => void saveCatalog(true), 700);
  }
}

async function validateCatalog() {
  try {
    state.validation = await invoke("validate_cc_catalog", {entries:state.entries.map(officialEntry)});
  } catch (error) {
    setStatus(String(error), true);
  }
  updateHealth();
  renderTable();
}

function updateHealth() {
  const h = state.validation && state.validation.health || {};
  el("#catalog-health-entries").textContent = h.entries == null ? state.entries.length : h.entries;
  el("#catalog-health-url").textContent = h.withUrl || 0;
  el("#catalog-health-missing-url").textContent = h.withoutUrl || 0;
  el("#catalog-health-broken").textContent = h.brokenLinks || 0;
  el("#catalog-health-duplicates").textContent = h.duplicateGroups || 0;
  el("#catalog-health-type").textContent = h.withType || 0;
  el("#catalog-health-compact").textContent = h.compactReady || 0;
  const missing = Math.max(0, (h.entries || 0) - (h.compactReady || 0));
  const compat = el("#catalog-compatibility");
  compat.textContent = tr("compatible") + " · " + (missing ? "Compact CC Links: " + missing : tr("compactCompatible"));
  compat.classList.toggle("warning", missing > 0);
}

function rebuildFilters() {
  function fill(select, values, label, current) {
    select.innerHTML = "";
    const first = document.createElement("option");
    first.value = ""; first.textContent = label; select.appendChild(first);
    [...new Set(values.filter(Boolean))].sort((a,b) => a.localeCompare(b)).forEach(value => {
      const option = document.createElement("option");
      option.value = value; option.textContent = value; select.appendChild(option);
    });
    select.value = current;
  }
  fill(el("#catalog-type-filter"), state.entries.map(x => x.type), tr("allTypes"), state.typeFilter);
  fill(el("#catalog-resource-filter"), state.entries.map(x => x.resourceType), tr("allResources"), state.resourceFilter);
}

function createCellInput(entry, field, readonly) {
  const input = document.createElement("input");
  input.className = "catalog-cell-input";
  input.value = entry[field] || "";
  input.readOnly = !!readonly;
  if (readonly) input.tabIndex = -1;
  input.addEventListener("focus", () => {
    state.selectedKey = entry._key;
    renderPreview();
    highlightSelected();
  });
  if (!readonly) {
    input.addEventListener("change", () => {
      entry[field] = input.value;
      markModified(entry);
      rebuildFilters();
      renderPreview();
    });
  }
  return input;
}

function highlightSelected() {
  const body = el("#catalog-table-body");
  if (!body) return;
  body.querySelectorAll(".catalog-row").forEach(row => row.classList.toggle("selected", row.dataset.key === state.selectedKey));
}

function renderTable() {
  const body = el("#catalog-table-body");
  if (!body) return;
  body.innerHTML = "";
  const all = filteredEntries();
  const pages = Math.max(1, Math.ceil(all.length / PAGE_SIZE));
  state.page = Math.min(state.page, pages - 1);
  const start = state.page * PAGE_SIZE;
  const visible = all.slice(start, start + PAGE_SIZE);
  if (!visible.length) {
    const empty = document.createElement("div");
    empty.className = "catalog-empty"; empty.textContent = tr("noRows"); body.appendChild(empty); return;
  }

  visible.forEach(entry => {
    const row = document.createElement("div");
    row.className = "catalog-grid catalog-row catalog-row-" + rowVisualState(entry);
    row.dataset.key = entry._key;
    row.addEventListener("click", event => {
      if (event.target.closest("button,input")) return;
      state.selectedKey = entry._key; renderPreview(); highlightSelected();
    });
    row.append(createCellInput(entry,"creatorConverter",false), createCellInput(entry,"fileName",false));

    const urlCell = document.createElement("div");
    urlCell.className = "catalog-url-cell";
    urlCell.appendChild(createCellInput(entry,"url",false));
    if (entry.url) {
      const link = document.createElement("button");
      link.type = "button"; link.tabIndex = -1; link.className = "catalog-mini-icon"; link.title = tr("openLink");
      link.innerHTML = '<i class="fa-solid fa-arrow-up-right-from-square"></i>';
      link.addEventListener("click", async event => {
        event.stopPropagation();
        try { await invoke("open_external_url", {url:entry.url}); } catch (error) { setStatus(String(error), true); }
      });
      urlCell.appendChild(link);
    }
    row.append(urlCell, createCellInput(entry,"tumblrHandle",false), createCellInput(entry,"type",false),
      createCellInput(entry,"resourceType",true), createCellInput(entry,"instance",true), createCellInput(entry,"tgi",true));

    const actions = document.createElement("div");
    actions.className = "catalog-row-actions";
    const status = document.createElement("span");
    const visual = rowVisualState(entry);
    status.className = "catalog-row-status state-" + visual;
    status.textContent = tr(visual === "new" ? "fresh" : visual);
    const minus = document.createElement("button");
    minus.type = "button"; minus.tabIndex = -1; minus.className = "catalog-row-button"; minus.textContent = "−"; minus.title = tr("remove");
    minus.addEventListener("click", event => { event.stopPropagation(); void removeRow(entry); });
    const plus = document.createElement("button");
    plus.type = "button"; plus.tabIndex = -1; plus.className = "catalog-row-button"; plus.textContent = "+"; plus.title = tr("addBelow");
    plus.addEventListener("click", event => { event.stopPropagation(); addRowBelow(entry); });
    actions.append(status, minus, plus); row.appendChild(actions);
    body.appendChild(row);
  });

  const footer = document.createElement("div");
  footer.className = "catalog-page-footer";
  const text = document.createElement("span");
  text.textContent = (start + 1) + "–" + Math.min(start + PAGE_SIZE, all.length) + " / " + all.length;
  const nav = document.createElement("div");
  const prev = document.createElement("button");
  prev.className = "secondary-btn compact-btn"; prev.textContent = tr("prev"); prev.disabled = state.page === 0;
  prev.addEventListener("click", () => { state.page--; renderTable(); });
  const next = document.createElement("button");
  next.className = "secondary-btn compact-btn"; next.textContent = tr("next"); next.disabled = state.page >= pages - 1;
  next.addEventListener("click", () => { state.page++; renderTable(); });
  nav.append(prev,next); footer.append(text,nav); body.appendChild(footer);
  highlightSelected();
}

function suggestionsFor(entry) {
  const creator = String(entry.creatorConverter || "").trim().toLocaleLowerCase();
  if (!creator) return [];
  const peers = state.entries.filter(x => x._key !== entry._key && String(x.creatorConverter || "").trim().toLocaleLowerCase() === creator);
  const result = [];
  [["tumblrHandle",tr("tumblr")],["type",tr("type")],["url",tr("url")]].forEach(pair => {
    const field = pair[0], label = pair[1];
    if (entry[field]) return;
    const counts = new Map();
    peers.map(x => String(x[field] || "").trim()).filter(Boolean).forEach(value => counts.set(value,(counts.get(value)||0)+1));
    const best = [...counts.entries()].sort((a,b) => b[1]-a[1])[0];
    if (best) result.push({field:field,label:label,value:best[0]});
  });
  return result;
}

function sourceRootForPackage(path) {
  const value = String(path || "").toLocaleLowerCase().replaceAll("/","\\");
  const source = state.sources.filter(x => x.enabled !== false).find(x => value.startsWith(String(x.path || "").toLocaleLowerCase().replaceAll("/","\\")));
  return source && source.path || currentRoot();
}

async function ensurePreview(entry) {
  if (!entry || !entry.packagePath || state.preview[entry.packagePath] || state.previewLoading.has(entry.packagePath)) return;
  const root = sourceRootForPackage(entry.packagePath);
  if (!root) return;
  state.previewLoading.add(entry.packagePath);
  renderPreview();
  try { state.preview[entry.packagePath] = await invoke("get_package_preview",{folder:root,packagePath:entry.packagePath}); }
  catch { state.preview[entry.packagePath] = {thumbnailBase64:null,mimeType:null}; }
  state.previewLoading.delete(entry.packagePath);
  renderPreview();
}

function appendMeta(container,label,value) {
  if (value == null || String(value).trim() === "") return;
  const row = document.createElement("div"); row.className = "meta-row";
  const a = document.createElement("span"); a.className = "meta-label"; a.textContent = label;
  const b = document.createElement("strong"); b.textContent = String(value);
  row.append(a,b); container.appendChild(row);
}

function renderPreview() {
  const pane = el("#catalog-preview");
  if (!pane) return;
  const entry = state.entries.find(x => x._key === state.selectedKey);
  pane.innerHTML = "";
  if (!entry) {
    const empty = document.createElement("div"); empty.className = "preview-empty"; empty.textContent = tr("selectEntry"); pane.appendChild(empty); return;
  }

  const thumb = document.createElement("button");
  thumb.type = "button"; thumb.className = "package-preview-thumb-wrap"; thumb.title = tr("clickPreview");
  const preview = state.preview[entry.packagePath];
  if (preview && preview.thumbnailBase64) {
    const image = document.createElement("img");
    image.className = "package-preview-thumb"; image.alt = entry.fileName;
    image.src = "data:" + (preview.mimeType || "image/png") + ";base64," + preview.thumbnailBase64;
    thumb.appendChild(image);
  } else {
    const empty = document.createElement("span"); empty.className = "package-preview-thumb-empty";
    empty.innerHTML = '<i class="fa-regular fa-image"></i>';
    const small = document.createElement("small"); small.textContent = state.previewLoading.has(entry.packagePath) ? "…" : tr("noPreview");
    empty.appendChild(small); thumb.appendChild(empty);
  }
  thumb.disabled = !entry.packagePath;
  thumb.addEventListener("click", () => { if (entry.packagePath) invoke("reveal_path",{path:entry.packagePath}); });
  pane.appendChild(thumb);
  if (entry.packagePath) queueMicrotask(() => ensurePreview(entry));

  const header = document.createElement("div"); header.className = "preview-header";
  const h3 = document.createElement("h3"); h3.textContent = entry.fileName || "—";
  const badge = document.createElement("span"); badge.className = "status-badge state-" + rowVisualState(entry);
  badge.textContent = tr(rowVisualState(entry) === "new" ? "fresh" : rowVisualState(entry));
  header.append(h3,badge); pane.appendChild(header);

  const meta = document.createElement("div"); meta.className = "preview-meta";
  appendMeta(meta,tr("creator"),entry.creatorConverter); appendMeta(meta,tr("type"),entry.type);
  appendMeta(meta,tr("resourceType"),entry.resourceType); appendMeta(meta,tr("instance"),entry.instance);
  appendMeta(meta,tr("tgi"),entry.tgi); appendMeta(meta,tr("url"),entry.url); pane.appendChild(meta);

  const flags = document.createElement("div"); flags.className = "catalog-preview-flags";
  [["linkBroken",tr("markBroken")],["partnershipException",tr("partnership")]].forEach(pair => {
    const label = document.createElement("label");
    const check = document.createElement("input"); check.type = "checkbox"; check.checked = !!entry[pair[0]];
    check.addEventListener("change", () => { entry[pair[0]] = check.checked; markModified(entry); renderAll(); });
    label.append(check,document.createTextNode(pair[1])); flags.appendChild(label);
  });
  pane.appendChild(flags);

  const suggestions = suggestionsFor(entry);
  if (suggestions.length) {
    const box = document.createElement("div"); box.className = "catalog-suggestions";
    const title = document.createElement("strong"); title.textContent = tr("suggestions"); box.appendChild(title);
    suggestions.forEach(item => {
      const line = document.createElement("div");
      const text = document.createElement("span"); text.textContent = item.label + ": " + item.value;
      const button = document.createElement("button"); button.className = "secondary-btn compact-btn"; button.textContent = tr("applySuggestion");
      button.addEventListener("click", () => { entry[item.field] = item.value; markModified(entry); renderAll(); });
      line.append(text,button); box.appendChild(line);
    });
    pane.appendChild(box);
  }

  if (state.rowState.get(entry._key) !== "new") {
    const revert = document.createElement("button"); revert.type = "button"; revert.className = "secondary-btn catalog-revert"; revert.textContent = tr("revert");
    revert.addEventListener("click", () => revertRow(entry)); pane.appendChild(revert);
  }
}

function addRowBelow(entry) {
  const index = state.entries.indexOf(entry);
  const row = {
    entryId:"", creatorConverter:entry.creatorConverter || "", fileName:"", url:"",
    tumblrHandle:entry.tumblrHandle || "", type:"", resourceType:"", instance:"", tgi:"",
    linkBroken:false, partnershipException:false, packagePath:"", modifiedAt:"", _key:makeKey()
  };
  state.entries.splice(index+1,0,row); state.rowState.set(row._key,"new");
  state.selectedKey = row._key; void validateCatalog(); renderAll();
}

function internalConfirm(message) {
  return new Promise(resolve => {
    const overlay = document.createElement("div"); overlay.className = "modal-backdrop";
    const box = document.createElement("section"); box.className = "confirm-modal card";
    const p = document.createElement("p"); p.textContent = message;
    const actions = document.createElement("div"); actions.className = "confirm-actions";
    const cancel = document.createElement("button"); cancel.className = "secondary-btn"; cancel.textContent = tr("cancel");
    const ok = document.createElement("button"); ok.className = "primary-btn"; ok.textContent = tr("remove");
    const done = value => { overlay.remove(); resolve(value); };
    cancel.onclick = () => done(false); ok.onclick = () => done(true); overlay.onclick = event => { if (event.target === overlay) done(false); };
    actions.append(cancel,ok); box.append(p,actions); overlay.appendChild(box); document.body.appendChild(overlay);
  });
}

async function removeRow(entry) {
  if (state.rowState.get(entry._key) !== "new" && !(await internalConfirm(tr("confirmDelete")))) return;
  const index = state.entries.indexOf(entry);
  if (index >= 0) state.entries.splice(index,1);
  state.original.delete(entry._key); state.rowState.delete(entry._key);
  if (state.selectedKey === entry._key) state.selectedKey = state.entries[Math.min(index,state.entries.length-1)]?._key || "";
  await validateCatalog(); renderAll();
  if (state.autosave) void saveCatalog(true);
}

function revertRow(entry) {
  const original = state.original.get(entry._key);
  if (!original) return;
  Object.assign(entry,clone(original)); state.rowState.set(entry._key,"saved");
  void validateCatalog(); renderAll();
}

function renderSources() {
  function list(node, items) {
    node.innerHTML = "";
    if (!items.length) {
      const empty = document.createElement("div"); empty.className = "catalog-source-empty"; empty.textContent = tr("sourceEmpty"); node.appendChild(empty); return;
    }
    items.forEach((item,index) => {
      const row = document.createElement("div"); row.className = "catalog-path-row";
      const check = document.createElement("input"); check.type = "checkbox"; check.checked = item.enabled !== false;
      check.addEventListener("change", () => { item.enabled = check.checked; void saveProfileSettings(); renderSources(); });
      const path = document.createElement("code"); path.textContent = item.path; path.title = item.path;
      const remove = document.createElement("button"); remove.className = "catalog-row-button"; remove.textContent = "−";
      remove.addEventListener("click", () => { items.splice(index,1); void saveProfileSettings(); renderSources(); });
      row.append(check,path,remove); node.appendChild(row);
    });
  }
  list(el("#catalog-sources-list"),state.sources); list(el("#catalog-ignored-list"),state.ignored);
  el("#catalog-sources-summary").textContent = state.sources.filter(x=>x.enabled!==false).length + " · " + state.ignored.filter(x=>x.enabled!==false).length;
}

async function loadProfileSettings() {
  const root = currentRoot();
  if (!root) { state.workspace = null; renderSources(); return; }
  try {
    state.workspace = await invoke("load_workspace",{folder:root});
    const profile = (state.workspace.profiles || []).find(x => x.id === state.workspace.activeProfileId) || (state.workspace.profiles || [])[0];
    state.sources = clone(profile?.catalogSources || []);
    state.ignored = clone(profile?.catalogIgnored || []);
    state.path = state.path || profile?.catalogMasterFile || "";
    state.backupLimit = Number(profile?.catalogBackupLimit || 5);
    state.autosave = !!profile?.catalogAutosave;
    if (!state.sources.length) state.sources = [{path:root,enabled:true}];
    el("#catalog-autosave").checked = state.autosave;
    const backup = el("#catalog-backup-limit"); if (backup) backup.value = String(state.backupLimit);
    renderSources(); updatePath();
    if (state.path && !state.entries.length) {
      try { loadDocument(await invoke("open_cc_catalog",{path:state.path}),false); } catch {}
    }
  } catch (error) { setStatus(String(error),true); }
}

async function saveProfileSettings() {
  const root = currentRoot();
  if (!root || !state.workspace) return;
  const profile = (state.workspace.profiles || []).find(x => x.id === state.workspace.activeProfileId) || (state.workspace.profiles || [])[0];
  if (!profile) return;
  profile.catalogSources = clone(state.sources); profile.catalogIgnored = clone(state.ignored);
  profile.catalogMasterFile = state.path; profile.catalogBackupLimit = state.backupLimit; profile.catalogAutosave = state.autosave;
  try { state.workspace = await invoke("save_workspace",{folder:root,store:state.workspace}); } catch (error) { setStatus(String(error),true); }
}

function updatePath() {
  el("#catalog-path").textContent = state.path || tr("noMaster");
  el("#catalog-path").title = state.path || "";
  const disabled = !state.path;
  ["#catalog-save","#catalog-validate","#catalog-import-missing","#catalog-compare","#catalog-export-issues"].forEach(id => el(id).disabled = disabled);
}

function loadDocument(doc, notice) {
  state.path = doc.path; state.original.clear(); state.rowState.clear();
  state.entries = attachKeys(doc.entries || [],false); state.validation = doc.validation || null;
  state.selectedKey = state.entries[0]?._key || ""; state.page = 0; state.pendingDocument = null;
  updatePath(); void saveProfileSettings(); renderAll();
  if (notice !== false) setStatus(tr("openedOk"));
}

async function createMaster() {
  const path = await save({title:tr("create"),defaultPath:"S3CC-Master-Catalog.xlsx",filters:[{name:"Excel Workbook",extensions:["xlsx"]},{name:"CSV",extensions:["csv"]}]});
  if (!path) return;
  try { loadDocument(await invoke("create_cc_catalog",{path:path,language:state.language}),false); setStatus(tr("createdOk")); }
  catch (error) { setStatus(String(error),true); }
}

async function openMaster() {
  const path = await open({multiple:false,directory:false,title:tr("open"),filters:[{name:"CC Master",extensions:["xlsx","xls","xlsb","ods","csv"]}]});
  if (!path || Array.isArray(path)) return;
  try {
    const doc = await invoke("open_cc_catalog",{path:path});
    state.pendingDocument = doc;
    const summary = el("#catalog-import-modal-summary"); summary.innerHTML = "";
    [
      [tr("schema"),doc.schemaVersion || "Legacy / progressive"],[tr("rows"),doc.previewTotalRows],
      [tr("validRows"),doc.previewValidRows],[tr("problemRows"),doc.previewProblemRows],
      [tr("recognized"),(doc.recognizedColumns || []).join(", ") || "—"],
      [tr("typeDetected"),(doc.recognizedColumns || []).includes("type") ? "✓" : "—"],
      [tr("resourceDetected"),(doc.recognizedColumns || []).includes("resource_type") ? "✓" : "—"]
    ].forEach(pair => {
      const row = document.createElement("div"); row.className = "meta-row";
      const a = document.createElement("span"); a.className = "meta-label"; a.textContent = pair[0];
      const b = document.createElement("strong"); b.textContent = String(pair[1]); row.append(a,b); summary.appendChild(row);
    });
    el("#catalog-import-modal").classList.remove("hidden"); el("#catalog-import-modal").setAttribute("aria-hidden","false");
  } catch (error) { setStatus(String(error),true); }
}

async function saveCatalog(silent) {
  if (!state.path || state.saving) return;
  await validateCatalog();
  if ((state.validation?.health?.errors || 0) > 0) {
    if (!silent) setStatus(tr("saveBlocked"),true);
    return;
  }

  let targetPath = state.path;
  const extension = String(targetPath).split(".").pop().toLocaleLowerCase();
  if (extension !== "xlsx" && extension !== "csv") {
    if (silent) return;
    targetPath = await save({
      title:tr("save"),
      defaultPath:"S3CC-Master-Catalog.xlsx",
      filters:[{name:"Excel Workbook",extensions:["xlsx"]},{name:"CSV",extensions:["csv"]}]
    });
    if (!targetPath) return;
  }

  state.saving = true;
  try {
    const result = await invoke("save_cc_catalog",{path:targetPath,language:state.language,entries:state.entries.map(officialEntry),backupLimit:state.backupLimit});
    if (targetPath !== state.path) {
      state.path = targetPath;
      updatePath();
      await saveProfileSettings();
    }
    state.validation = result.validation; state.original.clear();
    state.entries.forEach(entry => { state.original.set(entry._key,clone(officialEntry(entry))); state.rowState.set(entry._key,"saved"); });
    if (!silent) setStatus(tr("savedOk"));
    renderAll();
  } catch (error) { setStatus(String(error),true); }
  state.saving = false;
}

async function choosePath(kind) {
  const path = await open({directory:true,multiple:false,title:kind === "source" ? tr("included") : tr("ignored")});
  if (!path || Array.isArray(path)) return;
  const list = kind === "source" ? state.sources : state.ignored;
  if (!list.some(x => x.path.toLocaleLowerCase() === path.toLocaleLowerCase())) list.push({path:path,enabled:true});
  await saveProfileSettings(); renderSources();
}

async function scanSources() {
  if (!state.sources.some(x => x.enabled !== false)) return;
  try {
    const result = await invoke("scan_cc_catalog_sources",{sources:state.sources,ignored:state.ignored});
    let enriched = 0, added = 0;
    const byName = new Map(state.entries.map(entry => [normalizeName(entry.fileName),entry]));
    (result.entries || []).forEach(scanned => {
      const key = normalizeName(scanned.fileName);
      const existing = byName.get(key);
      if (existing) {
        ["resourceType","instance","tgi","packagePath","modifiedAt"].forEach(field => {
          if (scanned[field] && existing[field] !== scanned[field]) { existing[field] = scanned[field]; markModified(existing); }
        });
        if (!existing.type && scanned.type) { existing.type = scanned.type; markModified(existing); }
        enriched++;
      } else {
        const row = attachKeys([scanned],true)[0]; state.entries.push(row); byName.set(key,row); added++;
      }
    });
    el("#catalog-scan-summary").textContent = (result.scannedPackages || 0) + " packages · " + (result.skippedIgnored || 0) + " ignored · " + (result.errors || []).length + " errors";
    setStatus(enriched + " enriched · " + added + " added");
    await validateCatalog(); renderAll();
  } catch (error) { setStatus(String(error),true); }
}

async function importMissing() {
  const path = await open({multiple:false,directory:false,title:tr("importMissing"),filters:[{name:"Missing CC Links",extensions:["txt"]}]});
  if (!path || Array.isArray(path)) return;
  try {
    state.missing = await invoke("import_missing_cc_links",{path:path,entries:state.entries.map(officialEntry)});
    el("#catalog-missing-panel").classList.remove("hidden"); el("#catalog-missing-panel").open = true; renderMissing();
  } catch (error) { setStatus(String(error),true); }
}

function entryFromMissing(item) {
  return {
    entryId:"", creatorConverter:item.creatorConverter || "", fileName:item.fileName || item.detectedName || "",
    url:item.url || "", tumblrHandle:"", type:item.type || "", resourceType:item.resourceType || "",
    instance:item.instance || "", tgi:item.tgi || "", linkBroken:item.status === "INVALID_URL",
    partnershipException:false, packagePath:"", modifiedAt:"", _key:makeKey()
  };
}

function renderMissing() {
  const data = state.missing; if (!data) return;
  el("#catalog-missing-summary").textContent = (data.total || data.items.length) + " · " + data.notInMaster + " NOT_IN_MASTER · " + data.possibleMatch + " POSSIBLE_MATCH · " + data.missingUrl + " MISSING_URL · " + data.invalidUrl + " INVALID_URL · " + data.missingType + " MISSING_TYPE";
  const list = el("#catalog-missing-list"); list.innerHTML = "";
  (data.items || []).forEach(item => {
    const card = document.createElement("article"); card.className = "catalog-queue-item" + (item._resolved ? " resolved" : "");
    const head = document.createElement("div"); head.className = "catalog-queue-head";
    const strong = document.createElement("strong"); strong.textContent = item.fileName || item.detectedName || "—";
    const badge = document.createElement("span"); badge.className = "status-badge"; badge.textContent = item.status; head.append(strong,badge);
    const small = document.createElement("small");
    small.textContent = [item.creatorConverter,item.type,item.url,item.usedBy ? tr("usedBy") + ": " + item.usedBy : ""].filter(Boolean).join(" · ");
    const actions = document.createElement("div"); actions.className = "catalog-queue-actions";
    const create = document.createElement("button"); create.className = "secondary-btn compact-btn"; create.textContent = tr("createNew"); create.disabled = !!item._resolved;
    create.onclick = () => {
      const row = entryFromMissing(item); state.entries.push(row); state.rowState.set(row._key,"new"); state.selectedKey = row._key; item._resolved = true;
      void validateCatalog(); renderAll(); renderMissing();
    };
    actions.appendChild(create);
    if (item.suggestedEntryId) {
      const apply = document.createElement("button"); apply.className = "secondary-btn compact-btn"; apply.textContent = tr("applyMatch"); apply.disabled = !!item._resolved;
      apply.onclick = () => {
        const target = state.entries.find(entry => entry.entryId === item.suggestedEntryId);
        if (target) {
          [["creatorConverter",item.creatorConverter],["url",item.url],["type",item.type],["resourceType",item.resourceType],["instance",item.instance],["tgi",item.tgi]]
            .forEach(pair => { if (pair[1] && !target[pair[0]]) target[pair[0]] = pair[1]; });
          markModified(target); state.selectedKey = target._key;
        }
        item._resolved = true; renderAll(); renderMissing();
      };
      actions.appendChild(apply);
    }
    const ignore = document.createElement("button"); ignore.className = "secondary-btn compact-btn"; ignore.textContent = item._resolved ? tr("resolved") : tr("ignore");
    ignore.onclick = () => { item._resolved = !item._resolved; renderMissing(); };
    actions.appendChild(ignore); card.append(head,small,actions); list.appendChild(card);
  });
}

async function compareMasters() {
  if (!state.path) return;
  const other = await open({multiple:false,directory:false,title:tr("compare"),filters:[{name:"CC Master",extensions:["xlsx","xls","xlsb","ods","csv"]}]});
  if (!other || Array.isArray(other)) return;
  try {
    state.comparison = await invoke("compare_cc_catalogs",{leftPath:state.path,rightPath:other});
    el("#catalog-compare-panel").classList.remove("hidden"); el("#catalog-compare-panel").open = true; renderComparison();
  } catch (error) { setStatus(String(error),true); }
}

function renderComparison() {
  const data = state.comparison; if (!data) return;
  el("#catalog-compare-summary").textContent = (data.same || 0) + " same · " + (data.different || []).length + " different · " + (data.onlyLeft || []).length + " only current · " + (data.onlyRight || []).length + " only compared";
  const list = el("#catalog-compare-list"); list.innerHTML = "";
  let total = 0;
  [["Different",data.different],["Only current",data.onlyLeft],["Only compared",data.onlyRight]].forEach(group => {
    (group[1] || []).slice(0,300).forEach(item => {
      total++;
      const card = document.createElement("article"); card.className = "catalog-queue-item";
      const strong = document.createElement("strong"); strong.textContent = group[0] + ": " + (item.left?.fileName || item.right?.fileName || item.key);
      const small = document.createElement("small"); small.textContent = (item.differentFields || []).join(", ");
      card.append(strong,small); list.appendChild(card);
    });
  });
  if (!total) { const empty = document.createElement("div"); empty.className = "catalog-empty"; empty.textContent = tr("comparisonNone"); list.appendChild(empty); }
}

async function exportIssues() {
  const path = await save({title:tr("exportIssues"),defaultPath:"S3CC-Missing-and-Issues.xlsx",filters:[{name:"Excel Workbook",extensions:["xlsx"]},{name:"CSV",extensions:["csv"]}]});
  if (!path) return;

  const extraRows = [];
  for (const item of state.missing?.items || []) {
    if (item._resolved) continue;
    extraRows.push({
      status:item.status || "MISSING_CC_LINK",
      entry:officialEntry(entryFromMissing(item))
    });
  }
  for (const item of state.comparison?.onlyRight || []) {
    if (item.right) extraRows.push({status:"ONLY_COMPARED_MASTER",entry:officialEntry(item.right)});
  }
  for (const item of state.comparison?.onlyLeft || []) {
    if (item.left) extraRows.push({status:"ONLY_CURRENT_MASTER",entry:officialEntry(item.left)});
  }

  try {
    await invoke("export_catalog_issues",{
      path:path,
      language:state.language,
      entries:state.entries.map(officialEntry),
      extraRows:extraRows
    });
    setStatus(path);
  } catch (error) { setStatus(String(error),true); }
}

function setStaticText() {
  el("#catalog-title").textContent = tr("title"); el("#catalog-intro").textContent = tr("intro");
  el("#catalog-create span").textContent = tr("create"); el("#catalog-open span").textContent = tr("open");
  el("#catalog-save span").textContent = tr("save"); el("#catalog-validate span").textContent = tr("validate");
  el(".catalog-autosave span").textContent = tr("auto"); el(".catalog-backup-limit span").textContent = "Backups"; el("#catalog-sources-title").textContent = tr("sources");
  el("#catalog-included-title").textContent = tr("included"); el("#catalog-ignored-title").textContent = tr("ignored");
  el("#catalog-add-source").textContent = "+ " + tr("add"); el("#catalog-add-ignored").textContent = "+ " + tr("add");
  el("#catalog-scan-sources span").textContent = tr("scan"); el("#catalog-search").placeholder = tr("search");
  el("#catalog-import-missing span").textContent = tr("importMissing"); el("#catalog-compare span").textContent = tr("compare");
  el("#catalog-export-issues span").textContent = tr("exportIssues"); el("#catalog-missing-title").textContent = tr("queue");
  el("#catalog-compare-title").textContent = tr("comparison"); el("#catalog-import-modal-title").textContent = tr("importTitle");
  el("#catalog-import-cancel").textContent = tr("cancel"); el("#catalog-import-accept").textContent = tr("openCatalog");

  [["#catalog-health-entries","entries"],["#catalog-health-url","withUrl"],["#catalog-health-missing-url","withoutUrl"],["#catalog-health-broken","broken"],["#catalog-health-duplicates","duplicates"],["#catalog-health-type","withType"],["#catalog-health-compact","compactReady"]]
    .forEach(pair => el(pair[0]).nextElementSibling.textContent = tr(pair[1]));

  const headers = [tr("creator"),tr("fileName"),tr("url"),tr("tumblr"),tr("type"),tr("resourceType"),tr("instance"),tr("tgi")];
  [...el("#catalog-table-header").children].slice(0,8).forEach((node,index) => node.textContent = headers[index]);

  const status = el("#catalog-status-filter");
  [tr("allStatuses"),tr("broken"),tr("duplicates"),tr("modified"),tr("fresh"),tr("errors"),tr("withUrl"),tr("missingUrl"),tr("invalidUrl")].forEach((label,index) => { if (status.options[index]) status.options[index].textContent = label; });
  const sort = el("#catalog-sort");
  [tr("creator"),tr("fileName"),tr("type"),tr("resourceType"),"Status","Modified date"].forEach((label,index) => { if (sort.options[index]) sort.options[index].textContent = label; });
  renderAll();
}

function renderAll() {
  updatePath(); updateHealth(); rebuildFilters(); renderSources(); renderTable(); renderPreview();
  el("#catalog-search").value = state.search; el("#catalog-status-filter").value = state.statusFilter; el("#catalog-sort").value = state.sort;
  el("#catalog-sort-direction").textContent = state.direction === "asc" ? tr("sortAsc") : tr("sortDesc");
}

el("#catalog-create")?.addEventListener("click",createMaster);
el("#catalog-open")?.addEventListener("click",openMaster);
el("#catalog-save")?.addEventListener("click",()=>saveCatalog(false));
el("#catalog-validate")?.addEventListener("click",async()=>{await validateCatalog();setStatus((state.validation?.health?.errors || 0) + " " + tr("errors") + " · " + (state.validation?.health?.warnings || 0) + " warnings");});
el("#catalog-add-source")?.addEventListener("click",()=>choosePath("source"));
el("#catalog-add-ignored")?.addEventListener("click",()=>choosePath("ignored"));
el("#catalog-scan-sources")?.addEventListener("click",scanSources);
el("#catalog-import-missing")?.addEventListener("click",importMissing);
el("#catalog-compare")?.addEventListener("click",compareMasters);
el("#catalog-export-issues")?.addEventListener("click",exportIssues);
el("#catalog-autosave")?.addEventListener("change",event=>{state.autosave=event.currentTarget.checked;void saveProfileSettings();});
el("#catalog-backup-limit")?.addEventListener("change",event=>{state.backupLimit=Math.max(1,Math.min(10,Number(event.currentTarget.value)||5));void saveProfileSettings();});
el("#catalog-search")?.addEventListener("input",event=>{state.search=event.currentTarget.value;state.page=0;persistUi();renderTable();});
el("#catalog-type-filter")?.addEventListener("change",event=>{state.typeFilter=event.currentTarget.value;state.page=0;persistUi();renderTable();});
el("#catalog-resource-filter")?.addEventListener("change",event=>{state.resourceFilter=event.currentTarget.value;state.page=0;persistUi();renderTable();});
el("#catalog-status-filter")?.addEventListener("change",event=>{state.statusFilter=event.currentTarget.value;state.page=0;persistUi();renderTable();});
el("#catalog-sort")?.addEventListener("change",event=>{state.sort=event.currentTarget.value;state.page=0;persistUi();renderTable();});
el("#catalog-sort-direction")?.addEventListener("click",()=>{state.direction=state.direction==="asc"?"desc":"asc";persistUi();renderAll();});
el("#catalog-table-header")?.addEventListener("click",event=>{
  const button=event.target.closest("[data-sort]"); if(!button)return;
  const field=button.dataset.sort;
  if(state.sort===field) state.direction=state.direction==="asc"?"desc":"asc"; else {state.sort=field;state.direction="asc";}
  persistUi();renderAll();
});
el("#catalog-import-cancel")?.addEventListener("click",()=>{state.pendingDocument=null;el("#catalog-import-modal").classList.add("hidden");});
el("#catalog-import-accept")?.addEventListener("click",()=>{if(state.pendingDocument)loadDocument(state.pendingDocument,true);el("#catalog-import-modal").classList.add("hidden");});
el("#catalog-import-modal")?.addEventListener("click",event=>{if(event.target===el("#catalog-import-modal"))el("#catalog-import-cancel").click();});

window.addEventListener("s3cc-language-changed",event=>{state.language=event.detail||"en";setStaticText();});
window.addEventListener("s3cc-folder-changed",()=>void loadProfileSettings());

void loadProfileSettings();
setStaticText();
