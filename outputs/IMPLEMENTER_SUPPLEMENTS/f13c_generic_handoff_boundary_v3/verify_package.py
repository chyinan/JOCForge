from __future__ import annotations
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(ok,msg):
    if not ok: raise SystemExit(msg)
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    manifest=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id")=="f13c_generic_config_result_handoff_v3","package id")
    evidence=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence.get("native_image",{}).get("hash_match") is True,"image hash join")
    require(evidence.get("parser",{}).get("dmixmod_internal_slot")=="0x11","dmixmod slot")
    require(evidence.get("generic_handoff",{}).get("targeted_downmix_key_read")=="NOT_FOUND","targeted read status")
    require(evidence.get("unbound",{}).get("final_f13c_writer_or_consumer")=="UNBOUND","writer boundary")
    require(evidence.get("execution",{}).get("dynamic_observation")=="NOT_RUN","dynamic status")
    for row in manifest.get("payloads",[]):
        name=row.get("name")
        require(isinstance(name,str) and Path(name).name==name,"unsafe name")
        p=(ROOT/name).resolve()
        require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    side=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(ROOT/"MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT/"VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification state")
    print("PASS: hash-bound parser-to-generic handoff and UNBOUND consumer verified")
if __name__=="__main__": main()
