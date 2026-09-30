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

## Duplicate quarantine preview
Exact/Content duplicate members can be selected for a quarantine preview.

Proposed destination:
```text
<Packages parent>\S3CC Organizer\Quarantine\<timestamp>\...
```

The preview calculates SHA-256, size, collisions and a human-readable manifest.

Quarantine is intentionally preview-only:
```text
canExecute=false
```

No quarantine move/delete command exists before runtime validation.

## Next step
Stop feature expansion temporarily and validate locally:

```text
cargo check
cargo test
Tauri build
real .package tests
large-folder performance test
```
