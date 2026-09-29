# Real package validation matrix

These are real user-provided packages reserved for validating the first read-only scanner milestone.

> Important: the classifications below are **test expectations inferred from the sample purpose/file name only** until the package bytes are successfully parsed. They must not be treated as validated results.

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
3. Compressed CASP/OBJD resources decode successfully.
4. File name is never used as classification evidence.
5. Scanner reports every detected catalog resource.
6. The automatic destination is generated only from parsed resource metadata.
7. Changing EN/PT/ES changes labels/folder preview only, never internal identity.

## Expected behavior for non-catalog packages

A package such as PrismHome must not become CAS/Buy/Build merely because its filename or nearby resources suggest a purpose.

Until gameplay/tuning classification is implemented, a package with no supported CASP/OBJD catalog resource should remain:

```text
Unknown
```

or an explicitly supported future gameplay classification.

## Acceptance examples

Only after byte-level parsing confirms the corresponding flags should results such as these be accepted:

```text
CAS / Accessories / Male / <Age> / <Subtype>
CAS / Accessories / <Gender> / <Age> / Glasses
Buy / Vehicles / Cars
```

If the real resource flags disagree with the filename, **the resource flags win**.
