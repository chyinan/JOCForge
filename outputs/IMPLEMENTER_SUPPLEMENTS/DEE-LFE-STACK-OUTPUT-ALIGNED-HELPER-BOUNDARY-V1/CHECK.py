from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-STACK-OUTPUT-ALIGNED-HELPER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "c74c838bc2467338dc2e4a5547b1a2056f1bb921d06b9d4e7c61098b501edb99"
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
require(evidence.get("static_stack_alignment", {}).get("selected_helper") == "FUN_1403174c0", "helper selection mismatch")
require(evidence.get("static_stack_alignment", {}).get("branch_type") == "STATIC_ABI_BASED_NOT_RUNTIME_CAPTURE", "branch evidence type mismatch")
require(evidence.get("alias_and_parity", {}).get("stack_output_to_original_RDI_alias") == "UNBOUND", "alias was overstated")
require(evidence.get("scope", {}).get("raw_listing_or_instruction_bytes_included") is False, "raw listing boundary mismatch")
require(evidence.get("scope", {}).get("pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} alias=UNBOUND")
