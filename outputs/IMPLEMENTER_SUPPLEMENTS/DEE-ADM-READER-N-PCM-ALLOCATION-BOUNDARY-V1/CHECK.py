from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-READER-N-PCM-ALLOCATION-BOUNDARY-V1"
SOURCE_MANIFEST = "2cebbfe36bb6dce9f4935dcaaf2145e66ef693c6be378632ec350d3bc9d78718"
PARENT_MANIFEST = "de85b168909fdd0b56d9c4ef6cff83a5b4bec68cc421a511174e0e4b75732d7a"
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

initializer = evidence["pcm_initializer"]
require(initializer["required_elements"] == "N * sample_count", "element extent")
require(initializer["growth_condition"] == "capacity < N * sample_count", "capacity growth")
require(initializer["row_pointer_count"] == "N", "row count")
require(initializer["row_pointer_stride_bytes"] == "4 * sample_count", "row stride")
require(evidence["overflow"]["n_times_sample_count_guard"] == "NONE_OBSERVED", "overflow boundary")
require(evidence["overflow"]["overflow_allocator_argument"] == "UINT64_MAX", "byte overflow behavior")
require(evidence["status"]["allocator_outcome"] == "NOT_CAPTURED", "runtime allocation")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} extent=N*S overflow_boundary=BOUND")
