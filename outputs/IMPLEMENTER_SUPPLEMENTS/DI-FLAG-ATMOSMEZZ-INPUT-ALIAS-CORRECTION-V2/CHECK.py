from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DI-FLAG-ATMOSMEZZ-INPUT-ALIAS-CORRECTION-V2"
SOURCE_MANIFEST_SHA256 = "5f4774fe7f935863014417e61719e83a1554c559e456e80475c9b1b1ffdea128"
SOURCE_SEAL_SHA256 = "5d1d13b04a382d8f495515175c709b7823fb9d955536a629e1974d950a52fcc8"


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
require(sha256(ROOT / "UPSTREAM-PACKAGE-SEAL.json") == SOURCE_SEAL_SHA256, "upstream package seal hash mismatch")
require(upstream_seal.get("manifest_sha256") == SOURCE_MANIFEST_SHA256, "upstream seal manifest binding mismatch")
require(sources.get("source_manifest_list_sha256") == SOURCE_MANIFEST_SHA256, "source ledger manifest binding mismatch")
require(sources.get("source_package_seal_sha256") == SOURCE_SEAL_SHA256, "source ledger seal binding mismatch")
require(evidence.get("status", {}).get("alias_to_selected_source") == "USER_ASSERTED/UNBOUND", "alias provenance overclaim")
require(evidence.get("status", {}).get("attempt_1") == "CANCELLED_AT_64_6_PERCENT; NO_FINAL_EC3_RECEIPT; NOT_PASS", "attempt 1 status overclaim")
require(evidence.get("status", {}).get("attempt_2") == "CANCELLED_AT_74_8_PERCENT; NO_FINAL_EC3_RECEIPT; NOT_PASS", "attempt 2 status overclaim")
require(vectors.get("status") == "CANCELLED_ATTEMPTS_ONLY", "vector scope mismatch")
require(vectors.get("final_ec3_verified") is False, "output status overclaim")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} alias=USER_ASSERTED/UNBOUND attempts=CANCELLED_NOT_PASS")
