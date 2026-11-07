from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-EAC3-JOC-DATA-RATE-WRITER-RETURN-CONSUMER-BOUNDARY-V1"
SOURCE_MANIFEST = "94efe989c4d57916a0f86977f49b0a9966be878b36a84ad1386308e843171210"
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

require(evidence["writer_state"]["stored_value_type"] == "U32", "writer value type")
require(evidence["writer_state"]["destination_object_field_offset_hex"] == "0x4", "destination field")
require(evidence["status"]["post_return_pointer"] == "BOUND_STATIC", "return edge")
require(evidence["status"]["destination_plus_0x4_first_reader"] == "UNBOUND", "reader boundary")
require(evidence["post_return_path"]["second_argument_consumed"] is False, "callee argument use")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
require(evidence["scope"]["dee_or_runtime_capture"] == "NOT_RUN", "runtime status")
print(f"PASS package={PACKAGE_ID} return_edge=BOUND destination_reader=UNBOUND")
