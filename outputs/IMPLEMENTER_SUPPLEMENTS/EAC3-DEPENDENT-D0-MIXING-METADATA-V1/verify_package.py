#!/usr/bin/env python3
"""Verify the limited Flat7x D0 mixing-metadata segment contract and vectors."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "eac3-dependent-d0-mixing-metadata-v1"
STATUS = "IMPLEMENTER_READY_LIMITED_D0_MIXING_INFO_SEGMENT"
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


def encode_segment(inputs: dict) -> str:
    fields: list[tuple[int, int]] = []
    mixing = inputs["mixing"]
    fields.append((int(mixing is not None), 1))
    if mixing is not None:
        fields.extend([
            (mixing["mixmdo"], 2),
            (mixing["ltrtcmixlev"], 3),
            (mixing["lorocmixlev"], 3),
            (mixing["ltrtsurmixlev"], 3),
            (mixing["lorosurmixlev"], 3),
        ])
    blocks = inputs["block_mix"]
    block_mixing_present = any(value is not None for value in blocks)
    fields.append((int(block_mixing_present), 1))
    if block_mixing_present:
        for value in blocks:
            fields.append((int(value is not None), 1))
            if value is not None:
                fields.append((value, 5))
    fields.append((int(inputs["infomdate"]), 1))
    return "".join(format(value, f"0{width}b") for value, width in fields)


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
    "role": "D0", "strmtyp": 1, "acmod": 5, "lfeon": 0,
    "numblkscod": 3, "audio_blocks": 6, "infomdate_data": "caller_owned"
}, "target conditions")
require(contract["syntax"]["start_cursor_bits"] == [68, 76], "parent cursors")
require(contract["syntax"]["end_boundary"] == "immediately after infomdate", "end boundary")
require(contract["production_allowed"] is False, "contract production gate")
require(vectors.get("execution") == "NOT_RUN", "vectors remain NOT_RUN")
require(vectors.get("contains_media_or_audio_payload") is False, "payload excluded")

by_id = {item["id"]: item for item in vectors["vectors"]}
for vector_id in ("no-mixing-no-block-info", "mixing-present-infomdate-present", "block-mix-values-without-mixing"):
    vector = by_id[vector_id]
    bits = encode_segment(vector["inputs"])
    require(bits == vector["expected_segment_bits"], f"{vector_id} bits")
    require(len(bits) == vector["expected_segment_bit_count"], f"{vector_id} bit count")
    require(vector["start_bit_offset"] + len(bits) == vector["expected_end_bit_offset"], f"{vector_id} end cursor")

capacity = by_id["preflight-and-range-rejections"]
require(capacity["buffer_bytes"] == 10, "capacity vector buffer size")
require(capacity["available_bits"] == 12 and capacity["required_bits"] == 17, "capacity vector")
require(capacity["invalid_mixmdo"] == [4], "mixmdo rejection vector")
require(capacity["invalid_mix_levels"] == [8], "mix-level rejection vector")
require(capacity["invalid_block_mix_values"] == [32], "block-value rejection vector")
require(capacity["invalid_start_offsets"] == [67, 69], "cursor rejection vector")
require(provenance["production_allowed"] is False, "provenance gate")
require(provenance["standard"]["id"] == "ETSI TS 102 366 V1.4.1 (2017-09)", "official standard identity")
require(provenance["standard"]["clauses"] == ["E.1.2.2", "E.1.2.0"], "standard clauses")
print(f"PASS: {PACKAGE_ID}; {len(FILES)} payloads; D0 mixing/infomdate flags only; production_allowed=false")
