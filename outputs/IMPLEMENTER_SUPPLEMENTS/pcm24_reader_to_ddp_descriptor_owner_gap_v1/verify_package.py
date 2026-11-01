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
require(m["schema"]=="washed-pcm24-reader-to-ddp-descriptor-owner-gap.v1","schema mismatch")
require(m["package_id"]=="pcm24_reader_to_ddp_descriptor_owner_gap_v1","package mismatch")
require(m["status"]=="CONVERTER_READER_ROW_VIEW_AND_DDP_OFFSET0_CONSUMER_BOUND; CROSS_IMAGE_STORE_OWNER_GENERATION_UNBOUND","status mismatch")
require(m["implementation_ready"]=="NO","readiness mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["converter_endpoint"]["context7_adapter_callsite_argument"].startswith("caller-supplied"),"reader callsite mismatch")
require(o["ddp_endpoint"]["descriptor_offset_0_role"]=="source-row pointer table","DDP field mismatch")
require(o["cross_image_edge"]["converter_borrowed_row_view_to_ddp_descriptor_offset_0"]=="UNBOUND/NO","join promoted")
require(o["cross_image_edge"]["same_parent_generation"]=="UNBOUND/NO","generation promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("FUN_180" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("-----BEGIN" not in t,"key-like block leaked")
print("PCM24_READER_TO_DDP_ENDPOINTS=PASS")
print("CROSS_IMAGE_OWNER_GENERATION_JOIN=UNBOUND")
