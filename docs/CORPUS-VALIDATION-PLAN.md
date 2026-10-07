# Full Mods corpus validation plan

## Primary corpus

The authoritative real-world corpus for the next classification audit is the user's complete local The Sims 3 Packages tree:

```text
C:\Users\David\Documents\Electronic Arts\The Sims 3\Mods\Packages
```

This replaces the earlier ZIP-batch plan as the primary audit source. The previously supplied ZIPs remain useful historical samples, but new classifier decisions should be validated against the complete folder whenever Codex/local execution is available.

The audit must preserve the existing directory structure as evidence. Folder names may help identify relationships between packages, but **must never be treated as sufficient proof of a package category**.

## Safety rules

The corpus pass is read-only:

- do not move, rename, delete or rewrite any package;
- do not change `Resource.cfg`;
- do not reorganize the live Mods tree during analysis;
- do not upload the user's packages or commit them to GitHub;
- use internal DBPF/resource evidence as the classifier authority;
- keep ambiguous packages as review/unknown instead of guessing.

Any organization test must use a disposable copy of the relevant files.

## What the audit should collect

For every `.package`, record at minimum:

- original relative path;
- SHA-256 and file size;
- DBPF/resource type census;
- CASP / OBJD / OBJK evidence;
- S3SA assemblies and internal signatures when available;
- XML / ITUN / STBL / NMAP evidence;
- CLIP / pose-list evidence;
- Store/custom-content source evidence;
- detected creator;
- detected mod name;
- gameplay category metadata;
- classification confidence;
- suggested physical destination;
- classification reason/evidence;
- related/companion package candidates;
- `Resource.cfg` coverage for the proposed destination;
- final state: Classified, Needs Review, Mixed, Unknown or Invalid.

The audit should specifically look for patterns that improve:

- gameplay scripts by creator and mod name;
- NRaas modules;
- poses and animations;
- tuning and overrides;
- Store content;
- Build/Buy objects and s3pi-aligned subcategories;
- CAS content;
- sliders/morphs;
- translations/localization carriers;
- compatibility patches, add-ons and companion packages;
- ambiguous mixed-resource packages.

## Classifier acceptance rule

A new automatic classifier is only acceptable when it is based on repeatable internal evidence and remains conservative across the full corpus.

Reducing the Unknown/Needs Review count is **not** a success if it increases false positives. When evidence is weak, the Manager should keep the package in place and expose it for review.

Physical organization of script mods should prefer:

```text
Gameplay\Creator\Mod Name
```

when creator and mod identity are supported safely. Gameplay category remains metadata/filter information instead of forcing another physical folder level.

## Resource.cfg acceptance

For every proposed move, compare the resulting relative path against the actual `Resource.cfg` that covers the source package.

The Manager may compact taxonomy levels only when necessary to keep the destination loadable. It must preserve a required literal package prefix (for example `Packages\`) and must not claim a path is safe when advanced traversal directives cannot be interpreted reliably.

## Codex pass

When Codex quota/local access is available, give Codex access to the complete folder above and the current Manager repository.

Codex should:

1. run the current scanner against the complete corpus;
2. export a before snapshot;
3. identify systematic wrong, generic and unresolved classifications;
4. inspect representative package internals for each proposed new rule;
5. implement only rules that generalize safely;
6. run the scanner again against the full corpus;
7. compare before/after counts and changed destinations;
8. flag every classification change for review;
9. add resource-grounded regression tests;
10. run `cargo test --manifest-path src-tauri/Cargo.toml --lib` and the frontend build.

It must not reorganize the live Packages folder.

## Performance and data-integrity acceptance

Run the compiled application later against the complete tree or a faithful disposable copy.

1. Record cold and warm scan times, packages/s and cache hit behavior.
2. Run Duplicates, Conflicts and Dependencies and record completeness/truncation warnings.
3. Verify repeat scans produce stable identities and evidence.
4. Verify changing UI language does not alter internal classification identity.
5. Preview a mixed selection containing safe and unresolved packages; only safe classified files may be scheduled for movement.
6. Organize a disposable copy, Restore it, and compare original relative paths, size and SHA-256.
7. Test Quarantine separately on disposable duplicates.
8. Stress search/filter/list virtualization with the real package count.

Do not claim full-corpus classifier coverage or performance acceptance until this pass has actually been executed.
