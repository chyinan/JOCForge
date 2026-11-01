from __future__ import annotations
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(condition:bool,message:str)->None:
    if not condition: raise SystemExit(message)
raw=(ROOT/"MANIFEST.json").read_bytes();expected=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(raw).hexdigest()==expected,"manifest hash mismatch")
m=json.loads(raw)
require(m["schema"]=="washed-authorized-adm-seven-input-dbmd-branch-matrix.v1","schema mismatch")
require(m["package_id"]=="authorized_adm_seven_input_dbmd_branch_matrix_v1","package id mismatch")
require(m["status"]=="ALL_SEVEN_AUTHORIZED_DBMD_TUPLES_NEGATIVE_SENTINEL; POSITIVE_DELTA_NOT_EXERCISED","status mismatch")
require(m["implementation_readiness"]=="NEGATIVE_SENTINEL_RULE_SUPPORTED_FOR_THE_SEVEN_INPUTS; POSITIVE_BRANCH_UNTESTED","scope mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve();require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe/missing payload")
 d=p.read_bytes();require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["sample_count"]==7 and len(o["samples"])==7,"sample count mismatch")
require(o["negative_sentinel_count"]==7 and o["positive_delta_count"]==0,"branch counts mismatch")
require(len({s["input_sha256"] for s in o["samples"]})==7,"input hashes are not unique")
require(all(s["tuple"]=={"hours_or_sentinel":-1,"minutes":0,"subsecond_ticks":0} and s["branch"]=="NEGATIVE_SENTINEL" for s in o["samples"]),"tuple or branch mismatch")
require(o["direct_helper_observation_count"]==3,"direct helper coverage mismatch")
require(o["summary_constraints"]["raw_dbmd_bytes_in_summary"] is False and o["summary_constraints"]["audio_pcm_decoded"] is False,"data boundary widened")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("FUN_180" not in t and "RVA" not in t,"native coordinate leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("AUTHORIZED_SEVEN_DBMD_BRANCH_MATRIX=PASS")
print("POSITIVE_DBMD_DELTA=NOT_EXERCISED")
