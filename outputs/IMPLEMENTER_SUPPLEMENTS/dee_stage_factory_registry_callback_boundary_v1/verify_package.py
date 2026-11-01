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
require(m["schema"]=="washed-dee-stage-factory-registry-callback-boundary.v1","schema mismatch")
require(m["package_id"]=="dee_stage_factory_registry_callback_boundary_v1","package mismatch")
require(m["status"]=="REGISTRY_CALLBACK_CALLSITE_BOUND; PCM_TO_DDP_ENTRY_AND_CONCRETE_FACTORY_OVERRIDE_UNBOUND","status mismatch")
require(m["implementation_ready"]=="NO","readiness mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["category_registration"]["key"]=="pcm_to_ddp","category key mismatch")
require(o["category_registration"]["proves_stage_registry_callback_identity"] is False,"name/category promoted")
require(o["registry"]["callback_target_selection"]=="UNBOUND","callback target promoted")
require(o["stage_factory"]["concrete_override_or_module"]=="UNBOUND/NO","concrete factory promoted")
require(o["ddp_edge"]["descriptor_offset_0_writer"]=="UNBOUND/NO","DDP writer promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("FUN_180" not in t and "FUN_140" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("STAGE_FACTORY_REGISTRY_CALLBACK_BOUNDARY=PASS")
print("PCM_TO_DDP_CALLBACK_AND_CONCRETE_FACTORY=UNBOUND")
