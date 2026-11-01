from __future__ import annotations
import hashlib
import json
import re
from pathlib import Path

ROOT=Path(__file__).resolve().parent

def require(condition: bool,message: str)->None:
    if not condition:
        raise SystemExit(message)

manifest_raw=(ROOT/"MANIFEST.json").read_bytes()
expected=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(manifest_raw).hexdigest()==expected,"manifest hash mismatch")
manifest=json.loads(manifest_raw)
require(manifest["schema"]=="washed-first-love-dee-output-reference.v1","schema mismatch")
require(manifest["package_id"]=="first_love_dee_output_metadata_reference_v1","package id mismatch")
require(manifest["status"]=="SAMPLE_DEE_OUTPUT_METADATA_VERIFIED; STRICT_EAC3_CHECK_FAIL; RENDER_NOT_RUN","status mismatch")
require(manifest["implementation_ready"]=="NO","implementation readiness mismatch")
require(manifest["audio_algorithm_inference_allowed"] is False,"algorithm scope widened")
for entry in manifest["payloads"]:
 p=(ROOT/entry["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 data=p.read_bytes()
 require(len(data)==entry["bytes"],"payload size mismatch")
 require(hashlib.sha256(data).hexdigest()==entry["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["input"]["sha256"]=="db3ccb02260882bbd830d22c0ab207e6b8a6916743e87c939e2caa8fb4be4335","input hash mismatch")
require(o["output"]["sha256"]=="284fa11e3a5200a2bf6fd0eb3ccba5a11177f2c955cf8ab60ee7576f37339261","output hash mismatch")
require(o["output"]["parser"]["version"]=="0.20.1","OpenJOC version mismatch")
require(o["output"]["parser"]["joc_parsed_payloads"]==8048,"JOC AU count mismatch")
require(o["output"]["parser"]["deployed_compatibility"]["status"]=="PASS","compatibility result mismatch")
require(o["output"]["parser"]["etsi_strict"]["status"]=="FAIL","strict result was softened")
require(o["acceptance"]["spatial_render"]=="NOT_RUN" and o["acceptance"]["playback"]=="NOT_RUN","render/listening promoted")
require(o["native_gap"]["ddp_owner_member_0x34_runtime_join"]=="NOT_CAPTURED","native gap promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("FUN_180" not in t,"native symbol leaked")
 require(re.search(r"(?<!\w)RVA(?!\w)\s*(?:0x|[0-9])",t) is None,"instruction coordinate leaked")
 require("-----BEGIN" not in t,"key-like block leaked")
print("FIRST_LOVE_DEE_OUTPUT_REFERENCE=PASS")
print("OPENJOC_0_20_1_JOC_PARSE=PASS")
print("ETSI_STRICT=FAIL; COMPATIBILITY=PASS")
print("SPATIAL_RENDER=NOT_RUN; JOCFORGE_OUTPUT=NOT_CLAIMED")
