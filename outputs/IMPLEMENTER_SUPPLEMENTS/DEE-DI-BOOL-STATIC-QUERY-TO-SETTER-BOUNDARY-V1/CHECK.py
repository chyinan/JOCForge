from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
def require(ok,msg):
 if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","schema")
require(m["package_id"]==s["package_id"]=="DEE-DI-BOOL-STATIC-QUERY-TO-SETTER-BOUNDARY-V1","identity")
require(m["status"]==s["status"],"status")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest hash")
require(s["payload_file_count"]==len(m["payloads"])==5 and s["payload_hash_audit"]=="PASS_5_OF_5","payload audit")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload inventory")
for x in m["payloads"]:
 b=(root/x["file"]).read_bytes()
 require(len(b)==x["bytes"] and hashlib.sha256(b).hexdigest()==x["sha256"],"payload "+x["file"])
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(e["static_registration"]["registered_member"]=="param_1+0x270 = Priv+0x290","Bool registration")
require(e["conditional_query"]["key_match"]=="exact byte-key match","query semantics")
require(e["conditional_query"]["actual_xml_key_to_query"]=="UNBOUND","XML query boundary")
require(e["typed_setter"]["target"]=="FUN_18016a540","setter target")
require(e["f06_measurement"]["bool_to_f06_same_owner_generation_join"]=="UNBOUND","F06 join")
require(e["limits"]["audio_or_measurement_effect"]=="NOT_RUN","effect status")
print("DEE-DI-BOOL-STATIC-QUERY-TO-SETTER-BOUNDARY-V1_CHECK=PASS")
