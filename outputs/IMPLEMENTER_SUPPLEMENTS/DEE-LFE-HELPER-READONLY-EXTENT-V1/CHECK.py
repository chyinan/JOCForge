from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-HELPER-READONLY-EXTENT-V1"
SOURCE_MANIFEST_SHA256 = "52604072ce7175b32c60d66b507137955de5ed168b400af255865c4afa5e9d92"
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
for helper in ("aligned_helper", "unaligned_helper"):
    record = evidence.get("alignment_helpers", {}).get(helper, {})
    require(record.get("element_reads", "").startswith("256 F32"), f"extent mismatch: {helper}")
    require(record.get("byte_extent") == "0x400 bytes", f"byte extent mismatch: {helper}")
    require(record.get("writes_to_input") is False, f"write claim mismatch: {helper}")
require(evidence.get("buffer_continuity", {}).get("buffer_content_at_e350") == "UNBOUND", "buffer continuity was overstated")
require(evidence.get("scope", {}).get("numeric_parity") == "NOT_RUN", "parity status mismatch")
require(evidence.get("scope", {}).get("pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} extent=256_F32")
