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
    require(m.get("package_id")=="f09_4_d45_pcm_atmos_route_nonhit_attempt10_v1","package id")
    require(m.get("implementation_delivery")=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    e=json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8")); a=e.get("attempt10",{})
    require(a.get("filter_stage")=="pcm_to_atmos_ddp" and a.get("input_type")=="ADM","route/input scope")
    require(a.get("data_rate_kbps")==1024 and a.get("other_job_settings_unchanged") is True,"job preservation")
    require(a.get("exit_code")==0 and a.get("target_owner_history_functions_hit") is False,"run result")
    require(a.get("payload_or_bitstream_read") is False and a.get("playback")=="NOT_RUN","output boundary")
    require(e.get("separate_context",{}).get("later_nonzero_state")=="OPEN","reachability overclaim")
    vectors=json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(all(x.get("expected_output")=="NOT_SUPPLIED" for x in vectors.get("vectors",[])),"vector output invented")
    for row in m.get("payloads",[]):
        name=row.get("name"); require(isinstance(name,str) and Path(name).name==name,"unsafe name")
        p=(ROOT/name).resolve(); require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    side=(ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(ROOT / "MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification status")
    print("PASS: attempt10 scope, D45 non-hit, and output boundaries verified")
if __name__=="__main__": main()
