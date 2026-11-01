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
require(m["schema"]=="washed-f09-4-native-refresh-counter-scope.v1","schema mismatch")
require(m["package_id"]=="f09_4_native_refresh_counter_scope_v1","package mismatch")
require(m["status"]=="NATIVE_F09_4_U5_PARITY_NOT_CLOSED; F10_SEGMENT_COUNTER_IS_DISTINCT","status mismatch")
require(m["implementation_readiness"]=="APPROVED_MODEL_RULE_AVAILABLE; NATIVE_PARITY_NOT_ESTABLISHED","scope widened")
require(m["audio_algorithm_inference_allowed"] is False,"audio algorithm scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["native_static_observations"]["marker_segment_grouping"]["produces_u5_e1_9_frame_code"] is False,"F10 promoted to F09.4")
require(o["native_static_observations"]["actual_u5_table_e1_9_producer"]=="NOT_CLOSED","native parity promoted")
require(o["approved_model_rule"]["lane0_vector"]["table_e1_9_code"]==5,"model vector mismatch")
require(o["approved_model_rule"]["native_parity"]=="NOT_ESTABLISHED","model promoted to native")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("FUN_180" not in t and "FUN_140" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("F09_4_MODEL_RULE_REFERENCE=PASS")
print("NATIVE_U5_PARITY=NOT_CLOSED")
