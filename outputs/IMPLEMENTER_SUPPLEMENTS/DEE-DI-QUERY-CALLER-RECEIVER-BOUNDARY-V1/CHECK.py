from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-DI-QUERY-CALLER-RECEIVER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "a13c0bfe74eb2b2f3d5c0816e0c7c19f54cdb7ecb31d9755541949a7890ad024"
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
require(evidence.get("getter_call", {}).get("container_receiver") == "Priv+0xec0 ParametersContainer", "receiver field mismatch")
require(evidence.get("getter_call", {}).get("rdx_preserved") is True, "query argument preservation mismatch")
require(evidence.get("query_key_boundary", {}).get("exact_dialogue_intelligence_key_join") == "UNBOUND_NOT_CAPTURED", "key join was overstated")
require(evidence.get("audio_effect", {}).get("audio_or_joc_effect") == "UNBOUND", "audio effect was overstated")
require(evidence.get("scope", {}).get("dee_or_ui_execution") == "NOT_RUN", "runtime status mismatch")
require(evidence.get("scope", {}).get("media_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_list_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} key_selection=UNBOUND")
