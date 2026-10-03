# Mesh polycount analysis

The Manager exposes mesh/polycount information from the selected package under **Technical Details**.

## Supported resources

- CAS meshes: `GEOM (0x015A1849)`
- Object meshes: `MLOD (0x01D10F34)` and `MODL (0x01661233)`
- `NMAP` is used only as internal-name evidence to identify CAS LOD names such as LOD0/LOD1/LOD2/LOD3.

File names are never used as mesh/LOD evidence.

## GEOM

The parser reads:

- `NumVerts`;
- vertex-format widths;
- vertex data extent;
- face-point item widths;
- `NumFacePoints`.

The Sims 3 GEOM format uses three face points per triangle, therefore the displayed triangle count is `NumFacePoints / 3`.

If several GEOM resources exist but their internal LOD identity cannot be established safely, the Manager shows per-resource counts and deliberately avoids inventing one combined package polycount.

## MLOD / MODL

The parser locates the embedded `MLOD` block and reads each mesh group:

- vertex count;
- primitive count;
- primitive type;
- mesh flags;
- group name hash.

TriangleList, TriangleFan and TriangleStrip use the declared primitive count as triangle count. QuadList is converted to two triangles per primitive. Unsupported/ambiguous primitive types remain visible with an unavailable triangle total rather than being guessed.

Known resource-group conventions are displayed as:

- `MLOD / 0x00000000` → LOD 0 / High;
- `MODL / 0x00000001` → lower object LOD;
- `MLOD / 0x00010000` → high sun-shadow resource;
- `MLOD / 0x00010001` → low sun-shadow resource.

Shadow resources are reported separately and excluded from the visible LOD maximum. A normal mesh marked `ShadowCaster` is still counted.

## Safety / performance

Mesh parsing is on demand when the user opens Technical Details. This avoids decompressing every GEOM/MLOD/MODL during the initial Mods scan.

Decoded mesh resources larger than 256 MiB are skipped and surfaced as warnings.

The UI displays:

- highest safely-known visible triangle count;
- corresponding vertex count;
- totals grouped by known LOD;
- every parsed mesh resource;
- group count, internal name/TGI and warnings.

## Validation status

Synthetic Rust regression tests cover GEOM face counting, MLOD group counting, quad conversion, CAS internal LOD names, object LOD/shadow mapping and ShadowCaster handling.

These tests are committed but still require execution in the later Rust/Tauri validation stage. Real-package validation should compare selected results against S3PE/known mesh tools before a global Polycount filter is enabled.
