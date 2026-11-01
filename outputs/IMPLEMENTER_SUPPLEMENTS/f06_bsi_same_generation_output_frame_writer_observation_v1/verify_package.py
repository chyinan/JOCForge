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
    require(m.get("package_id")=="f06_bsi_same_generation_output_frame_writer_observation_v1","package id")
    e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
    run=e.get("run",{})
    require(run.get("dee_exit_code")==0 and run.get("whole_frame_count")==4247,"run/output size")
    require(e.get("same_generation_writer",{}).get("append_size_is_one_complete_frame") is True,"frame append")
    header=e.get("header_only",{})
    require(header.get("bytes_examined")==7 and header.get("dialnorm_u5")==17,"header observation")
    require(e.get("semantic_limits",{}).get("bsi_u16_to_standard_ec3_dialnorm_numeric_relation")=="NOT_BOUND","numeric mapping")
    require(e.get("owner_lifetime",{}).get("constructor_time_first_write")=="NOT_LOCATED","owner default")
    require(e.get("exclusions",{}).get("ec3_body_bytes_read")==0,"body read")
    vectors=json.loads((ROOT/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(all(v.get("expected_output")=="NOT_SUPPLIED" for v in vectors.get("vectors",[])),"vector scope")
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
    print("PASS: one-frame DEE reference, writer-generation and U16 boundary verified")
if __name__=="__main__": main()
