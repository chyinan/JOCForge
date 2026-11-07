from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-SELECTOR-SOURCE-OFFSET-CORRECTION-V2"
SOURCE_MANIFEST_SHA256 = "46473d85233accc9a6a98d00516f2910023175437fc03e28a202c0e67868c2ed"
SOURCE_MANIFEST_CHECKSUM_FILE_SHA256 = "e72d4362f5a265b90b6663ac7414ad809423eb81a9fb65027e438a22ff8ba36c"


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
upstream_manifest_list = ROOT / "UPSTREAM-MANIFEST.sha256"

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
require(sha256(upstream_manifest_list) == SOURCE_MANIFEST_CHECKSUM_FILE_SHA256, "upstream checksum file hash mismatch")
require(any(line.strip() == SOURCE_MANIFEST_SHA256 + "  MANIFEST.json" for line in upstream_manifest_list.read_text(encoding="utf-8").splitlines()), "upstream manifest entry mismatch")
require(sources.get("upstream_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest reference mismatch")
require(evidence.get("status", {}).get("lvar9_plus_0x40_plus_0x44_as_selector_source") == "SUPERSEDED", "old source offset not superseded")
require(evidence.get("status", {}).get("adm_row_crosswalk") == "UNBOUND", "ADM crosswalk overclaim")
require(vectors.get("status") == "STATIC_SOURCE_OFFSETS_ONLY", "vector scope mismatch")

print(f"PASS package={PACKAGE_ID} payloads={len(payloads)} corrected_offsets=0x94/0x98 ADM_crosswalk=UNBOUND")
