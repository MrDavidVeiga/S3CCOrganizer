# Veiga's S3CC Manager v1.0.0 Beta 1

First public beta of Veiga's S3CC Manager.

## Included in this beta

### Organizer
- Recursive .package scanning.
- Resource-based classification instead of filename guessing.
- CASP and OBJD classification plus supported package families.
- EN / PT / ES localized organization destinations.
- Package preview and technical evidence.
- Organization Planner with Before → After preview.
- Collision checks and transactional organization.
- Restore manifest generation.

### Duplicates
- Exact byte duplicates.
- Normalized-content duplicates.
- Retexture, recategorized and related-variant evidence.
- Reversible Quarantine.
- Quarantine restore and interrupted-transaction recovery.

### Conflicts
- Shared-TGI analysis.
- Identical-resource vs real-override distinction.
- Visual, catalog, gameplay, script and text conflict evidence.
- Conservative Resource.cfg priority evidence.
- Persistent Intentional Override decisions.
- Session-only ignore.

### Restore
- Restore preview.
- Mods-root guard.
- SHA-256 validation.
- Transaction rollback.
- Preservation of files added after organization.

### Sims3Pack → Package
- One or multiple Sims3Pack inputs.
- Embedded DBPF detection.
- Localized manifest naming.
- CASP fallback when a manifest name is unavailable.
- Separate conversion.
- Combined conversion.
- No-overwrite output.

## Release staging

Features already under development for later Manager versions are intentionally not exposed in this beta. Structure, Health, Snapshots, Inbox, Metadata, the advanced Technical Toolkit, the advanced Mesh Analyzer and CC Catalog are reserved for later releases in the public roadmap.

## Beta notice

This is a testing release. Back up important Mods folders before testing write operations and report reproducible issues with the affected package/Sims3Pack when possible.
