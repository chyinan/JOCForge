from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-TIME-BASE-HOST-SETTER-PROXY-BOUNDARY-V1"
SOURCE_MANIFEST = "30a929a9e8a4890fe53ec1041bf0178faed0501be01a92ffeec17ee0034548ef"
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

edge = evidence["generic_setter"]
require(edge["explicit_stage_descriptor_plus_0x550_operand"] is False, "+0x550 operand")
require(edge["bounded_direct_callsite_count"] == 8, "bounded callsite count")
require(edge["concrete_proxy_vtable_target"] == "UNBOUND", "proxy target")
require(evidence["status"]["xml_time_base_start_end_to_ddp_descriptor"] == "UNBOUND", "XML transfer")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} generic_setter=BOUND ddp_target=UNBOUND")
