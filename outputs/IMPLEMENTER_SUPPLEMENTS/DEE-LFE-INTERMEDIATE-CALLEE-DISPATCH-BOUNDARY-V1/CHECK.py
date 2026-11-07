from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-INTERMEDIATE-CALLEE-DISPATCH-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "5a5854ae6ac7576879046b3bb04aa667512944eaf57498d97f152f939b588658"
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
require(evidence.get("source_package", {}).get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "evidence source hash mismatch")
require(evidence.get("callee_dispatch", {}).get("pointee_access_before_first_helper") == "NONE_OBSERVED", "first-helper boundary mismatch")
require(evidence.get("callee_dispatch", {}).get("helper_read_write_extent") == "UNBOUND", "helper extent was overstated")
require(evidence.get("later_boundary", {}).get("buffer_content_continuity_after_helper") == "UNBOUND", "content continuity was overstated")
require(evidence.get("scope", {}).get("dee_execution") == "NOT_RUN", "DEE execution status mismatch")
require(evidence.get("scope", {}).get("pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(evidence.get("scope", {}).get("raw_listing_or_decompilation_included") is False, "raw-listing boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} helper_extent=UNBOUND")
