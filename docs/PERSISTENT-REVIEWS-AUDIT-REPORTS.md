# Persistent Conflict Reviews and Audit Reports

This milestone adds two non-destructive review features before local build validation.

## Persistent Intentional Overrides

The Conflicts page can save an `Intentional Override` decision permanently.

The decision is not keyed only by filename or folder path. The analyzer creates a stable `decisionKey` from the complete SHA-256 hashes of the two packages in the conflict pair.

This means:

- renaming either package does not invalidate the saved decision;
- moving either package within the selected Mods tree does not invalidate the saved decision;
- changing the bytes of either package creates a new decision key and requires review again.

The finding also exposes each package's complete file SHA-256 for audit purposes.

### Storage

Saved decisions are stored outside the selected Packages tree:

```text
<Packages parent>\S3CC Organizer\Review\conflict-decisions-v1.json
```

The store contains:

- decision key;
- decision type;
- SHA-256 of both packages;
- last known relative paths;
- update timestamp.

Only `intentional_override` is persisted.

`Ignored This Session` remains intentionally temporary and disappears after a fresh session.

### UI behavior

A persistent decision appears as:

```text
Saved Intentional Override
```

The normal `Intentional Override` filter includes saved decisions.

Clearing a saved intentional decision removes it from the local review store. It never alters either package or Resource.cfg.

## Audit Report

The sidebar exposes an Audit Report panel.

Export creates two files for the currently selected Mods root:

```text
<Packages parent>\S3CC Organizer\Reports\
    S3CC-Organizer-Audit-YYYYMMDD-HHMMSS.md
    S3CC-Organizer-Audit-YYYYMMDD-HHMMSS.json
```

Reports never overwrite an existing file.

### Markdown report

The human-readable Markdown report includes, when available:

- selected root and interface language;
- Organizer scan/classification summary;
- package status, category, proposed destination and classification evidence;
- Duplicates groups and relations;
- Conflicts findings;
- saved/temporary review decision shown for each conflict;
- persistent Intentional Override records, including records that are not present in the current conflict result;
- available performance timings.

If Scan, Duplicates or Conflicts has not been run in the current session, the section is explicitly marked as not analyzed.

### JSON report

The JSON companion is intended for machine-readable audits and later comparison tooling.

It contains:

- schema version;
- generation time;
- Organizer results;
- Duplicates analysis;
- Conflicts analysis;
- review decisions;
- Restore preview, when present;
- quarantine preview, when present;
- cache information;
- performance information.

Before writing files, the backend validates that the JSON payload is valid JSON.

## Safety

These features do not:

- delete packages;
- move packages;
- rewrite package resources;
- modify Resource.cfg;
- execute quarantine;
- execute Restore;
- change automatic conflict classification.

They record user review decisions and export already available analysis data only.
