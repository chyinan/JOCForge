from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
def require(ok,msg):
 if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","schema")
require(m["package_id"]==s["package_id"]=="DEE-TRIM-SURROUND-CANDIDATE-REGISTER-BOUNDARY-V1","identity")
require(m["status"]==s["status"],"status")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest")
require(s["payload_file_count"]==len(m["payloads"])==5 and s["payload_hash_audit"]=="PASS_5_OF_5","payload audit")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload set")
for x in m["payloads"]:
 d=(root/x["file"]).read_bytes()
 require(len(d)==x["bytes"] and hashlib.sha256(d).hexdigest()==x["sha256"],"payload hash")
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(e["candidate"]["function"]=="FUN_1800b5c60","candidate")
require(e["candidate"]["read"]=="U16 [RDI+0x968]","read")
require(e["candidate"]["store"]=="U16 [*(RDI+0x4330)+0x240]","store")
require(e["identity"]["incoming_rdi_to_registered_stringtype_param_1_plus_0x968"]=="UNBOUND","owner")
require(e["semantics"]["numeric_mapping"]=="UNBOUND","mapping")
require(e["execution"]["dee_run"]=="NOT_RUN","runtime")
print("DEE-TRIM-SURROUND-CANDIDATE-REGISTER-BOUNDARY-V1_CHECK=PASS")
