from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-PROCESSFRAME-M-TO-EAC3-WRITER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "ea654ef938b0dd5020ddef2cfaf93bcdc78e80b466a87877c5662af49fd21ae3"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
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
require(upstream.get("package_id") == "sample6-process-frame-eac3-writer-boundary-wash-v1", "upstream package mismatch")
require(sha256(ROOT / "UPSTREAM-MANIFEST.json") == SOURCE_MANIFEST_SHA256, "upstream manifest hash mismatch")
require(upstream_seal.get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "upstream package seal mismatch")
require(evidence.get("status", {}).get("adm_N_to_process_frame_M") == "UNBOUND", "count-domain join overclaim")
require(evidence.get("status", {}).get("process_frame_M_to_stuffer_frame_block_counters") == "UNBOUND", "stuffer-domain join overclaim")
require(evidence.get("status", {}).get("36_38_native_execution") == "NOT_RUN", "runtime overclaim")
require(vectors.get("status") == "SYMBOLIC_CONDITION_CHECKS_NOT_NATIVE", "vector evidence class mismatch")
require(vectors.get("dee_run") is False, "runtime overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} source_manifest={SOURCE_MANIFEST_SHA256[:12]} N_to_M=UNBOUND M_to_stuffer=UNBOUND")
