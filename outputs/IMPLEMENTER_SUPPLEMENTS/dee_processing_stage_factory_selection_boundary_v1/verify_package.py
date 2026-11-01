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
require(m["schema"]=="washed-dee-processing-stage-factory-selection.v1","schema mismatch")
require(m["package_id"]=="dee_processing_stage_factory_selection_boundary_v1","package mismatch")
require(m["status"]=="REGISTRY_CONFIG_AND_STAGE_FACTORY_SLOT_BOUND; PCM_TO_DDP_CONCRETE_OVERRIDE_UNBOUND","status mismatch")
require(m["implementation_ready"]=="NO","readiness mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["registry_to_stage_config"]["names_include_pcm_to_ddp"] is True,"config name missing")
require(o["registry_to_stage_config"]["name_match_is_dynamic_type_proof"] is False,"name promoted to type")
require(o["data_boundary"]["concrete_factory_override"]=="UNBOUND/NO","override promoted")
require(o["data_boundary"]["descriptor_offset_0_writer_found"] is False,"descriptor writer promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("FUN_180" not in t and "FUN_140" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("DEE_STAGE_REGISTRY_AND_FACTORY_BOUNDARY=PASS")
print("PCM_TO_DDP_CONCRETE_OVERRIDE=UNBOUND")
