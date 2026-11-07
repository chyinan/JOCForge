from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-READER-N-ROW-LOOP-BOUNDARY-V1"
SOURCE_MANIFEST = "de85b168909fdd0b56d9c4ef6cff83a5b4bec68cc421a511174e0e4b75732d7a"
PARENT_MANIFEST = "febecb984e28ab1a9d4db891531c924a55984e073b8e177c4ee1a42b9a689796"
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
consumer = evidence["selected_reader_consumer"]
require(consumer["n_type"] == "U64", "Reader N type")
require(consumer["loop_bound"] == "uVar1 starts at 0 and continues while uVar1 < N", "N loop")
require(consumer["per_row_byte_extent"] == "sample_count * 4", "row extent")
require(evidence["separate_allocation_gate"]["gate_controls_n"] is False, "separate allocation gate")
require(evidence["status"]["runtime_n_value"] == "NOT_CAPTURED", "runtime N status")
require(evidence["status"]["general_channel_count_range"] == "NOT_ESTABLISHED", "range boundary")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} N_row_loop=BOUND_STATIC range=UNBOUND")
