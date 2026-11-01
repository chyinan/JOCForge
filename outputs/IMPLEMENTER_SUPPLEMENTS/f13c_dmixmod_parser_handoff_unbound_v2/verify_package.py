from __future__ import annotations
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parent

def require(ok, message):
    if not ok:
        raise SystemExit(message)

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    raw=(ROOT / "MANIFEST.json").read_bytes()
    manifest=json.loads(raw)
    require(manifest.get("package_id")=="f13c_dmixmod_parser_handoff_unbound_v2", "package id")
    require(manifest.get("implementation_delivery")=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY", "scope")
    evidence=json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence.get("native_image",{}).get("hash_match") is True, "image identity")
    parser=evidence.get("parser_fields",[])
    require(any(x.get("key")=="dmixmod" and x.get("internal_slot")=="0x11" for x in parser), "dmixmod slot")
    require(evidence.get("optional_field_behavior",{}).get("native_value_after_absence")=="UNBOUND", "native default")
    unbound=evidence.get("unbound",[])
    require("DME preferred_downmix_mode to native dmixmod conversion" in unbound, "adapter edge")
    require("Final F13C writer/consumer branch" in unbound, "writer edge")
    for row in manifest.get("payloads",[]):
        name=row.get("name")
        require(isinstance(name,str) and Path(name).name==name, "unsafe payload")
        path=(ROOT/name).resolve()
        require(path.parent==ROOT.resolve() and path.is_file(), "payload path")
        require(path.stat().st_size==row.get("bytes") and digest(path)==row.get("sha256"), "payload hash")
    seal=(ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(seal and seal[0]==digest(ROOT / "MANIFEST.json"), "manifest seal")
    verification=json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(verification.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"}, "verification")
    print("PASS: native dmixmod parser scope and open edges verified")

if __name__=="__main__":
    main()

