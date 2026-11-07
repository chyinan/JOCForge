from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-CARRIER-BLOCK-PARAM3-CALLER-STACK-PAIR-BOUNDARY-V1"
SOURCE_ID="carrier_block_param3_caller_stack_pair_boundary_wash_v1"
SOURCE_MANIFEST="a9e6754e2f334d148f23d7aa8b437eee15cb43996f0884dd9c364bd3e0bbd894"
SOURCE_SEAL="8b162b5e16262b0647e73471b349091c15007450f6b7cdb4a7d5b73875d0db8b"
STATUS="PARAM3_CALLER_STACK_PAIR_BOUND_INITIAL_ZERO_ONLY; CALLEE_READ_VALUE_UNEVALUATED; ARENA_AND_PLAN_B_OWNER_JOINS_UNBOUND; RUNTIME_NOT_RUN"
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
    require(e.get("status")==STATUS and e["caller"]["argument_3"]=="&local_2c8","caller-edge")
    require(e["stack_pair"]["first_local_initial_value"]==0 and e["stack_pair"]["second_local_initial_value"]==0,"initial-zero")
    require(e["callee_relation"]["value_at_callee_read"]=="NOT_EVALUATED" and e["callee_relation"]["param3_semantic_owner"]=="OPAQUE","value-boundary")
    require(e["owner_joins"]["typed_puVar16_arena_owner"]=="UNBOUND" and e["owner_joins"]["typed_plan_b_local68_descriptor"]=="UNBOUND","joins")
    require(len(src.get("parent_packages",[]))==3 and all(x.get("verification")=="MATCHED_ON_DISK" for x in src["parent_packages"]),"parents")
    require(src.get("raw_static_export_or_query_script_included") is False and e["scope_exclusions"]["media_pcm_payload_key_or_license_read"] is False,"scope")
    require(v["vectors"][0]["expected_disposition"]["local140_at_callee_read"]=="NOT_EVALUATED","vector")
    print("PASS package="+PID+"; stack_pair=BOUND_AT_CALL_ENTRY; value_at_read=NOT_EVALUATED; joins=UNBOUND")
if __name__=="__main__": main()
