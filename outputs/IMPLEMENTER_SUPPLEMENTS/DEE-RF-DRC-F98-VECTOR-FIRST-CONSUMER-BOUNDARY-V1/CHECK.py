from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-RF-DRC-F98-VECTOR-FIRST-CONSUMER-BOUNDARY-V1"
SOURCE_MANIFEST = "fd20dba27f69640e3127053171eaf432395f338c5dd47cba3abcc54af561184a"
SOURCE_SEAL = "f226d9472c522492d9c3ffd9b3a3f4746bfac86fbb24d5d59c19b7d34363a93b"
PAYLOADS = {"EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json"}


def require(value: bool, message: str) -> None:
    if not value:
        raise RuntimeError(f"FAIL {PACKAGE_ID}: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
require(manifest.get("package_id") == PACKAGE_ID, "package id")
require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST, "source manifest")
require(manifest.get("source_package_seal_sha256") == SOURCE_SEAL, "source seal")
require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest hash")
require({row.get("name") for row in manifest.get("payloads", [])} == PAYLOADS | {"CHECK.py"}, "payload inventory")
for row in manifest["payloads"]:
    payload = ROOT / row["name"]
    require(payload.is_file() and payload.stat().st_size == row["bytes"], "payload size")
    require(sha256(payload) == row["sha256"], "payload hash")

require(evidence["vector"]["destination_owner_field_offset_hex"] == "0xf98", "destination field")
require(evidence["vector"]["exact_offset_references_after_copy"] == 0, "same-caller boundary")
require(evidence["status"]["f98_first_consumer"] == "UNBOUND", "consumer status")
require(evidence["status"]["rf_drc_encoding_setup"] == "UNBOUND", "encoding status")
require(evidence["next_call"]["explicit_destination_field_offset_hex"] == "0xfb0", "next destination")
require(evidence["next_call"]["same_owner_join_to_prior_f98_vector"] == "UNBOUND", "separate vector")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} F98_consumer=UNBOUND")
