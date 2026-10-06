# Veiga's S3CC Manager

A Windows desktop tool for **The Sims 3** to analyze, classify and organize custom content (`.package`) without deleting user files.

## Goals

The Organizer is designed around six independent views:

- **Organizer** — classify packages from their actual game resources and propose a folder structure.
- **Duplicates** — distinguish exact copies, content duplicates, related variants/retextures and files that only share resources.
- **Conflicts** — compare resource definitions and avoid treating every shared TGI/pattern as a real conflict.
- **Structure** — manually create nested folders, move files/folders, rename folders and safely undo recent manual structure actions.
- **Tools** — profiles/rules, protected folders, package metadata, health/Resource.cfg, snapshots, folder comparison, Inbox/New CC, groups, technical search, side-by-side comparison, conservative dependencies, Sims3Pack→Package conversion and unified history.
- **Restore** — preview and execute transactional restoration from Organizer manifests.

The application is implemented in **Rust + Tauri 2 + Vite**, following the same desktop stack used by Veiga's S3CC Packer/Splitter. S3PI is used only as technical reference for known The Sims 3 resource/category information; S3PI code is not embedded as a dependency.

## Safety principles

1. Never delete CC automatically.
2. Never call two packages duplicates only because their names or sizes match.
3. Never call two packages conflicting only because they share a pattern or resource key.
4. Show the reason for every duplicate/conflict classification.
5. Automated organization writes a restore manifest before moving files.
6. Manual Structure operations never delete or silently overwrite content and are recorded in a local operation history.
7. Restoring a previous layout must preserve files that were added later.

## Language-aware taxonomy

Internal classification uses stable language-independent enums. **User-facing category labels and generated folder names follow the current interface language.**

For the same internal classification:

```text
English:
CAS/Clothing/Male/YA-A/Top/

Português:
CAS/Roupas/Masculino/Jovem Adulto-Adulto/Parte de Cima/

Español:
CAS/Ropa/Masculino/Adulto Joven-Adulto/Parte Superior/
```

Changing the interface language never changes resource identity or the meaning of an existing restore manifest. New organization operations use the current interface language.

## Reversible folder organization

Before organization, the app records the previous layout in a human-readable `.txt` restore manifest. Each tracked file stores:

- original relative path;
- categorized relative path;
- SHA-256;
- size.

When restoring:

- files known by the manifest return to their previous paths;
- files added after organization are **not deleted**;
- those new files are moved under a localized safe folder while preserving their relative subfolders:
  - English: `Not Categorized`
  - Português: `Não Categorizado`
  - Español: `Sin categorizar`
- the "not categorized" folder follows the interface language active at restore time;
- destination collisions are handled conservatively and surfaced to the user.

See [docs/RESTORE-MANIFEST.md](docs/RESTORE-MANIFEST.md).

## First scan milestone

The first read-only scanner is now implemented. It recursively reads `.package` files, parses DBPF indexes/resources, detects CASP and OBJD catalog resources, and produces localized suggested destinations without moving files.

See [docs/FIRST-SCAN-MILESTONE.md](docs/FIRST-SCAN-MILESTONE.md).

## Organizer Planner milestone

The read-only Organizer Planner is implemented. It previews selected file moves, checks SHA-256 collisions, calculates folders to create and generates the restore-manifest text without touching the filesystem.

See [docs/ORGANIZER-PLANNER.md](docs/ORGANIZER-PLANNER.md).

## Organization execution and Restore

Transactional organization and Restore are now implemented behind mandatory preflight checks. Organization writes a complete baseline TXT manifest before moves; Restore verifies SHA-256 identities, handles later-added packages through the localized Not Categorized folder, and performs rollback on execution failure.

See [docs/ORGANIZATION-EXECUTION.md](docs/ORGANIZATION-EXECUTION.md) and [docs/RESTORE-MANIFEST.md](docs/RESTORE-MANIFEST.md).

## Duplicates analyzer

The read-only Duplicates engine is implemented. It separates byte-identical packages, normalized-content duplicates, retextures, recategorized variants and related variants using decompressed resource fingerprints.

See [docs/DUPLICATES-ANALYZER.md](docs/DUPLICATES-ANALYZER.md).

## Conflicts analyzer

The read-only conflict engine is implemented. It indexes shared TGIs, compares decompressed payload hashes, distinguishes identical shared resources from visual/catalog/gameplay/script/text overrides, and adds conservative Resource.cfg priority evidence.

See [docs/CONFLICTS-ANALYZER.md](docs/CONFLICTS-ANALYZER.md) and [docs/RESOURCE-CFG.md](docs/RESOURCE-CFG.md).

## Package family classifier

The scanner now recognizes safe non-CASP/OBJD families by authoritative resource types: skin tones, hair tones, sliders/morphs, patterns, script mods, tuning and localization. Ambiguous resource families remain Needs Review.

See [docs/PACKAGE-FAMILY-CLASSIFIER.md](docs/PACKAGE-FAMILY-CLASSIFIER.md).

## Analysis UX and cache

Long-running Scan, Duplicates and Conflicts operations now expose progress and cooperative cancellation. Resource/file fingerprints are cached persistently, Restore shows manifest history, and every scanned package can expose on-demand technical TGI/compression/hash details.

See [docs/ANALYSIS-UX-CACHE.md](docs/ANALYSIS-UX-CACHE.md).

Technical Details also exposes on-demand mesh/polycount analysis for CAS GEOM and object MLOD/MODL resources. See [docs/MESH-POLYCOUNT.md](docs/MESH-POLYCOUNT.md).

## Pre-build hardening

The application now persists UI preferences, supports a resizable sidebar, exposes safe file-manager navigation, shows cache/performance diagnostics, displays technical classification evidence in the Planner, blocks Restore across different Mods roots, supports persistent Intentional Override decisions plus session-only ignore marks, and provides reversible duplicate Quarantine outside Packages with SHA-256 preflight, rollback, manifest history and explicit restore.

See [docs/PREBUILD-HARDENING.md](docs/PREBUILD-HARDENING.md).

Interrupted Quarantine and Restore operations now expose a cautious recovery action in History. The ZIP corpus census is available as a read-only local tool; see [docs/CORPUS-VALIDATION-PLAN.md](docs/CORPUS-VALIDATION-PLAN.md).

## Persistent reviews and audit reports

Intentional Conflict overrides can now be saved locally using content-stable package SHA-256 decision keys, while `Ignored This Session` remains temporary. The app can also export a complete Markdown + JSON audit snapshot covering classifications, duplicates, conflicts and review decisions.

See [docs/PERSISTENT-REVIEWS-AUDIT-REPORTS.md](docs/PERSISTENT-REVIEWS-AUDIT-REPORTS.md).

## Manual Structure manager

The Structure tab can create nested folders, move files or folders between folders, and rename folders inside the selected Mods root. All operations reject path traversal and destination overwrite, and successful actions are recorded in the manual-operations audit log.

See [docs/MANUAL-STRUCTURE-MANAGER.md](docs/MANUAL-STRUCTURE-MANAGER.md).

## Advanced tools

The Tools workspace adds profiles and custom organization rules, protected folders, per-package tags/test status/favorites, Keep Together groups, read-only mode, before/after plan visualization, manual undo, empty-folder reporting, Resource.cfg coverage/load-order inspection, Mods snapshots, two-root comparison, safe New CC Inbox import, technical TGI/SHA search, package comparison, conservative dependency evidence, selection export and unified operation history.

Free-form package notes are intentionally not included in this phase.

See [docs/ADVANCED-TOOLS.md](docs/ADVANCED-TOOLS.md).

## Status

The repository is organized around four layers:

```text
UI
 └─ Tauri commands
     ├─ package scanner + family classifier
     ├─ resource/classification engine
     ├─ duplicate + conflict analyzers
     ├─ planner + organization + restore/manifest engine
     ├─ manual Structure + review/audit persistence
     └─ advanced Tools workspace + local workspace metadata
```

Implemented in code:

- CASP/OBJD scanning and localized organization destinations;
- additional package-family classification for skins, hair tones, sliders/morphs, patterns, scripts, tuning and localization;
- read-only Duplicates analysis;
- read-only Conflicts analysis with conservative Resource.cfg priority evidence;
- organization Planner;
- transactional organization execution with full baseline manifest;
- transactional Restore with rollback and preservation of later-added packages;
- manual Structure management for nested folder creation, file/folder moves and folder renaming;
- persistent Intentional Override decisions keyed by package SHA-256 pairs;
- Markdown + JSON audit reports including manual Structure history;
- profiles, rules and protected folders;
- tags, favorites and test status stored by SHA-256;
- Keep Together package groups;
- global read-only protection for the selected Mods tree;
- manual Structure undo and empty-folder detection/removal;
- Resource.cfg coverage and load-order inspection;
- Mods snapshots and two-folder comparison;
- safe Inbox/New CC scanning and verified copy import;
- SHA-256-stable manual review destinations for unresolved packages;
- dependency-based Keep Together group suggestions requiring explicit approval;
- reversible duplicate Quarantine with a durable manifest, no-overwrite transfer, rollback, History restore and interrupted-transaction recovery;
- advanced TGI/SHA technical search and TXT/CSV/JSON selection export;
- on-demand GEOM/MLOD/MODL vertex and triangle/polycount analysis with LOD-aware summaries;
- Sims3Pack conversion with localized manifest naming, CASP fallback, set splitting and no-overwrite output;
- side-by-side normalized package comparison;
- conservative package dependency evidence;
- unified visual operation history.

Still pending before release:

- successful Rust/Tauri build in an available execution environment;
- runtime validation of write actions through the compiled Tauri application;
- running the added Rust/ZIP audit regression suites in an available execution environment;
- analyzing the already-supplied Store and mixed ZIPs at the resource/payload level;
- large mixed Mods-folder performance testing;
- additional authoritative package-family classifiers only when real samples prove safe resource-level rules;
- portable/release workflows after runtime validation.
