from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-WAV-SELECTOR3-STATIC-READ-EOF-BOUNDARY-V2"
UPSTREAM_MANIFEST = "13f753a6ccc95572e3a124fc83bca7b0fc642c89550f8de7b2c45d4e5240da98"
UPSTREAM_SEAL = "264c51eeb8d5cd55eec840caea58f0f458523cd0fffc83abeb2fa42b75c3afe1"
UPSTREAM_VECTOR = "81a1e90622ab68cb3a8d1bab9f131c51e2236833c7c5a82c444a9268869bc3a3"
UPSTREAM_SUMS = "2f8d698e5c959b57c577382ccd94c95122f428bea348810b1d8b45ff316e55d8"
EXPECTED = {
    "CHECK.py", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md",
    "SOURCE-MANIFESTS.json", "UPSTREAM-INPUT-VECTOR.json", "UPSTREAM-MANIFEST.json",
    "UPSTREAM-PACKAGE-SEAL.json", "UPSTREAM-SHA256SUMS.txt"
}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"FAIL: {message}")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
up_manifest = json.loads((ROOT / "UPSTREAM-MANIFEST.json").read_text(encoding="utf-8"))
up_vector = json.loads((ROOT / "UPSTREAM-INPUT-VECTOR.json").read_text(encoding="utf-8"))

require(manifest.get("schema") == "jocforge.wash_supplement_manifest.v1", "manifest schema")
require(manifest.get("package_id") == PACKAGE_ID, "manifest package id")
require(manifest.get("source_manifest_sha256") == UPSTREAM_MANIFEST, "upstream manifest binding")
require(manifest.get("contains_private_key_or_license") is False, "secret boundary")
require(manifest.get("contains_pcm_or_media") is False, "media boundary")
require(manifest.get("contains_native_source_or_decompilation") is False, "source boundary")
require(seal.get("schema") == "jocforge.wash_supplement_seal.v1", "seal schema")
require(seal.get("package_id") == PACKAGE_ID, "seal package id")
require(seal.get("manifest_sha256") == sha(ROOT / "MANIFEST.json"), "manifest seal hash")
payloads = manifest.get("payloads")
require(isinstance(payloads, list) and {p.get("name") for p in payloads} == EXPECTED, "payload inventory")
for item in payloads:
    path = ROOT / item["name"]
    require(path.is_file(), f"payload missing: {item['name']}")
    require(path.stat().st_size == item["bytes"], f"payload size: {item['name']}")
    require(sha(path) == item["sha256"], f"payload hash: {item['name']}")
require(seal.get("payload_file_count") == len(EXPECTED), "seal count")
require(seal.get("payload_hash_audit") == f"PASS_{len(EXPECTED)}_OF_{len(EXPECTED)}", "seal audit")
require(sha(ROOT / "UPSTREAM-MANIFEST.json") == UPSTREAM_MANIFEST, "upstream manifest")
require(sha(ROOT / "UPSTREAM-PACKAGE-SEAL.json") == UPSTREAM_SEAL, "upstream seal")
require(sha(ROOT / "UPSTREAM-INPUT-VECTOR.json") == UPSTREAM_VECTOR, "upstream vector")
require(sha(ROOT / "UPSTREAM-SHA256SUMS.txt") == UPSTREAM_SUMS, "upstream sums")
require(up_manifest.get("status") == "STATIC_SELECTOR3_CALLBACK_AND_EOF_BODY_BOUND_RUNTIME_SELECTION_NOT_CAPTURED", "upstream status")
require(up_manifest.get("static_registration", {}).get("selector") == 3, "selector")
require(up_manifest.get("static_registration", {}).get("open_callback") == "FUN_1803ec660", "callback")
require(up_manifest.get("static_storage_type", {}).get("vtable_label") == "atmos_storage_master_s::vftable", "storage type")
require(up_manifest.get("static_read", {}).get("body") == "FUN_1804ccb00", "read body")
require(up_manifest.get("static_read", {}).get("eof_when_current_greater_or_equal_limit") == "returned_count remains zero", "EOF rule")
require(up_manifest.get("sample_boundary_arithmetic", {}).get("actual_final_sample_returned") == "NOT_CAPTURED", "last-sample limit")
require(up_vector.get("not_run", {}).get("runtime_storage_reader_execution") is True, "runtime reader boundary")
require(evidence.get("status", {}).get("runtime_provider_selection") == "NOT_CAPTURED", "runtime selection")
require(evidence.get("status", {}).get("actual_endSample_minus_one_returned") == "NOT_CAPTURED", "final sample status")
require(vectors.get("runtime_provider_selection") == "NOT_CAPTURED", "vector runtime status")
require(vectors.get("raw_audio_or_payload_included") is False, "vector media status")
require(sources.get("supersedes_supplement_manifest_sha256") == "7e9c04964c41a692046a3b50f04563224e791c31fea5af2336638a18bab42224", "superseded V1 manifest reference")
print(f"PASS {PACKAGE_ID} static_read=FUN_1804ccb00 runtime_final_sample=NOT_CAPTURED")

