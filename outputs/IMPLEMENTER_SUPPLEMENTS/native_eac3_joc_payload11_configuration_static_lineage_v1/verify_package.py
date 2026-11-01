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
require(m["schema"]=="washed-native-eac3-joc-payload11-configuration-lineage.v1","schema mismatch")
require(m["package_id"]=="native_eac3_joc_payload11_configuration_static_lineage_v1","package mismatch")
require(m["status"]=="TYPE11_STATIC_PRESENCE_RULES_BOUND; SAME_RUN_IMAGE_AND_GROUP_ID_PRESENCE_JOIN_UNBOUND","status mismatch")
require(m["implementation_ready"]=="NO","readiness mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["static_write_rules"]["codecdatae"]["presence_u1_written"] is True,"codecdatae write mismatch")
require(o["static_write_rules"]["payload_frame_aligned"]["presence_u1_written"] is True,"alignment write mismatch")
require(o["static_write_rules"]["create_duplicate"]["serialized"] is False,"duplicate flag omission mismatch")
require(o["static_write_rules"]["group_id"]["presence_value"]==0 and o["static_write_rules"]["group_id"]["value_serialized"] is False,"group id presence mismatch")
require(o["remaining_limits"]["payload11_static_module_to_first_love_or_potential_ai_loaded_ddp_module"]=="UNBOUND","same-run join promoted")
require(o["remaining_limits"]["native_group_id_present_zero_vs_openjoc_group_id_value_zero"]=="UNBOUND","group id join promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("FUN_180" not in t and "RVA" not in t,"native coordinate leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("NATIVE_TYPE11_CONFIGURATION_RULE=PASS")
print("SAME_RUN_IMAGE_AND_GROUP_PRESENCE=UNBOUND")
