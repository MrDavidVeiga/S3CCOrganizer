# Organizer Planner milestone

The Planner is a read-only safety layer between package classification and future file moves.

## Current behavior

The Planner:

- accepts only packages from the current scan;
- canonicalizes every selected path and verifies that it remains under the selected root;
- re-runs the scanner in the backend instead of trusting frontend classification state;
- allows automatic planning only for `classified` packages;
- blocks `Mixed`, `Unknown`, `Needs Review` and `Invalid`;
- validates generated folder components for Windows-invalid names and path traversal;
- computes SHA-256 and file size for planned files;
- detects existing destination files;
- distinguishes an identical-content collision from a different-content collision;
- never overwrites an existing file;
- calculates the folders that would need to be created;
- generates a human-readable restore manifest preview;
- never moves, renames, deletes or creates user files in this milestone.

The returned plan deliberately contains:

```text
canExecute=false
```

## Plan states

```text
ready
already_organized
collision_same_content
collision_different_content
blocked
```

Only `ready` items appear in the restore-manifest preview because only those items would be moved in the future execution milestone.

## Selection behavior

The Organizer UI selects eligible classified packages by default after a fresh scan.

`Select All` and `Select None` operate on the currently visible set, which means the active search query and status filter are both respected.

Items that are not eligible for automatic organization have disabled selection controls.

## Collision policy

If the generated destination already exists:

- same SHA-256 + same size -> `collision_same_content`;
- different hash or size -> `collision_different_content`.

Both states block execution. Silent overwrite is never allowed.

## Restore preview

The preview format includes:

```text
S3CC ORGANIZER RESTORE MANIFEST
version=1
created_at=<timestamp>
mode=PREVIEW
organization_language=<en|pt|es>
root=<selected root>
files=<ready count>

[file]
sha256=<SHA-256>
size=<bytes>
original=<old relative path>
organized=<new relative path>
[/file]
```

The organization language is audit metadata only. File identity is based on path, SHA-256 and size.
