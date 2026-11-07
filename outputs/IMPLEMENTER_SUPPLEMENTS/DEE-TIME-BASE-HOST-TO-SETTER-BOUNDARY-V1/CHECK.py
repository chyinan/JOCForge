from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-TIME-BASE-HOST-TO-SETTER-BOUNDARY-V1"
SOURCE_CHECKSUM_LIST_SHA256 = "ab65d0d6774e842e59d3dc9f85e33eb1aa66474635d91b270dc8cd0dd144c8dc"
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
require(evidence.get("host_path", {}).get("time_base_key_observed_on_this_call") is False, "time_base key was overstated")
require(evidence.get("typed_atmos_target_comparison", {}).get("host_setproperty_target") == "UNBOUND_DYNAMIC_OBJECT_VTABLE", "dynamic target boundary mismatch")
require(evidence.get("typed_atmos_target_comparison", {}).get("static_link_to_atmos_setter") == "NOT_FOUND", "setter join was overstated")
require(evidence.get("scope", {}).get("dee_or_xml_job_run") == "NOT_RUN", "runtime status mismatch")
require(evidence.get("scope", {}).get("media_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("checksum_list_sha256") == SOURCE_CHECKSUM_LIST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} host_join=UNBOUND")
