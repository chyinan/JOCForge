from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-PROCESSFRAME-PROFILE-INDEX-KEY1-PRODUCER-V1"
SOURCE_MANIFEST_SHA256 = "e50c225e4ef465271c93db438a264880714e8f52603d08de42cd471ae3589458"
SOURCE_SEAL_SHA256 = "8e5c4dcf212fee5406a11d7d34eb2f031fe6fe92ef54e967d69be75f14bfc679"


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
require(sha256(ROOT / "UPSTREAM-PACKAGE-SEAL.json") == SOURCE_SEAL_SHA256, "upstream seal hash mismatch")
require(upstream.get("package_id") == "sample6-core-selected-profile-index-wash-v1", "upstream package mismatch")
require(upstream_seal.get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "upstream seal binding mismatch")
require(sources.get("upstream_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest reference mismatch")
require(evidence.get("status", {}).get("runtime_selected_profile") == "NOT_CAPTURED", "runtime profile overclaim")
require(evidence.get("status", {}).get("runtime_C") == "NOT_CAPTURED", "capacity overclaim")
require(evidence.get("status", {}).get("adm_N_to_M") == "UNBOUND", "count-domain join overclaim")
require(vectors.get("status") == "STATIC_TABLE_AND_CONDITIONAL_DEFAULT_ONLY", "vector scope mismatch")
require(vectors.get("actual_profile_C_M_and_ADM_N_join") == "UNBOUND", "runtime join overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} source_manifest={SOURCE_MANIFEST_SHA256[:12]} runtime_profile=NOT_CAPTURED")
