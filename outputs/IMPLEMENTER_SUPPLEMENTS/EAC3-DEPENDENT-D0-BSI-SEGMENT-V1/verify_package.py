#!/usr/bin/env python3
"""Verify the limited D0 BSI-through-chanmap contract and package hashes."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "eac3-dependent-d0-bsi-segment-v1"
STATUS = "IMPLEMENTER_READY_LIMITED_D0_BSI_SEGMENT"
FILES = {
    "CONTRACT.md",
    "CONTRACT.json",
    "INPUT-ONLY-VECTORS.json",
    "PROVENANCE.json",
    "build_manifest.py",
    "verify_package.py",
}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
manifest = json.loads(manifest_bytes)
seal = json.loads((ROOT / "PACKAGE-SEAL.json").read_text(encoding="utf-8"))
contract = json.loads((ROOT / "CONTRACT.json").read_text(encoding="utf-8"))
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
provenance = json.loads((ROOT / "PROVENANCE.json").read_text(encoding="utf-8"))

require(manifest.get("package_id") == PACKAGE_ID, "manifest package identity")
require(manifest.get("status") == STATUS, "manifest limited scope")
require(manifest.get("production_allowed") is False, "manifest production gate")
require(seal.get("manifest_sha256") == hashlib.sha256(manifest_bytes).hexdigest(), "manifest seal")
require(seal.get("production_allowed") is False, "seal production gate")
listed = {entry["path"] for entry in manifest.get("files", [])}
require(listed == FILES, "manifest file set")
actual = {
    path.name
    for path in ROOT.iterdir()
    if path.is_file() and path.name not in {"MANIFEST.json", "PACKAGE-SEAL.json"}
}
require(actual == FILES, "on-disk file set")
for entry in manifest["files"]:
    path = ROOT / entry["path"]
    require(path.stat().st_size == entry["size_bytes"], f"size {path.name}")
    require(digest(path) == entry["sha256"], f"hash {path.name}")

scope = contract["implementation_scope"]
require(scope["proposed_function"] == "write_flat7x_d0_bsi_through_chanmap", "semantic function name")
require(scope["target_values"]["acmod"] == 5 and scope["target_values"]["lfeon"] == 0, "D0 target acmod/lfeon")
require(scope["target_values"]["chanmap"] == "0x1A00" and scope["target_values"]["chanmape"] == 1, "D0 target map")
require(scope["dialnorm_range"] == [1, 31], "valid dialnorm values")
require(scope["segment_bits_without_compr"] == 68 and scope["segment_bits_with_compr"] == 76, "segment lengths")
require(contract["production_allowed"] is False, "contract production gate")

require(vectors.get("execution") == "NOT_RUN", "vectors remain input-only")
vectors_by_id = {item["id"]: item for item in vectors["vectors"]}
no_compr = vectors_by_id["d0-bsi-through-chanmap-no-compr"]
with_compr = vectors_by_id["d0-bsi-through-chanmap-with-compr"]
capacity = vectors_by_id["full-segment-capacity-preflight"]
invalid = vectors_by_id["invalid-dialnorm"]
require(no_compr["expected_end_bit_offset"] == 68, "no-compr vector width")
require(no_compr["expected_bytes_hex_zero_initialized"] == "0b7747ff3a8411a000", "no-compr bytes")
require(with_compr["expected_end_bit_offset"] == 76, "compr vector width")
require(with_compr["expected_bytes_hex_zero_initialized"] == "0b7747ff3a8574b1a000", "compr bytes")
require(capacity["available_bits"] == 64 and capacity["required_bits"] == 68, "atomic preflight vector")
require(invalid["dialnorm_values"] == [0, 32], "reserved and out-of-range dialnorm vectors")
require(provenance["constraints"]["production_allowed"] is False, "provenance production gate")
print(f"PASS: {PACKAGE_ID}; {len(FILES)} payloads; D0 BSI segment only; production_allowed=false")
