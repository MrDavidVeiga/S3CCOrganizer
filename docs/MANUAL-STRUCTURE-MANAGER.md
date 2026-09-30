# Manual Structure Manager

The Structure tab provides explicit manual folder management inside the currently selected Mods root.

## Supported actions

### Create folders

The user can create:

- one new folder;
- nested folders in one action.

Example:

```text
Creator\Hair\Female
```

When the current folder is:

```text
CAS
```

the result is:

```text
CAS\Creator\Hair\Female
```

Existing destination paths are never overwritten.

### Move files or folders

Any selected file or folder inside the selected Mods root can be moved into another existing folder inside the same root.

The operation blocks:

- moving the selected root itself;
- destinations outside the selected root;
- moving a folder into itself;
- moving a folder into one of its descendants;
- destination collisions.

Files and folders are moved with a filesystem rename. No destination overwrite is allowed.

### Rename folders

Folders inside the selected root can be renamed.

The selected root itself cannot be renamed.

The new name is validated as a single filesystem component and Windows-invalid/reserved names are rejected.

Existing destinations are never overwritten.

## Path safety

All user-supplied relative paths reject:

- absolute paths;
- `..` traversal;
- root/prefix components;
- symbolic-link traversal for folder creation.

Existing paths are canonicalized and must remain under the selected root.

## Audit log

Every successful manual operation is appended to:

```text
<Packages parent>\S3CC Organizer\Manual Operations\manual-operations-v1.jsonl
```

The log records:

- timestamp;
- operation type;
- source relative path when applicable;
- destination relative path.

Manual Structure operations are also included in exported Audit Reports.

## Analysis invalidation

After a manual create/move/rename operation, scan, duplicate and conflict results are invalidated in the UI because filesystem paths may have changed.

The user must rerun analysis before trusting previous path-based results.

## Safety boundary

The Structure tab does not:

- delete files;
- overwrite destinations;
- move content outside the selected root;
- rewrite package contents;
- modify Resource.cfg.
