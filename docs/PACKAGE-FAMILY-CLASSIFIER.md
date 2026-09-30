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

Base destination:

```text
CAS\Sliders
```

For direct morph sliders, the Organizer then reads the slider's **internal name**, not the CAS panel/category in which the creator placed the control.

Evidence order:

1. NMAP internal slider name;
2. STBL internal label as fallback when NMAP is missing or opaque.

This matters because a body slider can legitimately be exposed by the creator under a Head, Mouth or other CAS panel. The UI placement is therefore never used as anatomical evidence.

When the internal name safely identifies anatomy, the destination is refined, for example:

```text
CAS\Sliders\Face\Eyes
CAS\Sliders\Face\Nose
CAS\Sliders\Face\Mouth & Lips
CAS\Sliders\Body\Arms
CAS\Sliders\Body\Legs
CAS\Sliders\Body\Shoulders
CAS\Sliders\Body\Waist
CAS\Sliders\Head\Hats
```

PT/ES use localized folder labels.

Specific anatomy wins over generic words. For example:

```text
"Nose Tip Height"   -> Face\Nose
"Shoulder Height"   -> Body\Shoulders
"Bloom_ArmTwist..." -> Body\Arms
"Bloom_LegLenght..."-> Body\Legs
```

If the internal name is too generic or opaque (for example `Tip Width` or `Outer Curve`), the package remains conservatively at:

```text
CAS\Sliders
```

No filename heuristic is used to force a subfolder.

Companion packages that provide shared slider labels for several anatomical regions also remain at `CAS\Sliders`, because assigning the companion to one region would split a real package set.

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
