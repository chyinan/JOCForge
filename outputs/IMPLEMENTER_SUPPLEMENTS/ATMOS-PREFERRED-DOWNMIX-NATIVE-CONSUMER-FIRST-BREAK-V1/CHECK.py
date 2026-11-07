from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-PREFERRED-DOWNMIX-NATIVE-CONSUMER-FIRST-BREAK-V1"
SOURCE_MANIFEST_SHA256 = "1c75cf415e566ccf260e031ef8bb818c293c7d32fcd04de28a78b29502c4e508"


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
require(upstream.get("package") == "F13C-preferred-downmix-native-consumer-first-break-v1", "upstream package mismatch")
require(sha256(ROOT / "UPSTREAM-MANIFEST.json") == SOURCE_MANIFEST_SHA256, "upstream manifest hash mismatch")
require(sha256(ROOT / "UPSTREAM-SHA256SUMS.txt") == evidence.get("source_package", {}).get("sha256sums_sha256"), "upstream checksum-list hash mismatch")
require(evidence.get("status", {}).get("xml_property_to_typed_native_ddp_dmixmod") == "UNBOUND", "typed consumer status overclaim")
require(evidence.get("status", {}).get("audio_effect") == "UNBOUND", "audio effect status overclaim")
require(evidence.get("status", {}).get("ab_execution") == "NOT_RUN", "A/B status overclaim")
require(vectors.get("status") == "PREPARED_NOT_RUN", "input vector execution status mismatch")
require(all(row.get("execution") == "NOT_RUN" for row in vectors.get("variants", [])), "candidate run status overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} source_manifest={SOURCE_MANIFEST_SHA256[:12]} typed_consumer=UNBOUND audio=UNBOUND AB=NOT_RUN")
