from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
def require(ok,msg):
 if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","schema")
require(m["package_id"]==s["package_id"]=="PUBLIC-ENABLE-ATMOS71-BINARY-PATCHER-AU0-BOUNDARY-V1","identity")
require(m["status"]==s["status"],"status")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest")
require(s["payload_file_count"]==len(m["payloads"])==5 and s["payload_hash_audit"]=="PASS_5_OF_5","payloads")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload inventory")
for x in m["payloads"]:
 d=(root/x["file"]).read_bytes()
 require(len(d)==x["bytes"] and hashlib.sha256(d).hexdigest()==x["sha256"],"payload "+x["file"])
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(e["source_repo"]["commit"]=="459b23b6eef609edc831642a3084e4d0af7e1e32","commit")
require(e["patcher_code_facts"]["patcher_parses_or_validates_au0"] is False,"AU0 scope")
require(e["validation_coverage"]["au0_integrity_or_parser_test"]=="NOT_IN_REPOSITORY","AU0 evidence")
require(e["user_reported_au0_failure"]["status"]=="USER_REPORTED_FAILURE_FILE_ID_UNBOUND","failure identity")
require(e["implementation_readiness"] is False,"readiness")
print("PUBLIC-ENABLE-ATMOS71-BINARY-PATCHER-AU0-BOUNDARY-V1_CHECK=PASS")
