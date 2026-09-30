# Analysis UX: progress, cancellation, cache, history and diagnostics

This milestone improves large Mods-folder workflows without changing organization safety rules.

## Progress and cancellation

The following read-only analyses expose live operation state:

- Organizer Scan
- Duplicates
- Conflicts

Each operation reports:

- processed package count;
- total package count;
- current package;
- current phase;
- comparison progress messages where applicable.

The UI polls this state while the backend work runs in a blocking worker.

Cancellation is cooperative:

- package loops check cancellation between packages;
- Duplicates checks during variant-pair generation;
- Conflicts checks during shared-resource pair generation.

Cancelling a refresh preserves the previous valid result instead of clearing the page.

Organization execution and Restore are not part of this cancellation system because they are transactional filesystem operations with their own rollback handling.

## Persistent fingerprint cache

Duplicates, Conflicts and Technical Details share a persistent cache containing:

- canonical package path;
- file size;
- filesystem modification time;
- complete file SHA-256;
- DBPF version;
- resource Type / Group / Instance;
- decompressed payload SHA-256;
- decompressed payload size;
- on-disk resource size;
- memory size;
- compression flag;
- parse/decompression error, when present.

A cache entry is reused only when both file size and modification timestamp still match.

Changed files are rebuilt automatically.

Deleted files are removed from the in-memory cache during a full Duplicates/Conflicts pass before the cache is saved.

### Cache location

For a selected root such as:

```text
Mods\Packages
```

the cache is stored outside the package tree:

```text
Mods\S3CC Organizer\Cache\fingerprints-v1.json
```

The Organizer never writes cache files inside the selected package hierarchy.

### Corrupt or incompatible cache

If the cache JSON cannot be read, or its version is unsupported, the Organizer silently starts with a clean cache.

This affects performance only; it never changes package files.

## Unreadable packages

The full-file SHA-256 can still be cached even when DBPF/resource parsing fails.

An unreadable or partially decoded package:

- can still participate in byte-identical Exact Duplicate detection;
- cannot participate in normalized Content Duplicate classification;
- cannot participate in resource-level Conflict analysis;
- exposes the parse error in Technical Details.

This prevents partial resource fingerprints from creating false duplicate results.

## Technical Details

The Organizer preview includes an on-demand Technical Details action.

It reports:

- file SHA-256;
- DBPF major/minor version;
- resource count;
- whether the fingerprint came from cache;
- resource type and class;
- complete TGI;
- compression;
- disk size;
- memory size;
- decompressed payload SHA-256.

The resource list is scrollable. The UI renders the first 500 rows at once to avoid freezing on unusually large packages.

No technical-details request modifies a package.

## Restore history

When a Mods root is selected, the Restore page lists manifests from:

```text
<selected-root-parent>\S3CC Organizer\Restore Manifests
```

Each row includes:

- manifest filename;
- transaction status;
- number of tracked files;
- whether the manifest belongs to the currently selected root.

Invalid manifests are visible but disabled.

A manifest belonging to another selected root is also disabled in the history list.

Selecting a valid history row immediately runs the normal Restore preview; it does not execute Restore.

Manual manifest selection remains available.

## Cache concurrency

The UI prevents Scan, Duplicates and Conflicts from running concurrently.

Technical Details is also disabled while a heavy analysis is active.

This avoids competing writers for the same fingerprint cache and keeps progress reporting unambiguous.

## No workflow dependency

All features in this milestone are application runtime features.

They do not depend on GitHub Actions or a build workflow.
