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
require(m["schema"]=="washed-adm-opener-setup-selector3-table-copy.v1","schema mismatch")
require(m["package_id"]=="adm_opener_setup_selector3_table_copy_static_closure_v1","package mismatch")
require(m["status"]=="SELECTOR3_SETUP_TABLE_CONTENT_COPY_STATIC_BOUND; POINTER_ALIAS_FALSE; RUNTIME_GENERATION_UNBOUND","status mismatch")
require(m["implementation_ready"]=="NO","readiness mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["scope"]["selector"]==3,"selector mismatch")
require(o["host_copy_and_dispatch"]["clone_allocates_new_table"] is True,"clone allocation missing")
require(o["host_copy_and_dispatch"]["records_copied"]=="all six-byte table records","record copy scope mismatch")
require(o["setup_consumer"]["source_table_pointer_relationship"]=="opener table content is copied into new setup table; not a pointer alias","pointer alias claim mismatch")
require(o["disposition"]["runtime_job_generation_token"]=="UNBOUND","generation promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("FUN_180" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("SELECTOR3_TABLE_CONTENT_COPY=PASS")
print("POINTER_ALIAS=NO; RUNTIME_GENERATION=UNBOUND")
