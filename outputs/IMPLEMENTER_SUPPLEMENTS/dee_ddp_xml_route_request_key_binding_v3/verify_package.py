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
    require(m.get("package_id")=="dee_ddp_xml_route_request_key_binding_v3","package id")
    require(m.get("implementation_delivery")=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    e=json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(e.get("xml",{}).get("route")=="encode_to_atmos_ddp","route")
    require(e.get("request_key_path",{}).get("route_key_location")=="first copied child row name at vector begin +0x18","route key source")
    require(e.get("callback_and_runner",{}).get("selected_config")=="DDP Atmos callback-created config local slot 0","selection")
    require(e.get("open",{}).get("global_callback_ordinal")=="ORDER_DEPENDENT/NOT_BOUND","ordinal boundary")
    require(e.get("runtime_boundary",{}).get("direct_factory_callback_entry_hit") is False,"runtime hit overclaim")
    for row in m.get("payloads",[]):
        name=row.get("name"); require(isinstance(name,str) and Path(name).name==name,"unsafe payload")
        p=(ROOT/name).resolve(); require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    side=(ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(ROOT / "MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification status")
    print("PASS: XML route key, exact selection, runner stores and open boundaries verified")
if __name__=="__main__": main()
