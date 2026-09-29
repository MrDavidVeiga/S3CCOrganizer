# Veiga's S3CC Organizer

A Windows desktop tool for **The Sims 3** to analyze, classify and organize custom content (`.package`) without deleting user files.

## Goals

The Organizer is designed around three independent views:

- **Organizer** — classify packages from their actual game resources and propose a folder structure.
- **Duplicates** — distinguish exact copies, content duplicates, related variants/retextures and files that only share resources.
- **Conflicts** — compare resource definitions and avoid treating every shared TGI/pattern as a real conflict.

The application is implemented in **Rust + Tauri 2 + Vite**, following the same desktop stack used by Veiga's S3CC Packer/Splitter. S3PI is used only as technical reference for known The Sims 3 resource/category information; S3PI code is not embedded as a dependency.

## Safety principles

1. Never delete CC automatically.
2. Never call two packages duplicates only because their names or sizes match.
3. Never call two packages conflicting only because they share a pattern or resource key.
4. Show the reason for every duplicate/conflict classification.
5. Create a restore point before moving files.
6. Restoring a previous layout must preserve files that were added later.

## Language-aware taxonomy

Internal classification uses stable language-independent enums. **User-facing category labels and generated folder names follow the current interface language.**

For the same internal classification:

```text
English:
CAS/Clothing/Male/YA-A/Top/

Português:
CAS/Roupas/Masculino/Jovem Adulto-Adulto/Parte de Cima/

Español:
CAS/Ropa/Masculino/Adulto Joven-Adulto/Parte Superior/
```

Changing the interface language never changes resource identity or the meaning of an existing restore manifest. New organization operations use the current interface language.

## Reversible folder organization

Before organization, the app records the previous layout in a human-readable `.txt` restore manifest. Each tracked file stores:

- original relative path;
- categorized relative path;
- SHA-256;
- size.

When restoring:

- files known by the manifest return to their previous paths;
- files added after organization are **not deleted**;
- those new files are moved under a localized safe folder while preserving their relative subfolders:
  - English: `Not Categorized`
  - Português: `Não Categorizado`
  - Español: `Sin categorizar`
- the "not categorized" folder follows the interface language active at restore time;
- destination collisions are handled conservatively and surfaced to the user.

See [docs/RESTORE-MANIFEST.md](docs/RESTORE-MANIFEST.md).

## Status

The repository is being bootstrapped around four layers:

```text
UI
 └─ Tauri commands
     ├─ package scanner
     ├─ resource/classification engine
     ├─ duplicate + conflict analyzers
     └─ restore/manifest engine
```

The first implementation milestone focuses on safe scanning, restore manifests and analyzer data models before enabling automatic moves.
