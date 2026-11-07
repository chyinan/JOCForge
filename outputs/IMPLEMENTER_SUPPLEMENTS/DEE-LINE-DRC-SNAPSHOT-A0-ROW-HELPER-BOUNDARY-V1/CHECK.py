from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LINE-DRC-SNAPSHOT-A0-ROW-HELPER-BOUNDARY-V1"
SOURCE_MANIFEST = "7fdb3c6b7a177e5fd4ed325246a337a162ba2f51e35f1559073cac32fdbfcfb5"
SOURCE_SEAL = "45408259fae5efc4e376fd74ac8fb316a8b5775dda109ac18d7ca1ef420f87e0"
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

route = evidence["same_record_route"]
require(route["source_field_offset"] == "+0xa0", "field identity")
require(route["source_type"] == "U32", "field type")
edge = evidence["first_row_helper_call"]
require(edge["explicit_snapshot_plus_0xa0_value_or_address"] is False, "row helper argument")
require(edge["selected_row_owner_join_to_snapshot_plus_0xa0"] == "UNBOUND", "row owner")
require(evidence["status"]["codec_or_bsi_input"] == "UNBOUND", "codec/BSI status")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} A0_to_row_helper=UNBOUND")
