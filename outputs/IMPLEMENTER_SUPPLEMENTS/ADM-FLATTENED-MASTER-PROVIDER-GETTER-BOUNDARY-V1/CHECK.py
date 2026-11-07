from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-FLATTENED-MASTER-PROVIDER-GETTER-BOUNDARY-V1"
UPSTREAM_MANIFEST = "994d15a917c8e6fce1581ecef20413bcdaec8affa3414e8c507766ae6d766af0"
UPSTREAM_SEAL = "937ae9d6d8db1bdd41f2358927f86ed60fd1581bfec58a96f2f93bb4ae77001c"
TARGET_IMAGE = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"
EXPECTED = {
    "CHECK.py", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md",
    "SOURCE-MANIFESTS.json", "UPSTREAM-EVIDENCE.json", "UPSTREAM-INPUT-ONLY-VECTORS.json",
    "UPSTREAM-MANIFEST.json", "UPSTREAM-MANIFEST-SHA256.txt",
    "UPSTREAM-PACKAGE-SEAL.json", "UPSTREAM-SOURCE-IDENTITIES.json", "UPSTREAM-REPORT.md",
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
up_evidence = json.loads((ROOT / "UPSTREAM-EVIDENCE.json").read_text(encoding="utf-8"))
up_vectors = json.loads((ROOT / "UPSTREAM-INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
up_ids = json.loads((ROOT / "UPSTREAM-SOURCE-IDENTITIES.json").read_text(encoding="utf-8"))

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
require(seal.get("payload_file_count") == len(EXPECTED), "seal payload count")
require(seal.get("payload_hash_audit") == f"PASS_{len(EXPECTED)}_OF_{len(EXPECTED)}", "seal audit")

require(sha(ROOT / "UPSTREAM-MANIFEST.json") == UPSTREAM_MANIFEST, "upstream manifest")
require(sha(ROOT / "UPSTREAM-PACKAGE-SEAL.json") == UPSTREAM_SEAL, "upstream seal")
require((ROOT / "UPSTREAM-MANIFEST-SHA256.txt").read_text(encoding="ascii").strip() == f"{UPSTREAM_MANIFEST}  MANIFEST.json", "upstream checksum")
require(up_manifest.get("package_id") == "sample6-flattened-master-provider-getter-boundary-wash-v1", "upstream package id")
require(up_manifest.get("target_image", {}).get("sha256") == TARGET_IMAGE, "upstream image hash")
require(up_evidence.get("resolver_factory_branch", {}).get("wrapper_type") == "atmos_storage_flatten_master", "wrapper type")
require(up_evidence.get("outer_provider_slot", {}).get("target") == "FUN_1804F5AF0 @ 0x1804F5AF0", "outer slot target")
require(up_evidence.get("outer_provider_slot", {}).get("nested_target") == "UNBOUND", "inner provider target")
require(up_evidence.get("flattened_master_typed_outputs", {}).get("selected_adm_reader_profile_owner_join") == "UNBOUND", "reader/profile owner join")
require(up_evidence.get("disposition", {}).get("format_plus_0x0A_to_selected_adm_track_count") == "UNBOUND", "physical N join")
require(up_vectors.get("vectors") and len(up_vectors["vectors"]) == 4, "upstream vector inventory")
require(up_ids.get("binary", {}).get("sha256") == TARGET_IMAGE, "upstream source identity")
require(sources.get("upstream_manifest_sha256") == UPSTREAM_MANIFEST, "source lineage")
require(sources.get("parent_receipts") and len(sources["parent_receipts"]) == 4, "parent receipt inventory")
require(evidence.get("status", {}).get("inner_provider_target_and_owner") == "UNBOUND", "inner owner status")
require(evidence.get("status", {}).get("N_to_compositemaster_format_u16") == "UNBOUND", "N join status")
require(evidence.get("status", {}).get("arbitrary_track_support") == "NOT_CLAIMED", "support boundary")
require(vectors.get("sample_track_support") == "NOT_ASSERTED", "vector support boundary")
require(vectors.get("pcm_or_payload_included") is False, "vector media boundary")
print(f"PASS {PACKAGE_ID} typed_getter=BOUND inner_provider=UNBOUND N_join=UNBOUND")

