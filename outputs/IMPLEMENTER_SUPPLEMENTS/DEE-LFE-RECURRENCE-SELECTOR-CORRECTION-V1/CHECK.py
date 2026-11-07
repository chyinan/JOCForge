from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-RECURRENCE-SELECTOR-CORRECTION-V1"
SOURCE_MANIFEST_SHA256 = "af45b96787219eb31b9a62c64e5c1b62590d2824ac37132404cf339e3509893d"
SOURCE_SEAL_SHA256 = "0efe3846f472ae11bf27b874ab69f4ec78fa79569a41551b1088e709534f656b"


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
upstream = json.loads((ROOT / "UPSTREAM-EVIDENCE-MANIFEST.json").read_text(encoding="utf-8"))
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
require(sha256(ROOT / "UPSTREAM-EVIDENCE-MANIFEST.json") == SOURCE_MANIFEST_SHA256, "upstream manifest hash mismatch")
require(sha256(ROOT / "UPSTREAM-PACKAGE-SEAL.json") == SOURCE_SEAL_SHA256, "upstream seal hash mismatch")
require(upstream.get("package_id") == "dee_lfe_base_layer_recurrence_branch_wash_v1", "upstream package mismatch")
require(upstream_seal.get("status") == "SEALED_PARTIAL", "upstream seal status mismatch")
require(evidence.get("status", {}).get("descriptor_stride_to_recurrence_selector_in_two_lfe_dispatchers") == "RETRACTED_NOT_A_JOIN", "correction status mismatch")
require(evidence.get("status", {}).get("correlation_root_cause") == "NOT_ESTABLISHED", "causality overclaim")
require(vectors.get("status") == "NO_NEW_SAMPLE_VECTOR", "vector scope mismatch")
require(vectors.get("numeric_parity") == "NOT_RUN", "parity overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} source={SOURCE_MANIFEST_SHA256[:12]} lfe_selector=ZERO_STATIC root_cause=UNBOUND")
