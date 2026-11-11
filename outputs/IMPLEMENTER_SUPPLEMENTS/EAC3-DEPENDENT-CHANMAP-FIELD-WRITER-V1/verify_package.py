from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
EXPECTED = {
    "CONTRACT.md",
    "CONTRACT.json",
    "INPUT-ONLY-VECTORS.json",
    "PROVENANCE.json",
    "build_manifest.py",
    "verify_package.py",
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest_path = ROOT / "MANIFEST.json"
seal_path = ROOT / "PACKAGE-SEAL.json"
manifest_bytes = manifest_path.read_bytes()
manifest = json.loads(manifest_bytes)
seal = json.loads(seal_path.read_text(encoding="utf-8"))
require(manifest["package_id"] == "eac3-dependent-chanmap-field-writer-v1", "package id")
require(manifest["status"] == "IMPLEMENTER_READY_LIMITED_FIELD_SERIALIZER", "package scope")
require(manifest["production_allowed"] is False, "end-to-end production gate")
require(seal["manifest_sha256"] == hashlib.sha256(manifest_bytes).hexdigest(), "manifest seal")
require(seal["production_allowed"] is False, "seal production gate")
listed = {entry["path"] for entry in manifest["files"]}
require(listed == EXPECTED, "manifest file set")
actual = {
    path.name
    for path in ROOT.iterdir()
    if path.is_file() and path.name not in {"MANIFEST.json", "PACKAGE-SEAL.json"}
}
require(actual == EXPECTED, "on-disk package file set")
for entry in manifest["files"]:
    path = ROOT / entry["path"]
    require(path.stat().st_size == entry["size_bytes"], f"size: {path.name}")
    require(digest(path) == entry["sha256"], f"hash: {path.name}")
contract = json.loads((ROOT / "CONTRACT.json").read_text(encoding="utf-8"))
scope = contract["implementation_scope"]
require(scope["target_map_u16"] == "0x1A00", "target map")
require(scope["target_map_bits_msb_indexed"] == [3, 4, 6], "map bits")
require(scope["bit_sequence_at_offset_zero"] == "10001101000000000", "field bits")
require(scope["limited_field_serializer_allowed"] is True, "limited implementation scope")
require(contract["end_to_end_production_allowed"] is False, "contract production gate")
require(contract["evidence_classes"]["DEE_record_5c_to_writer_join"] == "UNBOUND", "native boundary")
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
require(vectors["execution"] == "NOT_RUN", "vectors must stay input-only")
require(vectors["vectors"][0]["chanmap_u16"] == "0x1A00", "target vector")
require(len(vectors["vectors"][3]["cases"]) == 16, "one-hot coverage")
print(f"PASS package={manifest['package_id']} files={len(listed)} production_allowed=false")
