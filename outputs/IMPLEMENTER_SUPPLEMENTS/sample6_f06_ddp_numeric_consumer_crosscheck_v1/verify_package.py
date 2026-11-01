from __future__ import annotations
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

manifest_raw = (ROOT / "MANIFEST.json").read_bytes()
expected_manifest_hash = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(manifest_raw).hexdigest() == expected_manifest_hash, "manifest hash mismatch")
manifest = json.loads(manifest_raw)
require(manifest["schema"] == "washed-sample6-f06-ddp-numeric-consumer.v1", "schema mismatch")
require(manifest["package_id"] == "sample6_f06_ddp_numeric_consumer_crosscheck_v1", "package id mismatch")
require(manifest["status"] == "EVIDENCE_ONLY; PCM24_SOURCE_AND_DIALNORM_JOINS_UNBOUND", "status mismatch")
require(manifest["implementation_ready"] == "NO", "implementation readiness mismatch")
require(manifest["audio_algorithm_inference_allowed"] is False, "audio algorithm scope widened")
for entry in manifest["payloads"]:
    name = entry["file"]
    payload_path = (ROOT / name).resolve()
    require(payload_path.is_relative_to(ROOT.resolve()), "payload escapes package root")
    require(payload_path.is_file(), "payload missing")
    payload = payload_path.read_bytes()
    require(len(payload) == entry["bytes"], "payload byte count mismatch")
    require(hashlib.sha256(payload).hexdigest() == entry["sha256"], "payload hash mismatch")
observation = json.loads((ROOT / "OBSERVATION.json").read_text(encoding="utf-8"))
require(observation["scope"]["native_job_count"] == 1, "run count mismatch")
require(observation["scope"]["second_native_job_started"] is False, "second run recorded")
require(observation["f06_result_copy"]["field_type"] == "Float32", "F06 type mismatch")
require(observation["f06_result_copy"]["bits"] == "0xc1864780", "F06 result bits mismatch")
require(observation["f06_result_copy"]["unit"] == "NOT_STATED", "F06 unit was invented")
require(observation["f06_result_copy"]["f06_float_to_dialnorm_u16_rule"] == "UNBOUND", "DialNorm edge promoted")
require(observation["f06_result_copy"]["generation_token"] == "UNBOUND", "generation join promoted")
require(observation["ddp_internal_row_numeric_consumer"]["one_observed_lane_bits"] == "0x38158000", "row observation mismatch")
require(observation["ddp_internal_row_numeric_consumer"]["pcm24_input_scalar"] == "NOT_CAPTURED", "PCM source join promoted")
require(observation["not_observed"]["selected_pcm24_reader_branch_and_bulk_store"] == "NOT_HIT", "PCM24 branch promoted")
require(observation["not_observed"]["five_bit_serializer_consumer"] == "NOT_HIT", "serializer consumer promoted")
require(observation["output_observation"]["output_retained_now"] is False, "deleted output claimed retained")
require(observation["output_observation"]["strict_validation_status"] == "FAIL_ALL_4247_ACCESS_UNITS", "strict result mismatch")
require(observation["output_observation"]["render_verified"] is False, "render claimed")
require(observation["output_observation"]["playback"] == "NOT_RUN", "playback promoted")
for name in ("WASH-REVIEW.md", "OBSERVATION.json", "VERIFICATION.json"):
    text = (ROOT / name).read_text(encoding="utf-8")
    require("FUN_180" not in text, "native function symbol leaked")
    require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])", text) is None, "instruction coordinate leaked")
    require("D:\\" not in text and "C:\\" not in text, "absolute path leaked")
    require("-----BEGIN" not in text, "key-like block leaked")
print("SAMPLE6_F06_DDP_CROSSCHECK=PASS")
print("AUDIO_ALGORITHM_INFERENCE=FORBIDDEN")
print("PCM24_SOURCE_AND_DIALNORM_JOINS=UNBOUND")
