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
    cancelAnalysis: "Cancel",
    cancelling: "Cancelling…",
    cancelled: "Analysis cancelled",
    technicalDetails: "Technical Details",
    technicalDetailsLoading: "Reading package resources…",
    technicalDetailsError: "Could not load technical details",
    showingResourceLimit: "Showing the first 500 resources",
    fileSha256: "File SHA-256",
    dbpfVersion: "DBPF version",
    compression: "Compression",
    diskSize: "Disk size",
    memorySize: "Memory size",
    payloadHash: "Payload SHA-256",
    resourceTechnicalDetails: "Resource details",
    cacheReused: "Cache reused",
    cacheUpdated: "Cache updated",
    restoreHistory: "Restore History",
    restoreHistoryHint: "Manifests previously created for the selected Mods folder.",
    refreshHistory: "Refresh",
    noRestoreHistory: "No restore manifests were found for this Mods folder.",
    invalidManifest: "Invalid manifest",
    manifestFiles: "files",
    differentRoot: "Different Mods root",
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
    duplicatesIntro: "Find true duplicates and distinguish them from related CC variants without deleting anything.",
    analyzeDuplicates: "Analyze Duplicates",
    analyzingDuplicates: "Analyzing package fingerprints…",
    duplicatesReady: "Duplicate analysis complete",
    duplicatesFailed: "Duplicate analysis failed",
    duplicateSearch: "Search duplicate findings…",
    allDuplicateTypes: "All findings",
    exactDuplicate: "Exact Duplicate",
    contentDuplicate: "Content Duplicate",
    retexture: "Retexture",
    recategorizedVariant: "Recategorized Variant",
    relatedVariant: "Related Variant",
    exactGroups: "Exact Groups",
    contentGroups: "Content Groups",
    retextures: "Retextures",
    relatedVariants: "Related Variants",
    analyzeDuplicatesToBegin: "Analyze the selected Mods folder to find duplicate relationships.",
    selectDuplicateFinding: "Select a finding to see why it was detected.",
    duplicateMembers: "Files in this group",
    duplicateReason: "Why it was flagged",
    fileHash: "File SHA-256",
    normalizedFingerprint: "Normalized fingerprint",
    resourceCount: "Resource count",
    sharedResources: "Shared resources",
    identicalTgiPayloads: "Same TGI + same payload",
    changedSameTgi: "Same TGI + different payload",
    sharedStructural: "Shared structural resources",
    changedTextures: "Changed textures/materials",
    changedCatalog: "Changed catalog resources",
    evidence: "Evidence",
    exactExplanation: "The entire .package file has the same SHA-256. These files are byte-for-byte identical.",
    contentExplanation: "The package containers differ, but the normalized set of TGI keys and decompressed resource payload hashes is identical.",
    retextureExplanation: "Structural resources match while texture/material content differs. This is a related visual variant, not a duplicate to delete.",
    recategorizedExplanation: "Substantive non-catalog content matches while catalog resources differ. This is a recategorized variant, not a duplicate to delete.",
    relatedExplanation: "Structural resources match, but the remaining package content differs. It is related content and requires user review.",
    unreadablePackages: "Unreadable packages",
    variantAnalysisTruncated: "Variant relation list was limited for performance.",
    noDuplicateFindings: "No findings match the current search and filter.",
    duplicatesNext: "Duplicate analysis is implemented in read-only mode. No file is deleted or moved.",
    conflictsIntro: "Compare shared TGIs by decompressed payload and classify the impact instead of treating every overlap as a conflict.",
    analyzeConflicts: "Analyze Conflicts",
    analyzingConflicts: "Analyzing shared resources…",
    conflictsReady: "Conflict analysis complete",
    conflictsFailed: "Conflict analysis failed",
    conflictSearch: "Search conflict findings…",
    allConflictTypes: "All findings",
    sharedIdentical: "Shared Identical",
    visualOverride: "Visual Override",
    catalogOverride: "Catalog Override",
    gameplayOverride: "Gameplay Override",
    scriptConflict: "Script Conflict",
    textOverride: "Text Override",
    potentialConflict: "Potential Conflict",
    mixedOverride: "Mixed Override",
    packagePairs: "Package Pairs",
    realOverrides: "Overrides",
    scriptConflicts: "Script",
    potentialConflicts: "Potential",
    analyzeConflictsToBegin: "Analyze the selected Mods folder to inspect shared resources.",
    selectConflictFinding: "Select a finding to inspect its resource evidence.",
    conflictReason: "Why it was classified",
    conflictSamePayloadExplanation: "These packages share one or more TGIs with byte-identical decompressed payloads. This is shared content, not a conflict.",
    conflictVisualExplanation: "The same visual resource TGI exists in both packages but its decompressed payload differs. This is a visual override relationship.",
    conflictCatalogExplanation: "The same catalog resource TGI exists in both packages but its payload differs. Catalog behavior or categorization may be overridden.",
    conflictGameplayExplanation: "The same gameplay tuning TGI exists in both packages but its payload differs. Load order may change gameplay behavior.",
    conflictScriptExplanation: "The same script assembly TGI exists in both packages with a different payload. This is a high-risk script collision.",
    conflictTextExplanation: "The same STBL TGI exists with different text payloads. This is a text/localization override.",
    conflictPotentialExplanation: "The same TGI exists with a different payload, but its resource family is not yet classified deeply enough for a stronger conclusion.",
    conflictMixedExplanation: "This package pair contains multiple classes of differing shared resources.",
    sharedCount: "Shared TGIs",
    samePayloadCount: "Identical payloads",
    differentPayloadCount: "Different payloads",
    resourceClass: "Resource class",
    payloadA: "Payload A",
    payloadB: "Payload B",
    impact: "Impact",
    noConflictFindings: "No findings match the current search and filter.",
    conflictAnalysisTruncated: "Conflict pair list was limited for performance.",
    resourceCfg: "Resource.cfg",
    resourceCfgWarnings: "Resource.cfg warnings",
    loadPriority: "Load priority",
    matchingRule: "Matching rule",
    loadOrder: "Load order",
    higherPriorityWins: "Higher Resource.cfg priority takes precedence for this pair.",
    samePriorityUnknown: "Both packages have the same Resource.cfg priority. The winner is not inferred.",
    partialPriorityUnknown: "Only one package matched a PackedFile rule. The winner is not inferred.",
    unmatchedPriorityUnknown: "Neither package matched a PackedFile rule. The winner is not inferred.",
    missingResourceCfg: "No Resource.cfg was found at the selected root or its parent.",
    advancedCfgUnknown: "This Resource.cfg uses advanced traversal or conditional directives. The Organizer will not infer a winner from Priority alone.",
    likelyHigherPriority: "Higher priority",
    conflictsNext: "Resource-level conflict analysis is implemented in read-only mode.",
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
    cancelAnalysis: "Cancelar",
    cancelling: "Cancelando…",
    cancelled: "Análise cancelada",
    technicalDetails: "Detalhes Técnicos",
    technicalDetailsLoading: "Lendo resources do package…",
    technicalDetailsError: "Não foi possível carregar os detalhes técnicos",
    showingResourceLimit: "Exibindo os primeiros 500 resources",
    fileSha256: "SHA-256 do arquivo",
    dbpfVersion: "Versão DBPF",
    compression: "Compressão",
    diskSize: "Tamanho no disco",
    memorySize: "Tamanho em memória",
    payloadHash: "SHA-256 do payload",
    resourceTechnicalDetails: "Detalhes dos resources",
    cacheReused: "Cache reutilizado",
    cacheUpdated: "Cache atualizado",
    restoreHistory: "Histórico de Restauração",
    restoreHistoryHint: "Manifestos criados anteriormente para a pasta de Mods selecionada.",
    refreshHistory: "Atualizar",
    noRestoreHistory: "Nenhum manifesto de restauração foi encontrado para esta pasta de Mods.",
    invalidManifest: "Manifesto inválido",
    manifestFiles: "arquivos",
    differentRoot: "Outra pasta de Mods",
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
    duplicatesIntro: "Encontre duplicados reais e diferencie-os de variantes relacionadas de CC sem apagar nada.",
    analyzeDuplicates: "Analisar Duplicados",
    analyzingDuplicates: "Analisando fingerprints dos packages…",
    duplicatesReady: "Análise de duplicados concluída",
    duplicatesFailed: "Falha na análise de duplicados",
    duplicateSearch: "Pesquisar resultados de duplicados…",
    allDuplicateTypes: "Todos os resultados",
    exactDuplicate: "Duplicado Exato",
    contentDuplicate: "Duplicado por Conteúdo",
    retexture: "Retexture",
    recategorizedVariant: "Variante Recategorizada",
    relatedVariant: "Variante Relacionada",
    exactGroups: "Grupos Exatos",
    contentGroups: "Grupos por Conteúdo",
    retextures: "Retextures",
    relatedVariants: "Variantes Relacionadas",
    analyzeDuplicatesToBegin: "Analise a pasta de Mods selecionada para encontrar relações de duplicidade.",
    selectDuplicateFinding: "Selecione um resultado para ver por que ele foi detectado.",
    duplicateMembers: "Arquivos deste grupo",
    duplicateReason: "Por que foi marcado",
    fileHash: "SHA-256 do arquivo",
    normalizedFingerprint: "Fingerprint normalizado",
    resourceCount: "Quantidade de resources",
    sharedResources: "Resources compartilhados",
    identicalTgiPayloads: "Mesmo TGI + mesmo conteúdo",
    changedSameTgi: "Mesmo TGI + conteúdo diferente",
    sharedStructural: "Resources estruturais compartilhados",
    changedTextures: "Texturas/materiais alterados",
    changedCatalog: "Resources de catálogo alterados",
    evidence: "Evidências",
    exactExplanation: "O arquivo .package inteiro possui o mesmo SHA-256. Esses arquivos são idênticos byte por byte.",
    contentExplanation: "Os containers dos packages diferem, mas o conjunto normalizado de TGIs e hashes dos resources descomprimidos é idêntico.",
    retextureExplanation: "Os resources estruturais coincidem, enquanto texturas ou materiais diferem. É uma variante visual relacionada, não um duplicado para apagar.",
    recategorizedExplanation: "O conteúdo substancial fora do catálogo coincide, enquanto os resources de catálogo diferem. É uma variante recategorizada, não um duplicado para apagar.",
    relatedExplanation: "Os resources estruturais coincidem, mas o restante do conteúdo do package difere. É conteúdo relacionado e requer revisão.",
    unreadablePackages: "Packages não legíveis",
    variantAnalysisTruncated: "A lista de relações entre variantes foi limitada por desempenho.",
    noDuplicateFindings: "Nenhum resultado corresponde à pesquisa e ao filtro atuais.",
    duplicatesNext: "A análise de duplicados está implementada em modo somente leitura. Nenhum arquivo é apagado ou movido.",
    conflictsIntro: "Compare TGIs compartilhados pelo payload descomprimido e classifique o impacto em vez de tratar toda sobreposição como conflito.",
    analyzeConflicts: "Analisar Conflitos",
    analyzingConflicts: "Analisando resources compartilhados…",
    conflictsReady: "Análise de conflitos concluída",
    conflictsFailed: "Falha na análise de conflitos",
    conflictSearch: "Pesquisar resultados de conflitos…",
    allConflictTypes: "Todos os resultados",
    sharedIdentical: "Compartilhado Idêntico",
    visualOverride: "Override Visual",
    catalogOverride: "Override de Catálogo",
    gameplayOverride: "Override de Gameplay",
    scriptConflict: "Conflito de Script",
    textOverride: "Override de Texto",
    potentialConflict: "Conflito Potencial",
    mixedOverride: "Override Misto",
    packagePairs: "Pares de Packages",
    realOverrides: "Overrides",
    scriptConflicts: "Script",
    potentialConflicts: "Potenciais",
    analyzeConflictsToBegin: "Analise a pasta de Mods selecionada para inspecionar resources compartilhados.",
    selectConflictFinding: "Selecione um resultado para inspecionar as evidências por resource.",
    conflictReason: "Por que foi classificado",
    conflictSamePayloadExplanation: "Esses packages compartilham um ou mais TGIs com payload descomprimido idêntico. Isso é conteúdo compartilhado, não conflito.",
    conflictVisualExplanation: "O mesmo TGI de resource visual existe nos dois packages, mas o payload descomprimido difere. É uma relação de override visual.",
    conflictCatalogExplanation: "O mesmo TGI de catálogo existe nos dois packages, mas o payload difere. O comportamento ou a categorização de catálogo pode ser sobrescrito.",
    conflictGameplayExplanation: "O mesmo TGI de tuning/gameplay existe nos dois packages, mas o payload difere. A ordem de carregamento pode alterar o comportamento no jogo.",
    conflictScriptExplanation: "O mesmo TGI de assembly/script existe nos dois packages com payload diferente. É uma colisão de script de alto risco.",
    conflictTextExplanation: "O mesmo TGI STBL existe com textos diferentes. É um override de texto/localização.",
    conflictPotentialExplanation: "O mesmo TGI existe com payload diferente, mas a família do resource ainda não está classificada em profundidade suficiente para uma conclusão mais forte.",
    conflictMixedExplanation: "Este par de packages contém várias classes de resources compartilhados com conteúdo diferente.",
    sharedCount: "TGIs compartilhados",
    samePayloadCount: "Payloads idênticos",
    differentPayloadCount: "Payloads diferentes",
    resourceClass: "Classe do resource",
    payloadA: "Payload A",
    payloadB: "Payload B",
    impact: "Impacto",
    noConflictFindings: "Nenhum resultado corresponde à pesquisa e ao filtro atuais.",
    conflictAnalysisTruncated: "A lista de pares de conflito foi limitada por desempenho.",
    resourceCfg: "Resource.cfg",
    resourceCfgWarnings: "Avisos do Resource.cfg",
    loadPriority: "Prioridade de carregamento",
    matchingRule: "Regra correspondente",
    loadOrder: "Ordem de carregamento",
    higherPriorityWins: "A prioridade mais alta do Resource.cfg tem precedência neste par.",
    samePriorityUnknown: "Os dois packages possuem a mesma prioridade no Resource.cfg. A ferramenta não infere qual vence.",
    partialPriorityUnknown: "Apenas um package correspondeu a uma regra PackedFile. A ferramenta não infere qual vence.",
    unmatchedPriorityUnknown: "Nenhum dos dois packages correspondeu a uma regra PackedFile. A ferramenta não infere qual vence.",
    missingResourceCfg: "Nenhum Resource.cfg foi encontrado na pasta selecionada nem na pasta pai.",
    likelyHigherPriority: "Prioridade mais alta",
    conflictsNext: "A análise de conflitos por resource está implementada em modo somente leitura.",
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
    cancelAnalysis: "Cancelar",
    cancelling: "Cancelando…",
    cancelled: "Análisis cancelado",
    technicalDetails: "Detalles Técnicos",
    technicalDetailsLoading: "Leyendo resources del package…",
    technicalDetailsError: "No se pudieron cargar los detalles técnicos",
    showingResourceLimit: "Mostrando los primeros 500 resources",
    fileSha256: "SHA-256 del archivo",
    dbpfVersion: "Versión DBPF",
    compression: "Compresión",
    diskSize: "Tamaño en disco",
    memorySize: "Tamaño en memoria",
    payloadHash: "SHA-256 del payload",
    resourceTechnicalDetails: "Detalles de los resources",
    cacheReused: "Caché reutilizada",
    cacheUpdated: "Caché actualizada",
    restoreHistory: "Historial de Restauración",
    restoreHistoryHint: "Manifiestos creados anteriormente para la carpeta de Mods seleccionada.",
    refreshHistory: "Actualizar",
    noRestoreHistory: "No se encontraron manifiestos de restauración para esta carpeta de Mods.",
    invalidManifest: "Manifiesto inválido",
    manifestFiles: "archivos",
    differentRoot: "Otra carpeta de Mods",
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
    duplicatesIntro: "Encuentra duplicados reales y distínguelos de variantes relacionadas de CC sin eliminar nada.",
    analyzeDuplicates: "Analizar Duplicados",
    analyzingDuplicates: "Analizando fingerprints de los packages…",
    duplicatesReady: "Análisis de duplicados completado",
    duplicatesFailed: "Error en el análisis de duplicados",
    duplicateSearch: "Buscar resultados de duplicados…",
    allDuplicateTypes: "Todos los resultados",
    exactDuplicate: "Duplicado Exacto",
    contentDuplicate: "Duplicado por Contenido",
    retexture: "Retexture",
    recategorizedVariant: "Variante Recategorizada",
    relatedVariant: "Variante Relacionada",
    exactGroups: "Grupos Exactos",
    contentGroups: "Grupos por Contenido",
    retextures: "Retextures",
    relatedVariants: "Variantes Relacionadas",
    analyzeDuplicatesToBegin: "Analiza la carpeta de Mods seleccionada para encontrar relaciones de duplicidad.",
    selectDuplicateFinding: "Selecciona un resultado para ver por qué fue detectado.",
    duplicateMembers: "Archivos de este grupo",
    duplicateReason: "Por qué fue marcado",
    fileHash: "SHA-256 del archivo",
    normalizedFingerprint: "Fingerprint normalizado",
    resourceCount: "Cantidad de resources",
    sharedResources: "Resources compartidos",
    identicalTgiPayloads: "Mismo TGI + mismo contenido",
    changedSameTgi: "Mismo TGI + contenido diferente",
    sharedStructural: "Resources estructurales compartidos",
    changedTextures: "Texturas/materiales modificados",
    changedCatalog: "Resources de catálogo modificados",
    evidence: "Evidencias",
    exactExplanation: "El archivo .package completo tiene el mismo SHA-256. Estos archivos son idénticos byte por byte.",
    contentExplanation: "Los contenedores de los packages difieren, pero el conjunto normalizado de TGIs y hashes de los resources descomprimidos es idéntico.",
    retextureExplanation: "Los resources estructurales coinciden mientras las texturas o materiales difieren. Es una variante visual relacionada, no un duplicado para eliminar.",
    recategorizedExplanation: "El contenido sustancial fuera del catálogo coincide mientras los resources de catálogo difieren. Es una variante recategorizada, no un duplicado para eliminar.",
    relatedExplanation: "Los resources estructurales coinciden, pero el resto del contenido del package difiere. Es contenido relacionado y requiere revisión.",
    unreadablePackages: "Packages no legibles",
    variantAnalysisTruncated: "La lista de relaciones entre variantes fue limitada por rendimiento.",
    noDuplicateFindings: "Ningún resultado coincide con la búsqueda y el filtro actuales.",
    duplicatesNext: "El análisis de duplicados está implementado en modo de solo lectura. Ningún archivo se elimina ni se mueve.",
    conflictsIntro: "Compara TGIs compartidos por el payload descomprimido y clasifica el impacto en lugar de tratar cada coincidencia como conflicto.",
    analyzeConflicts: "Analizar Conflictos",
    analyzingConflicts: "Analizando resources compartidos…",
    conflictsReady: "Análisis de conflictos completado",
    conflictsFailed: "Error en el análisis de conflictos",
    conflictSearch: "Buscar resultados de conflictos…",
    allConflictTypes: "Todos los resultados",
    sharedIdentical: "Compartido Idéntico",
    visualOverride: "Override Visual",
    catalogOverride: "Override de Catálogo",
    gameplayOverride: "Override de Gameplay",
    scriptConflict: "Conflicto de Script",
    textOverride: "Override de Texto",
    potentialConflict: "Conflicto Potencial",
    mixedOverride: "Override Mixto",
    packagePairs: "Pares de Packages",
    realOverrides: "Overrides",
    scriptConflicts: "Script",
    potentialConflicts: "Potenciales",
    analyzeConflictsToBegin: "Analiza la carpeta de Mods seleccionada para inspeccionar resources compartidos.",
    selectConflictFinding: "Selecciona un resultado para inspeccionar sus evidencias por resource.",
    conflictReason: "Por qué fue clasificado",
    conflictSamePayloadExplanation: "Estos packages comparten uno o más TGIs con payload descomprimido idéntico. Es contenido compartido, no un conflicto.",
    conflictVisualExplanation: "El mismo TGI de resource visual existe en ambos packages, pero el payload descomprimido es diferente. Es una relación de override visual.",
    conflictCatalogExplanation: "El mismo TGI de catálogo existe en ambos packages, pero el payload es diferente. El comportamiento o la categorización del catálogo puede quedar sobrescrito.",
    conflictGameplayExplanation: "El mismo TGI de tuning/gameplay existe en ambos packages, pero el payload es diferente. El orden de carga puede cambiar el comportamiento del juego.",
    conflictScriptExplanation: "El mismo TGI de assembly/script existe en ambos packages con un payload diferente. Es una colisión de script de alto riesgo.",
    conflictTextExplanation: "El mismo TGI STBL existe con textos diferentes. Es un override de texto/localización.",
    conflictPotentialExplanation: "El mismo TGI existe con un payload diferente, pero la familia del resource todavía no está clasificada con suficiente profundidad para una conclusión más fuerte.",
    conflictMixedExplanation: "Este par de packages contiene varias clases de resources compartidos con contenido diferente.",
    sharedCount: "TGIs compartidos",
    samePayloadCount: "Payloads idénticos",
    differentPayloadCount: "Payloads diferentes",
    resourceClass: "Clase del resource",
    payloadA: "Payload A",
    payloadB: "Payload B",
    impact: "Impacto",
    noConflictFindings: "Ningún resultado coincide con la búsqueda y el filtro actuales.",
    conflictAnalysisTruncated: "La lista de pares de conflicto fue limitada por rendimiento.",
    resourceCfg: "Resource.cfg",
    resourceCfgWarnings: "Avisos de Resource.cfg",
    loadPriority: "Prioridad de carga",
    matchingRule: "Regla correspondiente",
    loadOrder: "Orden de carga",
    higherPriorityWins: "La prioridad más alta de Resource.cfg tiene precedencia en este par.",
    samePriorityUnknown: "Ambos packages tienen la misma prioridad en Resource.cfg. La herramienta no infiere cuál gana.",
    partialPriorityUnknown: "Solo un package coincidió con una regla PackedFile. La herramienta no infiere cuál gana.",
    unmatchedPriorityUnknown: "Ninguno de los dos packages coincidió con una regla PackedFile. La herramienta no infiere cuál gana.",
    missingResourceCfg: "No se encontró Resource.cfg en la carpeta seleccionada ni en su carpeta superior.",
    advancedCfgUnknown: "Este Resource.cfg usa directivas avanzadas de recorrido o condición. El Organizer no inferirá un ganador solo por Priority.",
    likelyHigherPriority: "Prioridad más alta",
    conflictsNext: "El análisis de conflictos por resource está implementado en modo de solo lectura.",
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
  duplicatesAnalysis: null,
  duplicatesBusy: false,
  duplicatesError: "",
  duplicatesNotice: "",
  duplicatesSearch: "",
  duplicatesFilter: "all",
  duplicateSelectedId: "",
  conflictsAnalysis: null,
  conflictsBusy: false,
  conflictsError: "",
  conflictsNotice: "",
  conflictsSearch: "",
  conflictsFilter: "all",
  conflictSelectedId: "",
  operations: { scan: null, duplicates: null, conflicts: null },
  technicalDetails: {},
  technicalDetailsLoading: "",
  technicalDetailsErrors: {},
  restoreHistory: [],
  restoreHistoryLoading: false,
  restoreHistoryError: "",
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
  analyzeDuplicatesBtn: document.querySelector("#analyze-duplicates-btn"),
  duplicatesState: document.querySelector("#duplicates-state"),
  duplicatesSearch: document.querySelector("#duplicates-search"),
  duplicatesFilter: document.querySelector("#duplicates-filter"),
  duplicatesEmpty: document.querySelector("#duplicates-empty"),
  duplicatesResults: document.querySelector("#duplicates-results"),
  duplicatesList: document.querySelector("#duplicates-list"),
  duplicatesPreview: document.querySelector("#duplicates-preview"),
  dupStatPackages: document.querySelector("#dup-stat-packages"),
  dupStatExact: document.querySelector("#dup-stat-exact"),
  dupStatContent: document.querySelector("#dup-stat-content"),
  dupStatRetexture: document.querySelector("#dup-stat-retexture"),
  dupStatRelated: document.querySelector("#dup-stat-related"),
  analyzeConflictsBtn: document.querySelector("#analyze-conflicts-btn"),
  conflictsState: document.querySelector("#conflicts-state"),
  conflictsSearch: document.querySelector("#conflicts-search"),
  conflictsFilter: document.querySelector("#conflicts-filter"),
  conflictsEmpty: document.querySelector("#conflicts-empty"),
  conflictsResults: document.querySelector("#conflicts-results"),
  conflictsList: document.querySelector("#conflicts-list"),
  conflictsPreview: document.querySelector("#conflicts-preview"),
  confStatPairs: document.querySelector("#conf-stat-pairs"),
  confStatReal: document.querySelector("#conf-stat-real"),
  confStatScript: document.querySelector("#conf-stat-script"),
  confStatPotential: document.querySelector("#conf-stat-potential"),
  confStatShared: document.querySelector("#conf-stat-shared"),
  scanProgress: document.querySelector("#scan-progress"),
  scanProgressBar: document.querySelector("#scan-progress-bar"),
  scanProgressText: document.querySelector("#scan-progress-text"),
  scanCancelBtn: document.querySelector("#scan-cancel-btn"),
  duplicatesProgress: document.querySelector("#duplicates-progress"),
  duplicatesProgressBar: document.querySelector("#duplicates-progress-bar"),
  duplicatesProgressText: document.querySelector("#duplicates-progress-text"),
  duplicatesCancelBtn: document.querySelector("#duplicates-cancel-btn"),
  conflictsProgress: document.querySelector("#conflicts-progress"),
  conflictsProgressBar: document.querySelector("#conflicts-progress-bar"),
  conflictsProgressText: document.querySelector("#conflicts-progress-text"),
  conflictsCancelBtn: document.querySelector("#conflicts-cancel-btn"),
  restoreHistoryList: document.querySelector("#restore-history-list"),
  refreshRestoreHistoryBtn: document.querySelector("#refresh-restore-history-btn"),
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
  if (el.duplicatesSearch) el.duplicatesSearch.placeholder = t("duplicateSearch");
  renderStatusFilter();
  renderDuplicateFilter();
  renderConflictFilter();
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

function renderDuplicateFilter() {
  if (!el.duplicatesFilter) return;
  const options = [
    ["all", t("allDuplicateTypes")],
    ["exact_duplicate", t("exactDuplicate")],
    ["content_duplicate", t("contentDuplicate")],
    ["retexture", t("retexture")],
    ["recategorized_variant", t("recategorizedVariant")],
    ["related_variant", t("relatedVariant")],
  ];
  el.duplicatesFilter.innerHTML = "";
  for (const [value, label] of options) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = label;
    option.selected = state.duplicatesFilter === value;
    el.duplicatesFilter.appendChild(option);
  }
}

function duplicateKindLabel(kind) {
  return {
    exact_duplicate: t("exactDuplicate"),
    content_duplicate: t("contentDuplicate"),
    retexture: t("retexture"),
    recategorized_variant: t("recategorizedVariant"),
    related_variant: t("relatedVariant"),
  }[kind] || kind;
}

function duplicateExplanation(kind) {
  return {
    exact_duplicate: t("exactExplanation"),
    content_duplicate: t("contentExplanation"),
    retexture: t("retextureExplanation"),
    recategorized_variant: t("recategorizedExplanation"),
    related_variant: t("relatedExplanation"),
  }[kind] || "";
}

function flattenedDuplicateFindings() {
  const analysis = state.duplicatesAnalysis;
  if (!analysis) return [];
  return [
    ...(analysis.groups || []).map((item) => ({ ...item, findingType: "group" })),
    ...(analysis.relations || []).map((item) => ({ ...item, findingType: "relation" })),
  ];
}

function visibleDuplicateFindings() {
  const query = state.duplicatesSearch.trim().toLocaleLowerCase();
  return flattenedDuplicateFindings().filter((item) => {
    if (state.duplicatesFilter !== "all" && item.kind !== state.duplicatesFilter) return false;
    if (!query) return true;

    const memberNames = item.findingType === "group"
      ? (item.members || []).flatMap((member) => [member.name, member.relativePath])
      : [
          item.left?.name,
          item.left?.relativePath,
          item.right?.name,
          item.right?.relativePath,
          ...(item.evidence || []),
        ];

    return memberNames
      .filter(Boolean)
      .join(" ")
      .toLocaleLowerCase()
      .includes(query);
  });
}

function renderDuplicatesPreview() {
  if (!el.duplicatesPreview) return;
  const item = flattenedDuplicateFindings().find(
    (finding) => finding.id === state.duplicateSelectedId
  );

  el.duplicatesPreview.innerHTML = "";
  if (!item) {
    const empty = document.createElement("div");
    empty.className = "preview-empty";
    empty.textContent = t("selectDuplicateFinding");
    el.duplicatesPreview.appendChild(empty);
    return;
  }

  const header = document.createElement("div");
  header.className = "preview-header";
  const title = document.createElement("h3");
  title.textContent = duplicateKindLabel(item.kind);
  const badge = document.createElement("span");
  badge.className = `duplicate-kind duplicate-kind-${item.kind}`;
  badge.textContent = item.findingType === "group"
    ? `${item.members?.length || 0} ${t("packages")}`
    : "2";
  header.append(title, badge);

  const reason = document.createElement("div");
  reason.className = "duplicate-reason";
  const reasonTitle = document.createElement("strong");
  reasonTitle.textContent = t("duplicateReason");
  const reasonText = document.createElement("p");
  reasonText.textContent = duplicateExplanation(item.kind);
  reason.append(reasonTitle, reasonText);

  el.duplicatesPreview.append(header, reason);

  if (item.findingType === "group") {
    const listTitle = document.createElement("h4");
    listTitle.textContent = t("duplicateMembers");
    el.duplicatesPreview.appendChild(listTitle);

    const members = document.createElement("div");
    members.className = "duplicate-member-list";
    for (const member of item.members || []) {
      const card = document.createElement("article");
      card.className = "duplicate-member";
      const name = document.createElement("strong");
      name.textContent = member.name;
      const path = document.createElement("code");
      path.textContent = member.relativePath;
      const meta = document.createElement("small");
      const hash = member.fileSha256 ? member.fileSha256.slice(0, 16) : "—";
      meta.textContent = `${t("fileHash")}: ${hash}… · ${t("resourceCount")}: ${member.resourceCount}`;
      card.append(name, path, meta);
      members.appendChild(card);
    }
    el.duplicatesPreview.appendChild(members);
  } else {
    const pair = document.createElement("div");
    pair.className = "duplicate-pair";
    for (const [label, member] of [["A", item.left], ["B", item.right]]) {
      const card = document.createElement("article");
      const mark = document.createElement("b");
      mark.textContent = label;
      const name = document.createElement("strong");
      name.textContent = member?.name || "—";
      const path = document.createElement("code");
      path.textContent = member?.relativePath || "";
      card.append(mark, name, path);
      pair.appendChild(card);
    }
    el.duplicatesPreview.appendChild(pair);

    const metrics = document.createElement("div");
    metrics.className = "duplicate-metrics";
    const values = [
      [t("sharedResources"), item.sharedResourceCount],
      [t("identicalTgiPayloads"), item.identicalTgiPayloadCount],
      [t("changedSameTgi"), item.changedSameTgiCount],
      [t("sharedStructural"), item.sharedStructuralCount],
      [t("changedTextures"), item.changedTextureCount],
      [t("changedCatalog"), item.changedCatalogCount],
    ];
    for (const [label, value] of values) {
      const row = document.createElement("div");
      const span = document.createElement("span");
      span.textContent = label;
      const strong = document.createElement("strong");
      strong.textContent = String(value ?? 0);
      row.append(span, strong);
      metrics.appendChild(row);
    }
    el.duplicatesPreview.appendChild(metrics);

    if (item.evidence?.length) {
      const evidenceTitle = document.createElement("h4");
      evidenceTitle.textContent = t("evidence");
      const list = document.createElement("ul");
      list.className = "duplicate-evidence";
      for (const evidence of item.evidence) {
        const li = document.createElement("li");
        li.textContent = evidence;
        list.appendChild(li);
      }
      el.duplicatesPreview.append(evidenceTitle, list);
    }
  }
}

function renderDuplicates() {
  if (!el.analyzeDuplicatesBtn) return;
  el.analyzeDuplicatesBtn.disabled =
    !state.folder || state.duplicatesBusy || state.scanning || state.conflictsBusy;

  const stats = state.duplicatesAnalysis?.stats || {};
  el.dupStatPackages.textContent = stats.packagesScanned ?? 0;
  el.dupStatExact.textContent = stats.exactGroups ?? 0;
  el.dupStatContent.textContent = stats.contentGroups ?? 0;
  el.dupStatRetexture.textContent = stats.retextureRelations ?? 0;
  el.dupStatRelated.textContent =
    (stats.recategorizedRelations ?? 0) + (stats.relatedVariantRelations ?? 0);

  if (state.duplicatesBusy) {
    el.duplicatesState.textContent = t("analyzingDuplicates");
    el.duplicatesState.className = "scan-state busy";
  } else if (state.duplicatesError) {
    el.duplicatesState.textContent = `${t("duplicatesFailed")}: ${state.duplicatesError}`;
    el.duplicatesState.className = "scan-state error";
  } else if (state.duplicatesNotice) {
    el.duplicatesState.textContent = state.duplicatesNotice;
    el.duplicatesState.className = "scan-state";
  } else if (state.duplicatesAnalysis) {
    const unreadable = stats.unreadablePackages ?? 0;
    const parts = [t("duplicatesReady")];
    if (unreadable) {
      parts.push(`${t("unreadablePackages")}: ${unreadable}`);
    }
    if (stats.cacheHits) parts.push(`${t("cacheReused")}: ${stats.cacheHits}`);
    if (stats.cacheMisses) parts.push(`${t("cacheUpdated")}: ${stats.cacheMisses}`);
    if (stats.variantAnalysisTruncated) {
      parts.push(t("variantAnalysisTruncated"));
    }
    el.duplicatesState.textContent = parts.join(" · ");
    el.duplicatesState.className = "scan-state success";
  } else {
    el.duplicatesState.textContent = "";
    el.duplicatesState.className = "scan-state";
  }

  const hasAnalysis = !!state.duplicatesAnalysis;
  el.duplicatesEmpty.classList.toggle("hidden", hasAnalysis);
  el.duplicatesResults.classList.toggle("hidden", !hasAnalysis);
  if (!hasAnalysis) {
    renderDuplicatesPreview();
    return;
  }

  const findings = visibleDuplicateFindings();
  el.duplicatesList.innerHTML = "";

  if (!findings.length) {
    const empty = document.createElement("div");
    empty.className = "list-empty";
    empty.textContent = t("noDuplicateFindings");
    el.duplicatesList.appendChild(empty);
  } else {
    for (const finding of findings) {
      const button = document.createElement("button");
      button.type = "button";
      button.className =
        "duplicate-row" + (finding.id === state.duplicateSelectedId ? " active" : "");

      const main = document.createElement("div");
      main.className = "duplicate-row-main";
      const kind = document.createElement("strong");
      kind.textContent = duplicateKindLabel(finding.kind);
      const names = document.createElement("span");
      names.textContent = finding.findingType === "group"
        ? (finding.members || []).map((member) => member.name).join(" · ")
        : `${finding.left?.name || "—"} ↔ ${finding.right?.name || "—"}`;
      main.append(kind, names);

      const count = document.createElement("span");
      count.className = `duplicate-kind duplicate-kind-${finding.kind}`;
      count.textContent = finding.findingType === "group"
        ? String(finding.members?.length || 0)
        : String(finding.sharedStructuralCount ?? 0);

      button.append(main, count);
      button.addEventListener("click", () => {
        state.duplicateSelectedId = finding.id;
        renderDuplicates();
      });
      el.duplicatesList.appendChild(button);
    }
  }

  if (!findings.some((item) => item.id === state.duplicateSelectedId)) {
    state.duplicateSelectedId = findings[0]?.id || "";
  }
  renderDuplicatesPreview();
}

function renderConflictFilter() {
  if (!el.conflictsFilter) return;
  const options = [
    ["all", t("allConflictTypes")],
    ["script_conflict", t("scriptConflict")],
    ["gameplay_override", t("gameplayOverride")],
    ["catalog_override", t("catalogOverride")],
    ["visual_override", t("visualOverride")],
    ["text_override", t("textOverride")],
    ["potential_conflict", t("potentialConflict")],
    ["mixed_override", t("mixedOverride")],
    ["shared_identical", t("sharedIdentical")],
  ];
  el.conflictsFilter.innerHTML = "";
  for (const [value, label] of options) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = label;
    option.selected = state.conflictsFilter === value;
    el.conflictsFilter.appendChild(option);
  }
}

function conflictKindLabel(kind) {
  return {
    shared_identical: t("sharedIdentical"),
    visual_override: t("visualOverride"),
    catalog_override: t("catalogOverride"),
    gameplay_override: t("gameplayOverride"),
    script_conflict: t("scriptConflict"),
    text_override: t("textOverride"),
    potential_conflict: t("potentialConflict"),
    mixed_override: t("mixedOverride"),
  }[kind] || kind;
}

function loadOrderExplanation(finding) {
  return {
    resolved_by_priority: t("higherPriorityWins"),
    same_priority: t("samePriorityUnknown"),
    partially_matched: t("partialPriorityUnknown"),
    unmatched: t("unmatchedPriorityUnknown"),
    resource_cfg_missing: t("missingResourceCfg"),
    advanced_cfg_unresolved: t("advancedCfgUnknown"),
  }[finding?.loadOrderStatus] || t("samePriorityUnknown");
}

function conflictExplanation(kind) {
  return {
    shared_identical: t("conflictSamePayloadExplanation"),
    visual_override: t("conflictVisualExplanation"),
    catalog_override: t("conflictCatalogExplanation"),
    gameplay_override: t("conflictGameplayExplanation"),
    script_conflict: t("conflictScriptExplanation"),
    text_override: t("conflictTextExplanation"),
    potential_conflict: t("conflictPotentialExplanation"),
    mixed_override: t("conflictMixedExplanation"),
  }[kind] || "";
}

function visibleConflictFindings() {
  const findings = state.conflictsAnalysis?.findings || [];
  const query = state.conflictsSearch.trim().toLocaleLowerCase();
  return findings.filter((item) => {
    if (state.conflictsFilter !== "all") {
      const matchesPrimary = item.kind === state.conflictsFilter;
      const matchesImpact = (item.impactKinds || []).includes(state.conflictsFilter);
      if (!matchesPrimary && !matchesImpact) return false;
    }
    if (!query) return true;

    const haystack = [
      item.left?.name,
      item.left?.relativePath,
      item.right?.name,
      item.right?.relativePath,
      item.kind,
      ...(item.impactKinds || []),
      ...(item.evidence || []).flatMap((evidence) => [
        evidence.resourceLabel,
        evidence.resourceTypeHex,
        evidence.groupHex,
        evidence.instanceHex,
        evidence.resourceClass,
      ]),
    ]
      .filter(Boolean)
      .join(" ")
      .toLocaleLowerCase();

    return haystack.includes(query);
  });
}

function renderConflictsPreview() {
  if (!el.conflictsPreview) return;
  const finding = (state.conflictsAnalysis?.findings || []).find(
    (item) => item.id === state.conflictSelectedId
  );

  el.conflictsPreview.innerHTML = "";
  if (!finding) {
    const empty = document.createElement("div");
    empty.className = "preview-empty";
    empty.textContent = t("selectConflictFinding");
    el.conflictsPreview.appendChild(empty);
    return;
  }

  const header = document.createElement("div");
  header.className = "preview-header";
  const title = document.createElement("h3");
  title.textContent = conflictKindLabel(finding.kind);
  const badge = document.createElement("span");
  badge.className = `conflict-kind conflict-kind-${finding.severity}`;
  badge.textContent = finding.severity?.toUpperCase() || "";
  header.append(title, badge);

  const reason = document.createElement("div");
  reason.className = "conflict-reason";
  const reasonTitle = document.createElement("strong");
  reasonTitle.textContent = t("conflictReason");
  const reasonText = document.createElement("p");
  reasonText.textContent = conflictExplanation(finding.kind);
  reason.append(reasonTitle, reasonText);

  const pair = document.createElement("div");
  pair.className = "conflict-pair";
  for (const [label, member] of [["A", finding.left], ["B", finding.right]]) {
    const card = document.createElement("article");
    const mark = document.createElement("b");
    mark.textContent = label;
    const name = document.createElement("strong");
    name.textContent = member?.name || "—";
    const path = document.createElement("code");
    path.textContent = member?.relativePath || "";

    const priority = document.createElement("small");
    priority.textContent =
      `${t("loadPriority")}: ${member?.loadPriority ?? "—"}`;

    const rule = document.createElement("small");
    rule.textContent =
      `${t("matchingRule")}: ${member?.loadRule || "—"}`;

    card.append(mark, name, path, priority, rule);
    pair.appendChild(card);
  }

  const metrics = document.createElement("div");
  metrics.className = "conflict-metrics";
  for (const [label, value] of [
    [t("sharedCount"), finding.sharedResourceCount],
    [t("samePayloadCount"), finding.identicalPayloadCount],
    [t("differentPayloadCount"), finding.differentPayloadCount],
  ]) {
    const row = document.createElement("div");
    const span = document.createElement("span");
    span.textContent = label;
    const strong = document.createElement("strong");
    strong.textContent = String(value ?? 0);
    row.append(span, strong);
    metrics.appendChild(row);
  }

  el.conflictsPreview.append(header, reason, pair, metrics);

  const loadOrder = document.createElement("div");
  loadOrder.className = "conflict-load-order";
  const loadTitle = document.createElement("strong");
  loadTitle.textContent = t("loadOrder");
  const loadText = document.createElement("p");
  loadText.textContent = loadOrderExplanation(finding);
  loadOrder.append(loadTitle, loadText);

  if (finding.higherPriorityPath) {
    const winner = document.createElement("code");
    winner.textContent = `${t("likelyHigherPriority")}: ${finding.higherPriorityPath}`;
    loadOrder.appendChild(winner);
  }

  if (state.conflictsAnalysis?.resourceCfg?.path) {
    const cfg = document.createElement("code");
    cfg.textContent = `${t("resourceCfg")}: ${state.conflictsAnalysis.resourceCfg.path}`;
    loadOrder.appendChild(cfg);

    const cfgWarnings = state.conflictsAnalysis.resourceCfg.warnings || [];
    if (cfgWarnings.length) {
      const warningTitle = document.createElement("strong");
      warningTitle.textContent = t("resourceCfgWarnings");
      loadOrder.appendChild(warningTitle);

      for (const warning of cfgWarnings) {
        const warningLine = document.createElement("p");
        warningLine.textContent = warning;
        loadOrder.appendChild(warningLine);
      }
    }
  }

  el.conflictsPreview.appendChild(loadOrder);

  const evidenceTitle = document.createElement("h4");
  evidenceTitle.textContent = t("evidence");
  el.conflictsPreview.appendChild(evidenceTitle);

  const evidenceList = document.createElement("div");
  evidenceList.className = "conflict-evidence-list";

  for (const evidence of finding.evidence || []) {
    const card = document.createElement("article");
    card.className = `conflict-evidence ${evidence.samePayload ? "same" : "different"}`;

    const top = document.createElement("div");
    top.className = "conflict-evidence-top";
    const label = document.createElement("strong");
    label.textContent = evidence.resourceLabel;
    const impact = document.createElement("span");
    impact.textContent = conflictKindLabel(evidence.impactKind);
    top.append(label, impact);

    const tgi = document.createElement("code");
    tgi.textContent =
      `${evidence.resourceTypeHex} · ${evidence.groupHex} · ${evidence.instanceHex}`;

    const details = document.createElement("div");
    details.className = "conflict-evidence-details";
    for (const [metaLabel, value] of [
      [t("resourceClass"), evidence.resourceClass],
      [t("payloadA"), evidence.leftPayloadSha256],
      [t("payloadB"), evidence.rightPayloadSha256],
    ]) {
      const row = document.createElement("div");
      const span = document.createElement("span");
      span.textContent = metaLabel;
      const strong = document.createElement("strong");
      strong.textContent =
        typeof value === "string" && value.length > 24 ? `${value.slice(0, 24)}…` : value;
      row.append(span, strong);
      details.appendChild(row);
    }

    card.append(top, tgi, details);
    evidenceList.appendChild(card);
  }

  el.conflictsPreview.appendChild(evidenceList);
}

function renderConflicts() {
  if (!el.analyzeConflictsBtn) return;
  el.analyzeConflictsBtn.disabled =
    !state.folder || state.conflictsBusy || state.scanning || state.duplicatesBusy;

  const analysis = state.conflictsAnalysis;
  const stats = analysis?.stats || {};
  const realOverrides =
    (stats.visualOverrides ?? 0) +
    (stats.catalogOverrides ?? 0) +
    (stats.gameplayOverrides ?? 0) +
    (stats.scriptConflicts ?? 0) +
    (stats.textOverrides ?? 0) +
    (stats.mixedOverrides ?? 0);

  el.confStatPairs.textContent = stats.packagePairs ?? 0;
  el.confStatReal.textContent = realOverrides;
  el.confStatScript.textContent = stats.scriptConflicts ?? 0;
  el.confStatPotential.textContent = stats.potentialConflicts ?? 0;
  el.confStatShared.textContent = stats.sharedIdentical ?? 0;

  if (state.conflictsBusy) {
    el.conflictsState.textContent = t("analyzingConflicts");
    el.conflictsState.className = "scan-state busy";
  } else if (state.conflictsError) {
    el.conflictsState.textContent = `${t("conflictsFailed")}: ${state.conflictsError}`;
    el.conflictsState.className = "scan-state error";
  } else if (state.conflictsNotice) {
    el.conflictsState.textContent = state.conflictsNotice;
    el.conflictsState.className = "scan-state";
  } else if (analysis) {
    const parts = [t("conflictsReady")];
    if (stats.unreadablePackages) {
      parts.push(`${t("unreadablePackages")}: ${stats.unreadablePackages}`);
    }
    if (stats.cacheHits) parts.push(`${t("cacheReused")}: ${stats.cacheHits}`);
    if (stats.cacheMisses) parts.push(`${t("cacheUpdated")}: ${stats.cacheMisses}`);
    if (stats.analysisTruncated) {
      parts.push(t("conflictAnalysisTruncated"));
    }
    el.conflictsState.textContent = parts.join(" · ");
    el.conflictsState.className = "scan-state success";
  } else {
    el.conflictsState.textContent = "";
    el.conflictsState.className = "scan-state";
  }

  const hasAnalysis = !!analysis;
  el.conflictsEmpty.classList.toggle("hidden", hasAnalysis);
  el.conflictsResults.classList.toggle("hidden", !hasAnalysis);

  if (!hasAnalysis) {
    renderConflictsPreview();
    return;
  }

  const findings = visibleConflictFindings();
  el.conflictsList.innerHTML = "";

  if (!findings.length) {
    const empty = document.createElement("div");
    empty.className = "list-empty";
    empty.textContent = t("noConflictFindings");
    el.conflictsList.appendChild(empty);
  } else {
    for (const finding of findings) {
      const button = document.createElement("button");
      button.type = "button";
      button.className =
        "conflict-row" + (finding.id === state.conflictSelectedId ? " active" : "");

      const main = document.createElement("div");
      main.className = "conflict-row-main";
      const kind = document.createElement("strong");
      kind.textContent = conflictKindLabel(finding.kind);
      const names = document.createElement("span");
      names.textContent =
        `${finding.left?.name || "—"} ↔ ${finding.right?.name || "—"}`;
      main.append(kind, names);

      const side = document.createElement("div");
      side.className = "conflict-row-side";
      const count = document.createElement("span");
      count.textContent = String(finding.differentPayloadCount ?? 0);
      const dot = document.createElement("span");
      dot.className = `conflict-severity conflict-severity-${finding.severity}`;
      side.append(count, dot);

      button.append(main, side);
      button.addEventListener("click", () => {
        state.conflictSelectedId = finding.id;
        renderConflicts();
      });
      el.conflictsList.appendChild(button);
    }
  }

  if (!findings.some((item) => item.id === state.conflictSelectedId)) {
    state.conflictSelectedId = findings[0]?.id || "";
  }

  renderConflictsPreview();
}


function operationBusy(kind) {
  return kind === "scan"
    ? state.scanning
    : kind === "duplicates"
      ? state.duplicatesBusy
      : state.conflictsBusy;
}

function operationUi(kind) {
  return {
    scan: [el.scanProgress, el.scanProgressBar, el.scanProgressText, el.scanCancelBtn],
    duplicates: [el.duplicatesProgress, el.duplicatesProgressBar, el.duplicatesProgressText, el.duplicatesCancelBtn],
    conflicts: [el.conflictsProgress, el.conflictsProgressBar, el.conflictsProgressText, el.conflictsCancelBtn],
  }[kind];
}

function renderOperationProgress(kind) {
  const ui = operationUi(kind);
  if (!ui) return;
  const [container, bar, text, cancelButton] = ui;
  const status = state.operations[kind];
  const visible = operationBusy(kind) || status?.running || status?.phase === "cancelling";
  container.classList.toggle("hidden", !visible);
  if (!visible) return;

  const total = status?.total || 0;
  const processed = Math.min(status?.processed || 0, total || Number.MAX_SAFE_INTEGER);
  const percent = total > 0 ? Math.max(0, Math.min(100, (processed / total) * 100)) : 0;
  bar.style.width = `${percent}%`;

  const pieces = [];
  if (total > 0) pieces.push(`${processed}/${total}`);
  if (status?.current) pieces.push(status.current);
  else if (status?.phase) pieces.push(status.phase);
  if (status?.message) pieces.push(status.message);
  text.textContent = pieces.join(" · ");

  cancelButton.disabled = !!status?.cancelRequested;
  cancelButton.textContent = status?.cancelRequested ? t("cancelling") : t("cancelAnalysis");
}

function renderAllOperationProgress() {
  renderOperationProgress("scan");
  renderOperationProgress("duplicates");
  renderOperationProgress("conflicts");
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function monitorOperation(kind) {
  while (operationBusy(kind)) {
    try {
      state.operations[kind] = await invoke("get_operation_status", { kind });
      renderOperationProgress(kind);
    } catch (_) {
      // The primary operation result remains authoritative.
    }
    await sleep(180);
  }

  try {
    state.operations[kind] = await invoke("get_operation_status", { kind });
  } catch (_) {}
  renderOperationProgress(kind);
}

async function cancelAnalysis(kind) {
  try {
    await invoke("cancel_operation", { kind });
    state.operations[kind] = await invoke("get_operation_status", { kind });
    renderOperationProgress(kind);
  } catch (_) {}
}

function bytesLabel(value) {
  const bytes = Number(value || 0);
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function renderRestoreHistory() {
  if (!el.restoreHistoryList) return;
  el.restoreHistoryList.innerHTML = "";

  if (state.restoreHistoryLoading) {
    const line = document.createElement("div");
    line.className = "restore-history-empty";
    line.textContent = t("restorePreviewing");
    el.restoreHistoryList.appendChild(line);
    return;
  }

  if (!state.restoreHistory.length) {
    const line = document.createElement("div");
    line.className = "restore-history-empty";
    line.textContent = state.restoreHistoryError || t("noRestoreHistory");
    el.restoreHistoryList.appendChild(line);
    return;
  }

  for (const item of state.restoreHistory) {
    const button = document.createElement("button");
    button.type = "button";
    button.className =
      "restore-history-item" +
      (item.path === state.restoreManifest ? " active" : "") +
      (!item.valid || !item.matchesSelectedRoot ? " unavailable" : "");
    button.disabled = !item.valid || !item.matchesSelectedRoot;

    const main = document.createElement("div");
    const name = document.createElement("strong");
    name.textContent = item.fileName;
    const meta = document.createElement("span");
    meta.textContent = item.valid
      ? `${item.status} · ${item.files} ${t("manifestFiles")}`
      : t("invalidManifest");
    main.append(name, meta);

    const badge = document.createElement("span");
    badge.className = "restore-history-status";
    badge.textContent = item.matchesSelectedRoot ? item.status : t("differentRoot");

    button.append(main, badge);
    button.addEventListener("click", async () => {
      state.restoreManifest = item.path;
      state.restorePlan = null;
      state.restoreError = "";
      state.restoreNotice = "";
      render();
      await previewRestore();
    });
    el.restoreHistoryList.appendChild(button);
  }
}

async function loadRestoreHistory() {
  if (!state.folder) {
    state.restoreHistory = [];
    renderRestoreHistory();
    return;
  }
  state.restoreHistoryLoading = true;
  state.restoreHistoryError = "";
  renderRestoreHistory();
  try {
    state.restoreHistory = await invoke("list_restore_history", { folder: state.folder });
  } catch (error) {
    state.restoreHistory = [];
    state.restoreHistoryError = String(error);
  } finally {
    state.restoreHistoryLoading = false;
    renderRestoreHistory();
  }
}

async function loadTechnicalDetails(item) {
  if (!item || !state.folder || state.technicalDetailsLoading) return;
  state.technicalDetailsLoading = item.path;
  delete state.technicalDetailsErrors[item.path];
  renderPreview();

  try {
    state.technicalDetails[item.path] = await invoke("get_package_technical_details", {
      folder: state.folder,
      packagePath: item.path,
    });
  } catch (error) {
    state.technicalDetailsErrors[item.path] = String(error);
  } finally {
    state.technicalDetailsLoading = "";
    renderPreview();
  }
}

function renderTechnicalDetails(container, item) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "secondary-btn technical-details-btn";
  button.textContent = t("technicalDetails");
  button.disabled =
    state.technicalDetailsLoading === item.path ||
    state.scanning ||
    state.duplicatesBusy ||
    state.conflictsBusy;
  button.addEventListener("click", () => loadTechnicalDetails(item));
  container.appendChild(button);

  if (state.technicalDetailsLoading === item.path) {
    const loading = document.createElement("div");
    loading.className = "technical-details-state";
    loading.textContent = t("technicalDetailsLoading");
    container.appendChild(loading);
    return;
  }

  if (state.technicalDetailsErrors[item.path] && !state.technicalDetails[item.path]) {
    const error = document.createElement("div");
    error.className = "technical-details-state error";
    error.textContent = `${t("technicalDetailsError")}: ${state.technicalDetailsErrors[item.path]}`;
    container.appendChild(error);
  }

  const details = state.technicalDetails[item.path];
  if (!details) return;

  if (details.parseError) {
    const parseError = document.createElement("div");
    parseError.className = "technical-details-state error";
    parseError.textContent = details.parseError;
    container.appendChild(parseError);
  }

  const summary = document.createElement("div");
  summary.className = "technical-summary";
  appendMeta(summary, t("fileSha256"), details.fileSha256);
  appendMeta(summary, t("dbpfVersion"), details.dbpfMajor == null ? "—" : `${details.dbpfMajor}.${details.dbpfMinor ?? 0}`);
  appendMeta(summary, t("resources"), details.resourceCount);
  appendMeta(summary, t("cacheReused"), details.cacheHit ? "✓" : "—");
  container.appendChild(summary);

  const title = document.createElement("h4");
  title.className = "technical-details-title";
  title.textContent = t("resourceTechnicalDetails");
  container.appendChild(title);

  const list = document.createElement("div");
  list.className = "technical-resource-list";
  const visibleResources = (details.resources || []).slice(0, 500);
  for (const resource of visibleResources) {
    const row = document.createElement("article");
    row.className = "technical-resource";
    const top = document.createElement("div");
    const label = document.createElement("strong");
    label.textContent = resource.typeLabel;
    const compression = document.createElement("span");
    compression.textContent = resource.compression;
    top.append(label, compression);

    const tgi = document.createElement("code");
    tgi.textContent = resource.tgi;

    const meta = document.createElement("small");
    meta.textContent =
      `${t("diskSize")}: ${bytesLabel(resource.diskSize)} · ${t("memorySize")}: ${bytesLabel(resource.memorySize)}`;

    const hash = document.createElement("code");
    hash.textContent = `${t("payloadHash")}: ${resource.payloadSha256}`;

    row.append(top, tgi, meta, hash);
    list.appendChild(row);
  }
  container.appendChild(list);

  if ((details.resources || []).length > visibleResources.length) {
    const limited = document.createElement("div");
    limited.className = "technical-details-state";
    limited.textContent =
      `${t("showingResourceLimit")}: ${visibleResources.length}/${details.resources.length}`;
    container.appendChild(limited);
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

  const technical = document.createElement("div");
  technical.className = "technical-details";
  renderTechnicalDetails(technical, item);
  el.previewCard.appendChild(technical);
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
  renderDuplicates();
  renderConflicts();
  renderRestoreHistory();
  renderAllOperationProgress();

  el.folderPath.textContent = state.folder || t("noFolder");
  el.folderPath.title = state.folder;
  el.scanBtn.disabled =
    !state.folder ||
    state.scanning ||
    state.duplicatesBusy ||
    state.conflictsBusy ||
    state.planning ||
    state.executing;
  el.chooseFolderBtn.disabled =
    state.scanning ||
    state.duplicatesBusy ||
    state.conflictsBusy ||
    state.planning ||
    state.executing;

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
  state.duplicatesAnalysis = null;
  state.duplicatesError = "";
  state.duplicatesNotice = "";
  state.duplicatesSearch = "";
  state.duplicatesFilter = "all";
  state.duplicateSelectedId = "";
  state.conflictsAnalysis = null;
  state.conflictsError = "";
  state.conflictsNotice = "";
  state.conflictsSearch = "";
  state.conflictsFilter = "all";
  state.conflictSelectedId = "";
  state.technicalDetails = {};
  state.technicalDetailsLoading = "";
  state.technicalDetailsErrors = {};
  state.restoreHistory = [];
  closePlanModal();
  render();
  await loadRestoreHistory();
}

async function scanFolder(preserveSelection = false, preserveNotice = false) {
  if (!state.folder || state.scanning || state.planning || state.executing) return;

  const previousSelection = new Set(state.selectedForPlan);
  const previousNotice = state.notice;
  const previousItems = state.items;
  const previousStats = state.stats;
  const previousSelectedId = state.selectedId;

  state.scanning = true;
  state.operations.scan = null;
  void monitorOperation("scan");
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
    const message = String(error);
    if (message.includes("__S3CC_OPERATION_CANCELLED__")) {
      state.notice = t("cancelled");
      state.error = "";
      state.items = previousItems;
      state.stats = previousStats;
      state.selectedId = previousSelectedId;
      state.selectedForPlan = new Set(previousSelection);
    } else {
      state.error = message;
      state.items = [];
      state.stats = null;
      state.selectedForPlan.clear();
    }
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
    await loadRestoreHistory();
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
    await loadRestoreHistory();
    render();
  }
}

async function analyzeDuplicates() {
  if (!state.folder || state.duplicatesBusy) return;

  const previousAnalysis = state.duplicatesAnalysis;
  const previousSelectedId = state.duplicateSelectedId;

  state.duplicatesBusy = true;
  state.duplicatesNotice = "";
  state.operations.duplicates = null;
  void monitorOperation("duplicates");
  state.duplicatesError = "";
  state.duplicatesAnalysis = null;
  state.duplicateSelectedId = "";
  render();

  try {
    state.duplicatesAnalysis = await invoke("analyze_duplicates", {
      folder: state.folder,
    });
    const first = flattenedDuplicateFindings()[0];
    state.duplicateSelectedId = first?.id || "";
  } catch (error) {
    const message = String(error);
    if (message.includes("__S3CC_OPERATION_CANCELLED__")) {
      state.duplicatesError = "";
      state.duplicatesNotice = t("cancelled");
      state.duplicatesAnalysis = previousAnalysis;
      state.duplicateSelectedId = previousSelectedId;
    } else {
      state.duplicatesError = message;
    }
  } finally {
    state.duplicatesBusy = false;
    render();
  }
}

async function analyzeConflicts() {
  if (!state.folder || state.conflictsBusy) return;

  const previousAnalysis = state.conflictsAnalysis;
  const previousSelectedId = state.conflictSelectedId;

  state.conflictsBusy = true;
  state.conflictsNotice = "";
  state.operations.conflicts = null;
  void monitorOperation("conflicts");
  state.conflictsError = "";
  state.conflictsAnalysis = null;
  state.conflictSelectedId = "";
  render();

  try {
    state.conflictsAnalysis = await invoke("analyze_conflicts", {
      folder: state.folder,
    });
    state.conflictSelectedId = state.conflictsAnalysis?.findings?.[0]?.id || "";
  } catch (error) {
    const message = String(error);
    if (message.includes("__S3CC_OPERATION_CANCELLED__")) {
      state.conflictsError = "";
      state.conflictsNotice = t("cancelled");
      state.conflictsAnalysis = previousAnalysis;
      state.conflictSelectedId = previousSelectedId;
    } else {
      state.conflictsError = message;
    }
  } finally {
    state.conflictsBusy = false;
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
    if (state.tab === "restore" && state.folder) void loadRestoreHistory();
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
el.scanCancelBtn.addEventListener("click", () => cancelAnalysis("scan"));
el.duplicatesCancelBtn.addEventListener("click", () => cancelAnalysis("duplicates"));
el.conflictsCancelBtn.addEventListener("click", () => cancelAnalysis("conflicts"));
el.refreshRestoreHistoryBtn.addEventListener("click", loadRestoreHistory);
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

el.analyzeDuplicatesBtn.addEventListener("click", analyzeDuplicates);
el.analyzeConflictsBtn.addEventListener("click", analyzeConflicts);
el.conflictsSearch.addEventListener("input", (event) => {
  state.conflictsSearch = event.currentTarget.value;
  renderConflicts();
});
el.conflictsFilter.addEventListener("change", (event) => {
  state.conflictsFilter = event.currentTarget.value;
  renderConflicts();
});
el.duplicatesSearch.addEventListener("input", (event) => {
  state.duplicatesSearch = event.currentTarget.value;
  renderDuplicates();
});
el.duplicatesFilter.addEventListener("change", (event) => {
  state.duplicatesFilter = event.currentTarget.value;
  renderDuplicates();
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
