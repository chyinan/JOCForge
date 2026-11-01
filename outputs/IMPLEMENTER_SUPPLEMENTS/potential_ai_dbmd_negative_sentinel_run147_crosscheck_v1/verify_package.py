from __future__ import annotations
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(condition:bool,message:str)->None:
    if not condition: raise SystemExit(message)
raw=(ROOT/"MANIFEST.json").read_bytes();expected=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(raw).hexdigest()==expected,"manifest hash mismatch")
m=json.loads(raw)
require(m["schema"]=="washed-potential-ai-dbmd-negative-sentinel-run147.v1","schema mismatch")
require(m["package_id"]=="potential_ai_dbmd_negative_sentinel_run147_crosscheck_v1","package mismatch")
require(m["status"]=="POTENTIAL_AI_NEGATIVE_FFOA_SENTINEL_JOINED_TO_RUN147_METADATA; POSITIVE_DELTA_UNTESTED","status mismatch")
require(m["implementation_readiness"]=="CANDIDATE_HELPER_METADATA_RULE_ONLY; NO_HELPER_QUERY_NEEDED_FOR_NEGATIVE_SENTINEL","scope mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes();require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["input"]["sha256"]=="96e34c167b552928fa9ef287d358c74dac3a2d135a86a0ef4d1529c7d51e8b82","input hash mismatch")
require(o["dbmd"]["sha256"]=="f490e0d73d5d572a0915609e518db5088158612b3b6fbf478f43c9859fa9145e","DBMD hash mismatch")
require(o["dbmd"]["type9_tuple"]=={"signed_int8_hours_or_sentinel":-1,"uint8_minutes":0,"signed_int32_le_ticks":0},"tuple mismatch")
require(o["run147_join"]["eac3_access_units"]==6448 and o["run147_join"]["joc_present"] is True,"run147 metadata mismatch")
require(o["candidate_helper_rule"]["query"]=="NOT_RUN_NEGATIVE_SENTINEL","helper query status mismatch")
require(o["run147_wash_reference"]["package_id"]=="potential_ai_dee_output_metadata_reference_v1","output package join missing")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("FUN_180" not in t and "RVA" not in t,"native coordinate leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("POTENTIAL_AI_RUN147_DBMD_VECTOR=PASS")
print("POSITIVE_DBMD_DELTA=NOT_TESTED")
