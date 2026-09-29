# First scan milestone

This milestone is intentionally read-only. It does not move, rename or delete user files.

## What works in this milestone

1. Select a folder from the desktop UI.
2. Recursively find `.package` files.
3. Read Sims 3 DBPF 2.0 indexes, including all eight index-header common-field bits.
4. Read and decompress supported RefPack resources.
5. Detect CASP (`0x034AEECB`) and OBJD (`0x319E4F1D`) resources.
6. Read CASP catalog fields:
   - clothing type;
   - type flags;
   - age;
   - species;
   - gender;
   - clothing-category flags.
7. Read OBJD catalog fields:
   - room flags;
   - function category;
   - subcategory 1;
   - subcategory 2;
   - build category.
8. Normalize the game flags into user-facing categories.
9. Generate category/folder previews using the current UI language (EN/PT/ES).
10. Mark a package as:
    - Classified;
    - Mixed;
    - Needs Review;
    - Unknown;
    - Invalid.

## Conservative classification rule

The scanner does not infer a Buy/Build category from the package filename.

For CASP and OBJD, automatic destinations are based on parsed catalog fields. If a supported catalog resource cannot be decoded, it is surfaced for review.

A package with multiple catalog resources gets an automatic destination only when all decoded resources resolve to the same destination. Otherwise it is marked Mixed.

## Examples

English:

```text
CAS\Clothing\Female\YA-A\Top
Buy\Comfort\Sofas & Loveseats
Build\Windows
```

Português:

```text
CAS\Roupas\Feminino\Jovem Adulto-Adulto\Parte de Cima
Compra\Conforto\Sofás
Construção\Janelas
```

Español:

```text
CAS\Ropa\Femenino\Adulto Joven-Adulto\Parte Superior
Compra\Comodidad\Sofás
Construcción\Ventanas
```

## Not enabled yet

- moving packages into organized folders;
- duplicate fingerprinting;
- conflict analysis;
- restore execution.

Those operations build on the scan result and are intentionally kept disabled until real package samples validate the classification layer.
