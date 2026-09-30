#!/usr/bin/env python3
"""Read-only Sims 3 .package corpus census for ZIPs previously supplied by users.

Uses only Python's standard library. Does NOT classify by filename, extract
packages into their Mods folders, repair packages, or upload corpus contents.
Resource-type summaries are EVIDENCE CANDIDATES, never automatic classifications.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import struct
import tempfile
import zipfile

MAGIC = b"DBPF"
TYPE_LABELS = {
    0x034AEECB: "CASP",  # Confirmed catalog type; unknown types remain hexadecimal.
    0x319E4F1D: "OBJD",
    0x0166038C: "NMAP",
    0x00B2D882: "IMG",
    0x0333406C: "XML",
    0x0354796A: "SkinTone",
    0x03555BA8: "HairTone",
    0x0355E0A6: "BoneDelta",
    0x0358B08A: "FACE",
    0x03B33DDF: "ITUN",
    0x062C8204: "BBLN",
    0x067CAA11: "BGEO",
    0x073FAA07: "S3SA",
    0x220557DA: "STBL",
    0xB52F5055: "FBLN",
    0xD4D9FBE5: "Pattern",
    0x015A1849: "GEOM",
    0x736884F1: "VPXY",
    0x73E93EEB: "Manifest",
}
MORPHS = {0x0355E0A6, 0x0358B08A, 0x062C8204, 0x067CAA11, 0xB52F5055}
CATALOG = {0x034AEECB, 0x319E4F1D}


class PackageIndexError(ValueError):
    pass


def read_index(stream, size: int) -> Counter:
    """Validate the DBPF v2/v3 index and return a resource-type histogram."""
    header = stream.read(96)
    if len(header) != 96 or header[:4] != MAGIC:
        raise PackageIndexError("missing or truncated DBPF header")
    major, = struct.unpack_from("<I", header, 4)
    count, = struct.unpack_from("<I", header, 36)
    index_length, = struct.unpack_from("<I", header, 44)
    index_version, index_position = struct.unpack_from("<II", header, 60)
    if major != 2 or index_version != 3:
        raise PackageIndexError(f"unsupported DBPF/index version: {major}/{index_version}")
    if count > 2_000_000:
        raise PackageIndexError("index resource count exceeds safety limit")
    if not count:
        return Counter()
    if index_position < 96 or index_length > 64 * 1024 * 1024:
        raise PackageIndexError("index location/length is invalid")
    if index_position + index_length > size:
        raise PackageIndexError("index exceeds package length")
    stream.seek(index_position)
    data = stream.read(index_length)
    if len(data) != index_length or len(data) < 4:
        raise PackageIndexError("truncated DBPF index")
    flags, = struct.unpack_from("<I", data, 0)
    if flags & ~0xFF:
        raise PackageIndexError("unsupported DBPF index field mask")
    common = flags.bit_count()
    expected = 4 + 4 * common + 4 * (8 - common) * count
    if expected != index_length:
        raise PackageIndexError(f"index length mismatch: {index_length} != {expected}")

    offset = 4
    template = [0] * 8
    for field in range(8):
        if flags & (1 << field):
            template[field], = struct.unpack_from("<I", data, offset)
            offset += 4
    types = Counter()
    for _ in range(count):
        record = template.copy()
        for field in range(8):
            if not flags & (1 << field):
                record[field], = struct.unpack_from("<I", data, offset)
                offset += 4
        resource_type = record[0]
        chunk_offset = record[4]
        file_size = record[5] & 0x7FFF_FFFF
        if chunk_offset + file_size > size:
            raise PackageIndexError("resource payload extends beyond package")
        types[resource_type] += 1
    return types


def resource_evidence(types: Counter) -> list[str]:
    """Resource-only candidate families; final app classifier must verify them."""
    present = set(types)
    evidence = []
    if 0x073FAA07 in present:
        evidence.append("script_resource")
    if present & MORPHS and not present & CATALOG:
        evidence.append("slider_morph_candidate")
    if 0x0354796A in present:
        evidence.append("skin_tone_resource")
    if 0x03555BA8 in present:
        evidence.append("hair_tone_resource")
    if 0xD4D9FBE5 in present:
        evidence.append("pattern_resource")
    if present & {0x03B33DDF, 0x0333406C}:
        evidence.append("tuning_resource")
    if present == {0x220557DA} or (
        0x220557DA in present and present <= {0x220557DA, 0x0166038C, 0x73E93EEB}
    ):
        evidence.append("stbl_only_resource")
    if present & CATALOG:
        evidence.append("catalog_resources_need_payload_analysis")
    if not evidence:
        evidence.append("needs_resource_review")
    return evidence


def audit_archive(path: Path, max_file_bytes: int, remaining_budget: list[int],
                  all_hashes: dict[str, list[str]]) -> dict:
    report = {
        "archive": path.name,
        "compressed_bytes": path.stat().st_size,
        "package_count": 0,
        "valid_indexes": 0,
        "invalid_indexes": [],
        "skipped_oversize": [],
        "resource_types": Counter(),
        "candidate_evidence": Counter(),
        "nonpackage_members": 0,
        "total_package_uncompressed_bytes": 0,
    }
    with zipfile.ZipFile(path) as z:
        for item in z.infolist():
            if item.is_dir() or not item.filename.lower().endswith(".package"):
                report["nonpackage_members"] += 1
                continue
            report["package_count"] += 1
            if item.file_size > max_file_bytes or item.file_size > remaining_budget[0]:
                report["skipped_oversize"].append({
                    "member": item.filename,
                    "uncompressed_bytes": item.file_size,
                    "reason": "file or global decompression budget",
                })
                continue
            remaining_budget[0] -= item.file_size
            sha = hashlib.sha256()
            actual = 0
            try:
                with z.open(item) as member, tempfile.SpooledTemporaryFile(max_size=8 * 1024 * 1024) as scratch:
                    while block := member.read(1024 * 1024):
                        actual += len(block)
                        if actual > item.file_size or actual > max_file_bytes:
                            raise PackageIndexError("decompressed member exceeded declared budget")
                        sha.update(block)
                        scratch.write(block)
                    if actual != item.file_size:
                        raise PackageIndexError("uncompressed file size mismatch")
                    report["total_package_uncompressed_bytes"] += actual
                    digest = sha.hexdigest()
                    all_hashes[digest].append(f"{path.name}::{item.filename}")
                    scratch.seek(0)
                    types = read_index(scratch, actual)
            except (OSError, EOFError, zipfile.BadZipFile, PackageIndexError, RuntimeError) as error:
                report["invalid_indexes"].append({
                    "member": item.filename,
                    "error": str(error),
                })
                continue
            report["valid_indexes"] += 1
            report["resource_types"].update(types)
            report["candidate_evidence"].update(resource_evidence(types))
    for key in ("resource_types", "candidate_evidence"):
        if key == "resource_types":
            report[key] = {
                TYPE_LABELS.get(t, f"0x{t:08X}"): n
                for t, n in sorted(report[key].items())
            }
        else:
            report[key] = dict(sorted(report[key].items()))
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archives", type=Path, nargs="+", help="Original corpus ZIPs")
    parser.add_argument("--output", type=Path, required=True, help="Local JSON report path")
    parser.add_argument("--max-file-mb", type=int, default=1024)
    parser.add_argument("--max-total-gb", type=int, default=16)
    args = parser.parse_args()
    remaining = [args.max_total_gb * 1024**3]
    all_hashes: dict[str, list[str]] = defaultdict(list)
    archives = []
    for path in args.archives:
        if not path.is_file() or not zipfile.is_zipfile(path):
            parser.error(f"Not a readable ZIP file: {path}")
        archives.append(audit_archive(path, args.max_file_mb * 1024**2,
                                      remaining, all_hashes))
    duplicates = [
        {"sha256": sha, "occurrences": paths}
        for sha, paths in all_hashes.items() if len(paths) > 1
    ]
    result = {
        "version": 1,
        "analysis": "read_only_dbpf_index_census_not_final_classification",
        "archives": archives,
        "cross_corpus_byte_duplicate_groups": duplicates,
        "remaining_budget_bytes": remaining[0],
        "notes": [
            "Resource-type evidence alone does not prove anatomic or CAS placement.",
            "No package content is uploaded, written back or extracted to Mods.",
            "STBL/NMAP internal-name and CASP/OBJD payload analysis require deeper validation.",
        ],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Audited {len(archives)} ZIP(s). Saved {args.output}")
    for archive in archives:
        print(f"{archive['archive']}: {archive['package_count']} packages, "
              f"{archive['valid_indexes']} valid, {len(archive['invalid_indexes'])} invalid, "
              f"{len(archive['skipped_oversize'])} skipped")
    print(f"Cross-corpus byte-identical duplicate groups: {len(duplicates)}")


if __name__ == "__main__":
    main()
