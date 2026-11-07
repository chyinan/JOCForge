from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-DI-QUERY-VIRTUAL-CALLER-RDX-BOUNDARY-V2"
SOURCE_MANIFEST_SHA256 = "8a9157bbcc50468a853cff8582c7844329fe1b876140dea98e6abe1ce08d4063"
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
require(evidence.get("thunk_receiver", {}).get("static_getter_receiver") == "Priv+0xec0 ParametersContainer", "receiver owner mismatch")
require(evidence.get("thunk_receiver", {}).get("rdx_behavior") == "preserved unchanged by thunk", "RDX preservation mismatch")
require(evidence.get("virtual_caller_rdx", {}).get("param_2_record_owner") == "UNBOUND", "query owner was overstated")
require(evidence.get("virtual_caller_rdx", {}).get("exact_dialogue_intelligence_match") == "UNBOUND_NOT_CAPTURED", "query key was overstated")
require(sources.get("sources", [])[0].get("manifest_list_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} query_owner=UNBOUND")
