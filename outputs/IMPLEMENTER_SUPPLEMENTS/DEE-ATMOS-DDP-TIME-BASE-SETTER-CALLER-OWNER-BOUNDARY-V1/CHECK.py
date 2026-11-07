from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-ATMOS-DDP-TIME-BASE-SETTER-CALLER-OWNER-BOUNDARY-V1"
SOURCE_ID="DEE-ATMOS-DDP-TIME-BASE-SETTER-CALLER-OWNER-BOUNDARY-WASH-V1"
SOURCE_MANIFEST="9866ae6a18bccac39e5f63b201a3b13f3cc57394a4b37463c892f6cbc74996fe"
SOURCE_SUMS="a1efcf954fec2e0bcf34be53a45e0cb278e00ebfecca2bac3f6cae4713b8535b"
PAYLOADS={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"}
def require(value,message):
    if not value: raise SystemExit("FAIL "+message)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def load(name):
    value=json.loads((ROOT/name).read_text(encoding="utf-8")); require(isinstance(value,dict),"json:"+name); return value
def main():
    m=load("MANIFEST.json"); s=load("package_seal.json"); e=load("EVIDENCE.json"); src=load("SOURCE-MANIFESTS.json"); v=load("INPUT-ONLY-VECTORS.json")
    require(m.get("package_id")==PID and m.get("source_package_id")==SOURCE_ID,"identity")
    require(m.get("source_manifest_sha256")==SOURCE_MANIFEST and m.get("source_sha256sums_sha256")==SOURCE_SUMS,"source-hashes")
    require(s.get("manifest_sha256")==sha(ROOT/"MANIFEST.json") and s.get("checker_status")=="PASS_NORMAL_AND_PYTHON_O","seal")
    rows=m.get("payloads",[]); require({x.get("name") for x in rows}==PAYLOADS,"inventory")
    for row in rows:
        p=ROOT/row["name"]; require(p.is_file() and p.stat().st_size==row.get("bytes") and sha(p)==row.get("sha256"),"payload:"+row["name"])
    require(e.get("status")=="CALLER_OWNER_NOT_TYPED_JOINED_TO_PLUS_0X550","status")
    require(e["helper_argument"]["callee"]=="FUN_180146180(&local_1e0)" and e["helper_argument"]["typed_option_descriptor_join"]=="UNBOUND","edge")
    require(e["time_base"]["caller_owner_to_nested_descriptor_plus_0x550"]=="UNBOUND" and e["runtime"]=="NOT_RUN","boundary")
    require(e["scope_exclusions"]["dee_job_run"] is False and e["scope_exclusions"]["media_pcm_payload_key_or_license_read"] is False and e["scope_exclusions"]["implementation_contact"] is False,"scope")
    require(len(src.get("parent_manifests",[]))==3 and all(x.get("verification")=="MATCHED_ON_DISK" for x in src["parent_manifests"]),"parents")
    require(src.get("raw_ghidra_script_or_log_included") is False,"raw-sources")
    require(v["vectors"][0]["expected_disposition"]["typed_join_in_bounded_slice"]=="UNBOUND","vector")
    print("PASS package="+PID+"; typed_join=UNBOUND; runtime=NOT_RUN")
if __name__=="__main__": main()
