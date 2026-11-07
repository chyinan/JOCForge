from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-DI-FLAG-BOOL-164-CHILD-OWNER-BOUNDARY-V1"
SOURCE_ID="di-flag-bool-empire-loudness-subcontext-164-candidate-owner-field-producer-boundary-addendum-wash-v1"
SOURCE_MANIFEST="1491cf69d68d90801b471ce0a72eab073d9106ad2804bc661ec6ea552d9b642b"
SOURCE_SEAL="b284915082fa11be87756025091a90a5d651e5708502303d05cd2976f676612a"
STATUS="CODEC_CHILD_OWNER_BOUND; PLUS_0XDF0_POINTER_TARGET_OPAQUE; DLL_SUBCONTEXT_JOIN_UNBOUND"
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
    require(e.get("status")==STATUS and e["owner_path"]["candidate_pointer_load"]=="*(undefined8 *)(lVar6+0xdf0)","edge")
    require(e["owner_path"]["field_writer_or_constructor_in_caller"]=="NOT_OBSERVED" and e["owner_path"]["pointer_target"]=="OPAQUE","first-break")
    require(e["typed_owner_generation_join"]=="UNBOUND" and e["bool_to_result"]=="UNBOUND" and e["runtime"]=="NOT_RUN","boundary")
    require(e["global_absence_claim"] is False and src.get("raw_trace_script_or_log_included") is False,"scope")
    require(len(src.get("parent_receipts",[]))==3 and v["vectors"][0]["expected_disposition"]["typed_owner_generation_join"]=="UNBOUND","parents-vector")
    require(m.get("contains_native_source_or_decompilation") is False and m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False,"exclusions")
    print("PASS package="+PID+"; pointer_target=OPAQUE; join=UNBOUND; runtime=NOT_RUN")
if __name__=="__main__": main()
