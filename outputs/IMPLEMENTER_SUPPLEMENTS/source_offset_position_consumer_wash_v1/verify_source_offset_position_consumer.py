#!/usr/bin/env python3
"""Verify the sanitized source-offset and first range-consumer addendum."""
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
require(manifest.get("package_id") == "source-offset-position-consumer-wash-v1", "wrong package id")
require(manifest.get("recipient_role") == "wash-agent", "wrong recipient role")
require(manifest.get("implementation_agent_contacted") is False, "implementation side contact reported")
require(manifest.get("implementation_delivery") == "PROHIBITED", "delivery boundary changed")

expected_files = {
    "EVIDENCE.json",
    "CHECK.json",
    "README.md",
    "verify_source_offset_position_consumer.py",
}
files = manifest.get("files")
require(isinstance(files, list) and {item.get("path") for item in files} == expected_files, "unexpected payload file set")
for item in files:
    relative = Path(item["path"])
    require(not relative.is_absolute() and ".." not in relative.parts, "unsafe manifest path")
    target = (ROOT / relative).resolve()
    require(target.parent == ROOT.resolve(), f"payload path escaped package: {relative}")
    data = target.read_bytes()
    require(len(data) == item["bytes"], f"payload size mismatch: {relative.name}")
    require(sha256(data) == item["sha256"], f"payload digest mismatch: {relative.name}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
check = json.loads((ROOT / "CHECK.json").read_text(encoding="utf-8"))
require(manifest.get("identity") == evidence.get("identity"), "manifest/evidence identity mismatch")
require(evidence.get("schema") == "source-offset-position-consumer-evidence.v1", "evidence schema mismatch")
require(check.get("schema") == "source-offset-position-consumer-check.v1", "check schema mismatch")
require(check.get("check_status") == "PASS", "check receipt is not PASS")

identity = evidence["identity"]
samples = identity["samples"]
require(len(samples) == 7, "seven authorized sample identities required")
digests = [item["sha256"] for item in samples]
require(len(set(digests)) == 7, "sample identities are not unique")
require(all(re.fullmatch(r"[0-9a-f]{64}", digest) for digest in digests), "invalid sample SHA-256")
require(identity["authorized_sample_hashes_rechecked"] is True, "current sample hash recheck missing")

metadata = evidence["seven_sample_metadata_scan"]
require(metadata["source_file_hashes_match_prior_identity_record"] is True, "source identity did not match prior metadata receipt")
require(metadata["data_chunk_bytes_read_by_metadata_scanner"] == 0, "metadata scanner read audio data")
require(metadata["sample_values_decoded_or_saved"] is False, "audio samples were decoded or saved")
require(metadata["bext_time_reference_present_count"] == 0, "BEXT reference observation changed")
require(metadata["explicit_axml_offset_or_ffoa_field_present_count"] == 0, "explicit XML offset/FFOA observation changed")
require(metadata["programme_start_is_same_across_all_seven"] is True, "programme start distinction appeared")
require(metadata["adm_audio_programme_start_seconds_derived"] == [3610.0], "unexpected programme start scalar")
require(metadata["discriminating_offset_vs_ffoa_sample_identified"] is False, "sample pair was overclaimed")

native = evidence["r138_native_scalar_observation"]
require(native["native_ffoa_seconds"] == native["native_offset_seconds"] == 3610.0, "R138 native scalar pair changed")
require(native["other_six_native_offset_ffoa_pairs"] == "NOT_CAPTURED", "other sample pairs were overclaimed")

producer = evidence["typed_source_metadata_edge"]
require(producer["source_time_field_a"]["normalized_metadata_key"] == "offset", "source offset field mapping mismatch")
require(producer["source_time_field_b"]["normalized_metadata_key"] == "ffoa", "source FFOA field mapping mismatch")
require(producer["source_time_field_a"]["numeric_consumption_type"] == "IEEE754_BINARY64_SECONDS", "offset type mismatch")
require(producer["source_time_field_b"]["numeric_consumption_type"] == "IEEE754_BINARY64_SECONDS", "FFOA type mismatch")
require(producer["source_field_writer_implementation"] == "UNBOUND_VIRTUAL_METADATA_IMPLEMENTATION", "upstream writer was overclaimed")
require(producer["ffoa_auto_schema_semantics"] == "AUTO_RETAINS_FFOA_FROM_SOURCE", "FFOA auto semantics mismatch")
require(producer["offset_auto_resolution_rule"] == "UNBOUND", "offset auto resolution was overclaimed")

consumer = evidence["first_explicit_range_consumer"]
require(consumer["first_frame_of_action_relative_start_seconds"] == "max(offset_seconds, ffoa_seconds) - offset_seconds", "file-position FFOA branch mismatch")
require(consumer["multi_file_sample_count_is_separate_from_alignment_offset"] is True, "sample count domain was conflated")
require(consumer["r138_equal_value_result_samples"] == 0, "equal-pair arithmetic mismatch")
require(evidence["spatial_fit_position_join"]["first_typed_consumer_in_spatial_or_fit"] == "NOT_OBSERVED", "spatial/fit join was overclaimed")
require(evidence["spatial_fit_position_join"]["writer_ordinal_is_generation"] is False, "writer ordinal used as generation")
require(evidence["gate_status"]["model_execution"] == "NOT_RUN", "model execution must remain NOT_RUN")
require(evidence["gate_status"]["gate1_closed"] is False, "Gate 1 cannot be marked closed")

sanitized_text = "\n".join(
    path.read_text(encoding="utf-8")
    for path in (ROOT / "EVIDENCE.json", ROOT / "CHECK.json", ROOT / "README.md")
)
require(re.search(r"\b0x[0-9a-fA-F]{6,}\b", sanitized_text) is None, "raw address found")
require(re.search(r"\b[A-Z]:\\", sanitized_text) is None, "absolute path found")
require(re.search(r"(?i)\.dll\b|\bIDA\b|\bGhidra\b|\bWinDbg\b", sanitized_text) is None, "binary/tool identifier found")
require(evidence["delivery"]["contains_sample_names_or_paths"] is False, "sample labels or paths were included")
require(manifest.get("contains_audio_or_pcm") is False, "audio/PCM was included")
require(manifest.get("contains_private_license_material") is False, "license material was included")

print("SOURCE_OFFSET_POSITION_CONSUMER_VERIFY=PASS")
print("AUTHORIZED_SAMPLES=7_HASHES_MATCHED_METADATA_ONLY")
print("PROGRAMME_STARTS=ALL_3610_SECONDS; NO_DISCRIMINATING_PAIR_FOUND")
print("FFOA_AUTO=SOURCE_RETAIN; SOURCE_FIELD_WRITER=UNBOUND")
print("RANGE_ORIGIN_EDGE=MUST_HAVE; SPATIAL_FIT_JOIN=NOT_OBSERVED")
print("MODEL_REPLAY=NOT_RUN; GATE1_CLOSED=NO")
