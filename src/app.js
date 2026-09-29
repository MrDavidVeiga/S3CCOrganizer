import "@fortawesome/fontawesome-free/css/all.min.css";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const I18N = {
  en: {
    organizer: "Organizer",
    duplicates: "Duplicates",
    conflicts: "Conflicts",
    restore: "Restore",
    language: "Language",
    chooseModsFolder: "Choose Mods Folder",
    noFolder: "No folder selected.",
    scanCcs: "Scan CCs",
    scanning: "Scanning packages…",
    analyzeBefore: "Analyze before organizing",
    safetyMessage: "Packages are classified from their actual resources. Review the plan before any move.",
    readOnlyStage: "Analyze first, preview every move, then organize.",
    packages: "Packages",
    classified: "Classified",
    mixed: "Mixed",
    needsReview: "Needs Review",
    invalid: "Invalid",
    unknown: "Unknown",
    chooseToBegin: "Choose a Mods folder to begin.",
    search: "Search packages, categories or destinations…",
    allStatuses: "All statuses",
    selectPackage: "Select a package to see its classification.",
    detectedFrom: "Detected from",
    category: "Category",
    subCategory: "Subcategory",
    gender: "Gender",
    age: "Age",
    species: "Species",
    usageCategories: "Catalog categories",
    suggestedDestination: "Suggested destination",
    possibleDestinations: "Possible destinations",
    resources: "Resources",
    originalPath: "Current path",
    warnings: "Review notes",
    noDestination: "No automatic destination",
    noResults: "No packages match the current filters.",
    scanComplete: "Scan complete",
    scanFailed: "Scan failed",
    previewPlan: "Preview Plan",
    planning: "Building plan…",
    planFailed: "Plan failed",
    selectAllVisible: "Select All",
    selectNoneVisible: "Select None",
    selected: "Selected",
    organizationPlan: "Organization Plan",
    simulationOnly: "Preflight preview. Nothing moves until you choose Organize Selected.",
    readyToMove: "Ready",
    collisions: "Collisions",
    blocked: "Blocked",
    foldersToCreate: "Folders",
    manifestPreview: "Restore manifest preview",
    reviewBeforeMove: "Review every change before organizing.",
    close: "Close",
    cancel: "Cancel",
    organizeSelected: "Organize Selected",
    alreadyOrganized: "Already organized",
    collisionSame: "Same file exists",
    collisionDifferent: "Different file exists",
    current: "Current",
    proposed: "Proposed",
    noFoldersNeeded: "No new folders are needed.",
    selectedEligible: "eligible selected",
    notEligible: "Not eligible for automatic organization",
    executing: "Organizing selected packages…",
    executionComplete: "Organization completed",
    executionNoChanges: "Nothing needed to be moved",
    executionRolledBack: "Organization failed and was rolled back",
    confirmOrganizeTitle: "Organize selected packages?",
    confirmOrganizeMessage: "A restore manifest will be written before any move. Files will never overwrite existing destinations.",
    restoreIntro: "Choose a restore manifest to preview the exact rollback before changing files.",
    chooseManifest: "Choose Restore Manifest",
    noManifest: "No manifest selected.",
    previewRestore: "Preview Restore",
    executeRestore: "Restore Structure",
    tracked: "Tracked",
    readyToRestore: "Ready",
    newFiles: "New Files",
    chooseManifestToBegin: "Choose a restore manifest to begin.",
    restorePreviewing: "Analyzing restore manifest…",
    restoreReady: "Restore preview ready",
    restoreFailed: "Restore failed",
    restoring: "Restoring previous structure…",
    restoreComplete: "Restore completed",
    confirmRestoreTitle: "Restore previous structure?",
    confirmRestoreMessage: "Tracked files will return to their original paths. Files added later will be moved to Not Categorized. Nothing is overwritten.",
    alreadyRestored: "Already restored",
    readyNew: "New → Not Categorized",
    alreadyUncategorized: "Already Not Categorized",
    changed: "Changed",
    missing: "Missing",
    ambiguous: "Ambiguous",
    duplicatesNext: "Exact, content and related-variant detection will be connected after the catalog scanner is validated.",
    conflictsNext: "Resource-level conflict analysis will distinguish shared identical resources from real overrides.",
  },
  pt: {
    organizer: "Organizador",
    duplicates: "Duplicados",
    conflicts: "Conflitos",
    restore: "Restaurar",
    language: "Idioma",
    chooseModsFolder: "Escolher Pasta de Mods",
    noFolder: "Nenhuma pasta selecionada.",
    scanCcs: "Analisar CCs",
    scanning: "Analisando packages…",
    analyzeBefore: "Analise antes de organizar",
    safetyMessage: "Os packages são classificados pelos resources reais. Revise o plano antes de qualquer movimentação.",
    readOnlyStage: "Analise primeiro, visualize cada movimento e só depois organize.",
    packages: "Packages",
    classified: "Classificados",
    mixed: "Mistos",
    needsReview: "Requer Revisão",
    invalid: "Inválidos",
    unknown: "Desconhecidos",
    chooseToBegin: "Escolha uma pasta de Mods para começar.",
    search: "Pesquisar packages, categorias ou destinos…",
    allStatuses: "Todos os estados",
    selectPackage: "Selecione um package para ver a classificação.",
    detectedFrom: "Detectado por",
    category: "Categoria",
    subCategory: "Subcategoria",
    gender: "Gênero",
    age: "Idade",
    species: "Espécie",
    usageCategories: "Categorias do catálogo",
    suggestedDestination: "Destino sugerido",
    possibleDestinations: "Destinos possíveis",
    resources: "Resources",
    originalPath: "Caminho atual",
    warnings: "Notas para revisão",
    noDestination: "Sem destino automático",
    noResults: "Nenhum package corresponde aos filtros atuais.",
    scanComplete: "Análise concluída",
    scanFailed: "Falha na análise",
    previewPlan: "Visualizar Plano",
    planning: "Montando plano…",
    planFailed: "Falha no plano",
    selectAllVisible: "Selecionar Tudo",
    selectNoneVisible: "Selecionar Nenhum",
    selected: "Selecionados",
    organizationPlan: "Plano de Organização",
    simulationOnly: "Preview de segurança. Nada será movido até escolher Organizar Selecionados.",
    readyToMove: "Prontos",
    collisions: "Colisões",
    blocked: "Bloqueados",
    foldersToCreate: "Pastas",
    manifestPreview: "Preview do manifesto de restauração",
    reviewBeforeMove: "Revise todas as alterações antes de organizar.",
    close: "Fechar",
    cancel: "Cancelar",
    organizeSelected: "Organizar Selecionados",
    alreadyOrganized: "Já organizado",
    collisionSame: "Arquivo idêntico já existe",
    collisionDifferent: "Arquivo diferente já existe",
    current: "Atual",
    proposed: "Proposto",
    noFoldersNeeded: "Nenhuma nova pasta precisa ser criada.",
    selectedEligible: "aptos selecionados",
    notEligible: "Não elegível para organização automática",
    executing: "Organizando packages selecionados…",
    executionComplete: "Organização concluída",
    executionNoChanges: "Nenhum arquivo precisava ser movido",
    executionRolledBack: "A organização falhou e foi revertida",
    confirmOrganizeTitle: "Organizar os packages selecionados?",
    confirmOrganizeMessage: "Um manifesto de restauração será salvo antes de qualquer movimento. Nenhum arquivo sobrescreverá um destino existente.",
    restoreIntro: "Escolha um manifesto de restauração para visualizar exatamente o que será desfeito antes de alterar os arquivos.",
    chooseManifest: "Escolher Manifesto de Restauração",
    noManifest: "Nenhum manifesto selecionado.",
    previewRestore: "Visualizar Restauração",
    executeRestore: "Restaurar Estrutura",
    tracked: "Rastreados",
    readyToRestore: "Prontos",
    newFiles: "Arquivos Novos",
    chooseManifestToBegin: "Escolha um manifesto de restauração para começar.",
    restorePreviewing: "Analisando manifesto de restauração…",
    restoreReady: "Preview da restauração pronto",
    restoreFailed: "Falha na restauração",
    restoring: "Restaurando estrutura anterior…",
    restoreComplete: "Restauração concluída",
    confirmRestoreTitle: "Restaurar a estrutura anterior?",
    confirmRestoreMessage: "Os arquivos rastreados voltarão aos caminhos originais. Arquivos adicionados depois irão para Não Categorizado. Nada será sobrescrito.",
    alreadyRestored: "Já restaurado",
    readyNew: "Novo → Não Categorizado",
    alreadyUncategorized: "Já em Não Categorizado",
    changed: "Alterado",
    missing: "Ausente",
    ambiguous: "Ambíguo",
    duplicatesNext: "A detecção de cópias exatas, duplicados por conteúdo e variantes será conectada depois da validação do scanner de catálogo.",
    conflictsNext: "A análise de conflitos por resource distinguirá resources idênticos compartilhados de overrides reais.",
  },
  es: {
    organizer: "Organizador",
    duplicates: "Duplicados",
    conflicts: "Conflictos",
    restore: "Restaurar",
    language: "Idioma",
    chooseModsFolder: "Elegir Carpeta de Mods",
    noFolder: "Ninguna carpeta seleccionada.",
    scanCcs: "Analizar CCs",
    scanning: "Analizando packages…",
    analyzeBefore: "Analiza antes de organizar",
    safetyMessage: "Los packages se clasifican por sus resources reales. Revisa el plan antes de cualquier movimiento.",
    readOnlyStage: "Analiza primero, revisa cada movimiento y luego organiza.",
    packages: "Packages",
    classified: "Clasificados",
    mixed: "Mixtos",
    needsReview: "Requiere Revisión",
    invalid: "Inválidos",
    unknown: "Desconocidos",
    chooseToBegin: "Elige una carpeta de Mods para comenzar.",
    search: "Buscar packages, categorías o destinos…",
    allStatuses: "Todos los estados",
    selectPackage: "Selecciona un package para ver su clasificación.",
    detectedFrom: "Detectado por",
    category: "Categoría",
    subCategory: "Subcategoría",
    gender: "Género",
    age: "Edad",
    species: "Especie",
    usageCategories: "Categorías del catálogo",
    suggestedDestination: "Destino sugerido",
    possibleDestinations: "Destinos posibles",
    resources: "Resources",
    originalPath: "Ruta actual",
    warnings: "Notas para revisión",
    noDestination: "Sin destino automático",
    noResults: "Ningún package coincide con los filtros actuales.",
    scanComplete: "Análisis completado",
    scanFailed: "Error en el análisis",
    previewPlan: "Ver Plan",
    planning: "Creando plan…",
    planFailed: "Error en el plan",
    selectAllVisible: "Seleccionar Todo",
    selectNoneVisible: "Seleccionar Ninguno",
    selected: "Seleccionados",
    organizationPlan: "Plan de Organización",
    simulationOnly: "Vista previa de seguridad. Nada se moverá hasta elegir Organizar Seleccionados.",
    readyToMove: "Listos",
    collisions: "Colisiones",
    blocked: "Bloqueados",
    foldersToCreate: "Carpetas",
    manifestPreview: "Vista previa del manifiesto de restauración",
    reviewBeforeMove: "Revisa todos los cambios antes de organizar.",
    close: "Cerrar",
    cancel: "Cancelar",
    organizeSelected: "Organizar Seleccionados",
    alreadyOrganized: "Ya organizado",
    collisionSame: "Ya existe un archivo idéntico",
    collisionDifferent: "Ya existe un archivo diferente",
    current: "Actual",
    proposed: "Propuesto",
    noFoldersNeeded: "No es necesario crear carpetas nuevas.",
    selectedEligible: "aptos seleccionados",
    notEligible: "No apto para organización automática",
    executing: "Organizando packages seleccionados…",
    executionComplete: "Organización completada",
    executionNoChanges: "No era necesario mover archivos",
    executionRolledBack: "La organización falló y fue revertida",
    confirmOrganizeTitle: "¿Organizar los packages seleccionados?",
    confirmOrganizeMessage: "Se guardará un manifiesto de restauración antes de cualquier movimiento. Ningún archivo sobrescribirá un destino existente.",
    restoreIntro: "Elige un manifiesto de restauración para ver exactamente lo que se deshará antes de modificar archivos.",
    chooseManifest: "Elegir Manifiesto de Restauración",
    noManifest: "Ningún manifiesto seleccionado.",
    previewRestore: "Ver Restauración",
    executeRestore: "Restaurar Estructura",
    tracked: "Rastreados",
    readyToRestore: "Listos",
    newFiles: "Archivos Nuevos",
    chooseManifestToBegin: "Elige un manifiesto de restauración para comenzar.",
    restorePreviewing: "Analizando manifiesto de restauración…",
    restoreReady: "Vista previa de restauración lista",
    restoreFailed: "Error de restauración",
    restoring: "Restaurando estructura anterior…",
    restoreComplete: "Restauración completada",
    confirmRestoreTitle: "¿Restaurar la estructura anterior?",
    confirmRestoreMessage: "Los archivos rastreados volverán a sus rutas originales. Los archivos añadidos después irán a Sin categorizar. Nada será sobrescrito.",
    alreadyRestored: "Ya restaurado",
    readyNew: "Nuevo → Sin categorizar",
    alreadyUncategorized: "Ya en Sin categorizar",
    changed: "Modificado",
    missing: "Ausente",
    ambiguous: "Ambiguo",
    duplicatesNext: "La detección de copias exactas, duplicados por contenido y variantes se conectará después de validar el escáner de catálogo.",
    conflictsNext: "El análisis de conflictos por resource distinguirá resources idénticos compartidos de overrides reales.",
  },
};

const LANGUAGE_ORDER = ["en", "pt", "es"];

const state = {
  language: localStorage.getItem("s3cc-organizer-language") || "en",
  tab: "organizer",
  folder: "",
  items: [],
  stats: null,
  selectedId: "",
  selectedForPlan: new Set(),
  search: "",
  status: "all",
  scanning: false,
  planning: false,
  executing: false,
  error: "",
  planError: "",
  notice: "",
  plan: null,
  restoreManifest: "",
  restorePlan: null,
  restoreBusy: false,
  restoreError: "",
  restoreNotice: "",
  pendingAction: "",
};

if (!LANGUAGE_ORDER.includes(state.language)) state.language = "en";

const el = {
  tabs: [...document.querySelectorAll(".tabs button")],
  pages: [...document.querySelectorAll(".tool-page")],
  languageButton: document.querySelector("#language-button"),
  languageCode: document.querySelector("#language-code"),
  languageMenu: document.querySelector("#language-menu"),
  languageMenuItems: [...document.querySelectorAll(".lang-menu-item")],
  chooseFolderBtn: document.querySelector("#choose-folder-btn"),
  scanBtn: document.querySelector("#scan-btn"),
  planBtn: document.querySelector("#plan-btn"),
  selectionSummary: document.querySelector("#selection-summary"),
  folderPath: document.querySelector("#folder-path"),
  scanState: document.querySelector("#scan-state"),
  emptyState: document.querySelector("#empty-state"),
  resultsState: document.querySelector("#results-state"),
  packageList: document.querySelector("#package-list"),
  previewCard: document.querySelector("#preview-card"),
  searchInput: document.querySelector("#search-input"),
  statusFilter: document.querySelector("#status-filter"),
  selectAllBtn: document.querySelector("#select-all-btn"),
  selectNoneBtn: document.querySelector("#select-none-btn"),
  statPackages: document.querySelector("#stat-packages"),
  statClassified: document.querySelector("#stat-classified"),
  statMixed: document.querySelector("#stat-mixed"),
  statReview: document.querySelector("#stat-review"),
  statInvalid: document.querySelector("#stat-invalid"),
  planModal: document.querySelector("#plan-modal"),
  planCloseBtn: document.querySelector("#plan-close-btn"),
  planCloseFooterBtn: document.querySelector("#plan-close-footer-btn"),
  planExecuteBtn: document.querySelector("#plan-execute-btn"),
  planItems: document.querySelector("#plan-items"),
  planDirectories: document.querySelector("#plan-directories"),
  manifestPreviewText: document.querySelector("#manifest-preview-text"),
  planStatSelected: document.querySelector("#plan-stat-selected"),
  planStatReady: document.querySelector("#plan-stat-ready"),
  planStatCollisions: document.querySelector("#plan-stat-collisions"),
  planStatBlocked: document.querySelector("#plan-stat-blocked"),
  planStatFolders: document.querySelector("#plan-stat-folders"),
  chooseManifestBtn: document.querySelector("#choose-manifest-btn"),
  previewRestoreBtn: document.querySelector("#preview-restore-btn"),
  executeRestoreBtn: document.querySelector("#execute-restore-btn"),
  restoreManifestPath: document.querySelector("#restore-manifest-path"),
  restoreState: document.querySelector("#restore-state"),
  restoreEmpty: document.querySelector("#restore-empty"),
  restoreResults: document.querySelector("#restore-results"),
  restoreItems: document.querySelector("#restore-items"),
  restoreStatTracked: document.querySelector("#restore-stat-tracked"),
  restoreStatReady: document.querySelector("#restore-stat-ready"),
  restoreStatNew: document.querySelector("#restore-stat-new"),
  restoreStatCollisions: document.querySelector("#restore-stat-collisions"),
  restoreStatBlocked: document.querySelector("#restore-stat-blocked"),
  confirmModal: document.querySelector("#confirm-modal"),
  confirmTitle: document.querySelector("#confirm-title"),
  confirmMessage: document.querySelector("#confirm-message"),
  confirmCancelBtn: document.querySelector("#confirm-cancel-btn"),
  confirmActionBtn: document.querySelector("#confirm-action-btn"),
};

function t(key) {
  return I18N[state.language]?.[key] ?? I18N.en[key] ?? key;
}

function eligibleForPlan(item) {
  return item?.status === "classified" && !!item.destinationPath;
}

function statusLabel(status) {
  return {
    classified: t("classified"),
    mixed: t("mixed"),
    needs_review: t("needsReview"),
    unknown: t("unknown"),
    invalid: t("invalid"),
  }[status] || status;
}

function planStatusLabel(status) {
  return {
    ready: t("readyToMove"),
    already_organized: t("alreadyOrganized"),
    collision_same_content: t("collisionSame"),
    collision_different_content: t("collisionDifferent"),
    blocked: t("blocked"),
  }[status] || status;
}

function restoreStatusLabel(status) {
  return {
    ready_restore: t("readyToRestore"),
    already_restored: t("alreadyRestored"),
    ready_new: t("readyNew"),
    already_uncategorized: t("alreadyUncategorized"),
    collision_same_content: t("collisionSame"),
    collision_different_content: t("collisionDifferent"),
    changed: t("changed"),
    missing: t("missing"),
    ambiguous: t("ambiguous"),
  }[status] || status;
}

function renderLanguage() {
  document.documentElement.lang = state.language;
  for (const element of document.querySelectorAll("[data-i18n]")) {
    const key = element.dataset.i18n;
    element.textContent = t(key);
  }
  if (el.languageCode) el.languageCode.textContent = state.language.toUpperCase();
  el.languageButton?.setAttribute(
    "aria-expanded",
    String(!el.languageMenu?.classList.contains("hidden"))
  );
  if (el.searchInput) el.searchInput.placeholder = t("search");
  renderStatusFilter();
  renderSelectionSummary();
}

function renderStatusFilter() {
  const options = [
    ["all", t("allStatuses")],
    ["classified", t("classified")],
    ["mixed", t("mixed")],
    ["needs_review", t("needsReview")],
    ["unknown", t("unknown")],
    ["invalid", t("invalid")],
  ];
  const current = state.status;
  el.statusFilter.innerHTML = "";
  for (const [value, label] of options) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = label;
    option.selected = value === current;
    el.statusFilter.appendChild(option);
  }
}

function renderTabs() {
  for (const button of el.tabs) {
    button.classList.toggle("active", button.dataset.tab === state.tab);
  }
  for (const page of el.pages) {
    page.classList.toggle("hidden", page.id !== `page-${state.tab}`);
  }
}

function renderStats() {
  const stats = state.stats || {};
  el.statPackages.textContent = stats.packages ?? 0;
  el.statClassified.textContent = stats.classified ?? 0;
  el.statMixed.textContent = stats.mixed ?? 0;
  el.statReview.textContent = stats.needsReview ?? 0;
  el.statInvalid.textContent = stats.invalid ?? 0;
}

function visibleItems() {
  const query = state.search.trim().toLocaleLowerCase();
  return state.items.filter((item) => {
    if (state.status !== "all" && item.status !== state.status) return false;
    if (!query) return true;

    const haystack = [
      item.name,
      item.relativePath,
      item.category,
      item.subCategory,
      item.gender,
      item.age,
      item.destinationPath,
      ...(item.resourceTypes || []),
      ...(item.usageCategories || []),
      ...(item.candidateDestinations || []),
    ]
      .filter(Boolean)
      .join(" ")
      .toLocaleLowerCase();

    return haystack.includes(query);
  });
}

function renderSelectionSummary() {
  const count = state.selectedForPlan.size;
  el.selectionSummary.textContent = `${count} ${t("selectedEligible")}`;
  el.planBtn.disabled =
    count === 0 || state.scanning || state.planning || state.executing;
}

function appendMeta(container, label, value) {
  if (value == null || value === "" || (Array.isArray(value) && !value.length)) return;

  const row = document.createElement("div");
  row.className = "meta-row";
  const dt = document.createElement("span");
  dt.className = "meta-label";
  dt.textContent = label;
  const dd = document.createElement("strong");
  dd.textContent = Array.isArray(value) ? value.join(", ") : String(value);
  row.append(dt, dd);
  container.appendChild(row);
}

function renderPreview() {
  const item = state.items.find((candidate) => candidate.id === state.selectedId);
  el.previewCard.innerHTML = "";

  if (!item) {
    const empty = document.createElement("div");
    empty.className = "preview-empty";
    empty.textContent = t("selectPackage");
    el.previewCard.appendChild(empty);
    return;
  }

  const header = document.createElement("div");
  header.className = "preview-header";
  const name = document.createElement("h3");
  name.textContent = item.name;
  const badge = document.createElement("span");
  badge.className = `status-badge status-${item.status}`;
  badge.textContent = statusLabel(item.status);
  header.append(name, badge);

  const meta = document.createElement("div");
  meta.className = "preview-meta";
  appendMeta(meta, t("detectedFrom"), item.detectedFrom);
  appendMeta(meta, t("category"), item.category);
  appendMeta(meta, t("subCategory"), item.subCategory);
  appendMeta(meta, t("gender"), item.gender);
  appendMeta(meta, t("age"), item.age);
  appendMeta(meta, t("species"), item.species);
  appendMeta(meta, t("usageCategories"), item.usageCategories);
  appendMeta(meta, t("resources"), `${item.resourceCount} · ${(item.resourceTypes || []).join(", ")}`);
  appendMeta(meta, t("originalPath"), item.relativePath);
  appendMeta(meta, t("suggestedDestination"), item.destinationPath || t("noDestination"));

  if (!item.destinationPath && item.candidateDestinations?.length) {
    appendMeta(meta, t("possibleDestinations"), item.candidateDestinations);
  }
  if (!eligibleForPlan(item)) {
    appendMeta(meta, t("organizationPlan"), t("notEligible"));
  }

  el.previewCard.append(header, meta);

  if (item.warnings?.length) {
    const warnings = document.createElement("div");
    warnings.className = "warning-box";
    const title = document.createElement("strong");
    title.textContent = t("warnings");
    const list = document.createElement("ul");
    for (const warning of item.warnings) {
      const li = document.createElement("li");
      li.textContent = warning;
      list.appendChild(li);
    }
    warnings.append(title, list);
    el.previewCard.appendChild(warnings);
  }
}

function setPlanSelection(id, checked) {
  if (checked) state.selectedForPlan.add(id);
  else state.selectedForPlan.delete(id);
  state.plan = null;
  state.planError = "";
  renderSelectionSummary();
}

function createPackageRow(item) {
  const row = document.createElement("div");
  row.className =
    "package-row" +
    (item.id === state.selectedId ? " active" : "") +
    (state.selectedForPlan.has(item.id) ? " plan-selected" : "");
  row.title = item.path;
  row.tabIndex = 0;
  row.setAttribute("role", "button");

  const selection = document.createElement("label");
  selection.className = "package-check";
  selection.title = eligibleForPlan(item) ? t("selected") : t("notEligible");

  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.checked = state.selectedForPlan.has(item.id);
  checkbox.disabled = !eligibleForPlan(item);
  checkbox.addEventListener("click", (event) => event.stopPropagation());
  checkbox.addEventListener("change", (event) => {
    event.stopPropagation();
    setPlanSelection(item.id, checkbox.checked);
    row.classList.toggle("plan-selected", checkbox.checked);
  });
  selection.appendChild(checkbox);

  const main = document.createElement("div");
  main.className = "package-main";
  const name = document.createElement("strong");
  name.textContent = item.name;
  const details = document.createElement("span");
  details.textContent =
    [item.category, item.subCategory, item.gender, item.age]
      .filter(Boolean)
      .join(" › ") || statusLabel(item.status);
  main.append(name, details);

  const side = document.createElement("div");
  side.className = "package-side";
  const source = document.createElement("span");
  source.textContent = (item.detectedFrom || []).join(" + ") || "—";
  const badge = document.createElement("span");
  badge.className = `status-dot status-${item.status}`;
  badge.setAttribute("aria-label", statusLabel(item.status));
  side.append(source, badge);

  row.append(selection, main, side);

  const choose = () => {
    state.selectedId = item.id;
    renderResults();
  };
  row.addEventListener("click", choose);
  row.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      choose();
    }
  });

  return row;
}

function renderResults() {
  const hasScan = !!state.stats;
  el.emptyState.classList.toggle("hidden", hasScan);
  el.resultsState.classList.toggle("hidden", !hasScan);

  if (!hasScan) {
    renderPreview();
    renderSelectionSummary();
    return;
  }

  const items = visibleItems();
  el.packageList.innerHTML = "";

  if (!items.length) {
    const empty = document.createElement("div");
    empty.className = "list-empty";
    empty.textContent = t("noResults");
    el.packageList.appendChild(empty);
  } else {
    for (const item of items) el.packageList.appendChild(createPackageRow(item));
  }

  if (!items.some((item) => item.id === state.selectedId)) {
    state.selectedId = items[0]?.id || "";
  }

  renderPreview();
  renderSelectionSummary();
}

function renderPlan() {
  const plan = state.plan;
  if (!plan) return;

  const stats = plan.stats || {};
  const collisions =
    (stats.collisionSameContent ?? 0) + (stats.collisionDifferentContent ?? 0);

  el.planStatSelected.textContent = stats.selected ?? 0;
  el.planStatReady.textContent = stats.ready ?? 0;
  el.planStatCollisions.textContent = collisions;
  el.planStatBlocked.textContent = stats.blocked ?? 0;
  el.planStatFolders.textContent = stats.directoriesToCreate ?? 0;
  el.planExecuteBtn.disabled = !plan.canExecute || state.executing;

  el.planItems.innerHTML = "";
  for (const item of plan.items || []) {
    const card = document.createElement("article");
    card.className = `plan-item plan-${item.planStatus}`;

    const top = document.createElement("div");
    top.className = "plan-item-top";
    const name = document.createElement("strong");
    name.textContent = item.name;
    const badge = document.createElement("span");
    badge.className = `plan-status plan-status-${item.planStatus}`;
    badge.textContent = planStatusLabel(item.planStatus);
    top.append(name, badge);

    const paths = document.createElement("div");
    paths.className = "plan-paths";
    for (const [label, path] of [
      [t("current"), item.sourceRelativePath],
      [t("proposed"), item.destinationRelativePath || t("noDestination")],
    ]) {
      const line = document.createElement("div");
      const span = document.createElement("span");
      span.textContent = label;
      const code = document.createElement("code");
      code.textContent = path;
      line.append(span, code);
      paths.appendChild(line);
    }

    card.append(top, paths);

    if (item.warnings?.length) {
      const note = document.createElement("div");
      note.className = "plan-warning";
      note.textContent = item.warnings.join(" ");
      card.appendChild(note);
    }
    el.planItems.appendChild(card);
  }

  el.planDirectories.innerHTML = "";
  if (!plan.directoriesToCreate?.length) {
    const empty = document.createElement("div");
    empty.className = "planner-empty";
    empty.textContent = t("noFoldersNeeded");
    el.planDirectories.appendChild(empty);
  } else {
    for (const path of plan.directoriesToCreate) {
      const line = document.createElement("code");
      line.textContent = path;
      el.planDirectories.appendChild(line);
    }
  }

  el.manifestPreviewText.textContent = plan.manifestPreview || "";
}

function renderRestore() {
  const plan = state.restorePlan;
  el.restoreManifestPath.textContent = state.restoreManifest || t("noManifest");
  el.restoreManifestPath.title = state.restoreManifest;
  el.previewRestoreBtn.disabled = !state.restoreManifest || state.restoreBusy;
  el.executeRestoreBtn.disabled = !plan?.canExecute || state.restoreBusy;

  const stats = plan?.stats || {};
  el.restoreStatTracked.textContent = stats.tracked ?? 0;
  el.restoreStatReady.textContent = stats.readyRestore ?? 0;
  el.restoreStatNew.textContent = stats.newFiles ?? 0;
  el.restoreStatCollisions.textContent = stats.collisions ?? 0;
  el.restoreStatBlocked.textContent = stats.blocked ?? 0;

  if (state.restoreBusy) {
    el.restoreState.textContent = state.pendingAction === "restore"
      ? t("restoring")
      : t("restorePreviewing");
    el.restoreState.className = "scan-state busy";
  } else if (state.restoreError) {
    el.restoreState.textContent = `${t("restoreFailed")}: ${state.restoreError}`;
    el.restoreState.className = "scan-state error";
  } else if (state.restoreNotice) {
    el.restoreState.textContent = state.restoreNotice;
    el.restoreState.className = "scan-state success";
  } else if (plan) {
    el.restoreState.textContent = t("restoreReady");
    el.restoreState.className = "scan-state success";
  } else {
    el.restoreState.textContent = "";
    el.restoreState.className = "scan-state";
  }

  el.restoreEmpty.classList.toggle("hidden", !!plan);
  el.restoreResults.classList.toggle("hidden", !plan);
  el.restoreItems.innerHTML = "";

  if (!plan) return;

  for (const item of plan.items || []) {
    const card = document.createElement("article");
    card.className = `plan-item restore-${item.status}`;

    const top = document.createElement("div");
    top.className = "plan-item-top";
    const name = document.createElement("strong");
    name.textContent =
      item.sourceRelativePath ||
      item.destinationRelativePath ||
      item.sha256.slice(0, 16);
    const badge = document.createElement("span");
    badge.className = `plan-status restore-status-${item.status}`;
    badge.textContent = restoreStatusLabel(item.status);
    top.append(name, badge);

    const paths = document.createElement("div");
    paths.className = "plan-paths";
    if (item.sourceRelativePath) {
      const line = document.createElement("div");
      const label = document.createElement("span");
      label.textContent = t("current");
      const code = document.createElement("code");
      code.textContent = item.sourceRelativePath;
      line.append(label, code);
      paths.appendChild(line);
    }
    if (item.destinationRelativePath) {
      const line = document.createElement("div");
      const label = document.createElement("span");
      label.textContent = t("proposed");
      const code = document.createElement("code");
      code.textContent = item.destinationRelativePath;
      line.append(label, code);
      paths.appendChild(line);
    }

    card.append(top, paths);

    if (item.warnings?.length) {
      const note = document.createElement("div");
      note.className = "plan-warning";
      note.textContent = item.warnings.join(" ");
      card.appendChild(note);
    }

    el.restoreItems.appendChild(card);
  }
}

function openPlanModal() {
  renderPlan();
  el.planModal.classList.remove("hidden");
  el.planModal.setAttribute("aria-hidden", "false");
}

function closePlanModal() {
  el.planModal.classList.add("hidden");
  el.planModal.setAttribute("aria-hidden", "true");
}

function openConfirm(action) {
  state.pendingAction = action;
  if (action === "organize") {
    el.confirmTitle.textContent = t("confirmOrganizeTitle");
    el.confirmMessage.textContent = t("confirmOrganizeMessage");
    el.confirmActionBtn.textContent = t("organizeSelected");
  } else {
    el.confirmTitle.textContent = t("confirmRestoreTitle");
    el.confirmMessage.textContent = t("confirmRestoreMessage");
    el.confirmActionBtn.textContent = t("executeRestore");
  }
  el.confirmModal.classList.remove("hidden");
  el.confirmModal.setAttribute("aria-hidden", "false");
}

function closeConfirm() {
  if (state.executing || state.restoreBusy) return;
  state.pendingAction = "";
  el.confirmModal.classList.add("hidden");
  el.confirmModal.setAttribute("aria-hidden", "true");
}

function render() {
  renderLanguage();
  renderTabs();
  renderStats();
  renderRestore();

  el.folderPath.textContent = state.folder || t("noFolder");
  el.folderPath.title = state.folder;
  el.scanBtn.disabled =
    !state.folder || state.scanning || state.planning || state.executing;
  el.chooseFolderBtn.disabled =
    state.scanning || state.planning || state.executing;

  if (state.executing) {
    el.scanState.textContent = t("executing");
    el.scanState.className = "scan-state busy";
  } else if (state.planError) {
    el.scanState.textContent = `${t("planFailed")}: ${state.planError}`;
    el.scanState.className = "scan-state error";
  } else if (state.scanning) {
    el.scanState.textContent = t("scanning");
    el.scanState.className = "scan-state busy";
  } else if (state.error) {
    el.scanState.textContent = `${t("scanFailed")}: ${state.error}`;
    el.scanState.className = "scan-state error";
  } else if (state.notice) {
    el.scanState.textContent = state.notice;
    el.scanState.className = "scan-state success";
  } else if (state.stats) {
    el.scanState.textContent = t("scanComplete");
    el.scanState.className = "scan-state success";
  } else {
    el.scanState.textContent = "";
    el.scanState.className = "scan-state";
  }

  renderResults();
  if (state.plan && !el.planModal.classList.contains("hidden")) renderPlan();
}

async function chooseFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: t("chooseModsFolder"),
  });
  if (!selected || Array.isArray(selected)) return;

  state.folder = selected;
  state.items = [];
  state.stats = null;
  state.selectedId = "";
  state.selectedForPlan.clear();
  state.error = "";
  state.planError = "";
  state.notice = "";
  state.plan = null;
  closePlanModal();
  render();
}

async function scanFolder(preserveSelection = false, preserveNotice = false) {
  if (!state.folder || state.scanning || state.planning || state.executing) return;

  const previousSelection = new Set(state.selectedForPlan);
  const previousNotice = state.notice;

  state.scanning = true;
  state.error = "";
  state.planError = "";
  state.plan = null;
  if (!preserveNotice) state.notice = "";
  closePlanModal();
  render();

  try {
    const result = await invoke("scan_packages", {
      folder: state.folder,
      language: state.language,
    });

    state.items = result.items || [];
    state.stats = result.stats || null;

    const eligibleIds = new Set(
      state.items.filter(eligibleForPlan).map((item) => item.id)
    );

    state.selectedForPlan.clear();
    if (preserveSelection) {
      for (const id of previousSelection) {
        if (eligibleIds.has(id)) state.selectedForPlan.add(id);
      }
    } else {
      for (const id of eligibleIds) state.selectedForPlan.add(id);
    }

    state.selectedId = visibleItems()[0]?.id || "";
    if (preserveNotice) state.notice = previousNotice;
  } catch (error) {
    state.error = String(error);
    state.items = [];
    state.stats = null;
    state.selectedForPlan.clear();
  } finally {
    state.scanning = false;
    render();
  }
}

async function buildPlan() {
  if (!state.folder || !state.selectedForPlan.size || state.planning) return;

  state.planning = true;
  state.planError = "";
  state.notice = "";
  state.plan = null;
  render();

  try {
    state.plan = await invoke("build_organization_plan", {
      folder: state.folder,
      language: state.language,
      selectedPaths: [...state.selectedForPlan],
    });
  } catch (error) {
    state.planError = String(error);
  } finally {
    state.planning = false;
    render();
    if (state.plan) openPlanModal();
  }
}

async function executeOrganization() {
  if (!state.plan?.canExecute || state.executing) return;

  state.executing = true;
  state.planError = "";
  el.confirmActionBtn.disabled = true;
  render();

  try {
    const result = await invoke("execute_organization", {
      folder: state.folder,
      language: state.language,
      selectedPaths: [...state.selectedForPlan],
    });

    if (result.status === "COMPLETE") {
      state.notice = `${t("executionComplete")}: ${result.moved}`;
    } else if (result.status === "NO_CHANGES") {
      state.notice = t("executionNoChanges");
    } else {
      state.planError = `${t("executionRolledBack")}: ${(result.errors || []).join(" ")}`;
    }

    state.pendingAction = "";
    el.confirmModal.classList.add("hidden");
    closePlanModal();
  } catch (error) {
    state.planError = String(error);
  } finally {
    state.executing = false;
    el.confirmActionBtn.disabled = false;
    render();
  }

  if (!state.planError) {
    await scanFolder(false, true);
  }
}

async function chooseManifest() {
  const selected = await open({
    multiple: false,
    directory: false,
    title: t("chooseManifest"),
    filters: [{ name: "S3CC Organizer", extensions: ["txt"] }],
  });
  if (!selected || Array.isArray(selected)) return;

  state.restoreManifest = selected;
  state.restorePlan = null;
  state.restoreError = "";
  state.restoreNotice = "";
  render();
}

async function previewRestore() {
  if (!state.restoreManifest || state.restoreBusy) return;

  state.restoreBusy = true;
  state.restoreError = "";
  state.restoreNotice = "";
  state.restorePlan = null;
  render();

  try {
    state.restorePlan = await invoke("preview_restore", {
      manifestPath: state.restoreManifest,
      currentLanguage: state.language,
    });
  } catch (error) {
    state.restoreError = String(error);
  } finally {
    state.restoreBusy = false;
    render();
  }
}

async function executeRestore() {
  if (!state.restorePlan?.canExecute || state.restoreBusy) return;

  state.restoreBusy = true;
  state.restoreError = "";
  el.confirmActionBtn.disabled = true;
  render();

  try {
    const result = await invoke("execute_restore", {
      manifestPath: state.restoreManifest,
      currentLanguage: state.language,
    });

    if (result.status === "RESTORED") {
      state.restoreNotice =
        `${t("restoreComplete")}: ${result.restored} + ${result.newFilesRelocated} ${t("newFiles")}`;
    } else {
      state.restoreError =
        `${t("executionRolledBack")}: ${(result.errors || []).join(" ")}`;
    }

    state.pendingAction = "";
    el.confirmModal.classList.add("hidden");
  } catch (error) {
    state.restoreError = String(error);
  } finally {
    state.restoreBusy = false;
    el.confirmActionBtn.disabled = false;
    render();
  }

  if (!state.restoreError) {
    await previewRestore();
    if (state.restorePlan) state.restoreNotice = t("restoreComplete");
    render();
  }
}

function selectAllVisible() {
  for (const item of visibleItems()) {
    if (eligibleForPlan(item)) state.selectedForPlan.add(item.id);
  }
  state.plan = null;
  state.planError = "";
  renderResults();
}

function selectNoneVisible() {
  for (const item of visibleItems()) {
    state.selectedForPlan.delete(item.id);
  }
  state.plan = null;
  state.planError = "";
  renderResults();
}

for (const button of el.tabs) {
  button.addEventListener("click", () => {
    state.tab = button.dataset.tab || "organizer";
    render();
  });
}

el.languageButton.addEventListener("click", (event) => {
  event.stopPropagation();
  el.languageMenu?.classList.toggle("hidden");
  renderLanguage();
});

for (const button of el.languageMenuItems) {
  button.addEventListener("click", async (event) => {
    event.stopPropagation();
    const nextLanguage = button.dataset.lang;
    if (!LANGUAGE_ORDER.includes(nextLanguage)) return;

    el.languageMenu?.classList.add("hidden");
    if (nextLanguage === state.language) {
      renderLanguage();
      return;
    }

    state.language = nextLanguage;
    localStorage.setItem("s3cc-organizer-language", state.language);
    render();

    if (state.folder && state.stats) await scanFolder(true);
    if (state.restoreManifest && state.restorePlan) await previewRestore();
  });
}

document.addEventListener("click", (event) => {
  if (!event.target.closest("#lang-dropdown")) {
    el.languageMenu?.classList.add("hidden");
    renderLanguage();
  }
});

document.addEventListener("keydown", (event) => {
  if (event.key !== "Escape") return;
  if (!el.confirmModal.classList.contains("hidden")) closeConfirm();
  else if (!el.planModal.classList.contains("hidden")) closePlanModal();
});

el.chooseFolderBtn.addEventListener("click", chooseFolder);
el.scanBtn.addEventListener("click", () => scanFolder(false));
el.planBtn.addEventListener("click", buildPlan);
el.selectAllBtn.addEventListener("click", selectAllVisible);
el.selectNoneBtn.addEventListener("click", selectNoneVisible);
el.planCloseBtn.addEventListener("click", closePlanModal);
el.planCloseFooterBtn.addEventListener("click", closePlanModal);
el.planExecuteBtn.addEventListener("click", () => openConfirm("organize"));

el.planModal.addEventListener("click", (event) => {
  if (event.target === el.planModal) closePlanModal();
});

el.chooseManifestBtn.addEventListener("click", chooseManifest);
el.previewRestoreBtn.addEventListener("click", previewRestore);
el.executeRestoreBtn.addEventListener("click", () => openConfirm("restore"));

el.confirmCancelBtn.addEventListener("click", closeConfirm);
el.confirmActionBtn.addEventListener("click", async () => {
  if (state.pendingAction === "organize") await executeOrganization();
  else if (state.pendingAction === "restore") await executeRestore();
});

el.confirmModal.addEventListener("click", (event) => {
  if (event.target === el.confirmModal) closeConfirm();
});

el.searchInput.addEventListener("input", (event) => {
  state.search = event.currentTarget.value;
  renderResults();
});

el.statusFilter.addEventListener("change", (event) => {
  state.status = event.currentTarget.value;
  renderResults();
});

render();
