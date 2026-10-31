#!/usr/bin/env python3
"""Verify the sanitized FFOA-auto and range-consumer addendum."""
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
sidecar = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip()
require(sidecar == f"{manifest_hash}  MANIFEST.json", "manifest sidecar mismatch")
require(manifest.get("schema") == "wash-evidence-manifest.v1", "manifest schema mismatch")
require(manifest.get("package_id") == "ffoa-auto-retention-range-branch-wash-v1", "wrong package identity")
require(manifest.get("recipient_role") == "wash-agent", "wrong recipient role")
require(manifest.get("implementation_agent_contacted") is False, "implementation side contact reported")
require(manifest.get("implementation_delivery") == "PROHIBITED", "delivery boundary changed")

expected_files = {
    "EVIDENCE.json",
    "CHECK.json",
    "README.md",
    "verify_ffoa_auto_retention_range_branch.py",
}
files = manifest.get("files")
require(isinstance(files, list) and {item.get("path") for item in files} == expected_files, "unexpected package file set")
for item in files:
    relative = Path(item["path"])
    require(not relative.is_absolute() and ".." not in relative.parts, "unsafe payload path")
    target = (ROOT / relative).resolve()
    require(target.parent == ROOT.resolve(), f"payload escaped package: {relative}")
    data = target.read_bytes()
    require(len(data) == item["bytes"], f"payload byte length mismatch: {relative.name}")
    require(sha256(data) == item["sha256"], f"payload hash mismatch: {relative.name}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
check = json.loads((ROOT / "CHECK.json").read_text(encoding="utf-8"))
require(manifest.get("identity") == evidence.get("identity"), "manifest/evidence identity mismatch")
require(evidence.get("schema") == "ffoa-auto-retention-range-branch-evidence.v1", "evidence schema mismatch")
require(check.get("schema") == "ffoa-auto-retention-range-branch-check.v1", "check schema mismatch")
require(check.get("check_status") == "PASS", "check status is not PASS")

auto = evidence["ffoa_auto_policy"]
require(auto["configuration_field"] == "ffoa" and auto["default_value"] == "auto", "FFOA auto field identity mismatch")
require(auto["schema_description_semantics"] == "RETAIN_FFOA_FROM_SOURCE", "FFOA auto meaning mismatch")
require(auto["generic_property_loader_auto_branch"] == "SKIPS_EXPLICIT_SETTER_WHEN_VALUE_IS_AUTO", "auto branch behavior mismatch")
require(auto["source_ffoa_field_writer"] == "UNBOUND_VIRTUAL_METADATA_IMPLEMENTATION", "source writer was overclaimed")

consumer = evidence["first_frame_of_action_file_position_consumer"]
require(consumer["effective_relative_start_seconds"] == "max(offset_seconds, ffoa_seconds) - offset_seconds", "range formula mismatch")
require(consumer["r138_native_values_for_authorized_sample"]["offset_seconds"] == 3610.0, "R138 offset value mismatch")
require(consumer["r138_native_values_for_authorized_sample"]["ffoa_seconds"] == 3610.0, "R138 FFOA value mismatch")
require(consumer["r138_native_values_for_authorized_sample"]["resulting_relative_start_samples"] == 0, "equal-pair range result mismatch")
require(consumer["r138_native_values_for_authorized_sample"]["priority_discrimination"] == "NONE_EQUAL_INPUTS", "equal pair was treated as discriminating")
require(evidence["offset_field"]["offset_auto_resolution_rule"] == "UNBOUND", "offset auto rule was overclaimed")
require(evidence["remaining_edges"]["first_absolute_sample_position_consumer_in_spatial_or_fit"] == "NOT_OBSERVED", "spatial/fit consumer was overclaimed")
require(evidence["remaining_edges"]["model_execution"] == "NOT_RUN", "model execution must remain NOT_RUN")
require(evidence["remaining_edges"]["gate1_closed"] is False, "Gate 1 cannot be marked closed")

sanitized_text = "\n".join(
    path.read_text(encoding="utf-8")
    for path in (ROOT / "EVIDENCE.json", ROOT / "CHECK.json", ROOT / "README.md")
)
require(re.search(r"\b0x[0-9a-fA-F]{6,}\b", sanitized_text) is None, "raw address found")
require(re.search(r"\b[A-Z]:\\", sanitized_text) is None, "absolute path found")
require(re.search(r"(?i)\.dll\b|\bIDA\b|\bGhidra\b|\bWinDbg\b", sanitized_text) is None, "binary/tool identifier found")
require(manifest.get("contains_sample_names_or_paths") is False, "sample label/path included")
require(manifest.get("contains_raw_addresses") is False, "raw address included")
require(manifest.get("contains_audio_or_pcm") is False, "audio/PCM included")
require(manifest.get("contains_private_license_material") is False, "license material included")

print("FFOA_AUTO_RETENTION_RANGE_BRANCH_VERIFY=PASS")
print("FFOA_AUTO=SOURCE_RETAINED_BY_SCHEMA_AND_NO_SETTER_BRANCH")
print("FILE_POSITION_FOA_START=max(offset,ffoa)-offset")
print("OFFSET_AUTO_RULE=UNBOUND; SOURCE_WRITER=UNBOUND")
print("SPATIAL_FIT_POSITION_JOIN=NOT_OBSERVED; GATE1_CLOSED=NO")
