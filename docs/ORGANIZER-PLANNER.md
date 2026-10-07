# Manager Planner milestone

The Planner is the mandatory safety layer between package classification and organization execution.

## Current behavior

The Planner:

- accepts only packages from the current scan;
- canonicalizes every selected path and verifies that it remains under the selected root;
- re-runs or reuses the backend scanner instead of trusting frontend-only classification state;
- schedules automatic moves only for packages with a safe `classified` destination;
- keeps `Mixed`, `Unknown`, `Needs Review` and `Invalid` packages physically in place;
- exposes unresolved packages as a logical review state instead of silently moving them to a physical Not Categorized folder;
- validates generated folder components for Windows-invalid names and path traversal;
- computes SHA-256 and file size for planned files;
- detects existing destination files;
- leaves duplicate/collision cases in place for explicit review rather than using Not Categorized as an automatic fallback;
- never overwrites an existing file;
- respects the actual parsed `Resource.cfg` when the parser can determine coverage reliably;
- preserves required literal path prefixes such as `Packages\`;
- compacts the final taxonomy when needed to keep a destination loadable;
- calculates only the directories needed by executable moves;
- generates a human-readable restore manifest preview;
- does not move files itself; execution is a separate backend command.

A physical localized Not Categorized folder still has valid uses in explicit/manual workflows and Restore safety behavior, but it is **not** the automatic destination for unresolved classification during normal organization.

## Plan states

```text
ready
keep_uncategorized
already_organized
duplicate_skipped
collision_different_content
blocked
```

- `ready` — safe automatic move.
- `keep_uncategorized` — no safe automatic destination; stays in its current location.
- `already_organized` — source already matches the safe destination.
- `duplicate_skipped` — byte-identical destination/plan duplicate; stays in place for Duplicates/Quarantine review.
- `collision_different_content` — destination collision with different data; stays in place.
- `blocked` — a true safety blocker such as protected-folder/group/path constraints.

Only executable ready items appear in the restore-manifest preview.

## Execution eligibility

A plan can execute when:

- at least one safe item is ready;
- the workspace is not read-only;
- there are no true `blocked` items.

Packages kept for review and skipped collision/duplicate cases do not force unrelated safe packages to move, and they do not need to be relocated first.

## Selection behavior

The Manager UI selects eligible classified packages by default after a fresh scan.

`Select All` and `Select None` operate on the currently visible set, so the active search query and filters are respected.

Items that are not eligible for automatic organization remain visible for inspection.

## Collision policy

If a generated destination already exists:

- same SHA-256 + same size -> duplicate review, source remains in place;
- different hash or size -> collision review, source remains in place.

Silent overwrite is never allowed. Duplicate Quarantine is a separate explicit workflow.

## Resource.cfg policy

When the source package is covered by a reliably parsed PackedFile rule, the Planner verifies the destination against the same loading model.

A deep logical taxonomy may be compacted when required to remain loadable. The classification evidence keeps the logical reason so the UI can explain why the physical path differs.

## Restore preview

The preview includes the original relative path, organized relative path, SHA-256, size, language and root for every executable move.

The organization language is audit metadata only. File identity is based on path, SHA-256 and size.
