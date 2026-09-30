# Real corpus and performance acceptance (pre-build)

## Sources already supplied in the user's Library

These are original user-owned archives. Keep the ZIPs in the Library or local test machine; never commit them to GitHub.

| Corpus | Library filename | Stored ZIP bytes | Current status |
|---|---|---:|---|
| Sliders | #6 Sliders.zip | 2,212,950 | Previous byte-level analysis recorded in REAL-PACKAGE-VALIDATION.md |
| Store | #9 Store.zip | 387,532,385 | Found/materialized; index and payload census still requires execution |
| Mixed | Packages.zip | 439,679,809 | Found/materialized; index and payload census still requires execution |
| Additional | Packages(1).zip | 29,769,218 | Found/materialized; index and payload census still requires execution |

Do not infer the contents of the three pending ZIPs from their names alone. In particular, "Store" does not imply every file is an object/Buy catalog item.

The current corpus inspector produces a **resource/index census**, not a definitive category mapping. It intentionally does not assign anatomy, CAS category, Build/Buy subtype or game behavior from package filenames.

## Run a local read-only corpus census

After the ZIPs are available as local paths:

~~~bash
python tools/audit_package_corpora.py \
  "/path/#9 Store.zip" \
  "/path/Packages.zip" \
  "/path/Packages(1).zip" \
  --output reports/local-corpus/census.json

python -m unittest discover -s tools -p "test_*.py"
~~~

For Windows PowerShell, use one line and Windows filesystem paths instead of the Unix shell continuation syntax.

The auditor reads ZIP members into bounded temporary scratch files (RAM first, temporary storage for larger packages), inspects DBPF indexes and records per-family **evidence candidates**. It does **not** extract files to Mods, modify ZIPs, upload packages, use filenames as category proof or write any changes into a package. The output JSON stays local and is Git-ignored.

Safety limits (adjust only if needed):

- Maximum decompressed package size: 1 GiB by default;
- Maximum aggregate decompressed package bytes per run: 16 GiB by default;
- Maximum DBPF index size: 64 MiB;
- Suspicious/invalid indexes, unreadable members and skipped oversized files are reported separately, not silently classified.

## Deep classification review after census

For every family with unambiguous resource evidence, inspect actual decoded CASP/OBJD, XML/ITUN, morph/NMAP/STBL, S3SA or other defining payloads before introducing an automatic folder. A script-tuning bundle can contain several families; mixed files must remain Needs Review if no safe package-level family emerges.

Compare cross-ZIP SHA-256 identities. Distinguish exact duplicates from shared TGIs and genuine content variants using the existing Duplicates/Conflicts engines.

Add documented regression fixtures **from resource properties** before activating each new family. A ZIP's top-level folder name is only reference/ground truth for human review, never the classifier's input.

## Performance and data-integrity acceptance

Run the compiled application later on copies of the Sliders and larger mixed corpora, with the same Mods-root profile/rules across tests.

1. Record a cold scan and three warm scans; capture Scan.totalMs, package count and Needs Review count.
2. Run Duplicates, Conflicts and Dependencies, recording per-phase diagnostics, completeness/truncation flags and skipped large-resource counts.
3. Compare output sets and resource evidence across repeat scans: differences need a reason; language switches must not alter internal identities.
4. Select a sample of classified files in Planner; include mixed/unknowns in the sample to verify those remain blocked until manually reviewed.
5. Organize a disposable copy, then Restore and compare every original relative path, byte size and SHA-256.
6. Test Quarantine on disposable exact duplicates: occupied destination, tampered file, failed later move, interrupted journal and explicit recovery.
7. Stress UI list filtering/search with thousands of items; measure responsiveness separately from Rust scanner elapsed time.

The scanner now gathers slider morph IDs during the original package scan instead of opening all files for a second ID pass. The dependency analyzer bounds unusually large payloads and **discloses skipped resources** to prevent false impressions of completeness.

**Do not** mark pending ZIPs "analyzed" or claim an application performance improvement until the tests above run successfully with real bytes. Do not request a Tauri build or create build/release workflows as part of this document.
