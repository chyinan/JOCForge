from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-CARRIER-BLOCK-LOCAL140-AFFINE-SOURCE-BOUNDARY-V1"
SOURCE_ID="carrier_block_local140_affine_source_boundary_wash_v1"
SOURCE_MANIFEST="bbf85685eba4166a09b850ec8b689b3a5e2d922f996d881534e8ce6dae6f2a8b"
SOURCE_SEAL="7ee5fb02e6c80abd09ae714bcf31ca1446751c7d24fb67d367242e096318bebf"
IMAGE="3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"
STATUS="LOCAL140_STACK_AFFINE_SOURCE_BOUND; PARAM3_OWNER_OPAQUE; NO_TYPED_JOIN_TO_PUVAR16_OWNER_OR_PLAN_B_LOCAL68; RUNTIME_NOT_RUN"
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
    require(e.get("status")==STATUS and e["local_140"]["expression"]=="param_3[1] - 0x7586e + *param_3","expression")
    require(e["function_input"]["semantic_owner"]=="OPAQUE" and e["joins"]["local_140_to_puVar16_arena_owner"]=="UNBOUND","owner")
    require(e["joins"]["local_140_to_plan_b_local68_descriptor"]=="UNBOUND" and e["runtime"]=="NOT_RUN","join/runtime")
    require(len(src.get("parent_packages",[]))==3 and all(x.get("verification")=="MATCHED_ON_DISK" for x in src["parent_packages"]),"parents")
    require(src.get("raw_static_export_or_query_script_included") is False and e["scope_exclusions"]["media_pcm_payload_key_or_license_read"] is False,"scope")
    require(v["vectors"][0]["expected_disposition"]["param3_owner"]=="UNBOUND","vector")
    print("PASS package="+PID+"; param3_owner=OPAQUE; joins=UNBOUND; runtime=NOT_RUN")
if __name__=="__main__": main()
