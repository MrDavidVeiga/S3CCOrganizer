# Advanced Organizer Tools

This phase extends Veiga's S3CC Organizer without adding package notes.

## Tools workspace

The sixth main view, **Tools**, groups advanced features into focused subtabs so the main navigation stays compact.

### Profiles & Rules

The Organizer stores per-root workspace settings outside the selected package tree.

A profile can define:

- a destination prefix;
- category-only / minimalist organization;
- protected folders;
- custom rules.

Custom rules may match:

- classified category;
- classified subcategory;
- authoritative detected resource family;
- package filename text;
- current relative-path text.

A matching rule changes only the proposed destination. It never changes the technical classification.

Every custom destination still passes the Planner's Windows path and traversal validation.

### Protected folders

A protected folder and its descendants are blocked from automatic Planner moves.

The files remain visible to Scan, Duplicates, Conflicts, Health and technical tools.

### Read-only mode

Read-only mode blocks modifications to the selected Mods tree:

- automatic organization;
- Restore execution;
- Structure create/move/rename/undo;
- Inbox import;
- empty-folder removal.

Analysis remains available.

Organizer metadata, reports, cache and snapshots are stored outside the selected package tree and may still be updated.

## Package metadata

Package-local metadata is stored by SHA-256 rather than being written into the package.

Available fields:

- tags;
- test status: Untested / Working / Problem / Removed;
- favorite.

No free-form package notes are implemented in this phase.

Tags and status participate in Organizer search.

## Keep Together groups

A user can create a content-stable group from package SHA-256 values.

When **Keep Together** is enabled, the Planner blocks:

- selecting only part of the group;
- a plan that would distribute selected group members across different destination folders.

Related Duplicates findings and conservative dependency findings can be sent to the group selection UI.

## Before / After tree

The Planner displays the current and proposed relative paths side by side before organization.

This is a visualization only; the normal preflight, manifest and collision checks remain authoritative.

## Manual Undo

Structure history can undo the most recent reversible manual action.

Supported:

- create folder — only if the created folder is still empty;
- move — only if the recorded source is free;
- rename folder — only if the recorded source is free.

Undo never overwrites content.

Automated organization is reversed through the existing Restore system instead.

## Empty folders

Health analysis reports empty folders.

Removal is always an explicit user action and succeeds only when the folder is still empty at execution time.

There is no automatic folder cleanup.

## Resource.cfg and Load Order

Health analysis parses supported Resource.cfg directives:

- Priority;
- PackedFile.

It displays:

- each PackedFile rule;
- source line;
- priority;
- package coverage;
- package depth;
- the matching rule.

Advanced traversal/conditional directives remain conservative and mark precedence as unreliable.

Conflict pair load-order evidence continues to use the existing Conflict analyzer.

## Mods Health

The health report includes:

- total packages;
- readable / unreadable DBPF packages;
- empty folders;
- packages not covered by the supported Resource.cfg rules;
- .package files found outside the selected root in the nearby Mods tree.

The tool reports facts; it does not delete packages automatically.

## Snapshots

A snapshot stores:

- relative path;
- size;
- whole-file SHA-256.

It does not copy package contents.

Comparing a snapshot to current state reports:

- added;
- removed;
- modified;
- moved.

A move is reported only when the SHA-256 correspondence is unambiguous one-to-one. Duplicate hashes remain added/removed instead of being guessed as moves.

## Compare two Mods folders

Two roots can be compared using the same path + SHA-256 model as snapshots.

Neither root is modified.

## Inbox / New CC

An external folder can be scanned before importing packages into:

```text
<selected root>\New CC\...
```

Safety rules:

- source Inbox must be outside the selected Mods root and its parent tree;
- only readable DBPF .package files are import-ready;
- the relative Inbox subfolder structure is preserved;
- existing destinations block the whole preflight;
- source files are copied, not moved;
- every copy is verified by SHA-256 + size;
- partial copies are rolled back when the copy stage fails;
- read-only mode blocks execution;
- audit manifest logging is best effort after a successful verified import.

## Technical Search

Supported query forms:

```text
type:034AEECB
group:00000000
instance:0123456789ABCDEF
sha:ABC123
free text
```

Results can be exported as TXT, CSV or JSON.

If no technical-search rows are selected, export can use the current Organizer package selection.

## Side-by-side package comparison

Two packages can be compared by normalized TGI keys and decompressed payload SHA-256.

Each resource is reported as:

- identical;
- changed;
- only left;
- only right.

Unreadable/undecodable packages are rejected rather than being shown as empty comparisons.

## Conservative dependencies

Dependency analysis scans known reference-bearing resource families and searches their decompressed payloads for exact 16-byte little-endian TGI references that resolve to resources in another package.

These are **potential dependency relationships**, not a claim that the target is functionally required.

False positives remain possible because not every binary occurrence of a TGI is semantically a dependency.

The evidence includes:

- source package;
- source resource;
- target package;
- target resource;
- payload offset.

## Unified History

The History subtab merges:

- manual Structure operations;
- Restore/organization manifests;
- snapshots;
- Inbox import manifests.

Timestamps use a common RFC3339 representation where available.

## Audit Report

Audit JSON/Markdown now also captures the available state for:

- workspace profiles/rules/protected folders;
- tags/status/favorites;
- Keep Together groups;
- Mods Health;
- snapshots and current snapshot comparison;
- conservative dependencies;
- operation history;
- Inbox scan/import preview.

## Safety boundary

No feature in this phase automatically deletes a .package.

The only delete-like action is explicit removal of a folder that is verified empty immediately before removal.
