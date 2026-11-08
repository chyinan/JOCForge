from __future__ import annotations
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent
def require(ok,msg):
    if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"))
s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","manifest schema")
require(m["package_id"]==s["package_id"]=="DEE-XML-END-TO-READER-SAMPLE-RANGE-BOUNDARY-V1","package identity")
require(m["status"]==s["status"],"status")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest hash")
require(s["payload_file_count"]==len(m["payloads"])==5,"payload count")
require(s["payload_hash_audit"]=="PASS_5_OF_5","payload audit")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload set")
for item in m["payloads"]:
    data=(root/item["file"]).read_bytes()
    require(len(data)==item["bytes"],"size mismatch "+item["file"])
    require(hashlib.sha256(data).hexdigest()==item["sha256"],"payload hash "+item["file"])
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(e["field_schema"]["xml_path"]=="/encode_to_atmos_ddp/end","field path")
require(e["field_schema"]["input_only_value"]=="47","input vector")
require(e["normalizer"]["output_unit"]=="sample index","normalizer unit")
require(e["normalizer"]["numeric_value"]=="NOT_COMPUTED","numeric claim")
require(e["reader_consumer"]["field"]=="param_2[6] / CompositeMasterItem.endSample","reader field")
require(e["reader_consumer"]["runtime_owner_generation_join"]=="UNBOUND","owner boundary")
require(e["limits"]["endpoint_to_output_crop_effect"]=="NOT_EVALUATED","crop effect")
require(e["exclusions"]["dee_runtime"]=="NOT_RUN","runtime status")
print("DEE-XML-END-TO-READER-SAMPLE-RANGE-BOUNDARY-V1_CHECK=PASS")
