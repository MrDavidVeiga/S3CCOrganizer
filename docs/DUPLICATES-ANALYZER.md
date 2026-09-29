# Duplicates analyzer

The Duplicates page is read-only. It does not delete, move or rename packages.

## Goals

The analyzer separates true duplicates from related custom-content variants.

A shared filename, file size, resource type, mesh or TGI is never sufficient by itself to call two packages duplicates.

## Exact Duplicate

Two packages are Exact Duplicates only when their complete file SHA-256 is identical.

```text
file SHA-256 A == file SHA-256 B
→ Exact Duplicate
```

This means the two .package files are byte-for-byte identical.

## Content Duplicate

The DBPF container may differ while the resources are semantically identical.

For every readable resource the analyzer computes:

```text
Type
Group
Instance
SHA-256(decompressed payload)
payload size
```

The entries are sorted and hashed into a normalized content fingerprint.

```text
file SHA-256 differs
normalized resource fingerprint matches
→ Content Duplicate
```

This catches packages that contain the same resources even when container-level bytes differ.

## Recategorized Variant

A pair is classified as Recategorized Variant only when:

- both packages have catalog resources;
- the substantive non-catalog resource signature matches;
- the catalog signature differs.

Examples include a CAS item whose CASP categorization was changed while its mesh/textures remained the same.

This is a relationship, not a deletion recommendation.

## Retexture

A pair is classified as Retexture when:

- structural resources match by resource type + decompressed payload identity;
- both packages contain texture/material resources;
- their texture/material signatures differ.

Structural matching currently considers:

- GEOM;
- MODL;
- MLOD;
- VPXY;
- OBJK.

Texture/material matching currently considers:

- _IMG;
- TXTC;
- TXTF;
- MATD.

The structural comparison intentionally does not require the same TGI because cloned/repacked variants may carry identical payloads under different resource keys.

A Retexture is never presented as a duplicate-to-delete.

## Related Variant

When structural signatures match but the pair does not meet the stricter Retexture or Recategorized Variant rules, it is shown as Related Variant.

The user can inspect:

- shared resource count;
- same TGI + same payload;
- same TGI + different payload;
- shared structural resources;
- changed textures/materials;
- changed catalog resources;
- concrete resource evidence.

## Descriptive metadata

The recategorization comparison ignores descriptive-only resources such as:

- STBL;
- NMAP;
- package manifest.

They do not determine whether the underlying asset content is the same.

## Catalog resources

Catalog identity includes CASP, OBJD and documented Build/Buy catalog resource families such as fences, stairs, railing, fireplace, terrain brushes, pools and foundations.

## Invalid or unsupported packages

The full-file SHA-256 is computed before DBPF parsing.

Therefore an unreadable package can still participate in Exact Duplicate detection.

It cannot participate in normalized Content Duplicate or variant classification unless its resources are decoded successfully.

## Scale protection

Variant relations are generated only between packages that share a structural signature.

The UI currently caps generated variant relations at 10,000 to prevent very large retexture families from producing hundreds of thousands of pair rows.

When this limit is reached, the interface explicitly reports that the variant relation list was limited.

Exact Duplicate and Content Duplicate groups are not affected by this relation cap.
