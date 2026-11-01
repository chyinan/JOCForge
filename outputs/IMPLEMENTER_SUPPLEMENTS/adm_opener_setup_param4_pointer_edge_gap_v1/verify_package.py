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
require(m["schema"]=="washed-adm-opener-setup-pointer-edge.v1","schema mismatch")
require(m["package_id"]=="adm_opener_setup_param4_pointer_edge_gap_v1","package mismatch")
require(m["status"]=="CHNA_ROW_SLOT_TYPED; SETUP_PARAM4_POINTER_JOIN_UNBOUND","status mismatch")
require(m["implementation_ready"]=="NO","readiness mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe/missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["typed_opener_producer"]["object_row_slot"].startswith("zero-based CHNA TrackIndex"),"row slot mismatch")
require(o["typed_opener_producer"]["object_payload_key"].endswith("not CHNA TrackIndex"),"payload identity mismatch")
require(o["setup_pointer_edge"]["owner_generation_identity"]=="UNBOUND","generation join promoted")
require(o["setup_pointer_edge"]["pointer_equality"]=="NOT_EVALUATED","pointer equality promoted")
require(o["setup_pointer_edge"]["prior_runtime_breakpoints_hit"]==0,"breakpoint status mismatch")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("FUN_180" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("OPENER_SETUP_POINTER_EDGE=PASS")
print("POINTER_JOIN=UNBOUND; POINTER_EQUALITY=NOT_EVALUATED")
