from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
def require(ok,msg):
 if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"))
s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","schema")
require(m["package_id"]==s["package_id"]=="DEE-TIME-BASE-HOST-TEMPLATE-FIRST-BREAK-BOUNDARY-V1","identity")
require(m["status"]==s["status"],"status")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest hash")
require(s["payload_file_count"]==len(m["payloads"])==5 and s["payload_hash_audit"]=="PASS_5_OF_5","payload count/audit")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload set")
for x in m["payloads"]:
 d=(root/x["file"]).read_bytes()
 require(len(d)==x["bytes"] and hashlib.sha256(d).hexdigest()==x["sha256"],"payload "+x["file"])
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(e["host_literal_path"]["host_function"]=="FUN_1400b9d20","host edge")
require(e["host_literal_path"]["classification"]=="StageRunner/DocModel makeTemplate path","classification")
require(e["property_list_candidate"]["accepted_job_xml_value_to_property_entry"]=="UNBOUND","XML writer boundary")
require(e["ddp_field"]["field"]=="Priv+0x550","DDP field")
require(e["ddp_field"]["current_value_writer"]=="UNBOUND","DDP writer")
require(e["input_vector"]["expected_output"]=="NONE","no expected output")
require(e["boundaries"]["dee_run"]=="NOT_RUN","runtime status")
print("DEE-TIME-BASE-HOST-TEMPLATE-FIRST-BREAK-BOUNDARY-V1_CHECK=PASS")
