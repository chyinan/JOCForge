from __future__ import annotations
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

manifest_raw=(ROOT/"MANIFEST.json").read_bytes()
expected=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(manifest_raw).hexdigest()==expected,"manifest hash mismatch")
manifest=json.loads(manifest_raw)
require(manifest["schema"]=="washed-helper-time-producer-candidate.v1","schema mismatch")
require(manifest["package_id"]=="helper_time_producer_candidate_closure_v1","package id mismatch")
require(manifest["status"]=="AXML_START_AND_DBMD_FFOA_CANDIDATE_RULES_BOUND; HISTORICAL_DEE_HELPER_JOIN_AND_FIXTURE_DBMD_TUPLE_UNBOUND","status mismatch")
require(manifest["implementation_ready"]=="CANDIDATE_HELPER_RULE_ONLY; DEE_HOST_JOIN_OPEN","scope widened")
for entry in manifest["payloads"]:
    p=(ROOT/entry["file"]).resolve()
    require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
    data=p.read_bytes()
    require(len(data)==entry["bytes"],"payload size mismatch")
    require(hashlib.sha256(data).hexdigest()==entry["sha256"],"payload hash mismatch")
observation=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(observation["scope"]["input_sha256"]=="7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1","input identity mismatch")
require(observation["axml_start_time"]["fixture_vector"]["helper_observed_seconds"]==3610.0,"helper fixture observation mismatch")
require(observation["dbmd_first_frame_of_action"]["decoded_fixture_tuple"]=="NOT_CAPTURED","tuple promoted")
require(observation["prior_native_sample_observation"]["interpretation"].startswith("SAME_INPUT_AND_EQUAL_FINAL_VALUES_ONLY"),"causality inferred")
require(observation["open_edges"]["helper_candidate_image_to_historical_dee_loaded_helper"]=="UNBOUND","host helper identity promoted")
require(observation["open_edges"]["new_dee_job_or_codec_execution"]=="NOT_RUN","DEE execution promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
    text=(ROOT/name).read_text(encoding="utf-8")
    require("FUN_180" not in text,"native function symbol leaked")
    require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",text) is None,"instruction coordinate leaked")
    require("D:\\" not in text and "C:\\" not in text,"absolute path leaked")
    require("-----BEGIN" not in text,"key-like block leaked")
print("HELPER_TIME_CANDIDATE_RULES=PASS")
print("DEE_HOST_HELPER_JOIN=UNBOUND")
print("FIXTURE_DBMD_TUPLE=NOT_CAPTURED")
