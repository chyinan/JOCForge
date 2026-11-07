from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-ATMOS-MASTER-OPEN-PROVIDER-INIT-BOUNDARY-V1"
SOURCE_ID="DEE-ATMOS-MASTER-OPEN-PROVIDER-INIT-BOUNDARY-WASH-V1"
SOURCE_MANIFEST="2508a2ac601c187f87ca010102ea578279c407e3ee8b861f762101027322119a"
IMAGE="3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"
N_MANIFEST="1df59a5dcfb184649a2e2346c8a44f044d521462adb56897375f01ebe2b7de10"
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
    require(e.get("image",{}).get("sha256")==IMAGE and e["adm_n_parent"]["manifest_sha256"]==N_MANIFEST,"image-and-n-parent")
    require(s.get("manifest_sha256")==sha(ROOT/"MANIFEST.json") and s.get("checker_status")=="PASS_NORMAL_AND_PYTHON_O","seal")
    rows=m.get("payloads",[]); require({x.get("name") for x in rows}==PAYLOADS,"inventory")
    for row in rows:
        p=ROOT/row["name"]; require(p.is_file() and p.stat().st_size==row.get("bytes") and sha(p)==row.get("sha256"),"payload:"+row["name"])
    require(e["provider_initialization"]["optional_filesystem_setup"]["selected_result"]=="NOT_CALLED","optional-path")
    require(any(x.get("provider")=="ADM/BWF" for x in e["provider_initialization"]["registrations"]),"provider-registration")
    require(e["join_disposition"]["host_provider_handle_to_adm_n_layout_owner"]=="UNBOUND_AT_OPAQUE_PROVIDER_REGISTRY_BOUNDARY","join")
    require(e["join_disposition"]["same_offset_used_as_identity"] is False and e["runtime"]=="NOT_RUN","boundary")
    require(e["scope_exclusions"]["raw_ghidra_data_or_decompilation_included"] is False and e["scope_exclusions"]["media_pcm_payload_key_or_license_accessed"] is False,"scope")
    require(len(src.get("parent_receipts",[]))==4 and src.get("raw_ghidra_scripts_logs_or_decompilation_copied") is False,"sources")
    require(v["vectors"][0]["expected_disposition"]["typed_owner_join"]=="UNBOUND","vector")
    print("PASS package="+PID+"; host_to_adm_n=UNBOUND; runtime=NOT_RUN")
if __name__=="__main__": main()
