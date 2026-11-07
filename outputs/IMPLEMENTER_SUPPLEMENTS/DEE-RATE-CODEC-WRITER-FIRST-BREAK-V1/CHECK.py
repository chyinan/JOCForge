from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-RATE-CODEC-WRITER-FIRST-BREAK-V1"
SOURCE_CHECKSUM_LIST_SHA256 = "880719cd751c0b6ba88ce470c48232c75aa6c94232bebac28004ba4fd8282c77"
PAYLOAD_NAMES = ("CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {PACKAGE_ID}: {message}")


manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
require(manifest.get("source_checksum_list_sha256") == SOURCE_CHECKSUM_LIST_SHA256, "source checksum-list mismatch")
require(set(manifest.get("payloads", {})) == set(PAYLOAD_NAMES), "payload set mismatch")
for name in PAYLOAD_NAMES:
    raw = (ROOT / name).read_bytes()
    record = manifest["payloads"][name]
    require(record.get("bytes") == len(raw), f"byte count mismatch: {name}")
    require(record.get("sha256") == hashlib.sha256(raw).hexdigest(), f"SHA-256 mismatch: {name}")
require(evidence.get("rate_derived_config", {}).get("rate_code_input") == "param_1[10]", "rate code input mismatch")
require(evidence.get("writer_path", {}).get("rate_config_to_writer_state_pointer_join") == "UNBOUND", "writer-state join was overstated")
require(evidence.get("writer_path", {}).get("concrete_writer_target") == "UNBOUND_DYNAMIC", "writer target was overstated")
require(evidence.get("writer_path", {}).get("specific_bsi_bitrate_field_effect") == "NOT_ESTABLISHED", "BSI effect was overstated")
require(evidence.get("runtime_and_scope", {}).get("dee_execution") == "NOT_RUN", "DEE execution status mismatch")
require(evidence.get("runtime_and_scope", {}).get("pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("runtime_and_scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("checksum_list_sha256") == SOURCE_CHECKSUM_LIST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} writer_join=UNBOUND")
