from __future__ import annotations
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(ok,msg):
    if not ok: raise SystemExit(msg)
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    mb=(ROOT / "MANIFEST.json").read_bytes(); m=json.loads(mb)
    require(m.get("package_id")=="f06_bsi_u16_measurement_update_observation_v1","package id")
    require(m.get("implementation_delivery")=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    e=json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    r=e.get("run",{})
    require(r.get("input_sha256")=="7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1","input identity")
    require(r.get("ascii_alias_sha256_matches_original") is True,"ASCII alias identity")
    require(r.get("context_update_flag")==0 and r.get("measurement_update_branch")=="TAKEN","active branch")
    require(r.get("serializer_time_value")==0 and r.get("storage_type")=="SIGNED_U16","native scalar observation")
    terminal=e.get("terminal",{}); require(terminal.get("dee_exit_code_receipt")=="NOT_CAPTURED" and terminal.get("ec3_output_present_at_final_check") is False,"output status")
    require(e.get("static_context",{}).get("constructor_time_first_write")=="NOT_LOCATED","constructor default overclaim")
    require(e.get("boundaries",{}).get("final_ec3_au_owner_generation")=="UNBOUND/NO","final owner join overclaim")
    vectors=json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(all(x.get("expected_output")=="NOT_SUPPLIED" for x in vectors.get("vectors",[])),"vector expected output")
    for row in m.get("payloads",[]):
        name=row.get("name"); require(isinstance(name,str) and Path(name).name==name,"unsafe payload")
        p=(ROOT/name).resolve(); require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    side=(ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(ROOT / "MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification state")
    print("PASS: one-run active branch and serializer U16 boundary verified")
if __name__=="__main__": main()
