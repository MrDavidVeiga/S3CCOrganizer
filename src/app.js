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
    safetyMessage: "Packages are classified from their actual resources. Nothing is moved during this scan.",
    readOnlyStage: "This stage only analyzes files. Nothing is moved.",
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
    duplicatesNext: "Exact, content and related-variant detection will be connected after the catalog scanner is validated.",
    conflictsNext: "Resource-level conflict analysis will distinguish shared identical resources from real overrides.",
    restoreNext: "Restore manifests will be enabled only after the organizer move engine is validated.",
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
    safetyMessage: "Os packages são classificados pelos resources reais. Nada é movido durante esta análise.",
    readOnlyStage: "Esta etapa apenas analisa os arquivos. Nada é movido.",
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
    duplicatesNext: "A detecção de cópias exatas, duplicados por conteúdo e variantes será conectada depois da validação do scanner de catálogo.",
    conflictsNext: "A análise de conflitos por resource distinguirá resources idênticos compartilhados de overrides reais.",
    restoreNext: "A restauração por manifesto será ativada depois da validação do motor de organização.",
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
    safetyMessage: "Los packages se clasifican por sus resources reales. Nada se mueve durante este análisis.",
    readOnlyStage: "Esta etapa solo analiza los archivos. Nada se mueve.",
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
    duplicatesNext: "La detección de copias exactas, duplicados por contenido y variantes se conectará después de validar el escáner de catálogo.",
    conflictsNext: "El análisis de conflictos por resource distinguirá resources idénticos compartidos de overrides reales.",
    restoreNext: "La restauración mediante manifiesto se activará después de validar el motor de organización.",
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
  search: "",
  status: "all",
  scanning: false,
  error: "",
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
  folderPath: document.querySelector("#folder-path"),
  scanState: document.querySelector("#scan-state"),
  emptyState: document.querySelector("#empty-state"),
  resultsState: document.querySelector("#results-state"),
  packageList: document.querySelector("#package-list"),
  previewCard: document.querySelector("#preview-card"),
  searchInput: document.querySelector("#search-input"),
  statusFilter: document.querySelector("#status-filter"),
  statPackages: document.querySelector("#stat-packages"),
  statClassified: document.querySelector("#stat-classified"),
  statMixed: document.querySelector("#stat-mixed"),
  statReview: document.querySelector("#stat-review"),
  statInvalid: document.querySelector("#stat-invalid"),
};

function t(key) {
  return I18N[state.language]?.[key] ?? I18N.en[key] ?? key;
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
  el.searchInput.placeholder = t("search");
  renderStatusFilter();
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

function statusLabel(status) {
  return {
    classified: t("classified"),
    mixed: t("mixed"),
    needs_review: t("needsReview"),
    unknown: t("unknown"),
    invalid: t("invalid"),
  }[status] || status;
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
    ]
      .filter(Boolean)
      .join(" ")
      .toLocaleLowerCase();
    return haystack.includes(query);
  });
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

function renderResults() {
  const hasScan = !!state.stats;
  el.emptyState.classList.toggle("hidden", hasScan);
  el.resultsState.classList.toggle("hidden", !hasScan);

  if (!hasScan) {
    renderPreview();
    return;
  }

  const items = visibleItems();
  el.packageList.innerHTML = "";

  if (!items.length) {
    const empty = document.createElement("div");
    empty.className = "list-empty";
    empty.textContent = t("noResults");
    el.packageList.appendChild(empty);
  }

  for (const item of items) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "package-row" + (item.id === state.selectedId ? " active" : "");
    button.title = item.path;

    const main = document.createElement("div");
    main.className = "package-main";
    const name = document.createElement("strong");
    name.textContent = item.name;
    const details = document.createElement("span");
    const classification = [item.category, item.subCategory, item.gender, item.age]
      .filter(Boolean)
      .join(" › ");
    details.textContent = classification || statusLabel(item.status);
    main.append(name, details);

    const side = document.createElement("div");
    side.className = "package-side";
    const source = document.createElement("span");
    source.textContent = (item.detectedFrom || []).join(" + ") || "—";
    const badge = document.createElement("span");
    badge.className = `status-dot status-${item.status}`;
    badge.setAttribute("aria-label", statusLabel(item.status));
    side.append(source, badge);

    button.append(main, side);
    button.addEventListener("click", () => {
      state.selectedId = item.id;
      renderResults();
      renderPreview();
    });
    el.packageList.appendChild(button);
  }

  if (!items.some((item) => item.id === state.selectedId)) {
    state.selectedId = items[0]?.id || "";
  }
  renderPreview();
}

function render() {
  renderLanguage();
  renderTabs();
  renderStats();

  el.folderPath.textContent = state.folder || t("noFolder");
  el.folderPath.title = state.folder;
  el.scanBtn.disabled = !state.folder || state.scanning;
  el.chooseFolderBtn.disabled = state.scanning;

  if (state.scanning) {
    el.scanState.textContent = t("scanning");
    el.scanState.className = "scan-state busy";
  } else if (state.error) {
    el.scanState.textContent = `${t("scanFailed")}: ${state.error}`;
    el.scanState.className = "scan-state error";
  } else if (state.stats) {
    el.scanState.textContent = t("scanComplete");
    el.scanState.className = "scan-state success";
  } else {
    el.scanState.textContent = "";
    el.scanState.className = "scan-state";
  }

  renderResults();
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
  state.error = "";
  render();
}

async function scanFolder() {
  if (!state.folder || state.scanning) return;
  state.scanning = true;
  state.error = "";
  render();

  try {
    const result = await invoke("scan_packages", {
      folder: state.folder,
      language: state.language,
    });
    state.items = result.items || [];
    state.stats = result.stats || null;
    const visible = visibleItems();
    state.selectedId = visible[0]?.id || "";
  } catch (error) {
    state.error = String(error);
  } finally {
    state.scanning = false;
    render();
  }
}

for (const button of el.tabs) {
  button.addEventListener("click", () => {
    state.tab = button.dataset.tab || "organizer";
    renderTabs();
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

    // Re-run classification so user-facing categories and destination folders
    // immediately match the current interface language.
    if (state.folder && state.stats) {
      await scanFolder();
    }
  });
}

document.addEventListener("click", (event) => {
  if (!event.target.closest("#lang-dropdown")) {
    el.languageMenu?.classList.add("hidden");
    renderLanguage();
  }
});

el.chooseFolderBtn.addEventListener("click", chooseFolder);
el.scanBtn.addEventListener("click", scanFolder);
el.searchInput.addEventListener("input", (event) => {
  state.search = event.currentTarget.value;
  renderResults();
});
el.statusFilter.addEventListener("change", (event) => {
  state.status = event.currentTarget.value;
  renderResults();
});

render();
