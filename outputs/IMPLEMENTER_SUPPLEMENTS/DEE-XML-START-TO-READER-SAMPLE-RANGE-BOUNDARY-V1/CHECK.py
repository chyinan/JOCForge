from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
def require(ok,msg):
 if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","schema")
require(m["package_id"]==s["package_id"]=="DEE-XML-START-TO-READER-SAMPLE-RANGE-BOUNDARY-V1","identity")
require(m["status"]==s["status"],"status")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest hash")
require(s["payload_file_count"]==len(m["payloads"])==5 and s["payload_hash_audit"]=="PASS_5_OF_5","payload audit")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload set")
for x in m["payloads"]:
 b=(root/x["file"]).read_bytes()
 require(len(b)==x["bytes"] and hashlib.sha256(b).hexdigest()==x["sha256"],"payload hash")
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(e["field_schema"]["xml_path"]=="/encode_to_atmos_ddp/start","field path")
require(e["field_schema"]["input_only_value"]=="0","input vector")
require(e["normalizer"]["output_unit"]=="sample index" and e["normalizer"]["numeric_value"]=="NOT_COMPUTED","normalizer")
require(e["reader_consumer"]["field"]=="param_2[5] / CompositeMasterItem.startSample","reader field")
require(e["reader_consumer"]["runtime_owner_generation_join"]=="UNBOUND","owner boundary")
require(e["limits"]["endpoint_to_output_crop_effect"]=="NOT_EVALUATED","crop effect")
require(e["exclusions"]["dee_runtime"]=="NOT_RUN","runtime")
print("DEE-XML-START-TO-READER-SAMPLE-RANGE-BOUNDARY-V1_CHECK=PASS")
