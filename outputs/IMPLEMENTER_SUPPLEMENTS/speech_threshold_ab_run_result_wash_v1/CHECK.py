from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
def require(ok,msg):
 if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","schema")
require(m["package_id"]==s["package_id"]=="speech_threshold_ab_run_result_wash_v1","identity")
require(m["status"]==s["status"],"status")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest hash")
require(s["payload_file_count"]==len(m["payloads"])==5 and s["payload_hash_audit"]=="PASS_5_OF_5","payload audit")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload set")
for x in m["payloads"]:
 b=(root/x["file"]).read_bytes()
 require(len(b)==x["bytes"] and hashlib.sha256(b).hexdigest()==x["sha256"],"payload "+x["file"])
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(len(e["arm_results"])==2 and [x["threshold"] for x in e["arm_results"]]==[0,100],"arm pair")
require(all(x["dee_exit_code"]==10 and x["event_count"]==0 and not x["output_file_present"] for x in e["arm_results"]),"run outcomes")
require(e["observer_boundary"]["raw_module_base_status_field_present"] is True,"pointer issue disposition")
require(e["observer_boundary"]["raw_pointer_value_in_this_washed_package"] is False,"raw pointer exclusion")
require(e["disposition"]["measurement_effect"]=="UNBOUND_NOT_RUN","effect status")
require(e["disposition"]["cause_of_exit_10"]=="UNBOUND","cause status")
require(e["exclusions"]["raw_run_receipts_included"] is False,"receipt exclusion")
print("speech_threshold_ab_run_result_wash_v1_CHECK=PASS")
