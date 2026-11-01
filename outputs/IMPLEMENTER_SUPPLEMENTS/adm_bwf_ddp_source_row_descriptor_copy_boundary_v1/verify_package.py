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
require(m["schema"]=="washed-adm-bwf-ddp-source-row-descriptor-boundary.v1","schema mismatch")
require(m["package_id"]=="adm_bwf_ddp_source_row_descriptor_copy_boundary_v1","package id mismatch")
require(m["status"]=="DDP_DESCRIPTOR_FORWARDING_AND_ROW_COPY_STATICALLY_BOUND; PCM24_READER_TO_DDP_TABLE_OWNER_GENERATION_UNBOUND","status mismatch")
require(m["implementation_ready"]=="NO","readiness mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["ddp_descriptor"]["offset_0"]=="pointer to source-row pointer table","descriptor field mismatch")
require(o["ddp_descriptor"]["offset_16"]=="frame count; checked against allocated input capacity","frame count field mismatch")
require(o["row_copy"]["source_to_internal_copy_static"]=="BOUND","copy status mismatch")
require(o["open_edge"]["same_owner_and_generation"]=="UNBOUND/NO","owner edge promoted")
require(o["open_edge"]["parent_run_reader_branch_status"]=="NOT_HIT","reader branch promoted")
require(o["input_only_vector"]["runtime_observation"]=="NOT_RUN","vector promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("FUN_180" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("DDP_DESCRIPTOR_FORWARDING_AND_ROW_COPY=PASS")
print("PCM24_READER_OWNER_GENERATION_JOIN=UNBOUND")
