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
require(m["schema"]=="washed-historical-helper-image-identity-boundary.v1","schema mismatch")
require(m["package_id"]=="historical_helper_image_identity_boundary_v1","package mismatch")
require(m["status"]=="R138_HOST_IMAGE_MATCHED; HOST_HELPER_COMMAND_EDGE_BOUND; HISTORICAL_CHILD_HELPER_IMAGE_UNBOUND","status mismatch")
require(m["implementation_ready"]=="NO","readiness mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["scope"]["r138_host_sha256"]==o["scope"]["current_host_sha256"],"host image match lost")
require(o["static_host_command_edge"]["static_host_to_helper_command_edge"].startswith("BOUND"),"static edge missing")
require(o["static_host_command_edge"]["resolved_runtime_path_prefix"]=="DYNAMIC_AND_NOT_BOUND_TO_R138","runtime path overclaimed")
require(o["historical_r138_child_process"]["historical_child_helper_image_sha256"]=="UNBOUND","historical child identity promoted")
require(o["nonexecution"]["new_dee_job"]=="NOT_RUN","new job status mismatch")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("FUN_180" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("HOST_HELPER_STATIC_COMMAND_EDGE=PASS")
print("HISTORICAL_CHILD_HELPER_IMAGE=UNBOUND")
