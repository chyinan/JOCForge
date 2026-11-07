from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-TIME-BASE-DEFAULT-SETTER-PRODUCER-V1"
SOURCE_CHECKSUM_LIST_SHA256 = "6c8c4d072a0157efdc4ccd8491da38ca095d4e86d9e5b961d8fffb5bc964f4ba"
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
require(evidence.get("constructor_producer", {}).get("argument_source") == "FUN_180136220 supplies static default file_position", "default producer mismatch")
require(evidence.get("constructor_producer", {}).get("selected_job_xml_value_to_setter_argument") == "UNBOUND", "XML-to-setter join was overstated")
require(evidence.get("constructor_producer", {}).get("host_setproperty_virtual_target_to_setter") == "UNBOUND", "host setter join was overstated")
require(evidence.get("scope", {}).get("dee_execution") == "NOT_RUN", "DEE execution status mismatch")
require(evidence.get("scope", {}).get("media_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("checksum_list_sha256") == SOURCE_CHECKSUM_LIST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} selected_XML_value=UNBOUND")
