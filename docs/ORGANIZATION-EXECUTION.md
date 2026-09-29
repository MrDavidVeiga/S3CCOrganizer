# Organization execution

The Organizer only enables execution after a fresh backend preflight reports a clean plan.

## Safety order

1. Re-run the package scanner.
2. Rebuild the organization plan.
3. Reject blocked items and any destination collision.
4. Canonicalize all selected source paths.
5. Snapshot **every existing .package under the selected root**, not only the selected files.
6. Compute SHA-256 and size for the complete baseline snapshot.
7. Write a restore manifest with status `PENDING`.
8. Create only the required destination directories.
9. Before each move, verify source identity again.
10. Refuse the move if the destination appeared after preflight.
11. Move the file.
12. Verify the destination SHA-256 and size.
13. When all moves succeed, update the manifest to `COMPLETE`.

No overwrite is allowed.

## Why the manifest snapshots every package

An organization can target only part of a Mods folder.

If the manifest contained only moved files, packages that already existed but were not selected could later look like files added after organization.

Therefore every baseline package receives a manifest entry.

Moved package:

```text
original=Creator\Top.package
organized=CAS\Clothing\Female\YA-A\Top\Top.package
```

Existing package that was not moved:

```text
original=Gameplay\MyMod.package
organized=Gameplay\MyMod.package
```

During Restore, an `original == organized` entry is recognized as part of the old structure and is never treated as a new file.

## Transaction statuses

```text
PENDING
COMPLETE
ROLLED_BACK
ROLLBACK_INCOMPLETE
RESTORED
RESTORE_ROLLBACK_INCOMPLETE
```

A `PENDING` manifest is written before the first move.

If a move fails, all successfully moved files are processed in reverse order and returned to their original paths. Their identity is verified again during rollback.

- all rollback operations succeed -> `ROLLED_BACK`
- one or more rollback operations fail -> `ROLLBACK_INCOMPLETE`

## Manifest storage

Restore manifests are stored outside the selected Packages root:

```text
<Packages parent>\S3CC Organizer\Restore Manifests\
```

Example:

```text
Mods\Packages
Mods\S3CC Organizer\Restore Manifests\S3CC-Organizer-Restore-20260929-170000.txt
```

## Created directories

The manifest records every destination directory that did not exist during planning.

Restore may remove **only those recorded directories**, and only when they are empty.

Arbitrary user folders are never recursively deleted.
