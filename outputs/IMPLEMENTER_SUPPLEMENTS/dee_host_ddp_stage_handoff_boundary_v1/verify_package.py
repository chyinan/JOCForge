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
require(m["schema"]=="washed-dee-host-ddp-stage-handoff-boundary.v1","schema mismatch")
require(m["package_id"]=="dee_host_ddp_stage_handoff_boundary_v1","package mismatch")
require(m["status"]=="HOST_RUNAUDIOFILTER_TO_GENERIC_STAGE_HANDOFF_BOUND; DDP_DESCRIPTOR_OFFSET0_WRITER_AND_HOST_TO_DDP_PROCESS_UNBOUND","status mismatch")
require(m["implementation_ready"]=="NO","readiness mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["host_stage_handoff"]["handoff_target"]=="generic filter-stage object","host handoff mismatch")
require(o["host_stage_handoff"]["row_descriptor_argument_observed"] is False,"descriptor promoted")
require(o["ddp_endpoint_context"]["host_call_or_transfer_into_ddp_process"]=="NOT_FOUND_IN_SCOPED_HOST_AUDIO_FILTER_PATH","host DDP edge promoted")
require(o["path_disposition"]["generic_stage_object_to_ddp_adapter"]=="UNBOUND","stage adapter edge promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("FUN_180" not in t and "FUN_140" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("DEE_HOST_STAGE_HANDOFF_BOUNDARY=PASS")
print("HOST_TO_DDP_DESCRIPTOR=UNBOUND")
