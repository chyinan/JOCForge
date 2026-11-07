from __future__ import annotations
import hashlib, json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
PID='DEE-ATMOS-WAV-PRIMARY-CALLBACK-ENTRY-BOUNDARY-V1'
SOURCE_ID='DEE-ATMOS-WAV-PRIMARY-CALLBACK-ENTRY-BOUNDARY-WASH-V1'
SOURCE_MANIFEST='96e1a3d1c8ebba38f2c1d3dffe4f008eb96dbdce04fa7c0baafc7e41e990ba7c'
STATUS='PRIMARY_WAV_CALLBACK_ENTRY_BOUND; ARGUMENT_TYPES_AND_PHYSICAL_N_UNBOUND; RUNTIME_NOT_CAPTURED'
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

    print("PASS package="+PID+"; evidence_only=TRUE")
if __name__=="__main__": main()
