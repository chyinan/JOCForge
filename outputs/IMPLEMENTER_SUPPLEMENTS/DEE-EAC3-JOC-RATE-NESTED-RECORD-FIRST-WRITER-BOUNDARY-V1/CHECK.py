from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-EAC3-JOC-RATE-NESTED-RECORD-FIRST-WRITER-BOUNDARY-V1"
SOURCE_MANIFEST = "aa11109eb3d3557c6c200ac3869cf7c20ab755d6348692ac598733f105c84936"
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

nested = evidence["nested_record"]
require(nested["pointer_source"] == "writer receiver +0x2a8", "nested pointer source")
require(len(nested["fields_written"]) == 3, "write field count")
require(nested["constructor_or_type_owner"] == "UNBOUND_STATIC", "semantic owner")
require(evidence["status"]["codec_or_bsi_effect"] == "UNBOUND_STATIC", "codec/BSI status")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} nested_writer=BOUND semantic_owner=UNBOUND")
