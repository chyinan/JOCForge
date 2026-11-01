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
    require(m.get("package_id")=="sample6_metadata_parent_consumer_mismatch_contract_v1","package id")
    e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
    a=e.get("attempt06",{})
    require(a.get("exit_code")==2 and a.get("binary_identity")=="UNBOUND_LAUNCH_HASH_DIFFERS_FROM_CURRENT_TARGET_PATH","run identity")
    require(e.get("current_source_compare",{}).get("parent_callback_compared_at_this_predicate") is False,"predicate scope")
    require(e.get("current_source_route",{}).get("run06_executable_bound_to_this_source") is False,"source join")
    require(e.get("open",[]).count("Which of the two consumer fields differed at runtime")==1,"runtime gap")
    vectors=json.loads((ROOT/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(vectors.get("contains_output_expectations") is False,"vector scope")
    for row in m.get("payloads",[]):
        name=row.get("name")
        require(isinstance(name,str) and Path(name).name==name,"unsafe payload")
        p=(ROOT/name).resolve()
        require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    side=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(ROOT/"MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT/"VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification")
    print("PASS: attempt06 consumer failure boundary and identity gaps verified")
if __name__=="__main__": main()
