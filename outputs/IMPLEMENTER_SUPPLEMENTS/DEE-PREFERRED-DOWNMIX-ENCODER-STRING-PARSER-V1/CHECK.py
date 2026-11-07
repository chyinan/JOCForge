from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-PREFERRED-DOWNMIX-ENCODER-STRING-PARSER-V1"
SOURCE_MANIFEST_SHA256 = "1fffe96f847866e3e26a891685d9b005c5e2fa1bf9eebee911b8c31c623852f1"
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
require(evidence.get("encoder_option", {}).get("storage_offset") == "+0x360", "encoder storage offset mismatch")
require(evidence.get("composite_parser", {}).get("field_delimiter") == ":", "colon delimiter mismatch")
require(evidence.get("composite_parser", {}).get("colon_backslash_sequence_codepoints") == [58, 92], "colon-backslash token sequence mismatch")
require(evidence.get("composite_parser", {}).get("key_value_delimiter") == "=", "equals delimiter mismatch")
require(evidence.get("preferred_downmix_reader", {}).get("effective_value") == "UNBOUND", "effective value was overstated")
require(evidence.get("runtime_and_scope", {}).get("actual_child_argv") == "NOT_CAPTURED", "runtime argv was overstated")
require(evidence.get("preferred_downmix_reader", {}).get("first_audio_bsi_or_frame_writer_consumer") == "UNBOUND", "audio boundary mismatch")
require(evidence.get("runtime_and_scope", {}).get("media_or_pcm_read") is False, "media boundary mismatch")
require(evidence.get("runtime_and_scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} runtime_argv=NOT_CAPTURED")
