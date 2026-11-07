from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-PHYSICAL-N-TO-PROCESSFRAME-M-FIRST-BREAK-V1"
UPSTREAM_MANIFEST = "34bc62d82c87e54b0405e82d4dca44f709ba525693b82b5ecac77a1f56ed5180"
UPSTREAM_SEAL = "47d054d2e5ffffc335453da330ed698a0693d251f564616c41ee89da074d21ae"
METADATA_PARENT = "6a2ccb7cc514736fd16b1acc3e8f6f2243a98cb429443bdbfd90410519d398d9"
EXPECTED = {
    "CHECK.py", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md",
    "SOURCE-MANIFESTS.json", "UPSTREAM-EVIDENCE.json", "UPSTREAM-INPUT-ONLY-VECTORS.json",
    "UPSTREAM-MANIFEST.json", "UPSTREAM-MANIFEST-SHA256.txt",
    "UPSTREAM-PACKAGE-SEAL.json", "UPSTREAM-SOURCE-IDENTITIES.json", "UPSTREAM-REPORT.md",
}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
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
require(manifest.get("contains_native_source_or_decompilation") is False, "source boundary")
require(seal.get("schema") == "jocforge.wash_supplement_seal.v1", "seal schema")
require(seal.get("package_id") == PACKAGE_ID, "seal package id")
require(seal.get("manifest_sha256") == sha256(ROOT / "MANIFEST.json"), "manifest seal hash")
payloads = manifest.get("payloads")
require(isinstance(payloads, list) and {p.get("name") for p in payloads} == EXPECTED, "payload set")
for row in payloads:
    path = ROOT / row["name"]
    require(path.is_file(), f"missing payload: {row['name']}")
    require(path.stat().st_size == row["bytes"], f"payload size: {row['name']}")
    require(sha256(path) == row["sha256"], f"payload hash: {row['name']}")
require(seal.get("payload_file_count") == len(EXPECTED), "seal count")
require(seal.get("payload_hash_audit") == f"PASS_{len(EXPECTED)}_OF_{len(EXPECTED)}", "seal payload audit")

require(sha256(ROOT / "UPSTREAM-MANIFEST.json") == UPSTREAM_MANIFEST, "upstream manifest hash")
require(sha256(ROOT / "UPSTREAM-PACKAGE-SEAL.json") == UPSTREAM_SEAL, "upstream seal hash")
require((ROOT / "UPSTREAM-MANIFEST-SHA256.txt").read_text(encoding="ascii").strip() ==
        f"{UPSTREAM_MANIFEST}  MANIFEST.json", "upstream manifest checksum file")
require(up_manifest.get("package_id") == "sample6-physical-n-to-processframe-m-first-break-wash-v1", "upstream package id")
up_files = {row.get("name"): row.get("sha256") for row in up_manifest.get("files", [])}
require(up_files.get("EVIDENCE.json") == sha256(ROOT / "UPSTREAM-EVIDENCE.json"), "upstream evidence copy hash")
require(up_files.get("INPUT-ONLY-VECTORS.json") == sha256(ROOT / "UPSTREAM-INPUT-ONLY-VECTORS.json"), "upstream vector copy hash")
require(up_files.get("SOURCE-IDENTITIES.json") == sha256(ROOT / "UPSTREAM-SOURCE-IDENTITIES.json"), "upstream identity copy hash")
require(up_evidence.get("package_id") == "sample6-physical-n-to-processframe-m-first-break-wash-v1", "upstream evidence id")
require(up_vectors.get("vectors") and len(up_vectors["vectors"]) == 4, "upstream input vectors")
require(up_ids.get("input_metadata_receipt", {}).get("manifest_sha256") == METADATA_PARENT, "metadata parent hash")
require(sources.get("input_metadata_parent", {}).get("manifest_sha256") == METADATA_PARENT, "source metadata binding")
require(evidence.get("status", {}).get("N_to_compositemaster_format_u16") == "UNBOUND", "N-to-U16 must remain unbound")
require(evidence.get("status", {}).get("core_M_formula_from_physical_N") == "UNBOUND", "N-to-M formula must remain unbound")
require(evidence.get("status", {}).get("packet_plus_0x08_to_core_M") == "UNBOUND", "packet-to-M break must remain unbound")
require(vectors.get("N_to_format_u16_owner_join") == "UNBOUND", "vector owner join")
require(vectors.get("packet_channel_u16_to_core_M") == "UNBOUND", "vector M join")
require(vectors.get("expected_track_support_result") == "NOT_ASSERTED", "do not promote support")
require(manifest.get("source_manifest_sha256") == UPSTREAM_MANIFEST, "manifest source link")
print(f"PASS {PACKAGE_ID} sources=36/38 N_to_M=UNBOUND")

