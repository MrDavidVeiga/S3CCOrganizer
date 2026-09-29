# Package family classifier

The Organizer first attempts authoritative CASP/OBJD catalog classification.

Only when no supported catalog classification exists does it attempt package-level classification by resource family.

Filename text is never used as evidence.

## Supported package-level families

### Skin tones

Authoritative resource:

```text
SkinTone 0x0354796A
```

Destination:

```text
CAS\Genetics\Skin Tones
CAS\Genética\Tons de Pele
CAS\Genética\Tonos de Piel
```

### Hair tones

Authoritative resource:

```text
HairTone 0x03555BA8
```

Destination:

```text
CAS\Genetics\Hair Tones
```

with localized labels in PT/ES.

### Sliders / morph packages

Recognized resource families:

```text
BoneDelta 0x0355E0A6
FACE      0x0358B08A
BBLN      0x062C8204
BGEO      0x067CAA11
FBLN      0xB52F5055
```

Destination:

```text
CAS\Sliders
```

This classifier is only used when CASP/OBJD did not already classify the package, so clothing containing morph resources is not reclassified as a slider.

### Patterns

Authoritative resource:

```text
PTRN 0xD4D9FBE5
```

Destination:

```text
Patterns / Padrões / Patrones
```

Patterns remain a top-level family because Create-a-Style patterns can be used in more than one game context.

### Script mods

Authoritative resource:

```text
S3SA 0x073FAA07
```

Destination:

```text
Gameplay\Scripts
Jogabilidade\Scripts
Jugabilidad\Scripts
```

S3SA is authoritative even when the same package also contains XML, ITUN or STBL.

### Tuning

A package is considered Tuning when it contains XML/ITUN and its resource family is limited to:

```text
XML
ITUN
STBL
NMAP
Manifest
```

This prevents an object or visual package that merely contains an auxiliary XML resource from being classified as a tuning mod.

Destination:

```text
Gameplay\Tuning
Jogabilidade\Tuning
Jugabilidad\Tuning
```

### Localization

A package containing STBL and only STBL/NMAP/Manifest resources can be classified as Localization.

Destination:

```text
Localization
Localização
Localización
```

## Ambiguous families

If a non-catalog package contains more than one authoritative package family, such as SkinTone + PTRN, it becomes:

```text
Needs Review
```

All candidate destinations are shown.

The Organizer never chooses one arbitrarily.
