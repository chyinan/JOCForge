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
    require(m.get("package_id")=="dee_ddp_atmos_task_route_runner_storage_v2","package id")
    require(m.get("implementation_delivery")=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","delivery scope")
    e=json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(e.get("task_route",{}).get("filter_audio")=="encode_to_atmos_ddp","route")
    require(e.get("selection",{}).get("local_factory_slot")==0,"local slot")
    require(e.get("runner_storage",{}).get("direct_xml_to_member_store")=="NOT_ISOLATED/NO","XML store boundary")
    require(e.get("run148",{}).get("direct_factory_callback_entry_hit") is False,"callback hit overclaim")
    require(e.get("remaining",{}).get("global_callback_ordinal")=="ORDER_DEPENDENT/NOT_BOUND","ordinal")
    seal=e.get("seal_anomaly",{})
    require(seal.get("duplicate_manifest_row") is True and seal.get("unique_names")==6,"seal anomaly boundary")
    for row in m.get("payloads",[]):
        name=row.get("name"); require(isinstance(name,str) and Path(name).name==name,"unsafe name")
        p=(ROOT/name).resolve(); require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash or size")
    sealfile=(ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(sealfile and sealfile[0]==digest(ROOT / "MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification state")
    print("PASS: route, runner stores, payload hashes, seal caveat and open edges verified")
if __name__=="__main__": main()
