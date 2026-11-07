from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-TIME-AXIS-XML-START-END-SCHEMA-BOUNDARY-V1"
SOURCE_MANIFEST = "08fe6b2e98e173deb51044595c8362471e11c2fd2bba6480eace74cf807969a0"
SOURCE_SEAL = "c615ff7e7944725c73f54c7831f32ac4b646e195e1e9ec79fbb715d18fb615b8"
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

elements = {row["name"]: row for row in evidence["schema_fact"]["elements"]}
require(elements["start"]["required"] is False and elements["start"]["type"] == "xs:string", "start schema")
require(elements["start"]["schema_default"] == "first_frame_of_action", "start default")
require(elements["end"]["required"] is False and elements["end"]["type"] == "xs:string", "end schema")
require(elements["end"]["schema_default"] == "end_of_file", "end default")
require(evidence["schema_fact"]["executing_dee_binding"] == "UNBOUND", "XSD binding")
require(evidence["runtime_and_native_joins"]["selected_start_value"] == "UNBOUND", "selected start")
require(evidence["runtime_and_native_joins"]["selected_end_value"] == "UNBOUND", "selected end")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} schema_only=True runtime_values=UNBOUND")
