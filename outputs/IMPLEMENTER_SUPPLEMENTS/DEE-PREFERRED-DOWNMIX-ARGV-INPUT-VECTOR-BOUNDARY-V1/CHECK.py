from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-PREFERRED-DOWNMIX-ARGV-INPUT-VECTOR-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "6b27f90541a950ac46e4f6e466de2444d58074bff19aa5621837e09e7b0da344"
PAYLOAD_NAMES = ("CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {PACKAGE_ID}: {message}")


manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source hash mismatch")
require(set(manifest.get("payloads", {})) == set(PAYLOAD_NAMES), "payload set mismatch")
for name in PAYLOAD_NAMES:
    raw = (ROOT / name).read_bytes()
    record = manifest["payloads"][name]
    require(record.get("bytes") == len(raw), f"byte count mismatch: {name}")
    require(record.get("sha256") == hashlib.sha256(raw).hexdigest(), f"SHA-256 mismatch: {name}")
vector = evidence.get("typed_input_vector", {})
require(vector.get("member_offset") == "0x318", "input vector owner field mismatch")
require(vector.get("option_role") == "repeatable input-file path list", "input vector role mismatch")
require(vector.get("consumer_splits_colon_or_equal_fields") is False, "consumer delimiter behavior mismatch")
require(evidence.get("encoder_composite_boundary", {}).get("actual_child_argv") == "NOT_CAPTURED", "runtime argv was overstated")
require(evidence.get("encoder_composite_boundary", {}).get("delimiter_splitter") == "UNBOUND", "encoder delimiter boundary was overstated")
require(evidence.get("scope", {}).get("dee_execution") == "NOT_RUN", "DEE execution status mismatch")
require(evidence.get("scope", {}).get("media_or_pcm_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} encoder_join=UNBOUND")
