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
require(m["schema"]=="washed-sample6-dbmd-negative-ffoa-sentinel.v1","schema mismatch")
require(m["package_id"]=="sample6_dbmd_negative_ffoa_sentinel_crosscheck_v1","package mismatch")
require(m["status"]=="SAMPLE_DBMD_NEGATIVE_SENTINEL_BRANCH_CONFIRMED; HISTORICAL_DEE_HELPER_IMAGE_UNBOUND","status mismatch")
require(m["implementation_ready"]=="CANDIDATE_HELPER_METADATA_RULE_ONLY","scope widened")
require(m["audio_algorithm_inference_allowed"] is False,"algorithm scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["scope"]["dbmd_payload_sha256"]=="ea961b79063bc68f64b1210dba002a5729d1003427196123f93b0890939b75d9","payload hash mismatch")
require(o["scope"]["bounded_wash_read"]["audio_data_payload_bytes"]==0,"audio data read")
require(o["decoded_time_tuple"]["fields"][0]["value"]==-1,"sentinel mismatch")
require(o["decoded_time_tuple"]["fields"][1]["value"]==0 and o["decoded_time_tuple"]["fields"][2]["value"]==0,"tuple mismatch")
require(o["candidate_helper_branch"]["absolute_source_ffoa_write"].startswith("SUPPRESSED"),"sentinel branch mismatch")
require(o["candidate_helper_branch"]["final_helper_ffoa_seconds"]==3610.0,"final FFOA mismatch")
require(o["historical_identity"]["historical_dee_host_loaded_helper_image"]=="UNBOUND","host identity promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("FUN_180" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("SAMPLE6_DBMD_NEGATIVE_SENTINEL=PASS")
print("HISTORICAL_DEE_HELPER_IMAGE=UNBOUND")
