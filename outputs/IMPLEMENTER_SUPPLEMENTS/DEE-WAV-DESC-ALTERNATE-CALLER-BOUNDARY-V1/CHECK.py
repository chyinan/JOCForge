from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-WAV-DESC-ALTERNATE-CALLER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "1fb3a897112c44d42e9c32037872e0b9de2f7c6c4003e74e6bb57651e4ac9e73"
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
require(evidence.get("alternate_constructor_caller", {}).get("post_constructor_write", {}).get("field") == "Desc+0x38", "post-constructor field mismatch")
require(evidence.get("alternate_constructor_caller", {}).get("post_constructor_write_to_desc_plus_0x68") == "NOT_FOUND_IN_THIS_FUNCTION", "limited scope mismatch")
require(evidence.get("alternate_constructor_caller", {}).get("selected_selector3_descriptor_owner_join") == "UNBOUND", "owner join was overstated")
require(evidence.get("scope", {}).get("dee_execution") == "NOT_RUN", "DEE execution status mismatch")
require(evidence.get("scope", {}).get("media_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_json_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} alternate_instance_join=UNBOUND")
