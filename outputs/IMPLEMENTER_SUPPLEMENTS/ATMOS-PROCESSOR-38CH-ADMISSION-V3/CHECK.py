from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
require(sha256(manifest_path) == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
require(manifest.get("package_id") == "ATMOS-PROCESSOR-38CH-ADMISSION-V3", "package id")
payload = manifest.get("files")
require(isinstance(payload, list) and len(payload) == 4, "payload inventory")
actual_names = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
require(actual_names == {entry.get("file") for entry in payload}, "payload file set")
for entry in payload:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"hash {entry['file']}")

seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
sealed = seal.get("files")
require(isinstance(sealed, list) and seal.get("file_count") == len(sealed), "seal inventory")
require({entry.get("file") for entry in sealed} == actual_names | {"MANIFEST.json", "MANIFEST.sha256"}, "sealed file set")
for entry in sealed:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"sealed size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"sealed hash {entry['file']}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
require(evidence["source_package"]["manifest_sha256"] == "0403c0a19f6dcc918361d12e68b83d4e04179626d3fdfd213d3a83ab424c1eaf", "source manifest binding")
require(evidence["source_package"]["sealed_file_count"] == 18, "source package file count")
require("+0x0A aliases CompositeMaster+0x1A" in evidence["static_field_chain"]["master_descriptor_alias"], "master field alias")
require("state slot 4" in evidence["static_field_chain"]["processor_expected_count"], "processor count field")
require(evidence["best_supported_rule"]["runtime_human_cat_value"] == "NOT_CAPTURED", "runtime count not claimed")
require(evidence["existing_receipt_audit"]["new_runtime_attempt"].startswith("NOT_RUN"), "no runtime attempt")
require(vector["runtime_execution"] == "NOT_RUN" and vector["specific_human_cat_value_captured"] is False, "vector boundary")
require(evidence["exclusions"]["private_key_or_license_contents_read"] is False, "private material exclusion")

print("PASS: receipt-audited 38-channel static admission and explicit NOT_RUN runtime value")
