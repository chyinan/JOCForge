from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "RF-DRC-SNAPSHOT-A4-BRANCH-TARGET-INPUT-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "6761ae7bd37ff11ce25470c9c61c2d0313c98214d5a19d63a2cd444d4e0d778a"
SOURCE_SEAL_SHA256 = "6c81aa388e423ef2e2b548433fdfe964c3c3685f65ec03b339d350bfbde88b7f"


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
upstream_seal = json.loads((ROOT / "UPSTREAM-package_seal.json").read_text(encoding="utf-8"))

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
require(sha256(ROOT / "UPSTREAM-package_seal.json") == SOURCE_SEAL_SHA256, "upstream package seal hash mismatch")
require(upstream.get("package_id") == "rf_drc_snapshot_a4_branch_target_inputs_wash_v1", "upstream package mismatch")
require(upstream_seal.get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "upstream seal manifest binding mismatch")
require(sha256(ROOT / "UPSTREAM-SHA256SUMS.txt") == sources.get("upstream_sha256sums_sha256"), "upstream checksum list hash mismatch")
require(evidence.get("status", {}).get("snapshot_plus_0xa4_indirect_inner_state_transfer") == "UNBOUND", "indirect transfer overclaim")
require(evidence.get("status", {}).get("runtime_variant") == "NOT_CAPTURED", "runtime variant overclaim")
require(vectors.get("status") == "STATIC_BRANCH_PREDICATES_ONLY", "vector scope mismatch")
require(vectors.get("worker_input_owner_join") == "UNBOUND", "owner join overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} source={SOURCE_MANIFEST_SHA256[:12]} A4_transfer=UNBOUND")
