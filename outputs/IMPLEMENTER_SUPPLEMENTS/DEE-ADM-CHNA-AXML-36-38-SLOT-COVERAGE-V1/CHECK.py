from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-CHNA-AXML-36-38-SLOT-COVERAGE-V1"
SOURCE_MANIFEST = "e409fbffde7f08924edffac49c790669a06a94c63b5a83c55c20230a12775c51"
SOURCE_SEAL = "a82c7f6e331d6b81e1d95cb659c8e92ce5831169505ee2c8942045f1dbef90fb"
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

samples = evidence["sample_coverage"]
require([row["physical_n"] for row in samples] == [36, 38], "sample N")
require(all(row["slot_bijection"] == "EXACT_ONE_TO_ONE" for row in samples), "slot bijection")
require(all(row["missing_or_duplicate_indices"] is False for row in samples), "index coverage")
require(evidence["read_boundary"]["data_chunk_payload_read"] is False, "data payload")
require(evidence["read_boundary"]["pcm_bytes_read"] == 0, "PCM read")
require(evidence["limits"]["runtime_dee_acceptance"] == "NOT_RUN", "runtime status")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} N36_N38_slot_bijection=BOUND pcm_bytes=0")
