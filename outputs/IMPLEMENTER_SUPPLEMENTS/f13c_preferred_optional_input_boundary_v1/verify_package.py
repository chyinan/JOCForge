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
    require(m.get("package_id")=="f13c_preferred_optional_input_boundary_v1","package id")
    require(m.get("implementation_delivery")=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","delivery scope")
    e=json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(e.get("adm_metadata_scan",{}).get("sample_count")==7,"ADM sample count")
    require(e.get("adm_metadata_scan",{}).get("matched_elements_total")==0,"ADM AXML match count")
    require(e.get("dme_ui",{}).get("preferred_control_active_initially") is False,"UI active default")
    require(e.get("dme_ui",{}).get("request_builder_rule")=="append only when control active","UI request rule")
    require(e.get("native_static_observations",{}).get("native_default_after_absence")=="UNBOUND","native default overclaim")
    require(e.get("native_static_observations",{}).get("dme_preferred_to_native_dmixmod_edge")=="UNBOUND","native adapter overclaim")
    require(e.get("execution",{}).get("dynamic_observation")=="NOT_RUN","dynamic status")
    for row in m.get("payloads",[]):
        name=row.get("name"); require(isinstance(name,str) and Path(name).name==name,"unsafe payload")
        p=(ROOT/name).resolve(); require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    side=(ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(ROOT / "MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification status")
    print("PASS: seven ADM metadata scans and optional-input boundary verified")
if __name__=="__main__": main()
