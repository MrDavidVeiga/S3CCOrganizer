# Veiga's S3CC Manager — Release Roadmap

This document defines the planned public feature progression for Veiga's S3CC Manager.

The codebase may contain functionality ahead of the public release currently being prepared. Features can remain implemented internally while being staged for later releases after validation.

## Product direction

Veiga's S3CC Manager is not intended to be only a CAS catalog/library.

The product direction is:

> **Understand, diagnose, organize and maintain the entire The Sims 3 custom-content library.**

Development priorities:

1. Prefer features that add technical diagnosis, safety, recovery or cross-tool workflow.
2. Do not add a feature only because another CC manager has it.
3. When a common feature is useful, implement it in a way that provides additional technical value.
4. Keep Packer, Splitter and Manager complementary instead of duplicating each complete tool inside the others.
5. Every minor release should introduce at least one meaningful user-facing capability.
6. Patch releases are reserved primarily for fixes, refinements, translations, compatibility and performance.

## Versioning model

- **Major** — major expansion of the product concept or ecosystem.
- **Minor** — meaningful new functionality.
- **Patch** — bug fixes, UI refinement, translations, performance and compatibility.

Examples:

- v1.0.0 — first public feature set.
- v1.0.1 — fixes from initial testing.
- v1.1.0 — next major user-facing capability.
- v2.0.0 — CC Catalog ecosystem integration.

## v1.0.0 — Core Manager

The first public release must already make the application clearly more than a CAS catalog.

### Organizer

- recursive package scan;
- CASP/OBJD resource-based classification;
- safe package-family classification where authoritative evidence exists;
- localized folder taxonomy;
- package preview and technical information;
- selection and organization preview;
- Before → After plan;
- collision checks;
- transactional organization;
- restore manifest.

### Duplicates

- exact byte duplicates;
- normalized-content duplicates;
- recategorized/retexture/related-variant evidence;
- explanation of why packages were related;
- reversible Quarantine;
- Quarantine Restore;
- interrupted-transaction recovery.

### Conflicts

- shared-TGI analysis;
- identical-resource vs real-override distinction;
- visual/catalog/gameplay/script/text conflict categories;
- conservative Resource.cfg priority evidence;
- Intentional Override decisions;
- session-only ignore.

### Restore

- manifest preview;
- Mods-root verification;
- SHA-256 validation;
- rollback on failure;
- preservation of files added after organization;
- localized Not Categorized destination.

### Sims3Pack → Package

- one or multiple Sims3Pack inputs;
- embedded DBPF detection;
- localized manifest naming;
- CASP-name fallback;
- GUID-name avoidance where a readable identity exists;
- separate conversion;
- combined conversion;
- set handling;
- no-overwrite output;
- conversion preview.

### v1.0.x

Only stabilization work unless a release-blocking usability gap is discovered:

- bug fixes;
- runtime safety fixes;
- UI/UX corrections;
- translation fixes;
- scanner/classifier corrections supported by real samples;
- performance fixes.

## v1.1.0 — Structure & Undo

Primary new capability: direct Mods-folder structure management.

- create nested folders;
- move files;
- move folders;
- rename folders;
- no-overwrite safeguards;
- path validation;
- unified operation records;
- manual Undo;
- safe empty-folder cleanup.

## v1.2.0 — Health & Resource.cfg

Primary new capability: understand how the Mods structure affects loading.

- Resource.cfg discovery and inspection;
- folder coverage;
- package-depth checks;
- load-order/priority evidence;
- structural health warnings;
- protected-folder awareness;
- diagnostics without destructive automation.

The Manager should explain problems instead of producing generic risk scores.

## v1.3.0 — Snapshots & Inbox

Primary new capability: control what changes in the user's library.

### Snapshots

- capture Mods state;
- compare current state against a previous snapshot;
- compare two Mods roots;
- added/removed/modified/moved package reporting.

### Inbox

- scan newly downloaded CC before adding it to Mods;
- preview classification;
- inspect duplicates/conflicts where applicable;
- verified no-overwrite import.

## v1.4.0 — Metadata & Relationships

Primary new capability: user-maintained organization that survives file moves/renames.

- tags;
- favorites;
- test status;
- SHA-256-stable manual classifications;
- Keep Together groups;
- dependency-based group suggestions requiring explicit approval;
- protected folder integration.

Free-form notes remain intentionally excluded unless there is a later concrete need.

## v1.5.0 — Technical Toolkit

Primary new capability: inspect packages beyond their visible filename/category.

- search by File Name;
- search by Instance;
- search by TGI;
- search by SHA-256;
- side-by-side normalized package comparison;
- conservative package dependency evidence;
- technical selection export;
- unified technical diagnostics.

## v1.6.0 — Advanced Mesh Analyzer

Primary new capability: mesh inspection beyond a single “highest polycount” number.

- GEOM analysis;
- MLOD/MODL analysis;
- vertices;
- triangles/polycount;
- per-LOD summaries;
- LOD reduction visibility;
- visible vs shadow mesh handling;
- resource-level evidence;
- no generic “bad CC” threshold based only on polygon count.

## v1.7.x / v1.8.x / v1.9.x — Reserved

These version slots remain intentionally uncommitted.

They can be used for genuinely useful new capabilities discovered through:

- Patreon beta feedback;
- real-world package testing;
- compatibility needs;
- improvements that differentiate the Manager from other TS3 CC utilities.

Do not fill these versions merely to match a competitor's checklist.

## v2.0.0 — CC Catalog Ecosystem

The v2 milestone expands the Manager from a Mods-management utility into the persistent identity center for the Veiga S3CC tool ecosystem.

### CC Catalog

- create a new master spreadsheet inside the Manager;
- XLSX/CSV support;
- manual catalog entry without requiring package scan;
- Creator / Converter;
- File Name;
- URL;
- Tumblr Handle;
- Type;
- Resource Type;
- Instance;
- TGI;
- stable internal entry identity;
- editable editorial fields;
- technical fields populated from package evidence;
- thumbnails;
- search;
- filters;
- sorting;
- autosave;
- controlled backup retention;
- URL health;
- duplicate catalog checks;
- broken-link state;
- partnership/exception state;
- multiple CC Sources;
- ignored folders;
- source scan/enrichment;
- master comparison.

### Packer ↔ Manager bridge

- Packer reads the same master;
- Packer remains a consumer, not a second bulk catalog manager;
- quick correction of relevant catalog fields;
- Standard CC Links;
- Compact CC Links;
- automatic Missing CC Links;
- Manager imports and resolves Missing CC Links;
- stable schema with forward-tolerant parsing.

Conceptual flow:

```text
Veiga's S3CC Manager
        │
        ├── CC Catalog
        │       ↕
        │   Missing CC Links
        │       ↕
        ├── Veiga's S3CC Packer
        │
        └── Veiga's S3CC Splitter
```

## Cross-tool principle

The three tools should remain complementary:

### Manager

- identity;
- classification;
- organization;
- diagnostics;
- conflicts;
- duplicates;
- recovery;
- catalog.

### Packer

- analyze content used by Sims/Households/Lots/Saves;
- merge/package workflows;
- CC Links consumption/export.

### Splitter

- split merged packages;
- inspect/remove content from merged packages;
- duplicate-awareness around split outputs.

Do not move the complete Packer or Splitter workflows into Manager.

## Release gate

A version is not ready only because its code exists.

Before each public release:

1. run Rust tests;
2. run cargo check;
3. build the Tauri application;
4. test read/write workflows with real TS3 packages;
5. verify rollback/recovery where the release writes files;
6. test EN/PT/ES UI strings affected by the release;
7. test representative small and large libraries;
8. resolve release-blocking regressions;
9. publish the Patreon beta first when applicable;
10. promote the stabilized build to public release.

## Current implementation status

The current development branch already contains functionality planned for several future versions.

That is intentional.

Release staging controls **what is presented/enabled as a public version**, while implementation can stay ahead so later releases can be tested and refined before publication.
