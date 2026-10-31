#!/usr/bin/env python3
"""Verify the sanitized R138 reader-origin and same-call lineage package."""
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
manifest_digest = sha256(manifest_bytes)
sidecar = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip()
require(sidecar == f"{manifest_digest}  MANIFEST.json", "manifest sidecar mismatch")
require(manifest.get("schema") == "wash-evidence-manifest.v1", "unexpected manifest schema")
require(manifest.get("package_id") == "reader-origin-cursor-spatial-commit-wash-v1", "wrong package identity")
require(manifest.get("recipient_role") == "wash-agent", "wrong recipient role")
require(manifest.get("implementation_agent_contacted") is False, "implementation side was contacted")
require(manifest.get("implementation_delivery") == "PROHIBITED", "implementation delivery boundary changed")

files = manifest.get("files")
require(isinstance(files, list), "manifest file list missing")
expected_names = {
    "EVIDENCE.json",
    "CHECK.json",
    "README.md",
    "verify_reader_origin_cursor_spatial_commit.py",
}
require({entry.get("path") for entry in files} == expected_names, "unexpected package file set")
for entry in files:
    relative = Path(entry["path"])
    require(not relative.is_absolute() and ".." not in relative.parts, "unsafe manifest path")
    path = (ROOT / relative).resolve()
    require(path.parent == ROOT.resolve(), f"package file escaped its directory: {relative}")
    data = path.read_bytes()
    require(len(data) == entry["bytes"], f"byte length mismatch: {relative.name}")
    require(sha256(data) == entry["sha256"], f"content hash mismatch: {relative.name}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
check = json.loads((ROOT / "CHECK.json").read_text(encoding="utf-8"))
require(evidence.get("schema") == "reader-origin-cursor-spatial-commit-native-evidence.v1", "unexpected evidence schema")
require(check.get("schema") == "reader-origin-cursor-spatial-commit-check.v1", "unexpected check schema")
require(check.get("check_status") == "PASS", "check receipt is not PASS")
require(manifest.get("identity") == evidence.get("identity"), "manifest and evidence identity differ")

identity = evidence["identity"]
require(identity["authorized_input_sha256"] == identity["ascii_path_alias_sha256"], "authorized source alias hash mismatch")
require(identity["authorized_input_alias_hash_match"] is True, "authorized source alias identity not bound")
origin = evidence["reader_origin_and_cursor"]
selected = origin["selected_provider_call"]
position = origin["selected_call_absolute_adm_sample_position"]
expected_position = (
    origin["absolute_start_boundary_samples"]
    + origin["relative_start_sample"]
    + selected["cursor_before_samples"]
)
require(position["value"] == expected_position == 173286144, "selected-call ADM sample derivation mismatch")
require(position["status"] == "DERIVED_FOR_THIS_SELECTED_CALL", "position derivation was overclaimed")
require(position["native_absolute_position_field_observed"] is False, "native position field must remain not captured")
require(selected["cursor_before_samples"] + selected["requested_frame_samples"] <= selected["extent_samples"], "selected request exceeds reader extent")
require(origin["reader_state_initial_cursor"] == 0, "reader state initial cursor changed")
require(origin["source_offset_owner"] == "UNBOUND_AUTO_OFFSET_VS_FFOA_SOURCE", "offset owner was overclaimed")

lineage = evidence["same_call_codec_and_writer"]
codec = lineage["reader_provider_to_codec_input"]
writer = lineage["codec_output_to_file_writer"]
require(codec["provider_descriptor_token"] == codec["codec_input_descriptor_token"], "provider and codec descriptor tokens differ")
require(codec["same_descriptor_pointer_identity"] is True, "provider/codec pointer join missing")
require(codec["same_thread_and_parent_activation"] is True, "provider/codec activation join missing")
require(codec["codec_consumed_sample_span"] == selected["requested_frame_samples"], "provider/codec sample span mismatch")
require(codec["descriptor_field_1_semantics"] == "UNBOUND" and codec["descriptor_field_3_semantics"] == "UNBOUND", "raw descriptor semantics were guessed")
require(writer["encoded_buffer_begin_token"] == writer["writer_data_token"], "codec output/writer pointer token differs")
require(writer["encoded_byte_length"] == writer["writer_byte_length"], "codec output/writer lengths differ")
require(writer["writer_normal_return_observed"] is True, "writer normal return missing")
require(writer["writer_ordinal_is_generation_token"] is False, "writer ordinal cannot identify generation")
require(lineage["native_generation_field"] == "NOT_CAPTURED", "native generation status changed")

capture = evidence["capture"]
require(capture["raw_audio_or_pcm_persisted"] is False, "raw audio/PCM persistence reported")
require(capture["raw_pointer_values_persisted"] is False, "raw pointer persistence reported")
require(capture["model_execution"] == "NOT_RUN", "model execution must remain NOT_RUN")
require(capture["model_comparison"] == "NOT_EVALUATED", "model comparison must remain NOT_EVALUATED")
require(capture["gate1_closed"] is False, "Gate 1 cannot be marked closed")
require(evidence["delivery"]["implementation_agent_contacted"] is False, "implementation agent contact was reported")

sanitized_text = "\n".join(
    path.read_text(encoding="utf-8")
    for path in (ROOT / "EVIDENCE.json", ROOT / "CHECK.json", ROOT / "README.md")
)
require(re.search(r"\b0x[0-9a-fA-F]{6,}\b", sanitized_text) is None, "raw address found")
require(re.search(r"\b[A-Z]:\\", sanitized_text) is None, "absolute Windows path found")
require(re.search(r"(?i)\.dll\b|\bIDA\b|\bGhidra\b|\bWinDbg\b", sanitized_text) is None, "binary or analysis-tool identifier found")
require(manifest.get("contains_raw_addresses") is False, "raw addresses included")
require(manifest.get("contains_binary_excerpts") is False, "binary excerpts included")
require(manifest.get("contains_audio_or_pcm") is False, "audio/PCM included")
require(manifest.get("contains_reference_ec3_payload") is False, "reference payload included")
require(manifest.get("contains_private_license_material") is False, "private license material included")

print("READER_ORIGIN_CURSOR_SPATIAL_COMMIT_VERIFY=PASS")
print("AUTHORIZED_ADM_TO_ASCII_ALIAS=HASH_MATCH")
print("READER_CURSOR_TO_SELECTED_ABSOLUTE_SAMPLE=DERIVED_173286144")
print("PROVIDER_TO_CODEC_AND_CODEC_BUFFER_TO_WRITER=SAME_CALL_BOUND")
print("OFFSET_OWNER_AND_NATIVE_GENERATION=UNBOUND_OR_NOT_CAPTURED")
print("MODEL_REPLAY=NOT_RUN; GATE1_CLOSED=NO")
