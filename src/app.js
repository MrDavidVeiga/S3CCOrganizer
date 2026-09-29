import "@fortawesome/fontawesome-free/css/all.min.css";

const I18N = {
  en: {
    organizer: "Organizer",
    duplicates: "Duplicates",
    conflicts: "Conflicts",
    language: "Language",
    chooseModsFolder: "Choose Mods Folder",
    noFolder: "No folder selected.",
    scanCcs: "Scan CCs",
    analyzeBefore: "Analyze before organizing",
    safetyMessage: "Packages will be classified from their actual resources. No file is deleted automatically.",
    packages: "Packages",
    needsReview: "Needs Review",
    chooseToBegin: "Choose a Mods folder to begin.",
  },
  pt: {
    organizer: "Organizador",
    duplicates: "Duplicados",
    conflicts: "Conflitos",
    language: "Idioma",
    chooseModsFolder: "Escolher Pasta de Mods",
    noFolder: "Nenhuma pasta selecionada.",
    scanCcs: "Analisar CCs",
    analyzeBefore: "Analise antes de organizar",
    safetyMessage: "Os packages serão classificados pelos recursos reais. Nenhum arquivo será excluído automaticamente.",
    packages: "Packages",
    needsReview: "Requer Revisão",
    chooseToBegin: "Escolha uma pasta de Mods para começar.",
  },
  es: {
    organizer: "Organizador",
    duplicates: "Duplicados",
    conflicts: "Conflictos",
    language: "Idioma",
    chooseModsFolder: "Elegir Carpeta de Mods",
    noFolder: "Ninguna carpeta seleccionada.",
    scanCcs: "Analizar CCs",
    analyzeBefore: "Analiza antes de organizar",
    safetyMessage: "Los packages se clasificarán según sus recursos reales. Ningún archivo se eliminará automáticamente.",
    packages: "Packages",
    needsReview: "Requiere Revisión",
    chooseToBegin: "Elige una carpeta de Mods para comenzar.",
  },
};

const LANGUAGE_ORDER = ["en", "pt", "es"];
let language = localStorage.getItem("s3cc-organizer-language") || "en";
if (!LANGUAGE_ORDER.includes(language)) language = "en";

const tabs = [...document.querySelectorAll(".tabs button")];
const languageButton = document.querySelector("#language-button");

function renderLanguage() {
  document.documentElement.lang = language;
  for (const element of document.querySelectorAll("[data-i18n]")) {
    const key = element.dataset.i18n;
    element.textContent = I18N[language]?.[key] ?? I18N.en[key] ?? key;
  }
  if (languageButton) languageButton.textContent = `${language.toUpperCase()} ▾`;
}

for (const tab of tabs) {
  tab.addEventListener("click", () => {
    for (const button of tabs) button.classList.remove("active");
    tab.classList.add("active");
  });
}

languageButton?.addEventListener("click", () => {
  const current = LANGUAGE_ORDER.indexOf(language);
  language = LANGUAGE_ORDER[(current + 1) % LANGUAGE_ORDER.length];
  localStorage.setItem("s3cc-organizer-language", language);
  renderLanguage();
});

renderLanguage();

export function currentLanguage() {
  return language;
}
