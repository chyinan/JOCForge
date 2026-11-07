from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-STAGERUNNER-OBSERVED-STRING-OBJECT-BOUNDARY-V1"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
manifest = json.loads(manifest_bytes)
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
manifest_sha = hashlib.sha256(manifest_bytes).hexdigest()
require(manifest["package_id"] == PACKAGE_ID, "manifest package id mismatch")
require(seal["package_id"] == PACKAGE_ID, "seal package id mismatch")
require(seal["manifest_sha256"] == manifest_sha, "seal manifest binding mismatch")

payloads = {entry["file"]: entry for entry in manifest["payloads"]}
require(
    set(payloads) == {"REPORT.md", "EVIDENCE.json", "SOURCE-MANIFESTS.json", "CHECK.py"},
    "unexpected payload set",
)
for name, entry in payloads.items():
    path = ROOT / name
    require(path.is_file(), f"missing payload: {name}")
    require(path.stat().st_size == entry["bytes"], f"payload byte mismatch: {name}")
    require(sha256(path) == entry["sha256"], f"payload hash mismatch: {name}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
require(evidence["implementation_ready"] is False, "scope must remain reference-only")
require(evidence["status"].endswith("TIME_BASE_VALUE_NOT_CAPTURED"), "time_base status mismatch")
require(evidence["attempt_13"]["native_time_base_value"] == "NOT_CAPTURED", "value must remain not captured")
require(evidence["attempt_13"]["time_base_to_adm_owner_generation_join"] == "UNBOUND", "owner join mismatch")
require(evidence["scope"]["time_base_key_observed_at_these_callers"] is False, "caller scope mismatch")
require(evidence["observed_string_like_representation"]["cpp_type_identity"] == "CANDIDATE_ONLY", "type identity mismatch")
require("native source/decompilation" in json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))["excluded"], "exclusion missing")

print("PASS: hash-bound reference-only supplement; time_base remains NOT_CAPTURED/UNBOUND")
