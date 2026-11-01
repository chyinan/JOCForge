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
    require(m.get("package_id")=="f09_4_d45_pcm_route_attempt10_11_boundary_v1","package id")
    e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
    a=e.get("attempts",[])
    require(len(a)==2 and a[0]["n"]==10 and a[1]["n"]==11,"attempts")
    require(a[0]["breakpoints_armed"] is True and a[0]["target_hits"]==0,"attempt10")
    require(a[1]["target_breakpoints_armed"] is False and a[1]["target_status"]=="NOT_INSTRUMENTED","attempt11")
    require(e.get("static_pcm_gate",{}).get("accepted_frame_samples")==1536,"PCM guard")
    require(e.get("d45_state",{}).get("later_nonzero","").startswith("OPEN"),"later state")
    require(e.get("terminal",{}).get("bitstream_parse")=="NOT_RUN","bitstream")
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
    print("PASS: attempts10/11 and route-specific D45 boundary verified")
if __name__=="__main__": main()
