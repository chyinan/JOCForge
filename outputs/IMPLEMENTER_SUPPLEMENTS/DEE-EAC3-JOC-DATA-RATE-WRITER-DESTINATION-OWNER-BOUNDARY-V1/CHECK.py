from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-EAC3-JOC-DATA-RATE-WRITER-DESTINATION-OWNER-BOUNDARY-V1"
SOURCE_MANIFEST = "78428df8f49605218df857ce851052abf54dd19ac3f2ab179c81ea384f155d43"
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

writer = evidence["generic_writer"]
require(writer["stored_value_type"] == "U32", "stored type")
require(writer["destination_child_field_offset_hex"] == "0x4", "destination field")
require(writer["stored_value_source"] == {"owner_pointer_field": "writer receiver +0x2a8", "value_field_offset": "+0x10", "type": "U32"}, "value source")
require(writer["child_owner_type"] == "UNBOUND_STATIC", "owner status")
require(evidence["bounded_continuation"]["next_callee_consumes_second_argument"] is False, "RDX boundary")
require(evidence["status"]["destination_first_reader"] == "UNBOUND_STATIC", "reader status")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} store=BOUND owner_reader=UNBOUND")
