# Restore manifest

The Organizer creates a **human-readable TXT manifest** before it moves any file.

The TXT is intended to be understandable by users, while the parser treats its field names as stable keys.

## Example

```text
S3CC ORGANIZER RESTORE MANIFEST
version=1
created_at=2026-09-29T13:40:00-03:00
organization_language=pt
root=C:\Users\Player\Documents\Electronic Arts\The Sims 3\Mods\Packages

[file]
sha256=0123456789ABCDEF...
size=1048576
original=Hair\Creator\Hair.package
organized=CAS\Hair\Female\YA-A\Hair.package
[/file]
```

## Restore semantics

A restore operation scans the current organized root before changing anything.

### Files tracked by the manifest

A tracked file is restored to `original` only after identity checks.

Primary identity:
1. SHA-256
2. file size

The current relative path is not considered sufficient identity by itself.

### Files added after categorization

Any current `.package` that is not represented by the restore manifest is considered **new after categorization**.

Those files are never deleted. They are moved to a localized top-level directory:

- `Not Categorized` (en)
- `Não Categorizado` (pt)
- `Sin categorizar` (es)

Their current relative structure is preserved below that directory when possible.

Example:

```text
CAS\Hair\Female\NewHair.package
```

becomes:

```text
Não Categorizado\CAS\Hair\Female\NewHair.package
```

### Destination collisions

The restore engine must not overwrite silently.

If a destination already contains a different file, restoration records a collision and leaves both files safe. The UI must show the user which files require review.

### Empty directories

Directories created by the Organizer may be removed after a successful restore only when empty. User directories are never recursively deleted just because they were not part of the categorized layout.

## Why TXT?

The user explicitly requested a TXT snapshot so the previous folder structure can be inspected outside the application. The first version therefore uses TXT as the canonical persisted restore format rather than a hidden database.
