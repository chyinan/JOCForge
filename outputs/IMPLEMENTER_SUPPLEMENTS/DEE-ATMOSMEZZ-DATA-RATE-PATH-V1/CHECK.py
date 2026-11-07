from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOSMEZZ-DATA-RATE-PATH-V1"
SOURCE_MANIFEST_SHA256 = "2352b6835dd2d6b202dd71aa96ddaf6019487670b2d2c1609d70a20c6b03d191"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
upstream_hash_file = ROOT / "UPSTREAM-MANIFEST.sha256"

require(manifest.get("schema") == "jocforge.wash_supplement_manifest.v1", "manifest schema mismatch")
require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source binding mismatch")
require(manifest.get("contains_private_key_or_license") is False, "secret boundary mismatch")
require(manifest.get("contains_pcm_or_media") is False, "media boundary mismatch")
require(manifest.get("contains_native_source_or_decompilation") is False, "source boundary mismatch")
require(seal.get("package_id") == PACKAGE_ID, "seal package id mismatch")
require(seal.get("manifest_sha256") == sha256(ROOT / "MANIFEST.json"), "manifest seal mismatch")

payloads = manifest.get("payloads")
require(isinstance(payloads, list) and payloads, "payload list missing")
for item in payloads:
    path = ROOT / item["name"]
    require(path.is_file(), f"payload missing: {item['name']}")
    require(path.stat().st_size == item["bytes"], f"payload size mismatch: {item['name']}")
    require(sha256(path) == item["sha256"], f"payload hash mismatch: {item['name']}")
require(seal.get("payload_file_count") == len(payloads), "sealed payload count mismatch")
require(sources.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest reference mismatch")
require(upstream_hash_file.is_file(), "upstream checksum file missing")
require(sha256(upstream_hash_file) == SOURCE_MANIFEST_SHA256, "upstream manifest checksum file hash mismatch")
require(evidence.get("status", {}).get("xml_token_to_native_selector_code") == "UNBOUND", "rate mapping overclaim")
require(evidence.get("status", {}).get("dee_run") == "NOT_RUN", "runtime overclaim")
require(vectors.get("status") == "SCHEMA_AND_ONE_SELECTED_JOB_ONLY", "vector scope mismatch")
require(vectors.get("native_selector_mapping") == "UNBOUND", "selector mapping overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} source_manifest={SOURCE_MANIFEST_SHA256[:12]} XML_to_selector=UNBOUND")
