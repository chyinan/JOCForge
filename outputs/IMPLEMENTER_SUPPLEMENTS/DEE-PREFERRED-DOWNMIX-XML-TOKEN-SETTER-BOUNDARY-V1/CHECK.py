from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-PREFERRED-DOWNMIX-XML-TOKEN-SETTER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "bdd71adc1d6387fa4645416bb28760922a38064c9e9a651e6aec62a217280a1c"
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
tokens = evidence.get("xml_property", {}).get("exact_tokens", {})
require(tokens == {"loro": 0, "ltrt": 1, "ltrt-pl2": 2, "not_indicated": 3}, "enum mapping mismatch")
require(evidence.get("typed_value_path", {}).get("constructor_callback") == "null in the inspected XML construction path", "callback construction mismatch")
require(evidence.get("typed_value_path", {}).get("xml_ordinal_to_stored_code") == "UNBOUND", "stored-value join was overstated")
require(evidence.get("typed_reader", {}).get("audio_or_bsi_consumer") == "UNBOUND", "audio consumer was overstated")
require(evidence.get("scope", {}).get("dee_or_xml_job_run") == "NOT_RUN", "runtime status mismatch")
require(evidence.get("scope", {}).get("media_pcm_or_payload_read") is False, "media boundary mismatch")
require(evidence.get("scope", {}).get("private_key_or_license_read") is False, "key boundary mismatch")
require(sources.get("sources", [])[0].get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "source ledger mismatch")
print(f"PASS {PACKAGE_ID} payloads={len(PAYLOAD_NAMES)} stored_code=UNBOUND")
