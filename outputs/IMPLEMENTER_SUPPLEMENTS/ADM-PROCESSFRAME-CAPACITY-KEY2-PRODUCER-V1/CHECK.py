from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-PROCESSFRAME-CAPACITY-KEY2-PRODUCER-V1"
SOURCE_MANIFEST_SHA256 = "502c4dfbe4010f70a4c7485f81d1c48fb780d73b8c8274b0dbdd86674cf74ff0"
SOURCE_SEAL_SHA256 = "6a812cc6c3a12cbb4b4e8cedbbf5542c5f4d1ef0cfdad91b9b2807f37e211ba5"


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
upstream = json.loads((ROOT / "UPSTREAM-MANIFEST.json").read_text(encoding="utf-8"))
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
require(sha256(ROOT / "UPSTREAM-MANIFEST.json") == SOURCE_MANIFEST_SHA256, "upstream manifest hash mismatch")
require(sha256(ROOT / "UPSTREAM-PACKAGE-SEAL.json") == SOURCE_SEAL_SHA256, "upstream package seal hash mismatch")
require(upstream.get("package_id") == "sample6-core-capacity-key2-producer-wash-v1", "upstream package mismatch")
require(upstream_seal.get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "upstream seal manifest binding mismatch")
require(sources.get("upstream_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest reference mismatch")
require(evidence.get("status", {}).get("selected_profile") == "UNBOUND", "profile overclaim")
require(evidence.get("status", {}).get("active_capacity_C") == "UNBOUND", "capacity overclaim")
require(evidence.get("status", {}).get("C_to_ADM_N_or_track_count") == "UNBOUND", "count-domain join overclaim")
require(vectors.get("status") == "STATIC_PROFILE_ROWS_AND_SYMBOLIC_CONDITIONS_ONLY", "vector scope mismatch")
require(vectors.get("actual_C") == "UNBOUND", "active capacity overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} source_manifest={SOURCE_MANIFEST_SHA256[:12]} active_C=UNBOUND ADM_N_join=UNBOUND")
