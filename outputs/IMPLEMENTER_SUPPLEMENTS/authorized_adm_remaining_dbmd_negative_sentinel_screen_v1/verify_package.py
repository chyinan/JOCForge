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
require(m["schema"]=="washed-authorized-adm-dbmd-negative-sentinel-screen.v1","schema mismatch")
require(m["package_id"]=="authorized_adm_remaining_dbmd_negative_sentinel_screen_v1","package mismatch")
require(m["status"]=="THREE_REMAINING_AUTHORIZED_DBMD_TUPLES_NEGATIVE_SENTINEL; POSITIVE_DELTA_NOT_FOUND_IN_THIS_SCREEN","status mismatch")
require(m["implementation_readiness"]=="NEGATIVE_SENTINEL_CROSS_SAMPLE_SUPPORT; POSITIVE_DELTA_BRANCH_UNTESTED","scope widened")
require(m["audio_algorithm_inference_allowed"] is False,"algorithm scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(len(o["samples"])==3,"sample count mismatch")
require(all(s["branch"]=="NEGATIVE_SENTINEL" and s["tuple"]=={"hours_or_sentinel":-1,"minutes":0,"subsecond_ticks":0} for s in o["samples"]),"tuple/branch mismatch")
require(all(s["dbmd_payload_bytes"]==576 and s["record_types"]==[7,9,10] and all(x=="PASS" for x in s["checksums"]) for s in o["samples"]),"record validation mismatch")
require(o["read_scope"]["audio_data_payload_bytes_read_during_chunk_walk"]==0,"audio data read")
require(o["disposition"]["positive_delta_branch"]=="NOT_FOUND_IN_THIS_SCREEN","positive branch promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("FUN_180" not in t and "RVA" not in t,"native coordinate leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("THREE_AUTHORIZED_DBMD_SENTINELS=PASS")
print("POSITIVE_DELTA_BRANCH=NOT_FOUND_IN_THIS_SCREEN")
