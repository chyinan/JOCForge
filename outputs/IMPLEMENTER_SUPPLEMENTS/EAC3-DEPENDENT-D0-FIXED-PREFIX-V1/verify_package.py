#!/usr/bin/env python3
"""Verify the limited Flat7x D0 prefix contract and all package hashes."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "eac3-dependent-d0-fixed-prefix-v1"
STATUS = "IMPLEMENTER_READY_LIMITED_DEPENDENT_PREFIX"
FILES = {
    "CONTRACT.md",
    "CONTRACT.json",
    "INPUT-ONLY-VECTORS.json",
    "PROVENANCE.json",
    "build_manifest.py",
    "verify_package.py",
}
EXPECTED_BITS = "000010110111011101000111111111110011101010000"


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
require(scope["proposed_function"] == "write_flat7x_d0_fixed_prefix", "semantic helper name")
require(scope["preflight_bits"] == 45 and scope["end_cursor_bit_offset"] == 45, "prefix width")
require(scope["fields"][0]["value"] == "0x0B77", "syncword")
require(scope["fields"][1]["value"] == 1, "dependent strmtyp")
require(scope["fields"][3]["value"] == 2047, "D0 frmsiz")
require(scope["fields"][6]["value"] == 5 and scope["fields"][7]["value"] == 0, "D0 acmod/lfeon")
require(scope["fields"][8]["value"] == 16, "E-AC-3 bsid")
require(scope["chanmape_and_chanmap"].startswith("separate later"), "chanmap caller position")
require(contract["production_allowed"] is False, "contract production gate")

vector = vectors["vectors"][0]
require(vectors["execution"] == "NOT_RUN", "vectors remain input-only")
require(vector["expected_bit_string"] == EXPECTED_BITS, "fixed prefix vector")
require(vector["expected_bytes_hex_zero_initialized"] == "0b7747ff3a80", "fixed prefix bytes")
require(vector["expected_end_bit_offset"] == 45, "vector cursor")
capacity = vectors["vectors"][1]
require(capacity["id"] == "capacity-preflight-40-bits", "capacity vector label")
require(capacity["buffer_bytes"] * 8 == capacity["available_bits"] == 40, "capacity vector width")
require(capacity["available_bits"] < 45, "capacity vector rejects short buffer")
require(capacity["expected"] == "reject before mutating bytes or cursor", "capacity atomicity")
require(provenance["constraints"]["production_allowed"] is False, "provenance production gate")
print(f"PASS: {PACKAGE_ID}; {len(FILES)} payloads; 45-bit D0 prefix only; production_allowed=false")
