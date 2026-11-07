from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-DI-EMPIRE-BOOL-RESULT-FLOAT32-FIRST-BREAK-BOUNDARY-V1"
SOURCE_MANIFEST = "b79a9a9cabb53c33e126c0a6b5195185a031685bd2ebd01ec5fc66b970c04036"
SOURCE_SEAL = "bef27cd37618f210c7183c3561e6413477c900f5e4154ae5dd285067c9930c69"
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

path = evidence["same_owner_bool_path"]
require(path["setup_bool_source_offset"] == "+0x94", "setup Bool")
require(path["owner_bool_copy_offset"] == "+0x2c", "owner Bool copy")
require(path["config_bool_field_offset"] == "+0x44", "config Bool")
result = evidence["float32_result_path"]
require(result["result_type"] == "Float32", "result type")
require(result["subcontext_to_result_numeric_dependency"] == "UNBOUND", "result dependency")
require(evidence["status"]["runtime_branch_selection"] == "UNBOUND_NOT_CAPTURED", "runtime mode")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} Bool_to_subcontext=CONDITIONAL result_causality=UNBOUND")
