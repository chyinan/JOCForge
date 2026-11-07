from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-CARRIER-BLOCK-THIRD-SCALAR-OWNER-BREAK-BOUNDARY-V1"
SOURCE_ID="carrier_block_third_scalar_candidate_owner_break_wash_v1"
SOURCE_MANIFEST="ff6dac386ac871b5a0eebc14bc9ba39505fd849f016c2972aa1e80bf277c14b7"
SOURCE_SEAL="c5b9a6b3bfb041d33f8b492c4f73f351758dd20c7f510b3437011d37fef40f08"
STATUS="CANDIDATE_QWORD_STORE_ONE_DIRECT_CALLER_ARG_SETUP_BOUND; R8_BASE_TYPED_OWNER_UNBOUND; NO_JOIN_TO_PLAN_B_LOCAL68_DESCRIPTOR; THIRD_U32_FIRST_WRITER_SEMANTIC_OWNER_AND_PROFILE_TRANSFER_UNBOUND; KEYLESS_NO_GO; RUNTIME_NOT_RUN"
PAYLOADS={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"}
def require(value, message):
    if not value: raise SystemExit("FAIL "+message)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def read_json(name):
    value=json.loads((ROOT/name).read_text(encoding="utf-8"))
    require(isinstance(value,dict),"json-object:"+name)
    return value
def main():
    manifest=read_json("MANIFEST.json"); seal=read_json("package_seal.json")
    evidence=read_json("EVIDENCE.json"); sources=read_json("SOURCE-MANIFESTS.json"); vectors=read_json("INPUT-ONLY-VECTORS.json")
    require(manifest.get("package_id")==PID and manifest.get("source_package_id")==SOURCE_ID,"identity")
    require(manifest.get("source_manifest_sha256")==SOURCE_MANIFEST and manifest.get("source_package_seal_sha256")==SOURCE_SEAL,"source-binding")
    require(seal.get("manifest_sha256")==sha(ROOT/"MANIFEST.json") and seal.get("status")=="SEALED_FOR_CHECK","seal")
    rows=manifest.get("payloads",[]); require({x.get("name") for x in rows}==PAYLOADS,"inventory")
    for row in rows:
        path=ROOT/row["name"]
        require(path.is_file() and path.stat().st_size==row.get("bytes") and sha(path)==row.get("sha256"),"payload:"+row["name"])
    require(evidence.get("status")==STATUS and evidence["candidate_store"]["operand"]=="qword ptr [R8+0xb18]","candidate-store")
    require(evidence["candidate_store"]["only_inspected_direct_caller"]["argument_2"]=="param_2+0x348","caller-argument")
    require(evidence["plan_b_descriptor"]["third_u32_operand"]=="dword ptr [RDI+0xb1c]","plan-b-read")
    require(evidence["join_disposition"]["typed_owner_join"]=="UNBOUND" and evidence["join_disposition"]["generation_join"]=="UNBOUND","joins")
    require(evidence["join_disposition"]["candidate_is_first_writer_for_plan_b_third_u32"]=="NOT_ESTABLISHED","first-writer")
    require(evidence["runtime_gate"]["keyless_capture"]=="NO_GO/BLOCKED_BEFORE_LAUNCH" and evidence["runtime_gate"]["capture"]=="NOT_RUN","runtime")
    require(evidence["scope_exclusions"]["implementation_source_included"] is False and evidence["scope_exclusions"]["decompilation_included"] is False and evidence["scope_exclusions"]["private_key_or_license_included"] is False and evidence["scope_exclusions"]["media_pcm_or_payload_included"] is False,"scope-exclusions")
    require(len(sources.get("parent_packages",[]))==5 and all(x.get("verification")=="MATCHED_ON_DISK" for x in sources["parent_packages"]),"parent-hashes")
    require(vectors["vectors"][0]["expected_disposition"]["typed_owner_join"]=="UNBOUND","vector-boundary")
    require(manifest.get("contains_native_source_or_decompilation") is False and manifest.get("contains_private_key_or_license") is False and manifest.get("contains_pcm_or_media") is False,"manifest-exclusions")
    print("PASS package="+PID+"; owner/generation=UNBOUND; runtime=NOT_RUN")
if __name__=="__main__": main()
