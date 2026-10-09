# Auditoria técnica comparativa — funcionalidades comuns com PlumbobForge
Data: 2026-10-08. Escopo: Veiga's S3CC Manager, branch `fix/mods-copy-packages-invariant-20261008`, versus os repositórios públicos `PlumbobForge/PlumbobForge.Desktop` e `PlumbobForge/PlumbobForge.Backend`. Objetivo: corrigir lacunas **já existentes nos dois programas**, sem importar funcionalidades novas ou adotar código GPL-3.0.

> **Evidência:** inspeção estática de código-fonte. Nenhuma build, execução do app, benchmark, inspeção visual real ou teste Rust foi realizado. Os critérios abaixo são condições de aceite, não alegações de que passaram.

## Fonte e rastreabilidade
- PlumbobForge Desktop: `ViewModels/ContentManagerViewModel.Filters.cs`, `ViewModels/ContentManagerViewModel.Items.cs`, `ViewModels/ItemViewModel.cs`, `ViewModels/HealthViewModel.Conflicts.cs`, `Services/ThumbnailCache.cs`, `Services/UiStateService.cs`.
- PlumbobForge Backend: `Services/Sims3HealthService.cs`, `Services/PackageTypeService.cs`, `Services/ThumbnailService.cs`, `Services/CacheBuilderService.cs`.
- Manager: `src/app.js`, `src/style.css`, `index.html`, `src-tauri/src/scanner.rs`, `catalog.rs`, `conflicts.rs`, `duplicates.rs`, `cache.rs`, `package_details.rs`, `quarantine.rs`, `workspace.rs`, `resource_cfg.rs`, `executor.rs`.

## Matriz de constatações (antes de correções)

### C-01 — Filtros combináveis de CAS: diferença **confirmada**, prioridade ALTA
**PlumbobForge:** filtros simultâneos por tipo, categoria CAS, idade, gênero, roupa, habilitação; ordenação; persistência por filtro. Eventos de filtro com debounce de 30 ms, pesquisa 200 ms. `ContentManagerViewModel.Filters.cs`, `UiStateService.cs`.

**Manager:** o Organizador mostra pesquisa + estado de classificação apenas (`index.html`, `visibleItems()` em `src/app.js`). `ScanPackageItem.classifications` já inclui `CatalogClassification` por recurso CASP com `main_category`, `sub_category`, `age`, `gender`, `species`, `usage_categories`. Logo, NÃO é necessário alterar o roteamento físico das pastas ou inferir metadados pelos nomes.

**Risco crítico:** uma package contém múltiplos CASPs. Se um filtro por gênero for aplicado em um CASP e por idade em outro CASP, surgem falsos positivos. Todos os critérios de CASP devem satisfazer **um mesmo registro** em `classifications`. Packages de múltiplas idades/gêneros devem ser filtráveis sem forçar classificação única. As labels de idade são localizadas e podem incluir combinações; não tratar substring como identidade exata de faixa etária. Mudança de idioma requer revalidação/reset dos valores localizados.

**Aceite:** combinar categoria+subcategoria+idade+gênero+uso; filtros operam apenas na apresentação (sem alterar Planner/Restore); seleção já existente é preservada ao filtrar; pesquisa e seleção geral respeitam todos os filtros; opção All restaura todos; Reset remove filtros adicionais; resultados vazios exibem mensagem; opções indisponíveis não provocam exceção; critérios por CASP correlacionados; normalização Unicode; PT/EN/ES.

### C-02 — Cache visual de thumbnails: risco de memória e de corridas, prioridade ALTA
**PlumbobForge:** `Services/ThumbnailCache.cs`: LRU de 300 bitmaps reduzidos a ~180 px, diferencia estados grayscale, deduplica operações concorrentes (`_inFlight`); `ItemViewModel.cs`: semáforo de quatro cargas simultâneas; `ThumbnailService.cs`: armazenamento em disco e limpeza de thumbnails órfãs.

**Manager:** `get_package_preview` Rust procura recursos internos e fallback nos caches do jogo; UI carrega sob demanda e guarda `thumbnailBase64` em `state.packagePreviews` por caminho. `loadPackagePreview` e `loadDuplicateMemberPreview` não limitam explicitamente o número de prévias em memória; requisições assíncronas podem sobreviver à troca de raiz. O cache de fingerprints em `cache.rs` é **outro cache**, não resolve memória de imagens. Evitar ler repetidamente caches grandes do jogo por thumbnail é uma oportunidade adicional que exige medição.

**Aceite:** cache visual limitado por entradas e orçamento de bytes (Base64), descarte LRU, reaproveitamento Organizador/Duplicados/Conflitos, falhas recuperáveis, sem sequestro de resultados de outra raiz, sem clobber de operação em progresso, sem recomputação em cascata, nenhum acesso fora da raiz, prévias internas+fallback preservados, imagens sem vazamento após trocar pasta. Validação em CC com thumbs grandes/ausentes e rápida troca de seleção.

### C-03 — Conflitos agrupados: vantagem de navegação confirmada, prioridade ALTA / risco funcional MÉDIO
**PlumbobForge:** `Sims3HealthService.ScanLibraryConflictsAsync` cria clusters ligados por TGI com disjoint-set (componentes conexos), exibe cartões com múltiplos packages e contagem dos TGIs; `HealthViewModel.Conflicts.cs` ações por cartão.

**Manager:** `conflicts.rs` constrói pares por interseção de TGI, hashes de conteúdo descomprimido, avaliações `Resource.cfg`, classificação de impacto, `MAX_FINDINGS=20000` e amostra de `MAX_EVIDENCE_PER_FINDING=64`. A interface virtualiza pares e permite revisão segura um a um.

**Riscos:** cluster **não comprova conflito direto entre todos os membros**; transitividade A↔B↔C não significa A↔C. Misturar TGI, tipo, classes, payload idêntico, `Resource.cfg` inativo e casos intencionais num único agrupamento produz recomendações falsas. Se houver corte de resultados, indicar claramente que o agrupamento cobre apenas pares reportados. Alterar agrupamento **somente na camada de apresentação**, mantendo cada par/evidência/decisão/quarentena e contagens separadas. Não adotar heurística `Keep only this` sem prévia e confirmação.

**Aceite:** mostrar grupos relacionados expandidos/recolhidos; pares pertencentes continuam individualmente selecionáveis, revisáveis e exportáveis; nunca confundir número de grupos/pares/packages; seleção global age sobre os mesmos pares; filtros e pesquisa devem delimitar grupos; acessível via teclado; lista virtual permanece eficaz para bibliotecas grandes; não promover recursos idênticos a conflitos reais.

### C-04 — Ações contextuais de arquivo, prioridade MÉDIA-ALTA
**PlumbobForge:** renomear, favoritar, tags e habilitar diretamente no item (`ContentManagerViewModel.Items.cs`). **Manager:** metadados existem em Tools/workspace, mas o Organizador não expõe alterações diretas em cada prévia. Prévia já mostra favorito, tags e status. É possível expor favoritos/metadados por ação contextual sem alterar hashes ou mover arquivos.

**Aceite:** leitura/escrita somente para arquivo dentro da raiz, usar `set_package_metadata` existente, preservar tags/status ao favoritar, desabilitar durante ações ocupadas/readonly, indicar falhas, atualizar a prévia e estrelas imediatamente após sucesso, não reexecutar classificação nem acionar plano involuntariamente; não copiar operações de exclusão direta.

### C-05 — Estado visual / ordenação, prioridade MÉDIA
**PlumbobForge:** `UiStateService.cs` persiste ordenação, colunas, filtros, tamanho de thumbnails, painéis recolhidos. **Manager:** `persistPreferences` já salva pesquisa/estado/aba, mas não oferece ordenação ou filtros CAS complexos na aba Organizador. A lista é virtualizada.

**Aceite:** preferências serializáveis versionadas, valores inválidos revertidos a All/ordem padrão, trocas de idioma não tornam filtros invisíveis sem correspondências; retorno à mesma seleção; lista virtual não altera a ordem real de operações no backend; limpeza de lista não destrói dados em disco.

### C-06 — Desempenho de escaneamento, oportunidade NÃO comprovada sem benchmark
**PlumbobForge:** faz leitura rápida do índice DBPF (header/index), `Parallel.For` até 8 threads e índice TGI invertido. **Manager:** reutiliza hashes/payloads descomprimidos via cache persistente, também usa índice TGI invertido e faz análise mais rigorosa. Comparações de tempo sem mesmo corpus e hardware são indevidas.

**Antes de mexer no motor:** medir cold/warm em 100/1000/10000 packages, DBPF descomprimido/comprimido, com caches grandes e `Resource.cfg`; separar enumeração, leitura, descompressão, hashing, identificação dos pares e UI; registrar cancelamento e memória; evitar repetir descompressão por par; limites e truncamento explícitos.

### C-07 — Integridade / classificação: NÃO adotar regressões concorrentes
**PlumbobForge:** parte dos duplicados é agregada por `resource_count` + XOR de TGI; hash não é prova de igualdade de bytes. `HealthViewModel.Conflicts.cs` escolhe suposto vencedor por Overrides/nome e tem operação de desativação em lote. `Sims3HealthService.FixPatternPackage` substitui o original por `File.Move(... overwrite:true)`. Essas heurísticas **não** são apropriadas para o Manager.

**Manager:** manter identificação por SHA-256 de arquivo/payload, evidência por recurso, estado inativo via `Resource.cfg`, escolha humana dos arquivos a isolar, quarentena fora de Packages/Overrides, confirmação, manifesto, rollback, restauração e proteção de symlink/path traversal. Não transformar `.package.disabled` em ativo e não mover conteúdos de Packages para fora de sua própria árvore.

## Convergência e regras invioláveis
1. `Packages` permanece em `Packages` e `Overrides` em `Overrides`, independentemente do nome da pasta-pai.
2. UI WinUI 3 fiel nos temas System, Veiga Light, Veiga Dark, WinUI Light/Dark; hover/focus/disabled/contraste/scroll; PT/EN/ES não cortados.
3. Botões de ação são ações reais, caixas de seleção e menus seguem teclado e acessibilidade.
4. Não remover, sobrescrever, desativar, mover ou alterar recursos sem prévia, decisão explícita e mecanismo de restauração.
5. Não confundir `filesDiscovered`, `activePackagesFound`, `packagesScanned`, `cacheHits` e limites de pares.
6. Operações de leitura não disparam escrita; sem build/workflow até autorização expressa.

## Plano de verificação
- Estática: referências de ids/handlers, tokens CSS/temas, i18n, assinaturas Rust/JS, serialização, chaves e caminhos de arquivos.
- Casos de filtros: CASP mistos com idade/gênero cruzados, acessórios sem idade na pasta mas com idade nos metadados, packs com múltiplas categorias, category+usage distintos; status/seleção/search combinados; vazio; alteração de idioma.
- Casos de thumbnail: 0/1/101 itens, sem thumbnail, base64 grande, falha no `invoke`, navegação rápida, volta à mesma imagem, troca de pasta durante a chamada, erros que não acumulam indefinidamente.
- Casos de conflitos: 2/3/N packages em cadeias, conflito de tuning versus recurso benigno, pares inativos e intencionais, truncamento, seleção através de filtros, reabertura do detalhe.
- Eventos: teclado Ctrl/Shift/Enter/Escape, foco/rolagem, clique fora de menus, resize entre janelas largas e estreitas.
- Execução/visual/benchmarks: **pendentes**, pois o usuário expressamente proibiu builds/workflows.

## Status
Auditoria estática pré-correção registrada. As conclusões de superioridade são limitadas às áreas observáveis pelo código; não representam benchmarks nem testes em jogo. As implementações serão próprias, mantendo bibliotecas Rust/Tauri e os princípios de licença/autoria.
