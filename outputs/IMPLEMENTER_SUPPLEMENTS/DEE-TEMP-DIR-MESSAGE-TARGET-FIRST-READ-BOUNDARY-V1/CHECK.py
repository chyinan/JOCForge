from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
def require(ok,msg):
    if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"))
s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","manifest schema")
require(m["package_id"]==s["package_id"]=="DEE-TEMP-DIR-MESSAGE-TARGET-FIRST-READ-BOUNDARY-V1","package identity")
require(m["status"]==s["status"],"status mismatch")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest hash")
require(s["payload_file_count"]==len(m["payloads"])==5,"payload count")
require(s["payload_hash_audit"]=="PASS_5_OF_5","payload audit")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload set")
for item in m["payloads"]:
    data=(root/item["file"]).read_bytes()
    require(len(data)==item["bytes"],"size mismatch "+item["file"])
    require(hashlib.sha256(data).hexdigest()==item["sha256"],"hash mismatch "+item["file"])
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(e["receiver"]["first_instruction"] == "MOV EAX,dword ptr [RCX+0x48]", "receiver.first_instruction mismatch")
require(e["receiver"]["field_semantics"] == "UNBOUND", "receiver.field_semantics mismatch")
require(e["clean_temp"]["cleanup_effect"] == "UNBOUND", "clean_temp.cleanup_effect mismatch")
print("DEE-TEMP-DIR-MESSAGE-TARGET-FIRST-READ-BOUNDARY-V1_CHECK=PASS")
