#!/usr/bin/env python3
"""Verify the bounded I0 AC-3 header contract, vectors, and package hashes."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "flat7x-i0-ac3-header-segment-v1"
STATUS = "IMPLEMENTER_READY_LIMITED_I0_AC3_HEADER_SEGMENT"
FILES = {
    "CONTRACT.md",
    "CONTRACT.json",
    "INPUT-ONLY-VECTORS.json",
    "PROVENANCE.json",
    "build_manifest.py",
    "verify_package.py",
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def pack_fields(fields: list[tuple[int, int]]) -> tuple[str, str]:
    bits = "".join(format(value, f"0{width}b") for value, width in fields)
    padded = bits + "0" * ((-len(bits)) % 8)
    raw = bytes(int(padded[offset : offset + 8], 2) for offset in range(0, len(padded), 8))
    return bits, raw.hex()


manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
manifest = json.loads(manifest_bytes)
seal = json.loads((ROOT / "PACKAGE-SEAL.json").read_text(encoding="utf-8"))
contract = json.loads((ROOT / "CONTRACT.json").read_text(encoding="utf-8"))
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
provenance = json.loads((ROOT / "PROVENANCE.json").read_text(encoding="utf-8"))

require(manifest.get("package_id") == PACKAGE_ID, "manifest identity")
require(manifest.get("status") == STATUS, "limited status")
require(manifest.get("production_allowed") is False, "manifest production gate")
require(seal.get("manifest_sha256") == hashlib.sha256(manifest_bytes).hexdigest(), "seal manifest hash")
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

target = contract["target"]
require(target["role"] == "I0" and target["codec"] == "legacy_ac3_core", "I0 AC-3 target")
require(target["sample_rate_hz"] == 48000 and target["nominal_rate_kbps"] == 640, "target rate")
require(target["frame_bytes"] == 2560 and target["audio_blocks"] == 6, "target frame extent")
require(target["fixed_fields"] == {"syncword": "0x0B77", "fscod": 0, "bsid": 6, "acmod": 7, "lfeon": 1}, "target syntax tuple")
require(target["allowed_frmsizecod"] == [36, 37], "allowed 640-kbps size codes")
require(contract["syntax"]["end_bit_without_compr"] == 62, "no-compr width")
require(contract["syntax"]["end_bit_with_compr"] == 70, "compr width")
require(contract["implementation_scope"]["writes_full_frame"] is False, "header-only scope")
require(contract["production_allowed"] is False, "contract production gate")
require(vectors.get("execution") == "NOT_RUN", "input vectors remain unexecuted")
require(vectors.get("contains_media_or_audio_payload") is False, "payload exclusion")

by_id = {item["id"]: item for item in vectors["vectors"]}
for vector_id in ("i0-640-frmsizecod36-no-compr", "i0-640-frmsizecod37-with-compr"):
    vector = by_id[vector_id]
    values = vector["inputs"]
    code = values["frmsizecod"]
    compr = values["compr"]
    fields = [
        (0x0B77, 16),
        (values["crc1"], 16),
        (0, 2),
        (code, 6),
        (6, 5),
        (values["bsmod"], 3),
        (7, 3),
        (values["cmixlev"], 2),
        (values["surmixlev"], 2),
        (1, 1),
        (values["dialnorm"], 5),
        (int(compr is not None), 1),
    ]
    if compr is not None:
        fields.append((compr, 8))
    bits, encoded = pack_fields(fields)
    require(code in [36, 37], f"{vector_id} size code")
    require(bits == vector["expected_bits"], f"{vector_id} bit vector")
    require(encoded == vector["expected_bytes_hex_zero_padded"], f"{vector_id} bytes")
    require(len(bits) == vector["expected_end_bit_offset"], f"{vector_id} cursor")
    require(vector["derived_frame_bytes_at_48khz"] == 2560, f"{vector_id} extent")

rejections = by_id["preflight-and-range-rejections"]
require(rejections["available_bits"] == 64 and rejections["required_bits_with_compr"] == 70, "capacity vector")
require(rejections["invalid_frmsizecod"] == [35, 38], "size-code rejection vector")
require(rejections["invalid_dialnorm"] == [0, 32], "dialnorm rejection vector")
require(provenance["production_allowed"] is False, "provenance gate")
require(provenance["sources"][2]["identifier"] == "ETSI TS 102 366 V1.4.1 (2017-09)", "standard source identity")
print(f"PASS: {PACKAGE_ID}; {len(FILES)} payloads; I0 AC-3 header segment only; production_allowed=false")
