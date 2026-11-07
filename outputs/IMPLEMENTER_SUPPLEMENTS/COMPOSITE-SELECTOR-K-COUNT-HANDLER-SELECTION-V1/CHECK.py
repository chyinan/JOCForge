from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "COMPOSITE-SELECTOR-K-COUNT-HANDLER-SELECTION-V1"
SOURCE_MANIFEST_SHA256 = "82ec118eb0bc4bba8133bdcaadacf12c33f4c3b3fd86adcef6cb5419f179c516"
PAYLOAD_NAMES = ("CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {PACKAGE_ID}: {message}")


manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest binding mismatch")
require(set(manifest.get("payloads", {})) == set(PAYLOAD_NAMES), "payload set mismatch")
for name in PAYLOAD_NAMES:
    path = ROOT / name
    raw = path.read_bytes()
    record = manifest["payloads"][name]
    require(record.get("bytes") == len(raw), f"byte count mismatch: {name}")
    require(record.get("sha256") == hashlib.sha256(raw).hexdigest(), f"SHA-256 mismatch: {name}")
require(evidence.get("source_package", {}).get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "evidence source hash mismatch")
require(evidence.get("source_package", {}).get("separate_package_seal") == "NOT_PRESENT", "upstream seal status mismatch")
require(evidence.get("handler_selection", {}).get("actual_suffix_for_this_j_k") == "NOT_CAPTURED", "actual suffix must remain unobserved")
require(evidence.get("handler_selection", {}).get("selected_storage_handler") == "UNBOUND", "handler must remain unbound")
require(evidence.get("candidate_k_count_rule", {}).get("applicability_to_this_j_plus_0x08") == "UNBOUND_PENDING_SELECTED_HANDLER", "K-count applicability was overstated")
require(evidence.get("first_break", {}).get("count_writer_for_selected_handler") == "NOT_REACHED", "writer reach was overstated")
require(evidence.get("scope", {}).get("native_execution") == "NOT_RUN", "native execution status mismatch")
require(evidence.get("scope", {}).get("pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(evidence.get("scope", {}).get("raw_native_source_or_disassembly_included") is False, "raw-source boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} selected_handler=UNBOUND K_count_applicability=UNBOUND")
