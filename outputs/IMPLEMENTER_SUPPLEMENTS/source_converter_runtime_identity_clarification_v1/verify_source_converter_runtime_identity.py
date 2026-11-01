#!/usr/bin/env python3
"""Verify the sanitized source-converter runtime identity clarification."""
from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
manifest = json.loads(manifest_bytes)
manifest_hash = sha256(manifest_bytes)
require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip() == f"{manifest_hash}  MANIFEST.json", "manifest sidecar mismatch")
require(manifest.get("schema") == "wash-evidence-manifest.v1", "manifest schema mismatch")
require(manifest.get("package_id") == "source-converter-runtime-identity-clarification-wash-v1", "wrong package identity")
require(manifest.get("recipient_role") == "wash-agent", "wrong recipient role")
require(manifest.get("implementation_agent_contacted") is False, "implementation side contact reported")
require(manifest.get("implementation_delivery") == "PROHIBITED", "delivery boundary changed")

expected_files = {
    "EVIDENCE.json",
    "CHECK.json",
    "README.md",
    "verify_source_converter_runtime_identity.py",
}
files = manifest.get("files")
require(isinstance(files, list) and {item.get("path") for item in files} == expected_files, "unexpected payload file set")
for item in files:
    relative = Path(item["path"])
    require(not relative.is_absolute() and ".." not in relative.parts, "unsafe manifest path")
    target = (ROOT / relative).resolve()
    require(target.parent == ROOT.resolve(), f"payload escaped package: {relative}")
    data = target.read_bytes()
    require(len(data) == item["bytes"], f"payload size mismatch: {relative.name}")
    require(sha256(data) == item["sha256"], f"payload hash mismatch: {relative.name}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
check = json.loads((ROOT / "CHECK.json").read_text(encoding="utf-8"))
require(manifest.get("identity") == evidence.get("identity_roles"), "manifest/evidence identity mismatch")
require(evidence.get("schema") == "source-converter-runtime-identity-clarification.v1", "evidence schema mismatch")
require(check.get("schema") == "source-converter-runtime-identity-check.v1", "check schema mismatch")
require(check.get("check_status") == "PASS", "check status is not PASS")

identities = evidence["identity_roles"]
static_copy = identities["r139_static_image"]
runtime_image = identities["r128_runtime_image"]
disposition = evidence["role_and_build_disposition"]
require(static_copy["component_role"] == runtime_image["component_role"] == "ADM_INPUT_CONVERTER", "component role mismatch")
require(static_copy["sha256"] != runtime_image["sha256"], "distinct image hashes collapsed")
require(disposition["r139_and_r128_component_roles_match"] is True, "same semantic role not recorded")
require(disposition["r139_and_r128_images_byte_identical"] is False, "different byte identity not recorded")
require(disposition["version_or_build_incompatibility_inferred_from_hash_difference"] is False, "hash difference was overinterpreted")
require(disposition["r138_runtime_image_join_to_9972"] == "UNBOUND", "R138 converter join was overclaimed")
require(disposition["r139_ffoa_auto_contract_for_r138"] == "BUILD_IDENTITY_UNJOINED_PROVISIONAL", "R138 FFOA scope was overclaimed")
require(identities["r138_reference_run"]["input_converter_image_sha256"] == "NOT_CAPTURED", "R138 input converter identity changed")
require(identities["r128_runtime_storage_support"]["source_virtual_field_writer"] == "NOT_RESOLVED_IN_SCOPED_STATIC_SEARCH", "source writer was overclaimed")

range_status = evidence["range_and_remaining_edges"]
require(range_status["ffoa_auto_policy"] == "AUTO_SKIPS_EXPLICIT_SETTER_AND_RETAINS_SOURCE_FFOA", "FFOA-auto contract changed")
require(range_status["file_position_foa_start_rule"] == "max(offset_seconds, ffoa_seconds) - offset_seconds", "range formula changed")
require(range_status["offset_auto_resolution"] == "UNBOUND", "offset auto rule was overclaimed")
require(range_status["spatial_fit_first_sample_position_consumer"] == "NOT_OBSERVED", "spatial/fit join was overclaimed")
require(range_status["model_execution"] == "NOT_RUN", "model execution must remain NOT_RUN")
require(range_status["gate1_closed"] is False, "Gate 1 cannot be marked closed")

sanitized_text = "\n".join(
    path.read_text(encoding="utf-8")
    for path in (ROOT / "EVIDENCE.json", ROOT / "CHECK.json", ROOT / "README.md")
)
require(re.search(r"\b0x[0-9a-fA-F]{6,}\b", sanitized_text) is None, "raw address found")
require(re.search(r"\b[A-Z]:\\", sanitized_text) is None, "absolute path found")
require(re.search(r"(?i)\.dll\b|\bIDA\b|\bGhidra\b|\bWinDbg\b", sanitized_text) is None, "binary/tool identifier found")
require(manifest.get("contains_sample_names_or_paths") is False, "sample labels or paths included")
require(manifest.get("contains_raw_addresses") is False, "raw addresses included")
require(manifest.get("contains_audio_or_pcm") is False, "audio/PCM included")
require(manifest.get("contains_private_license_material") is False, "license material included")

print("SOURCE_CONVERTER_RUNTIME_IDENTITY_VERIFY=PASS")
print("R139_STATIC_AND_R128_RUNTIME=SAME_ROLE_DISTINCT_HASHES; NO_INCOMPATIBILITY_INFERENCE")
print("FFOA_AUTO=STATICALLY_CONFIRMED_FOR_9972_ONLY")
print("R138_CONVERTER_IMAGE=NOT_CAPTURED; R138_JOIN=UNBOUND")
print("SOURCE_WRITER=UNBOUND; SPATIAL_FIT_JOIN=NOT_OBSERVED; GATE1_CLOSED=NO")
