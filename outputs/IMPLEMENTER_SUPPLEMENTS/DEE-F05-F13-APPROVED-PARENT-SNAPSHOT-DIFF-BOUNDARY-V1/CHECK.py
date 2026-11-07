from __future__ import annotations
import hashlib, json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
PID='DEE-F05-F13-APPROVED-PARENT-SNAPSHOT-DIFF-BOUNDARY-V1'
SOURCE_ID='f05_f13_transaction_compare_parent_snapshot_diff_wash_v1'
SOURCE_MANIFEST='5e88d3742c8c9d127ca958f83931df08ea056ea9b808591d2d56fa26789a7acf'
STATUS='SIX_COMPARE_PATH_FILES_IDENTICAL_TO_APPROVED_PARENT; CURRENT_STATIC_TOKEN_FLOW_MATCHES; HISTORICAL_OPERANDS_UNBOUND'
PAYLOADS={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"}
def require(value,message):
    if not value: raise SystemExit("FAIL "+message)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def load(name):
    value=json.loads((ROOT/name).read_text(encoding="utf-8")); require(isinstance(value,dict),"json:"+name); return value
def main():
    m=load("MANIFEST.json"); s=load("package_seal.json"); e=load("EVIDENCE.json"); src=load("SOURCE-MANIFESTS.json"); v=load("INPUT-ONLY-VECTORS.json")
    require(m.get("package_id")==PID and m.get("source_package_id")==SOURCE_ID,"identity")
    require(m.get("source_manifest_sha256")==SOURCE_MANIFEST and e.get("status")==STATUS,"source-status")
    require(s.get("manifest_sha256")==sha(ROOT/"MANIFEST.json") and s.get("checker_status")=="PASS_NORMAL_AND_PYTHON_O","seal")
    rows=m.get("payloads",[]); require({x.get("name") for x in rows}==PAYLOADS,"inventory")
    for row in rows:
        p=ROOT/row["name"]; require(p.is_file() and p.stat().st_size==row.get("bytes") and sha(p)==row["sha256"],"payload:"+row["name"])
    require(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False,"exclusions")
    require(e.get("runtime")=="NOT_RUN","runtime")
    require(e["facts"]["historical_failure"].find("UNBOUND")>=0,"historical-boundary")

    print("PASS package="+PID+"; evidence_only=TRUE")
if __name__=="__main__": main()
