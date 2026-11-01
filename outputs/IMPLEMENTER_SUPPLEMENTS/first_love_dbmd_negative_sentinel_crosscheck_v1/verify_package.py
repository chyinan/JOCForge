from __future__ import annotations
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(condition:bool,message:str)->None:
    if not condition: raise SystemExit(message)
raw=(ROOT/"MANIFEST.json").read_bytes()
expected=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(raw).hexdigest()==expected,"manifest hash mismatch")
m=json.loads(raw)
require(m["schema"]=="washed-first-love-dbmd-negative-sentinel-crosscheck.v1","schema mismatch")
require(m["package_id"]=="first_love_dbmd_negative_sentinel_crosscheck_v1","package mismatch")
require(m["status"]=="FIRST_LOVE_NEGATIVE_FFOA_SENTINEL_BRANCH_MATCHES_CANDIDATE_HELPER; POSITIVE_DELTA_BRANCH_UNEXERCISED","status mismatch")
require(m["implementation_readiness"]=="CANDIDATE_HELPER_METADATA_RULE_ONLY; POSITIVE_BRANCH_NOT_TESTED","scope widened")
require(m["audio_algorithm_inference_allowed"] is False,"algorithm scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["scope"]["input_sha256"]=="db3ccb02260882bbd830d22c0ab207e6b8a6916743e87c939e2caa8fb4be4335","input identity mismatch")
require(o["scope"]["dbmd_payload_sha256"]=="f490e0d73d5d572a0915609e518db5088158612b3b6fbf478f43c9859fa9145e","DBMD hash mismatch")
require(o["dbmd_time_tuple"]["type9_hours_or_sentinel"]["value"]==-1,"sentinel mismatch")
require(o["dbmd_time_tuple"]["type9_minutes"]["value"]==0 and o["dbmd_time_tuple"]["type9_subsecond_ticks"]["value"]==0,"tuple mismatch")
require(o["candidate_helper"]["predicted_final_ffoa_seconds"]==o["candidate_helper"]["observed_final_ffoa_seconds"]==3610.0,"helper formula result mismatch")
require(o["cross_sample_context"]["positive_nonnegative_tuple_branch"]=="NOT_TESTED","positive branch promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("FUN_180" not in t and re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"native coordinate leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("FIRST_LOVE_DBMD_NEGATIVE_SENTINEL=PASS")
print("POSITIVE_DBMD_DELTA_BRANCH=NOT_TESTED")
