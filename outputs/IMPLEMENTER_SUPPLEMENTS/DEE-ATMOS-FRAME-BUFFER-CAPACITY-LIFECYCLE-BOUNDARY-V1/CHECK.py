from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOS-FRAME-BUFFER-CAPACITY-LIFECYCLE-BOUNDARY-V1"
SOURCE_MANIFEST = "8300eb50440b08d7cf3f436ed89661a90bfb1f270837e332d5496890b8b9fba5"
PARENT_COUNT_MANIFEST = "f30a57b2456311aeda1144708c8126f2843686327e864190afd8ac0a834dd766"
PARENT_IDENTITY_MANIFEST = "d72f1bb76bc0d1dcfbf757a424538e07ba1835836b1fe601f541e34e5c7943d7"
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
require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest hash")
require({row.get("name") for row in manifest.get("payloads", [])} == PAYLOADS | {"CHECK.py"}, "payload inventory")
for row in manifest["payloads"]:
    payload = ROOT / row["name"]
    require(payload.is_file() and payload.stat().st_size == row["bytes"], "payload size")
    require(sha256(payload) == row["sha256"], "payload hash")

owner = evidence["selected_owner"]
require(owner["initialization_flag_offset"] == "+0x68", "initialization flag")
require(owner["float_matrix_pointer_offset"] == "+0x80", "matrix pointer")
require(owner["channel_pointer_table_offset"] == "+0x88", "pointer table")
require(owner["stored_matrix_capacity"] == "NOT_PRESENT_IN_INSPECTED_OWNER_FIELDS", "matrix capacity")
require(evidence["allocation"]["capacity_recheck_on_initialized_path"] == "NOT_PRESENT_IN_INSPECTED_BODY", "allocation guard")
require(evidence["first_consumer"]["same_owner_capacity_guard"] == "NOT_PRESENT_IN_INSPECTED_BODY", "consumer guard")
require(evidence["status"]["same_owner_reuse_with_larger_count"] == "NOT_CAPTURED", "reuse status")
require(evidence["status"]["observed_overrun"] == "NOT_OBSERVED", "overrun status")
require(evidence["parent_receipts"][0]["manifest_sha256"] == PARENT_COUNT_MANIFEST, "count parent")
require(evidence["parent_receipts"][1]["manifest_sha256"] == PARENT_IDENTITY_MANIFEST, "identity parent")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} capacity_guard=NOT_SHOWN reuse=NOT_CAPTURED")
