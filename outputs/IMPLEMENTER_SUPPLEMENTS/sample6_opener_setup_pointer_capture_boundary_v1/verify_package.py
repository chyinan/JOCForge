from __future__ import annotations
import hashlib
import json
import re
from pathlib import Path

ROOT=Path(__file__).resolve().parent

def require(condition: bool,message: str)->None:
    if not condition:
        raise SystemExit(message)

manifest_raw=(ROOT/"MANIFEST.json").read_bytes()
expected=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(manifest_raw).hexdigest()==expected,"manifest hash mismatch")
manifest=json.loads(manifest_raw)
require(manifest["schema"]=="washed-sample6-opener-setup-pointer-capture.v1","schema mismatch")
require(manifest["package_id"]=="sample6_opener_setup_pointer_capture_boundary_v1","package id mismatch")
require(manifest["status"]=="OUTPUT_METADATA_OBSERVED; POINTER_IDENTITY_NOT_CAPTURED; INSTRUMENTATION_UNRESOLVED","status mismatch")
require(manifest["implementation_ready"]=="NO","implementation readiness mismatch")
require(manifest["audio_algorithm_inference_allowed"] is False,"audio algorithm scope widened")
for entry in manifest["payloads"]:
    p=(ROOT/entry["file"]).resolve()
    require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
    data=p.read_bytes()
    require(len(data)==entry["bytes"],"payload size mismatch")
    require(hashlib.sha256(data).hexdigest()==entry["sha256"],"payload hash mismatch")
observation=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(observation["scope"]["native_job_started_once"] is True,"run scope mismatch")
require(observation["scope"]["second_native_job_started"] is False,"second job was reported")
require(observation["instrumentation"]["breakpoints_hit"]==0,"breakpoint hit status mismatch")
require(observation["instrumentation"]["same_generation_pointer_equality"]=="NOT_EVALUATED","pointer join promoted")
require("DOES NOT SHOW THAT THE CODE SITES OR EVENTS ARE ABSENT" in observation["instrumentation"]["instrumentation_result"],"non-hit overstated")
require(observation["output_observation"]["sha256"]=="34d18a6fb0f477cbccd0b7d633a80b4191c0c535385ff66b8d9cbf7ff1440448","output hash mismatch")
require(observation["output_observation"]["output_retained_now"] is False,"deleted output claimed retained")
require(observation["output_observation"]["render_verified"] is False,"render claimed")
require(observation["output_observation"]["playback"]=="NOT_RUN","playback promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
    text=(ROOT/name).read_text(encoding="utf-8")
    require("FUN_180" not in text,"native function symbol leaked")
    require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",text) is None,"instruction coordinate leaked")
    require("D:\\" not in text and "C:\\" not in text,"absolute path leaked")
    require("-----BEGIN" not in text,"key-like block leaked")
print("SAMPLE6_POINTER_CAPTURE_BOUNDARY=PASS")
print("POINTER_IDENTITY=UNBOUND")
