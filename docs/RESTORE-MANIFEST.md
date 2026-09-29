# Restore manifest and Restore engine

The Organizer writes a **human-readable TXT manifest before moving any package**.

The TXT is both the user-readable snapshot and the canonical input for Restore.

## Example

```text
S3CC ORGANIZER RESTORE MANIFEST
version=1
created_at=2026-09-29T17:00:00-03:00
organization_language=pt
root=C:\Users\Player\Documents\Electronic Arts\The Sims 3\Mods\Packages
status=COMPLETE
files=3
created_directories=2
created_dir=CAS
created_dir=CAS\Roupas

[file]
sha256=0123456789ABCDEF...
size=1048576
original=Creator\Hair.package
organized=CAS\Cabelos\Feminino\Jovem Adulto-Adulto\Hair.package
[/file]

[file]
sha256=FEDCBA9876543210...
size=32768
original=Gameplay\MyMod.package
organized=Gameplay\MyMod.package
[/file]
```

## Complete baseline snapshot

The manifest contains every `.package` that existed under the selected root at the moment the organization began.

This is required even when only some packages are organized.

- moved file -> `original` and `organized` differ;
- pre-existing untouched file -> `original == organized`.

This allows Restore to distinguish a truly new file from a file that was already there before organization.

## Restore preflight

Restore never starts from paths alone.

For tracked files it checks:

1. expected organized path;
2. SHA-256;
3. file size;
4. expected original destination;
5. alternate location by identity when appropriate.

Possible states include:

```text
ready_restore
already_restored
changed
missing
ambiguous
collision_same_content
collision_different_content
```

Any changed, missing, ambiguous or colliding tracked item blocks automatic Restore.

## Files added after organization

After all baseline entries are matched to current files, any remaining current `.package` is considered new after organization.

It is never deleted.

Its proposed restore destination is:

- English: `Not Categorized\<current relative path>`
- Português: `Não Categorizado\<current relative path>`
- Español: `Sin categorizar\<current relative path>`

The **current interface language at restore time** determines this folder name.

Example:

```text
Current:
CAS\Cabelos\Feminino\NewHair.package

Restore in Portuguese:
Não Categorizado\CAS\Cabelos\Feminino\NewHair.package
```

If that destination already exists, Restore is blocked. No overwrite occurs.

## Restore execution

Restore itself is transactional:

1. build a fresh Restore preview;
2. refuse execution when any blocked item exists;
3. verify each source identity immediately before moving;
4. verify each destination immediately after moving;
5. if one action fails, reverse all already completed restore moves;
6. on success, mark the manifest `RESTORED`.

Files newly relocated to Not Categorized participate in the same rollback transaction.

## Directory cleanup

The organization manifest records directories created by the Organizer.

After a successful Restore, only those recorded directories are candidates for cleanup, and only if empty.

User directories are never recursively deleted.

## Unsafe manifest protection

Restore rejects:

- absolute entry paths;
- parent traversal such as `..`;
- malformed manifests;
- unsupported manifest versions;
- unsafe transaction states such as `PENDING` or `ROLLBACK_INCOMPLETE`.

The parser accepts the early `language=` field for backward compatibility, but new manifests use `organization_language=`.
