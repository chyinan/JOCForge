from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-DI-DIALOGUE-BOOL-STATIC-KEY-OWNER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "f2192a5956fab3a03be35d944ac1b28c767b9bf59f41e29674a911193d01d326"
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
require(evidence.get("parameters_container", {}).get("match_rule") == "memcmp over shorter length plus equal-length check", "getter comparison rule mismatch")
require(evidence.get("dialogue_intelligence_candidate", {}).get("static_exact_key_to_bool_result") == "BOUND", "static key binding mismatch")
require(evidence.get("runtime_and_effect", {}).get("dispatcher_runtime_key") == "NOT_CAPTURED", "runtime key selection was overstated")
require(evidence.get("runtime_and_effect", {}).get("audio_or_joc_effect") == "UNBOUND", "audio effect was overstated")
require(evidence.get("scope", {}).get("audio_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} runtime_selection=UNBOUND")
