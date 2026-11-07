from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-WAV-LOWER-STORAGE-RANGE-INITIALIZATION-BOUNDARY-V1"
SOURCE_SEAL_SHA256 = "34b826fe953744542d5e1b8687717f604680f4d47b209a8a1160ce3db4fabec7"
PAYLOAD_NAMES = ("CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {PACKAGE_ID}: {message}")


manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
require(manifest.get("source_package_seal_file_sha256") == SOURCE_SEAL_SHA256, "source seal hash mismatch")
require(set(manifest.get("payloads", {})) == set(PAYLOAD_NAMES), "payload set mismatch")
for name in PAYLOAD_NAMES:
    raw = (ROOT / name).read_bytes()
    record = manifest["payloads"][name]
    require(record.get("bytes") == len(raw), f"byte count mismatch: {name}")
    require(record.get("sha256") == hashlib.sha256(raw).hexdigest(), f"SHA-256 mismatch: {name}")
require(evidence.get("initial_range") == {"current": 0, "limit": 0, "interpretation": "empty initial range [0,0)"}, "initial range mismatch")
require(evidence.get("limit_source", {}).get("nonzero_writer") == "UNBOUND", "nonzero writer was overstated")
require(evidence.get("units", {}).get("classification") == "STATIC_CALL_CONTRACT_INFERENCE", "unit confidence mismatch")
require(evidence.get("scope", {}).get("dee_execution") == "NOT_RUN", "DEE execution status mismatch")
require(evidence.get("scope", {}).get("media_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(evidence.get("scope", {}).get("raw_ghidra_trace_or_attempt_log_included") is False, "raw-trace boundary mismatch")
require(sources.get("sources", [])[0].get("package_seal_file_sha256") == SOURCE_SEAL_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} initial_range=[0,0)")
