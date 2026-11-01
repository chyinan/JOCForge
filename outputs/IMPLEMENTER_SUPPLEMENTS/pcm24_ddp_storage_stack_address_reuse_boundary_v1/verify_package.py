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
    require(m.get("package_id")=="pcm24_ddp_storage_stack_address_reuse_boundary_v1","package id")
    e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
    require(all(x.get("host_stack_key_token_equals_ddp_record_destination_token") is True and x.get("address_relation")=="STACK_ADDRESS_REUSE_CANDIDATE_NOT_IDENTITY" for x in e.get("runs",[])),"alias identity")
    require(all(x.get("input_rehashed_during_run") is False for x in e.get("runs",[])),"input hash")
    require(e.get("host_storage_lookup",{}).get("classification")=="METADATA_LOOKUP_NOT_AUDIO_ROW_READ","lookup")
    require(e.get("ddp_internal_path",{}).get("selected_host_adm_to_ddp_owner")=="UNBOUND","owner join")
    for row in m.get("payloads",[]):
        name=row.get("name")
        require(isinstance(name,str) and Path(name).name==name,"unsafe name")
        p=(ROOT/name).resolve()
        require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    side=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(ROOT/"MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT/"VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification")
    print("PASS: metadata lookup, stack alias non-identity, and open owner edge verified")
if __name__=="__main__": main()
