# Conflicts analyzer

The Conflicts page is read-only. It never deletes, moves, renames or rewrites packages.

## Core rule

A shared TGI alone is not a conflict.

For every shared TGI, the analyzer compares the SHA-256 of the **decompressed resource payload**.

```text
same TGI + same payload
→ Shared Identical
→ informational
→ not a conflict
```

```text
same TGI + different payload
→ classify by resource family
```

## Impact classes

### Visual Override

Used for differing shared visual resources such as:

- _IMG
- GEOM
- MODL
- MLOD
- VPXY
- MATD
- TXTC
- TXTF
- OBJK

This means the two packages target the same visual resource identity with different content.

It is not automatically considered harmful: default replacements and intentional visual overrides can legitimately behave this way.

### Catalog Override

Used for differing shared catalog resources such as CASP, OBJD and supported Build/Buy catalog families.

Possible effects include:

- CAS categorization changes;
- Buy/Build category changes;
- object catalog metadata changes.

### Gameplay Override

Used for differing shared tuning resources currently including:

- XML
- ITUN

These can change gameplay behavior depending on which version takes precedence.

### Script Conflict

Used when two packages contain the same S3SA TGI with different payloads.

This is shown as high severity because two different script assemblies targeting the same resource identity should be reviewed carefully.

### Text Override

Used for STBL resources with the same TGI and different payloads.

This normally represents text/localization overriding rather than a gameplay or visual conflict.

### Potential Conflict

Used when the same TGI differs but the resource family does not yet have a stronger semantic classification.

The analyzer intentionally does not invent a stronger label.

### Mixed Override

Used when one package pair contains more than one non-identical impact class, for example visual and gameplay overrides together.

## Pair aggregation

Findings are grouped by package pair.

For each pair the UI shows:

- total shared TGIs;
- identical shared payloads;
- different shared payloads;
- all impact classes;
- Type / Group / Instance;
- payload hash A;
- payload hash B;
- resource class.

Up to 64 resource evidence rows are retained per pair. The finding reports when evidence was truncated.

The total pair list is capped at 20,000 for performance. Shared-resource indexing is TGI-based, so packages that share nothing are never compared pairwise.

## Resource.cfg

The analyzer looks for `Resource.cfg` in:

1. the selected root;
2. the selected root's parent.

This supports both choosing `Mods` and choosing the usual `Mods\Packages` folder.

The parser currently uses:

- `Priority <number>`;
- `PackedFile <pattern>`.

The current priority applies to subsequent PackedFile rules until another Priority line appears.

For a package, the analyzer records the highest matching PackedFile priority.

### Conservative load-order result

```text
priority A > priority B
→ A is reported as higher priority
```

```text
priority A == priority B
→ winner not inferred
```

```text
only one package matched a PackedFile rule
→ winner not inferred
```

```text
no Resource.cfg / no matching rule
→ winner not inferred
```

The analyzer does not currently infer an order between packages with equal priority.

## Current scope

Nested Resource.cfg discovery, Select/End condition evaluation, Scan traversal semantics and other advanced Resource.cfg commands are not yet used for winner prediction.

The UI only makes a precedence statement when a numeric Priority difference is directly supported by matching PackedFile rules.
