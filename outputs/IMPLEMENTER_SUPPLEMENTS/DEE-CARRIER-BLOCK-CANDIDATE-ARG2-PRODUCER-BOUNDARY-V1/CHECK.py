from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-CARRIER-BLOCK-CANDIDATE-ARG2-PRODUCER-BOUNDARY-V1"
SOURCE_ID="carrier_block_candidate_argument2_producer_break_wash_v1"
SOURCE_MANIFEST="93c8993b891844e7bc7f207acfeddac9823eb14b0540be88c418e0cff70902b3"
SOURCE_SEAL="a316b41b402fe7d90834265e860bb19ad43ce64011ea8a7db1e005dc44abfc15"
STATUS="CANDIDATE_ARG2_ARENA_AFFINE_CHAIN_BOUND; NO_TYPED_JOIN_TO_PLAN_B_LOCAL68; THIRD_U32_WRITER_OWNER_AND_CARRIER_TRANSFER_UNBOUND; RUNTIME_NOT_RUN"
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
        p=ROOT/row["name"]; require(p.is_file() and p.stat().st_size==row.get("bytes") and sha(p)==row.get("sha256"),"payload:"+row["name"])
    require(e.get("status")==STATUS and e["candidate_store"]["destination"]=="qword ptr [R8+0xb18]","candidate")
    chain=e["candidate_argument2_chain"]
    require(chain["producer_argument_2"]=="puVar16+0x9a4" and chain["byte_offset"]=="0x1348" and chain["derived_candidate_argument2_offset_from_puVar16"]=="0x1690","affine-chain")
    require(e["plan_b_descriptor"]["third_u32_read"]=="dword ptr [RDI+0xb1c]" and e["plan_b_descriptor"]["typed_pointer_generation_join_from_candidate_arena"]=="UNBOUND","plan-b")
    require(e["join_disposition"]["third_u32_first_writer"]=="UNBOUND" and e["runtime_gate"]["capture"]=="NOT_RUN","boundaries")
    require(len(src.get("parent_packages",[]))==3 and all(x.get("verification")=="MATCHED_ON_DISK" for x in src["parent_packages"]),"parents")
    require(src.get("raw_static_export_or_query_script_included") is False and e["scope_exclusions"]["media_or_payload_read"] is False,"scope")
    require(v["vectors"][0]["expected_disposition"]["typed_pointer_generation_join"]=="UNBOUND","vector")
    print("PASS package="+PID+"; candidate_first_writer=NOT_ESTABLISHED; runtime=NOT_RUN")
if __name__=="__main__": main()
