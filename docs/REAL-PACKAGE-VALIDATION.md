# Real package validation matrix

These are real user-provided packages reserved for validating the first read-only scanner milestone.

> Important: classifications must be accepted only after package bytes support them. File names are never authoritative classification evidence.

## Samples

| Sample | Size | Validation target |
|---|---:|---|
| `[littlecat] Plugs_M_YA - E.package` | 146,780 bytes | CAS accessory; verify CASP clothing type, male gender and YA/Elder age flags |
| `deniisu4t3ilkupfletcherglasses.package` | 328,860 bytes | CAS glasses/accessory; verify CASP accessory subtype and age/gender flags |
| `[Veiga Sims] PrismHome(1).package` | 825,086 bytes | Non-CAS gameplay/tuning package; confirm that the CASP/OBJD-only milestone does not invent a catalog destination |
| `F-P_2019-BMW-Z4.package` | 7,828,604 bytes | Buy object/vehicle; verify OBJD FunctionCategory=Vehicles and Car subcategory when present |

## Required checks

For every package:

1. DBPF header and index parse successfully.
2. Every index TGI is preserved exactly.
3. Compressed resources required by the classifier/analyzers decode successfully.
4. File name is never used as classification evidence.
5. Scanner reports every detected supported resource family.
6. The automatic destination is generated only from parsed resource metadata/evidence.
7. Changing EN/PT/ES changes labels/folder preview only, never internal identity.

## Expected behavior for non-catalog packages

A package such as PrismHome must not become CAS/Buy/Build merely because its filename or nearby resources suggest a purpose.

A package with no supported authoritative classification remains:

```text
Unknown
```

or an explicitly supported package-level family.

## Acceptance examples

Only after byte-level parsing confirms the corresponding flags/resources should results such as these be accepted:

```text
CAS / Accessories / Male / <Age> / <Subtype>
CAS / Accessories / <Gender> / <Age> / Glasses
Buy / Vehicles / Cars
CAS / Sliders
```

If the real resource flags disagree with the filename, **the resource flags win**.

---

## 2026-09-30 — real Sliders corpus

A user-provided Sliders folder was validated locally without committing any `.package` files to this repository.

This was a byte-level validation against the same Sims 3 DBPF index layout and RefPack compression rules implemented by the Organizer. It is not a substitute for the later compiled Tauri runtime validation.

### Corpus

```text
.package files: 299
DBPF parse success: 299
DBPF parse failures: 0
resource decode failures: 0
byte-identical duplicate groups: 0
```

### Scanner classification

Authoritative morph resource families found:

```text
BoneDelta
FACE
BBLN
BGEO
FBLN
```

Results:

```text
direct slider/morph packages: 290
STBL slider companion packages: 1
classified to CAS\Sliders: 291
conservative Unknown/Needs Review auxiliaries: 8
```

The STBL-only companion is classified as a slider only because its STBL entry keys match morph-resource instance IDs in other packages from the selected set. Its filename is not used as evidence.

The real corpus confirmed matches from the STBL companion to FBLN instances for the following slider concepts:

```text
Butt Size
Breast Size
Head Size
Hip Size
Shoulder Size
Waist Size
Calf Size
Chest Size
Neck Size
Thigh Size
```

### Auxiliary packages intentionally not auto-classified

Eight packages contain only resources such as GEOM, IMG, NMAP and/or LAYO without an authoritative supported family that proves a safe organization category.

They remain conservative instead of being classified from their filenames.

### Planner simulation — default profile

For the 291 scanner-classified packages:

```text
selected: 291
ready: 291
blocked: 0
already organized: 0
same-content destination collisions: 0
different-content destination collisions: 0
destination filename collisions inside CAS\Sliders: 0
directories to create:
  CAS
  CAS\Sliders
canExecute expectation: true
```

The eight non-classified auxiliaries remain untouched because they are not eligible for automatic organization.

### Organization/Restore copy simulation

A disposable extracted copy of the real Sliders corpus was used to simulate the Planner/execution/Restore file transaction without compiling the application.

Baseline identity was recorded as relative path + SHA-256 + size for all 299 packages.

Organization result:

```text
planned classified packages: 291
moved to CAS\Sliders: 291
conservative auxiliaries left in place: 8
total packages after organization: 299
destination overwrites: 0
hash/size verification failures: 0
```

Restore result:

```text
restored packages: 299
final relative-path match vs baseline: exact
final SHA-256 match vs baseline: exact
final size match vs baseline: exact
temporary CAS\Sliders directory remaining: no
temporary CAS directory remaining: no
```

The two non-package files in the supplied folder (`desktop.ini` and `lista_arquivos.txt`) were not moved or modified.

This validates the real corpus and transaction model, but the same scenario must still be repeated through the compiled Tauri commands after the first build.

### Duplicates validation

Using decompressed resource payload hashes and the same normalized fingerprint rules as the Duplicates analyzer:

```text
exact duplicate groups: 0
normalized-content duplicate groups: 0
retexture relations: 0
recategorized variant relations: 0
related variant relations: 0
```

### Conflicts validation

The corpus exposed a major false-positive source: many unrelated slider packages contain package-local NMAP metadata using the same TGI, commonly instance zero.

Before ignoring package-local NMAP/Manifest metadata, that pattern produced thousands of meaningless package pairs.

After the correction, only three cross-package override pairs remain:

1. two shared GEOM resources between an aWT eyeball mesh and BloomBase eye correction;
2. two shared GEOM resources between PYXIS Obscura eyeball mesh and the Buckley child/toddler eye mesh replacement;
3. one shared LAYO resource between the two Cmar SkinTonePanel variants.

All five shared GEOM payloads and the shared LAYO payload differ between the corresponding packages, so these are genuine override relationships rather than shared-identical resources.

LAYO (`0x025C95B6`) is recognized as a visual/UI resource by the conflict analyzer.

### Regression coverage added from this corpus

The code now includes regression checks for:

- NMAP and Manifest exclusion from cross-package conflict evidence;
- LAYO labeling and visual-override classification;
- STBL v2 key parsing;
- morph resource types used by slider companion matching;
- STBL-key-to-morph-instance companion evidence.

### Next validation stage

Do not create release/portable workflows yet.

The next stage, requested separately, is the first compiled Rust/Tauri test/build followed by runtime testing against a copy of real Mods content.
