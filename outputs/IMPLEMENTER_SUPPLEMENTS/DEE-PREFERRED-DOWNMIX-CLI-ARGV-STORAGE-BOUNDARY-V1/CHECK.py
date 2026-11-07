from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-PREFERRED-DOWNMIX-CLI-ARGV-STORAGE-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "b614131e246adc9950a73dd469d7a39d46fabba9596a6ed1da10e9401181134f"
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
require(evidence.get("argv_to_app_entry", {}).get("target_reads_or_forwards_R8") == "NOT_OBSERVED", "argv R8 use was overstated")
require(evidence.get("typed_encoder_storage", {}).get("argv_to_boost_parse_store_notify_write") == "UNBOUND_STATIC", "argv storage write was overstated")
require(evidence.get("runtime_and_scope", {}).get("actual_child_argv") == "NOT_CAPTURED", "runtime argv was overstated")
require(evidence.get("runtime_and_scope", {}).get("dee_execution") == "NOT_RUN", "DEE execution status mismatch")
require(evidence.get("runtime_and_scope", {}).get("media_or_pcm_read") is False, "media boundary mismatch")
require(evidence.get("runtime_and_scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} argv_write=UNBOUND")
