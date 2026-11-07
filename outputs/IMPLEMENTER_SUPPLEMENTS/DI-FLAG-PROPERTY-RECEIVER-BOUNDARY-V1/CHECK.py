from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DI-FLAG-PROPERTY-RECEIVER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "9bb2cd833efdd70256b42c5c52689010a84eed02ab3b47ad5b12abf3a1168971"


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
upstream_seal = json.loads((ROOT / "UPSTREAM-PACKAGE-SEAL.json").read_text(encoding="utf-8"))

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
require(upstream_seal.get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "upstream seal binding mismatch")
require(sources.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest reference mismatch")
require(len(sources.get("parent_packages", [])) == 3, "parent receipt count mismatch")
require(evidence.get("status", {}).get("same_receiver_wrapper_to_setter") == "BOUND_STATIC", "receiver edge status mismatch")
require(evidence.get("status", {}).get("generic_property_map_caller") == "UNBOUND", "generic caller overclaim")
require(evidence.get("status", {}).get("first_measurement_or_audio_consumer_of_this_plus_0x50") == "NOT_IDENTIFIED", "consumer overclaim")
require(vectors.get("status") == "NO_AUTHORIZED_SAMPLE_VECTOR_USED", "input scope mismatch")
require(vectors.get("di_ab") == "NOT_RUN", "runtime overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} source_manifest={SOURCE_MANIFEST_SHA256[:12]} owner=UNBOUND consumer=NOT_IDENTIFIED")
