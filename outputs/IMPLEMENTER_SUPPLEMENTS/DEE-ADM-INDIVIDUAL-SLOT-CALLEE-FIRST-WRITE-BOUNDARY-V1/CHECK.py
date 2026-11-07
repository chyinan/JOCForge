from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-ADM-INDIVIDUAL-SLOT-CALLEE-FIRST-WRITE-BOUNDARY-V1"
SOURCE_ID="adm_individual_slot_callee_first_write_wash_v1"
SOURCE_MANIFEST="2b1f18dda f1c880a183e27cbf91e0743fc8158d6c4972894e57572a11eedf5c5".replace(" ","")
SOURCE_SEAL="21e8b8a63258d9db63babee732582b22975141753ef8a8780c5d3bb595aa1095"
STATUS="CALLEE_FIRST_TRACKED_FIELD_STORE_FOUND; UID_SLOT_NOT_ESTABLISHED; TYPED_DESCRIPTOR_UNBOUND; POST_STORE_NOT_INSPECTED; RUNTIME_NOT_RUN"
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
    require(e.get("status")==STATUS and e["first_write"]["rva"]=="0x2809a8","first-write")
    require(e["first_write"]["proven_individual_record_plus_0x4c_write"] is False,"not-a-slot-write")
    require(e["slot_contract"]["slot_producer"]=="UNBOUND" and e["slot_contract"]["pre_assignment_guard"]=="UNBOUND","slot-boundary")
    require(e["slot_contract"]["post_store_body"]=="NOT_SEMANTICALLY_REVIEWED" and e["slot_contract"]["runtime"]=="NOT_RUN","scope")
    require(e["query_history"]["initial_query_predecoded_in_memory_instruction_count"]==17 and e["query_history"]["v2_query_behavior"].startswith("incremental"),"query-history")
    require(src["parent_package"]["evidence_manifest_sha256"]=="51baf681b94999d8f56b4e1e4178b917954c364e23c8397b5826a0dc086de181","parent")
    require(src.get("raw_logs_scripts_or_disassembly_included") is False and v["vectors"][0]["expected_disposition"]["slot_producer"]=="UNBOUND","source-vector")
    print("PASS package="+PID+"; slot_producer=UNBOUND; post_store=NOT_SEMANTICALLY_REVIEWED; runtime=NOT_RUN")
if __name__=="__main__": main()
