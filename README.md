# Veiga's S3CC Organizer

A Windows desktop tool for **The Sims 3** to analyze, classify and organize custom content (`.package`) without deleting user files.

## Goals

The Organizer is designed around three independent views:

- **Organizer** — classify packages from their actual game resources and propose a folder structure.
- **Duplicates** — distinguish exact copies, content duplicates, related variants/retextures and files that only share resources.
- **Conflicts** — compare resource definitions and avoid treating every shared TGI/pattern as a real conflict.

The application is implemented in **Rust + Tauri 2 + Vite**, following the same desktop stack used by Veiga's S3CC Packer/Splitter. S3PI is used only as technical reference for known The Sims 3 resource/category information; S3PI code is not embedded as a dependency.

## Safety principles

1. Never delete CC automatically.
2. Never call two packages duplicates only because their names or sizes match.
3. Never call two packages conflicting only because they share a pattern or resource key.
4. Show the reason for every duplicate/conflict classification.
5. Create a restore point before moving files.
6. Restoring a previous layout must preserve files that were added later.

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

## Status

The repository is organized around four layers:

```text
UI
 └─ Tauri commands
     ├─ package scanner + family classifier
     ├─ resource/classification engine
     ├─ duplicate + conflict analyzers
     └─ planner + organization + restore/manifest engine
```

Implemented in code:

- CASP/OBJD scanning and localized organization destinations;
- additional package-family classification for skins, hair tones, sliders/morphs, patterns, scripts, tuning and localization;
- read-only Duplicates analysis;
- read-only Conflicts analysis with conservative Resource.cfg priority evidence;
- organization Planner;
- transactional organization execution with full baseline manifest;
- transactional Restore with rollback and preservation of later-added packages.

Still pending before release:

- successful Rust/Tauri build in an available execution environment;
- runtime validation against the supplied real .package samples;
- correction of any parser/classifier differences found during those real-package tests;
- portable/release workflows after runtime validation.
