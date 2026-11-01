from __future__ import annotations
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(ok,msg):
    if not ok: raise SystemExit(msg)
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    m=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"))
    require(m.get("package_id")=="sample6_attempt05_drc_single_token_error_mapping_v1","package id")
    e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
    require(e.get("run",{}).get("exit_code")==2,"run exit")
    require(e.get("run",{}).get("binary_identity")=="UNBOUND_RUNNER_TARGET_AND_PROCESS_IMAGE","binary identity")
    require(e.get("conclusion",{}).get("classification")=="STATIC_CAUSE_CONSISTENT_WITH_RUNTIME_ERROR_NOT_PROVEN_FOR_ATTEMPT05","causal scope")
    require(e.get("static_drc_api",{}).get("source_to_attempt05_binary_join")=="UNBOUND","source join")
    source=json.loads((ROOT/"SOURCE-BINDING.json").read_text(encoding="utf-8"))
    require(source.get("incoming_manifest_sha256")=="b9a3008ecbe8af4a0542f016be6550888bcaa1b6b0a5992c342d9394e0f6fb55","source manifest")
    for row in m.get("payloads",[]):
        name=row.get("name")
        require(isinstance(name,str) and Path(name).name==name,"unsafe name")
        p=(ROOT/name).resolve()
        require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    side=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(ROOT/"MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT/"VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification")
    print("PASS: attempt05 terminal, static wrapper scope, and binary identity boundary verified")
if __name__=="__main__": main()
