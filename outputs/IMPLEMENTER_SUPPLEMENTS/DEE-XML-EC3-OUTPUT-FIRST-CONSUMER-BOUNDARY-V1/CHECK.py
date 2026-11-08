from __future__ import annotations
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent
def require(ok,msg):
    if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"))
s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","schema")
require(m["package_id"]==s["package_id"]=="DEE-XML-EC3-OUTPUT-FIRST-CONSUMER-BOUNDARY-V1","identity")
require(m["status"]==s["status"],"status")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest hash")
require(s["payload_file_count"]==len(m["payloads"])==5 and s["payload_hash_audit"]=="PASS_5_OF_5","payload audit")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload set")
for x in m["payloads"]:
 d=(root/x["file"]).read_bytes()
 require(len(d)==x["bytes"] and hashlib.sha256(d).hexdigest()==x["sha256"],"payload mismatch "+x["file"])
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(e["scope"]["match_count"]==0,"scope.match_count mismatch")
require(e["schema_candidate"]["xml_path"]=="/output/ec3","schema_candidate.xml_path mismatch")
require(e["first_break"]["status"]=="UNBOUND","first_break.status mismatch")
require(e["execution"]["dee_run"]=="NOT_RUN","execution.dee_run mismatch")
print("DEE-XML-EC3-OUTPUT-FIRST-CONSUMER-BOUNDARY-V1_CHECK=PASS")
