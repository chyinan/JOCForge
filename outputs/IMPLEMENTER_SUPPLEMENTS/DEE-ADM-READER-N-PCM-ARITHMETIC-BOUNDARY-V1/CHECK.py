from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-READER-N-PCM-ARITHMETIC-BOUNDARY-V1"
SOURCE_MANIFEST = "011ac807da1d8c663e29bf7639d116d98b7716d1321bfda5949188c83e333686"
PARENT_CAPACITY_MANIFEST = "2cebbfe36bb6dce9f4935dcaaf2145e66ef693c6be378632ec350d3bc9d78718"
PARENT_N_GATE_MANIFEST = "79ac8fc623f6078e130795ce07de5493021f67fdef48e39a429f9f0ec6325e28"
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

contract = evidence["integer_contract"]
require(contract["element_product"] == "N * sample_count, low 64-bit IMUL result", "element product")
require(contract["element_product_overflow_guard"] == "NONE_OBSERVED", "N*S guard")
require(contract["overflow_allocator_argument"] == "UINT64_MAX", "byte overflow sentinel")
safe = evidence["derived_safe_domain"]
require(safe["uniform_safe_s_for_n_1_to_128"]["maximum"] == 36028797018963967, "uniform safe S")
require(safe["uniform_safe_s_for_n_1_to_128"]["formula"] == "2^55 - 1", "uniform bound formula")
require(evidence["runtime_status"]["allocator_result"] == "NOT_CAPTURED", "runtime allocator")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} N*S_guard=NONE byte_overflow=UINT64_MAX")
