from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-OUTPUT-STACK-CLAMP-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "08564bc7c55924ad0d169a8fe9f6e27ca510e6c325af5d24c26abdea572454d6"
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
require(evidence.get("caller_output_buffer", {}).get("element_count") == 256, "buffer count mismatch")
require(evidence.get("post_return_consumer", {}).get("direct_write_to_rdi_original_input") is False, "input mutation status mismatch")
require(evidence.get("alias_and_continuity", {}).get("stack_arg7_to_rdi_original_input_alias") == "UNBOUND", "buffer alias was overstated")
require(evidence.get("alias_and_continuity", {}).get("e350_content_continuity") == "UNBOUND", "content continuity was overstated")
require(evidence.get("scope", {}).get("numeric_parity") == "NOT_RUN" or evidence.get("alias_and_continuity", {}).get("numeric_parity") == "NOT_RUN", "parity status mismatch")
require(evidence.get("scope", {}).get("pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} clamp_extent=256_F32")
