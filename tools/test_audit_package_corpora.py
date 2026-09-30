#!/usr/bin/env python3
"""Regression tests for the read-only ZIP/DBPF corpus auditor."""
import io
from pathlib import Path
import struct
import tempfile
import unittest
import zipfile
from collections import defaultdict

from audit_package_corpora import (
    PackageIndexError, audit_archive, read_index, resource_evidence,
)


def synthetic_package(resource_type=0x034AEECB):
    header = bytearray(96)
    header[:4] = b"DBPF"
    struct.pack_into("<I", header, 4, 2)    # major
    struct.pack_into("<I", header, 36, 1)   # index count
    struct.pack_into("<I", header, 44, 36)  # 4 flags + 8 dwords
    struct.pack_into("<I", header, 60, 3)   # index v3
    struct.pack_into("<I", header, 64, 96)  # index position
    index = struct.pack("<I8I", 0, resource_type, 0, 0, 1, 132, 4, 4, 0)
    return bytes(header) + index + b"DATA"


class CorpusAuditTests(unittest.TestCase):
    def test_valid_dbpf_index(self):
        data = synthetic_package()
        counts = read_index(io.BytesIO(data), len(data))
        self.assertEqual(counts[0x034AEECB], 1)

    def test_reject_invalid_payload_range(self):
        data = bytearray(synthetic_package())
        struct.pack_into("<I", data, 96 + 4 + 4 * 4, 0x7FFFFFFF)
        with self.assertRaises(PackageIndexError):
            read_index(io.BytesIO(data), len(data))

    def test_classifier_evidence_never_uses_filename(self):
        count = read_index(io.BytesIO(synthetic_package(0x067CAA11)),
                           len(synthetic_package(0x067CAA11)))
        self.assertIn("slider_morph_candidate", resource_evidence(count))

    def test_zip_processed_in_memory_without_extracting(self):
        with tempfile.TemporaryDirectory() as temp:
            folder = Path(temp)
            archive = folder / "collection.zip"
            data = synthetic_package()
            with zipfile.ZipFile(archive, "w") as out:
                out.writestr("CAS/one.package", data, compress_type=zipfile.ZIP_DEFLATED)
                out.writestr("CAS/two.package", data, compress_type=zipfile.ZIP_DEFLATED)
                out.writestr("notes.txt", "ignore")
            hashes = defaultdict(list)
            report = audit_archive(archive, 1024 * 1024, [1024 * 1024], hashes)
            self.assertEqual(report["valid_indexes"], 2)
            self.assertEqual(report["nonpackage_members"], 1)
            self.assertEqual(len([k for k, v in hashes.items() if len(v) == 2]), 1)
            self.assertFalse((folder / "CAS").exists())

    def test_budget_blocks_oversized_member(self):
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp) / "collection.zip"
            with zipfile.ZipFile(archive, "w") as out:
                out.writestr("oversized.package", synthetic_package())
            report = audit_archive(archive, 8, [1024], defaultdict(list))
            self.assertEqual(report["valid_indexes"], 0)
            self.assertEqual(len(report["skipped_oversize"]), 1)


if __name__ == "__main__":
    unittest.main()
