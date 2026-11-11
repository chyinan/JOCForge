#!/usr/bin/env python3
"""Verify the limited D0 informational metadata and addbsi suffix vectors."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "eac3-dependent-d0-info-addbsi-segment-v1"
STATUS = "IMPLEMENTER_READY_LIMITED_D0_INFO_ADDBSI_SUFFIX"
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


def enc(fields: list[tuple[int, int]]) -> str:
    return "".join(format(value, f"0{width}b") for value, width in fields)


def encode_suffix(vector: dict) -> str:
    inputs = vector["inputs"]
    information = inputs["information"]
    addbsi_hex = inputs["addbsi_hex"]
    fields: list[tuple[int, int]] = []
    if information is not None:
        fields.extend([
            (information["bsmod"], 3),
            (int(information["copyrightb"]), 1),
            (int(information["origbs"]), 1),
            (int(information["audio_production"] is not None), 1),
        ])
        audio = information["audio_production"]
        if audio is not None:
            fields.extend([
                (audio["mixlevel"], 5),
                (audio["roomtyp"], 2),
                (audio["convtyp"], 1),
            ])
        fields.append((information["sourcefscod"], 1))
    fields.append((int(addbsi_hex is not None), 1))
    if addbsi_hex is not None:
        raw = bytes.fromhex(addbsi_hex)
        fields.append((len(raw) - 1, 6))
        fields.extend((byte, 8) for byte in raw)
    return enc(fields)


manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
manifest = json.loads(manifest_bytes)
seal = json.loads((ROOT / "PACKAGE-SEAL.json").read_text(encoding="utf-8"))
contract = json.loads((ROOT / "CONTRACT.json").read_text(encoding="utf-8"))
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
provenance = json.loads((ROOT / "PROVENANCE.json").read_text(encoding="utf-8"))

require(manifest.get("package_id") == PACKAGE_ID, "manifest package id")
require(manifest.get("status") == STATUS, "manifest limited status")
require(manifest.get("production_allowed") is False, "manifest production gate")
require(seal.get("manifest_sha256") == hashlib.sha256(manifest_bytes).hexdigest(), "manifest seal")
require(seal.get("production_allowed") is False, "seal production gate")
listed = {entry["path"] for entry in manifest.get("files", [])}
require(listed == FILES, "manifest file set")
actual = {
    path.name for path in ROOT.iterdir()
    if path.is_file() and path.name not in {"MANIFEST.json", "PACKAGE-SEAL.json"}
}
require(actual == FILES, "on-disk file set")
for entry in manifest["files"]:
    path = ROOT / entry["path"]
    require(path.stat().st_size == entry["size_bytes"], f"size {path.name}")
    require(digest(path) == entry["sha256"], f"hash {path.name}")

target = contract["target"]
require(target == {
    "stream_type": 1, "substream_id": 0, "sample_rate_hz": 48000,
    "fscod": 0, "numblkscod": 3, "audio_blocks": 6,
    "acmod": 5, "lfeon": 0, "chanmap": "0x1A00",
    "start_cursor_source": "cursor after parent mixmdate/block-mix/infomdate writer"
}, "target context")
require(contract["syntax"]["information_fields_if_present"][0]["name"] == "bsmod", "information field order")
require(contract["syntax"]["addbsi"]["addbsil_value"] == "byte_count - 1", "addbsi length semantics")
require(contract["syntax"]["addbsi"]["byte_count_range"] == [1, 64], "addbsi length range")
require(contract["production_allowed"] is False, "contract production gate")
require(vectors.get("execution") == "NOT_RUN", "vectors remain NOT_RUN")
require(vectors.get("contains_media_or_audio_payload") is False, "payload excluded")

by_id = {item["id"]: item for item in vectors["vectors"]}
for vector_id in ("infomdate-absent-no-addbsi", "informational-fields-and-two-byte-addbsi", "audio-production-fields-without-addbsi"):
    vector = by_id[vector_id]
    infomdate = vector["inputs"]["infomdate_present"]
    require(infomdate == (vector["inputs"]["information"] is not None), f"{vector_id} infomdate consistency")
    bits = encode_suffix(vector)
    require(bits == vector["expected_suffix_bits"], f"{vector_id} bits")
    require(len(bits) == vector["expected_suffix_bit_count"], f"{vector_id} bit count")
    require(vector["start_bit_offset"] + len(bits) == vector["expected_end_bit_offset"], f"{vector_id} end cursor")

capacity = by_id["capacity-and-range-rejections"]
require(capacity["available_bits"] == 25 and capacity["required_bits"] == 30, "capacity preflight vector")
require(capacity["invalid_addbsi_lengths"] == [0, 65], "addbsi rejection vectors")
require(provenance["production_allowed"] is False, "provenance gate")
require(provenance["standard"]["id"] == "ETSI TS 102 366 V1.4.1 (2017-09)", "standard identity")
print(f"PASS: {PACKAGE_ID}; {len(FILES)} payloads; D0 info/addbsi suffix only; production_allowed=false")
