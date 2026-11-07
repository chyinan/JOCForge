from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DI-FLAG-PROPERTY-REGISTRATION-ENTRY-V1"
SOURCE_MANIFEST_SHA256 = "0d6d2d10b67e27659567825fb4bec472c5bb33d3cfdfcd383104ba34db331f83"
SOURCE_SEAL_SHA256 = "9bd97339c8ab809de9b7aa820ac2693170c911db0d9b2ad6c99a23796922a8af"


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
upstream_manifest = ROOT / "UPSTREAM-MANIFEST.sha256"

require(manifest.get("schema") == "jocforge.wash_supplement_manifest.v1", "manifest schema mismatch")
require(manifest.get("package_id") == PACKAGE_ID, "package id mismatch")
require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest binding mismatch")
require(manifest.get("contains_private_key_or_license") is False, "secret boundary mismatch")
require(manifest.get("contains_pcm_or_media") is False, "media boundary mismatch")
require(manifest.get("contains_native_source_or_decompilation") is False, "source boundary mismatch")
require(seal.get("package_id") == PACKAGE_ID, "seal package id mismatch")
require(seal.get("manifest_sha256") == sha256(ROOT / "MANIFEST.json"), "package manifest seal mismatch")

payloads = manifest.get("payloads")
require(isinstance(payloads, list) and payloads, "payload list missing")
for item in payloads:
    path = ROOT / item["name"]
    require(path.is_file(), f"payload missing: {item['name']}")
    require(path.stat().st_size == item["bytes"], f"payload size mismatch: {item['name']}")
    require(sha256(path) == item["sha256"], f"payload hash mismatch: {item['name']}")
require(seal.get("payload_file_count") == len(payloads), "sealed payload count mismatch")
require(sha256(upstream_manifest) == SOURCE_MANIFEST_SHA256, "upstream manifest list hash mismatch")
require(upstream_seal.get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "upstream seal manifest binding mismatch")
require(sha256(ROOT / "UPSTREAM-PACKAGE-SEAL.json") == SOURCE_SEAL_SHA256, "upstream seal hash mismatch")
require(sources.get("upstream_manifest_list_sha256") == SOURCE_MANIFEST_SHA256, "source manifest reference mismatch")
require(sources.get("upstream_package_seal_sha256") == SOURCE_SEAL_SHA256, "source seal reference mismatch")
require(evidence.get("status", {}).get("generic_map_runtime_receiver_join") == "UNBOUND", "generic map overclaim")
require(evidence.get("status", {}).get("measurement_audio_consumer") == "NOT_IDENTIFIED", "consumer overclaim")
require(vectors.get("status") == "NO_SAMPLE_VECTOR_USED", "vector scope mismatch")
require(vectors.get("di_ab") == "NOT_RUN", "runtime overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} source_manifest={SOURCE_MANIFEST_SHA256[:12]} generic_map=UNBOUND audio_consumer=NOT_IDENTIFIED")
