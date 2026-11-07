from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-F05-F13-FIRST-TRANSACTION-COMPARE-LINEAGE-BOUNDARY-V1"
SOURCE_ID="f05_f13_first_transaction_compare_lineage_wash_v1"
SOURCE_MANIFEST="61f33b558c3210ef6c50e15b44b55d1e7ffae8deaa2f665f6d24afa8f92ed2de"
STATUS="CURRENT_SOURCE_EXPECTED_ACTUAL_TOKEN_PATHS_BOUND_AND_EQUAL; HISTORICAL_RUNTIME_OPERANDS_AND_BINARY_JOIN_UNBOUND; GENERATION_VALIDATED_SEPARATELY"
PAYLOADS={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"}
def require(value,message):
    if not value: raise SystemExit("FAIL "+message)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def load(name):
    value=json.loads((ROOT/name).read_text(encoding="utf-8")); require(isinstance(value,dict),"json:"+name); return value
def main():
    m=load("MANIFEST.json"); s=load("package_seal.json"); e=load("EVIDENCE.json"); src=load("SOURCE-MANIFESTS.json"); v=load("INPUT-ONLY-VECTORS.json")
    require(m.get("package_id")==PID and m.get("source_package_id")==SOURCE_ID,"identity")
    require(m.get("source_manifest_sha256")==SOURCE_MANIFEST and e.get("source",{}).get("manifest_sha256")==SOURCE_MANIFEST,"source-binding")
    require(s.get("manifest_sha256")==sha(ROOT/"MANIFEST.json") and s.get("checker_status")=="PASS_NORMAL_AND_PYTHON_O","seal")
    rows=m.get("payloads",[]); require({x.get("name") for x in rows}==PAYLOADS,"inventory")
    for row in rows:
        p=ROOT/row["name"]; require(p.is_file() and p.stat().st_size==row.get("bytes") and sha(p)==row["sha256"],"payload:"+row["name"])
    require(e.get("status")==STATUS and e["first_comparison"]["path"].endswith(":274"),"compare")
    require(e["current_snapshot_result"]["divergence_before_first_compare"]=="NOT_OBSERVED","current-source")
    require(e["historical_failure"]["expected_actual_values"]=="UNBOUND" and e["historical_failure"]["failed_executable_sha256"]=="UNBOUND","historical-boundary")
    require(e["scope"]["tests_run"] is False and e["scope"]["dee_launched"] is False and e["scope"]["implementation_changed"] is False,"scope")
    require(len(src.get("working_tree_files",[]))==11 and len(src.get("approved_parent_receipts",[]))==2,"source-files")
    require(v["vectors"][0]["expected_disposition"]["historical_runtime_expected_actual"]=="UNBOUND","vector")
    print("PASS package="+PID+"; current_source_divergence=NOT_OBSERVED; historical_operands=UNBOUND")
if __name__=="__main__": main()
