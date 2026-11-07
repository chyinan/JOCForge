from __future__ import annotations
import hashlib, json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
PID='DEE-ADM-INDIVIDUAL-SLOT-CALLEE-NEXT-WRITE-BOUNDARY-V1'
SOURCE_ID='adm_individual_slot_callee_next_write_wash_v1'
SOURCE_MANIFEST='c36b7d09f43ee30eaf5350190cbac474c972234e096687bdd89bc99da57a9c1a'
STATUS='CALLEE_NEXT_OUTPUT_FIELD_STORE_FOUND; UID_SLOT_NOT_ESTABLISHED; SLOT_GUARD_UNBOUND; POST_NEXT_NOT_INSPECTED; RUNTIME_NOT_RUN'
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
    require("UID_SLOT_NOT_ESTABLISHED" in e.get("status"," ") and "SLOT_GUARD_UNBOUND" in e.get("status"," "),"slot-boundary")

    print("PASS package="+PID+"; evidence_only=TRUE")
if __name__=="__main__": main()
