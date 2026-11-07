from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-DI-BOOL-PLUS00-FIRST-CONSUMER-BOUNDARY-V1"
SOURCE_MANIFEST = "5800986d25422fca144fb41309605043df1514c2a794e985bb9310d8383360ba"
SOURCE_SEAL = "1c8649f401df4df0013d94e6e89acdc418baa8a479633cc01bf562aeca6b7142"
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

path = evidence["bool_path"]
require(path["source_field_offset_hex"] == "0x94", "Bool source")
require(path["normalized_configuration_field_offset_hex"] == "0x00", "normalized destination")
require(path["producer_relation"] == "BOUND_STATIC", "Bool mapping")
require(path["first_downstream_field_read"] == "+0x14", "first consumer field")
require(path["first_downstream_reader_uses_bool_derived_plus_0x00"] is False, "Bool consumption")
require(evidence["status"]["bool_to_meter_result_dependency"] == "UNBOUND", "result boundary")
joined = json.dumps(evidence, ensure_ascii=True) + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw native address leak")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} bool_to_config=BOUND result_effect=UNBOUND")
