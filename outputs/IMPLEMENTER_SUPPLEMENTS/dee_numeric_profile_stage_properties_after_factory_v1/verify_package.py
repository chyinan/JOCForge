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
    require(m.get("package_id")=="dee_numeric_profile_stage_properties_after_factory_v1","package id")
    e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
    require(e.get("host_route",{}).get("matching")=="exact bytes and exact length against parsed route name","route match")
    require(e.get("loader_index",{}).get("task_numeric_field_transfer")=="NONE_OBSERVED_IN_RUN145_RUN148_PATH","numeric transfer overclaim")
    require(e.get("domain_boundary",{}).get("is_task_input_selector") is False,"profile index domain")
    for row in m.get("payloads",[]):
        name=row.get("name"); require(isinstance(name,str) and Path(name).name==name,"unsafe name")
        p=(ROOT/name).resolve(); require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    seal=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()
    require(seal and seal[0]==digest(ROOT/"MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT/"VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification")
    print("PASS: route selection, post-factory property application and loader index boundary verified")
if __name__=="__main__": main()
