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
require(m["schema"]=="washed-potential-ai-dee-output-reference.v1","schema mismatch")
require(m["package_id"]=="potential_ai_dee_output_metadata_reference_v1","package id mismatch")
require(m["status"]=="SAMPLE_DEE_OUTPUT_METADATA_VERIFIED; STRICT_EAC3_CHECK_FAIL; RENDER_NOT_RUN","status mismatch")
require(m["implementation_ready"]=="NO","readiness mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"algorithm scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["input"]["sha256"]=="96e34c167b552928fa9ef287d358c74dac3a2d135a86a0ef4d1529c7d51e8b82","input hash mismatch")
require(o["output"]["sha256"]=="3f9a2b8b890e12e6ee9f09de28ce9eb89626da8f2a9900a562006d661cae59c2","output hash mismatch")
require(o["output"]["parser"]["version"]=="0.20.1","parser version mismatch")
require(o["output"]["parser"]["joc_parsed_payloads"]==6448,"JOC coverage mismatch")
require(o["output"]["parser"]["deployed_compatibility"]["status"]=="PASS","compatibility mismatch")
require(o["output"]["parser"]["etsi_strict"]["status"]=="FAIL","strict mismatch")
require(o["acceptance"]["spatial_render"]=="NOT_RUN" and o["acceptance"]["playback"]=="NOT_RUN","render promoted")
require(o["native_gap"]["same_owner_plus_0x34_runtime_join"]=="NOT_CAPTURED","runtime join promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("FUN_180" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("POTENTIAL_AI_DEE_OUTPUT_REFERENCE=PASS")
print("OPENJOC_0_20_1_JOC_PARSE=PASS; ETSI_STRICT=FAIL")
