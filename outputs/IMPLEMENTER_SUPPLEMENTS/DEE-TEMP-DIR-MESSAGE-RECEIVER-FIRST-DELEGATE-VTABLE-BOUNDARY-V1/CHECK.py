from __future__ import annotations
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

manifest = json.loads((root / "MANIFEST.json").read_text(encoding="utf-8"))
seal = json.loads((root / "package_seal.json").read_text(encoding="utf-8"))
require(manifest["schema"] == "jocforge.wash_supplement_manifest.v1", "manifest schema mismatch")
require(manifest["package_id"] == seal["package_id"] == "DEE-TEMP-DIR-MESSAGE-RECEIVER-FIRST-DELEGATE-VTABLE-BOUNDARY-V1", "package identity mismatch")
require(manifest["status"] == seal["status"], "status mismatch")
require(seal["manifest_sha256"] == hashlib.sha256((root / "MANIFEST.json").read_bytes()).hexdigest(), "manifest hash mismatch")
require(seal["payload_file_count"] == len(manifest["payloads"]) == 5, "payload count mismatch")
require(seal["payload_hash_audit"] == "PASS_5_OF_5", "payload audit not passed")
require(seal["checker_status"] in {"PENDING", "PASS_NORMAL_AND_PYTHON_O"}, "checker status mismatch")
expected_names = {"REPORT.md", "EVIDENCE.json", "SOURCE-MANIFESTS.json", "INPUT-ONLY-VECTORS.json", "CHECK.py"}
require({entry["file"] for entry in manifest["payloads"]} == expected_names, "payload set mismatch")
for payload in manifest["payloads"]:
    data = (root / payload["file"]).read_bytes()
    require(len(data) == payload["bytes"], f"size mismatch: {payload['file']}")
    require(hashlib.sha256(data).hexdigest() == payload["sha256"], f"hash mismatch: {payload['file']}")
evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
require(evidence["return_object"]["caller_local"] == "local_98", "caller local mismatch")
require(evidence["return_object"]["storage_size_bytes"] == 80, "caller local size mismatch")
require(evidence["first_delegate"]["vtable_slot"] == "+0x18", "vtable slot mismatch")
require(evidence["first_delegate"]["resolved_function_boundary"] == "FUN_14006c760", "target function boundary mismatch")
require(evidence["first_delegate"]["target_body_inspected"] is False, "target-body scope expanded")
require(evidence["clean_temp"]["explicit_argument_to_observed_message_call"] is False, "clean_temp input overclaim")
require(evidence["clean_temp"]["cleanup_effect"] == "UNBOUND", "cleanup effect overclaim")
require(evidence["runtime"]["owner_generation"] == "NOT_CAPTURED", "runtime owner claim overreach")
require(evidence["runtime"]["dee_run"] == "NOT_RUN", "run status changed")
require(evidence["runtime"]["implementation_ready"] is False, "readiness overclaim")
print("TEMP_MESSAGE_RECEIVER_SUPPLEMENT_CHECK=PASS")
