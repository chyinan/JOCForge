from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-FLATTENED-MASTER-INNER-PROVIDER-FIRST-BREAK-V1"
SOURCE_MANIFEST_SHA256 = "e859bccf68ea0aaf1a0353652b4e3905f6fe7088f8e111043ce47d0a7965a5cf"
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
require(evidence.get("source_package", {}).get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "evidence source mismatch")
require(evidence.get("wrapper_handoff", {}).get("outer_slot_behavior") == "forward wrapper+0x60 to inner provider virtual slot +0xA0", "provider edge mismatch")
require(evidence.get("inner_provider_return", {}).get("concrete_inner_provider_target") == "UNBOUND", "inner target was overstated")
require(evidence.get("inner_provider_return", {}).get("qword_zero_to_wrapper_format_plus_0x58") == "UNBOUND", "owner join was overstated")
require(evidence.get("resolver_route", {}).get("runtime_branch_selection") == "NOT_OBSERVED", "runtime route was overstated")
require(evidence.get("scope", {}).get("dee_execution") == "NOT_RUN", "DEE execution status mismatch")
require(evidence.get("scope", {}).get("media_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(evidence.get("scope", {}).get("raw_listing_or_decompilation_included") is False, "raw listing boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} inner_provider=UNBOUND")
