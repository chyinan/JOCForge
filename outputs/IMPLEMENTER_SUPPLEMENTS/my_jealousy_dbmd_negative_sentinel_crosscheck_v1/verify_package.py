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
require(m["schema"]=="washed-my-jealousy-dbmd-negative-sentinel-crosscheck.v1","schema mismatch")
require(m["package_id"]=="my_jealousy_dbmd_negative_sentinel_crosscheck_v1","package mismatch")
require(m["status"]=="MY_JEALOUSY_NEGATIVE_FFOA_SENTINEL_BRANCH_MATCHES_CANDIDATE_HELPER; POSITIVE_DELTA_UNTESTED","status mismatch")
require(m["implementation_readiness"]=="CANDIDATE_HELPER_METADATA_RULE_ONLY; POSITIVE_BRANCH_NOT_TESTED","scope widened")
require(m["audio_algorithm_inference_allowed"] is False,"algorithm scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["scope"]["input_sha256"]=="479c430a19f205fb83c1476eb30a4f0f763a405068b43222344a1acf87f5cd8d","input hash mismatch")
require(o["scope"]["dbmd_payload_sha256"]=="f490e0d73d5d572a0915609e518db5088158612b3b6fbf478f43c9859fa9145e","DBMD hash mismatch")
require(o["tuple"]["type9"]["hours_or_sentinel"]["value"]==-1,"sentinel mismatch")
require(o["candidate_helper"]["predicted_ffoa_seconds"]==o["candidate_helper"]["observed_ffoa_seconds"]==3610.0,"helper value mismatch")
require(o["relationship"]["positive_delta_branch"]=="NOT_TESTED","positive branch promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("FUN_180" not in t and re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"native coordinates leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("MY_JEALOUSY_NEGATIVE_SENTINEL=PASS")
print("POSITIVE_DBMD_DELTA=NOT_TESTED")
