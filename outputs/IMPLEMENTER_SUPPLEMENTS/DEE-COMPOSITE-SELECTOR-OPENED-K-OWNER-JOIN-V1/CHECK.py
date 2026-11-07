from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-COMPOSITE-SELECTOR-OPENED-K-OWNER-JOIN-V1"
UPSTREAM_MANIFEST = "8ba69c952f128bd64faec02670501315fc9de2a0f7fa6feb8ed1c0b4edab0bb3"
IMAGE_SHA256 = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"
EXPECTED = {
    "CHECK.py", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md",
    "SOURCE-MANIFESTS.json", "UPSTREAM-CHECK.json", "UPSTREAM-FUNCTION-HASHES.json",
    "UPSTREAM-MANIFEST.json", "UPSTREAM-OWNER-OPENED-K-JOIN.json",
    "UPSTREAM-SOURCE-RECEIPTS.json"
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
up_owner = json.loads((ROOT / "UPSTREAM-OWNER-OPENED-K-JOIN.json").read_text(encoding="utf-8"))
up_receipts = json.loads((ROOT / "UPSTREAM-SOURCE-RECEIPTS.json").read_text(encoding="utf-8"))
up_functions = json.loads((ROOT / "UPSTREAM-FUNCTION-HASHES.json").read_text(encoding="utf-8"))

require(manifest.get("schema") == "jocforge.wash_supplement_manifest.v1", "manifest schema")
require(manifest.get("package_id") == PACKAGE_ID, "manifest package id")
require(manifest.get("source_manifest_sha256") == UPSTREAM_MANIFEST, "source manifest binding")
require(manifest.get("contains_private_key_or_license") is False, "secret boundary")
require(manifest.get("contains_pcm_or_media") is False, "media boundary")
require(manifest.get("contains_native_source_or_decompilation") is False, "native source boundary")
require(seal.get("schema") == "jocforge.wash_supplement_seal.v1", "seal schema")
require(seal.get("package_id") == PACKAGE_ID, "seal package id")
require(seal.get("manifest_sha256") == sha(ROOT / "MANIFEST.json"), "manifest seal hash")
payloads = manifest.get("payloads")
require(isinstance(payloads, list) and {p.get("name") for p in payloads} == EXPECTED, "payload set")
for row in payloads:
    path = ROOT / row["name"]
    require(path.is_file(), f"missing payload: {row['name']}")
    require(path.stat().st_size == row["bytes"], f"payload size: {row['name']}")
    require(sha(path) == row["sha256"], f"payload hash: {row['name']}")
require(seal.get("payload_file_count") == len(EXPECTED), "payload count")
require(seal.get("payload_hash_audit") == f"PASS_{len(EXPECTED)}_OF_{len(EXPECTED)}", "payload audit")
require(sha(ROOT / "UPSTREAM-MANIFEST.json") == UPSTREAM_MANIFEST, "upstream manifest hash")
require(up_manifest.get("package_id") == "COMPOSITE-SELECTOR-OPENED-K-OWNER-JOIN-V1", "upstream id")
require(up_owner.get("target_image_sha256") == IMAGE_SHA256, "target image")
require(up_owner.get("join_result", {}).get("selector_descriptor_row_to_opened_K_slot") == "YES_STATIC_DIRECT_FIELD_COPY", "static J-to-K slot join")
require(up_owner.get("join_result", {}).get("same_per_item_owner_instance") == "YES_STATIC_POINTER_LINEAGE", "static owner")
require(up_owner.get("join_result", {}).get("same_runtime_opened_K_pointer_value") == "NOT_CAPTURED", "runtime pointer status")
require(up_owner.get("join_result", {}).get("same_runtime_source_generation") == "NOT_CAPTURED", "runtime generation")
require(up_owner.get("first_unresolved_runtime_edge", {}).get("status") == "NOT_CAPTURED", "first unresolved edge")
require(up_receipts.get("raw_source_material_in_package") is False, "raw source boundary")
require(up_functions.get("target_image_sha256") == IMAGE_SHA256, "function image binding")
require(evidence.get("status", {}).get("selector_descriptor_row_to_opened_K_slot") == "BOUND_STATIC_DIRECT_FIELD_COPY", "evidence static join")
require(evidence.get("status", {}).get("physical_N_to_K_count") == "UNBOUND", "physical count join")
require(vectors.get("physical_N_to_K_count") == "UNBOUND", "vector N-to-K boundary")
require(vectors.get("runtime_pointer_and_generation") == "NOT_CAPTURED", "vector runtime boundary")
print(f"PASS {PACKAGE_ID} static_J_K_slot=BOUND physical_N=UNBOUND")

