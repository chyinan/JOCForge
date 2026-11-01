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
    require(m.get("package_id")=="f12_measure_only_adm_scalar_source_boundary_v1","package id")
    require(m.get("implementation_delivery")=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    e=json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(e.get("sample",{}).get("axml_loudness_related_fields")==[],"AXML field absence")
    require(e.get("job_configuration",{}).get("custom_dialnorm_element_present") is False,"custom input presence")
    require(e.get("job_configuration",{}).get("metering_mode_role")=="measurement-method selector, not a loudness scalar","meter mode meaning")
    require(e.get("native_measurement",{}).get("unit")=="NOT_STATED_IN_NATIVE_LOG","unit overclaim")
    require(e.get("interpretation",{}).get("programme_to_native_source_row")=="NOT_PROVEN","source row join")
    vectors=json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(all(x.get("expected_output")=="NOT_SUPPLIED" for x in vectors.get("vectors",[])),"vector expectation")
    for row in m.get("payloads",[]):
        name=row.get("name"); require(isinstance(name,str) and Path(name).name==name,"unsafe name")
        p=(ROOT/name).resolve(); require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    side=(ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(ROOT / "MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification status")
    print("PASS: no user scalar, measure-only source and unit boundaries verified")
if __name__=="__main__": main()
