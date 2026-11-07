from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-ADM-INDIVIDUAL-SLOT-PRODUCER-FIRST-BREAK-BOUNDARY-V1"
SOURCE_ID="adm_individual_slot_source_first_break_wash_v1"
SOURCE_MANIFEST="51baf681b94999d8f56b4e1e4178b917954c364e23c8397b5826a0dc086de181"
SOURCE_SEAL="9b07a5b59fc3fef6696a2f5fcdc2c7bfd8cfff6127be1ab0f8ed18702f79e694"
STATUS="SELECTED_U_OPEN_BRANCH_NO_EXPLICIT_0X4C_ACCESS; ONE_HELPER_NO_EXPLICIT_0X4C_ACCESS; SLOT_PRODUCER_UNBOUND; RUNTIME_NOT_RUN"
PAYLOADS={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"}
def require(value,message):
    if not value: raise SystemExit("FAIL "+message)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def load(name):
    value=json.loads((ROOT/name).read_text(encoding="utf-8")); require(isinstance(value,dict),"json:"+name); return value
def main():
    m=load("MANIFEST.json"); s=load("package_seal.json"); e=load("EVIDENCE.json"); src=load("SOURCE-MANIFESTS.json"); v=load("INPUT-ONLY-VECTORS.json")
    require(m.get("package_id")==PID and m.get("source_package_id")==SOURCE_ID,"identity")
    require(m.get("source_manifest_sha256")==SOURCE_MANIFEST and m.get("source_package_seal_sha256")==SOURCE_SEAL,"source-binding")
    require(s.get("manifest_sha256")==sha(ROOT/"MANIFEST.json") and s.get("checker_status")=="PASS_NORMAL_AND_PYTHON_O","seal")
    rows=m.get("payloads",[]); require({x.get("name") for x in rows}==PAYLOADS,"inventory")
    for row in rows:
        p=ROOT/row["name"]; require(p.is_file() and p.stat().st_size==row.get("bytes") and sha(p)==row["sha256"],"payload:"+row["name"])
    require(e.get("status")==STATUS and e["helper_edge"]["next_call"]=="FUN_1802809a0 at RVA 0x4d8bbc","edge")
    require(e["result"]["slot_plus_0x4c_producer"]=="UNBOUND_IN_SELECTED_PATH" and e["result"]["pre_assignment_guard"]=="UNBOUND","unbound")
    require(e["post_call_condition"]["demonstrated_slot_guard"] is False and e["result"]["runtime_oob_behavior"]=="NOT_INFERRED","guard")
    require(e["execution_scope"]["runtime"]=="NOT_RUN" and e["execution_scope"]["media_pcm_payload_key_or_license_read"] is False,"scope")
    require(src.get("raw_trace_script_or_log_included") is False and v["vectors"][0]["expected_disposition"]["slot_producer"]=="UNBOUND","sources-vector")
    print("PASS package="+PID+"; slot_producer=UNBOUND; runtime=NOT_RUN")
if __name__=="__main__": main()
