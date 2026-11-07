from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-XML-TIME-CONTROL-READER-BOUNDARY-V1"


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
expected = {"REPORT.md", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "SOURCE-MANIFESTS.json", "CHECK.py"}
payloads = {item["file"]: item for item in manifest["payloads"]}
require(set(payloads) == expected, "payload set mismatch")
for name, item in payloads.items():
    path = ROOT / name
    require(path.is_file(), f"missing payload: {name}")
    require(path.stat().st_size == item["bytes"], f"payload size mismatch: {name}")
    require(sha256(path) == item["sha256"], f"payload hash mismatch: {name}")
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
require(evidence["implementation_ready"] is False, "time-control effect must remain reference-only")
require(evidence["start_end"]["xml_to_typed_range_owner"] == "UNBOUND", "start/end join must remain unbound")
require(evidence["input_offset_ffoa"]["xml_to_policy_owner"] == "UNBOUND", "offset/ffoa join must remain unbound")
require(evidence["input_offset_ffoa"]["host_response_to_converter_context"] == "UNBOUND", "converter join must remain unbound")
require(evidence["execution"]["dee_run"] == "NOT_RUN", "DEE must remain not run")
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
require(vectors["status"] == "SOURCE_DOCUMENT_VALUES_ONLY; NOT_EXECUTED", "input vector status mismatch")
require([item["name"] for item in vectors["domains"]] == ["encode_to_atmos_ddp_time_control", "input_audio_adm_offset_ffoa"], "time domains collapsed")
print(f"PASS mode={'optimized' if not __debug__ else 'normal'} time domains=distinct runtime=NOT_RUN")
