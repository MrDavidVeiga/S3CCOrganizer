# Pre-build hardening

This milestone closes application-side work before the first local Rust/Tauri build.

## UI preferences
Stored locally under `s3cc-organizer-preferences-v1`:
- language and active tab;
- last Mods folder;
- Organizer/Duplicates/Conflicts normal filters and searches;
- sidebar width.

Session-only Conflict marks are not persisted.

## Safe navigation
Backend commands:
- `open_directory`
- `reveal_path`

They only open existing paths. The UI can reveal a package and open the cache/manifest directories.

## Cache and diagnostics
The sidebar exposes cache entry count, size, path, Open Cache and Clear Cache.

Clear Cache only removes the Organizer fingerprint cache after internal confirmation.

Performance diagnostics report:
- Scan total;
- Duplicates total, hashing, DBPF load, resource decode/fingerprint and comparison;
- Conflicts total, hashing, DBPF load, resource decode/fingerprint and comparison.

## Planner evidence
Automatic destinations now expose technical evidence.

CASP evidence includes clothingType, typeFlags, age/species/gender flags and clothing category.

OBJD evidence includes FunctionCategory, SubCategory1, SubCategory2 and BuildCategory.

Package-family classifications expose the resource family used.

## Restore root guard
Restore receives the currently selected Mods root and canonicalizes it against the manifest root.

Execution requires the two roots to match. The UI shows Manifest root vs Selected Mods root before Restore.

## Conflict review marks
Current-session annotations:
- Intentional Override
- Ignored This Session

These marks do not alter packages or Resource.cfg and are cleared on a fresh analysis or app restart.

## Reversible duplicate quarantine
Exact/Content duplicate members can be selected for a quarantine preflight.

Destination:
```text
<Packages parent>\S3CC Organizer\Quarantine\<timestamp>\...
```

Execution is explicit and transactional. It rechecks SHA-256 + size, never overwrites, writes a durable quarantine manifest *before* moving data, rolls back on failure and can be restored later from Unified History.

File movement uses atomic no-overwrite destination creation (hard links where supported; create-new and verified copy otherwise). The same preview session directory is preserved through execution. Original bytes are removed from their prior location only after the newly-created destination is verified.

Transaction statuses include `PENDING`, `COMPLETE`, `ROLLED_BACK`, `ROLLBACK_INCOMPLETE`, `RESTORE_PENDING`, `RESTORE_INCOMPLETE`, and `RESTORED`. On interrupted transactions, the History view offers **Recover Quarantine**. Recovery preflights *every* file and only returns verified single-location files to their original paths. If two copies exist or their contents differ, the operation refuses to delete or overwrite and requires manual inspection.

Quarantine ZIP/data inventory: [CORPUS-VALIDATION-PLAN.md](CORPUS-VALIDATION-PLAN.md). Added Rust regression cases cover success, an interrupted manifest, tampering, an occupied restore destination, a simulated late failure and no-overwrite transfer. These tests are source-committed, **not yet executed**.

No package is deleted.

## Manual review
Unknown/Mixed/Needs Review packages can receive an explicit user-approved destination stored by SHA-256. Invalid Windows/traversal destinations are rejected. Authoritative automatic classifications continue to take priority.

## Dependency group suggestions
Conservative dependency evidence is aggregated into reviewable Keep Together suggestions. No group is created without explicit user approval.

## Large-folder safety and performance

The scanner collects slider morph IDs during its initial package scan instead of reopening all morph packages. STBL companion checks reopen only STBL carriers. Dependency analysis bounds large resource payload scans and reports how many were skipped, including translated UI warnings. The runtime/performance impact is **not measured yet**.

The read-only ZIP census tool and its Python regression tests are in `tools/`. Existing Library ZIPs are listed in `CORPUS-VALIDATION-PLAN.md`; the larger ZIPs still need executable byte-level validation.

## Next step

The **v1.0.0 public scope is now frozen** to Organizer + Duplicates + Conflicts + Restore + Sims3Pack → Package.

Development code may remain ahead of the public roadmap, but release validation should focus first on the v1.0.0 feature set:

```text
cargo check
cargo test
Tauri build
real .package tests
real Sims3Pack conversion tests
organization / quarantine / restore write tests
large-folder performance test
EN/PT/ES regression pass
```

Later functionality is staged according to [RELEASE-ROADMAP.md](RELEASE-ROADMAP.md).
