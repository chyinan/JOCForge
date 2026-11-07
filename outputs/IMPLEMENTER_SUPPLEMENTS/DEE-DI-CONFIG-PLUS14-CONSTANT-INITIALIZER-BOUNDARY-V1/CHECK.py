from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-DI-CONFIG-PLUS14-CONSTANT-INITIALIZER-BOUNDARY-V1"
SOURCE_MANIFEST = "b81be512aee5d5cc296a36daf6eb639554d627c4b3c5cf479a17db22a618573d"
SOURCE_SEAL = "9f237b610e9bf32ff5bcd97c7b409236264a37174272b7ee4cee0705f01c278d"
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

field = evidence["normalized_meter_configuration"]
require(field["field_byte_offset_hex"] == "0x14", "field offset")
require(field["first_write"]["value"] == 1 and field["later_write"]["value"] == 1, "constant values")
require(field["first_write"]["order"].startswith("before"), "first-write order")
require(field["bool_to_field_plus_0x14_relation"].startswith("NOT_SAME_SOURCE"), "Bool relation")
require(evidence["status"]["setup_bool_plus_0x94_to_config_plus_0x14"] == "UNBOUND", "Bool edge")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
require(evidence["scope"]["dee_or_runtime_capture"] == "NOT_RUN", "runtime status")
print(f"PASS package={PACKAGE_ID} plus14_constant=BOUND bool_relation=UNBOUND")
