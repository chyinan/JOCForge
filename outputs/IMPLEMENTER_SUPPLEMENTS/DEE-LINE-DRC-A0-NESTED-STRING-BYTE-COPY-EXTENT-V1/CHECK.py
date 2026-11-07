from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LINE-DRC-A0-NESTED-STRING-BYTE-COPY-EXTENT-V1"
SOURCE_MANIFEST_SHA256 = "31a411a8ab940f1e2f4dd3fe5faef02144ffaf7479073fb755c88f926532099d"
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
require(evidence.get("generic_copy", {}).get("helper") == "FUN_18059a7e0", "copy helper mismatch")
require(evidence.get("generic_copy", {}).get("record_type") == "generic string-like object; not codec/BSI", "type classification mismatch")
require(evidence.get("audio_boundary", {}).get("snapshot_plus_0xa0_join") == "UNBOUND", "snapshot join was overstated")
require(evidence.get("audio_boundary", {}).get("codec_or_bsi_owner") == "UNBOUND", "codec/BSI owner was overstated")
require(evidence.get("scope", {}).get("dee_execution") == "NOT_RUN", "runtime status mismatch")
require(evidence.get("scope", {}).get("media_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} codec_bsi=UNBOUND")
