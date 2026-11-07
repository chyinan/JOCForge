from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOSMEZZ-DATA-RATE-SELECTOR-CONSTRUCTOR-EDGE-V1"
SOURCE_MANIFEST_SHA256 = "5c92d48a60682928e4fb9abd6f04c7e2062c19e31b27587a93ddf8452f284e81"


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
require(sha256(upstream_hash_file) == SOURCE_MANIFEST_SHA256, "upstream manifest-list hash mismatch")
require(sources.get("upstream_manifest_sha256_file") == SOURCE_MANIFEST_SHA256, "source ledger binding mismatch")
require(evidence.get("status", {}).get("six_xml_string_to_selector_codes") == "BOUND_STATIC", "static mapping status mismatch")
require(evidence.get("status", {}).get("workflow_parameter_map_to_lookup_input_same_run") == "UNBOUND", "runtime join overclaim")
require(evidence.get("status", {}).get("dee_run") == "NOT_RUN", "runtime overclaim")
require(vectors.get("status") == "STATIC_LOOKUP_ONLY_NOT_RUNTIME", "vector scope mismatch")
require(vectors.get("runtime_workflow_map_and_branch") == "NOT_CAPTURED", "runtime branch overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} source_manifest={SOURCE_MANIFEST_SHA256[:12]} table=BOUND_STATIC runtime_join=UNBOUND")
