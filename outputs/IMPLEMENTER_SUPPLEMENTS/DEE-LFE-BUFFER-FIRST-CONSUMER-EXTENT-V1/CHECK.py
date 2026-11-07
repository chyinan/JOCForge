from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-BUFFER-FIRST-CONSUMER-EXTENT-V1"
SOURCE_MANIFEST_SHA256 = "c13bad284c9b330e1fb70823a51a91ddc4ee4af874f9432a09dea4d2c723f751"
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
extent = evidence.get("input_extent", {})
require(extent.get("read_bytes") == 16 and len(extent.get("direct_rcx_reads", [])) == 4, "input extent mismatch")
require(extent.get("direct_store_through_rcx") is False, "buffer mutation status mismatch")
require(evidence.get("later_lfe_path", {}).get("buffer_content_at_e350") == "UNBOUND", "content continuity was overstated")
require(evidence.get("scope", {}).get("numeric_parity") == "NOT_RUN" or evidence.get("later_lfe_path", {}).get("numeric_parity") == "NOT_RUN", "parity status mismatch")
require(evidence.get("scope", {}).get("media_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} input_read_bytes=16")
