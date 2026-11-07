from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-PREFERRED-DOWNMIX-STAGE-CONSUMER-FIRST-BREAK-V1"
SOURCE_MANIFEST_SHA256 = "e66c7c13900ddec00aa0f5353e75dbe1ec9d2a648b6216252a3193e730c9318b"
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
require(evidence.get("typed_reader", {}).get("formatter_exact_entry_direct_callers") == 0, "formatter caller boundary mismatch")
require(evidence.get("first_per_source_candidate", {}).get("typed_join_to_mixmode_owner") == "UNBOUND", "owner join was overstated")
require(evidence.get("first_break", {}).get("audio_consumer") == "UNBOUND", "audio consumer was overstated")
require(evidence.get("scope", {}).get("dee_or_xml_job_run") == "NOT_RUN", "runtime status mismatch")
require(evidence.get("scope", {}).get("media_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} audio_consumer=UNBOUND")
