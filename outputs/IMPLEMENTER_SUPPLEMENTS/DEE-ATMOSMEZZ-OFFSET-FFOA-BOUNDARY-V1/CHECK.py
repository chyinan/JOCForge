from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOSMEZZ-OFFSET-FFOA-BOUNDARY-V1"
SOURCE_MANIFEST = "fb51cbd50eec6876cda6394b51b106b5bca6b98b91a1d853878a836f2e7ec94b"
SOURCE_SEAL = "356633d861d3fc765f7eae888ba901030980ef2410281611b2b1e1ccc286adbe"
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

schema = evidence["schema_fact"]
require(schema["input_offset"]["type"] == "xs:string" and schema["input_offset"]["schema_default"] == "auto", "offset schema")
require(schema["input_ffoa"]["type"] == "xs:string" and schema["input_ffoa"]["schema_default"] == "auto", "ffoa schema")
require(evidence["host_normalizer"]["host_map_to_converter_privcontext"] == "UNBOUND", "host converter join")
require(evidence["converter_consumer"]["literal_auto_directly_accepted"] is False, "literal auto behavior")
require(evidence["status"]["owner_generation"] == "UNBOUND", "owner status")
require(evidence["status"]["reader_seek_sample"] == "NOT_CAPTURED", "reader seek status")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} schema=BOUND host_to_converter=UNBOUND")
