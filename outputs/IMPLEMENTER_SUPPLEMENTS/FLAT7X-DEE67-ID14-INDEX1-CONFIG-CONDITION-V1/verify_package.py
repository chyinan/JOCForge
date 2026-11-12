from __future__ import annotations
import hashlib, json
from pathlib import Path, PurePosixPath
ROOT=Path(__file__).resolve().parent
PACKAGE_ID="FLAT7X-DEE67-ID14-INDEX1-CONFIG-CONDITION-V1"
EXPECTED={"FINDINGS.md","NATIVE-CONDITION.json","SOURCE-RECEIPTS.json","verify_package.py"}
def req(value,message):
    if not value: raise SystemExit("PACKAGE_CHECK=FAIL "+message)
def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def obj(name): return json.loads((ROOT/name).read_text(encoding="utf-8"))
def main():
    raw=(ROOT/"MANIFEST.json").read_bytes(); manifest=json.loads(raw); manifest_hash=hashlib.sha256(raw).hexdigest()
    req(manifest.get("schema")=="jocforge.wash_implementer_supplement.v1" and manifest.get("package_id")==PACKAGE_ID,"manifest")
    seen=set()
    for entry in manifest.get("files",[]):
        name=entry["path"]; relative=PurePosixPath(name)
        req(name not in seen and not relative.is_absolute() and all(part not in {"",".",".."} for part in relative.parts),"path")
        path=ROOT.joinpath(*relative.parts)
        req(path.is_file() and path.stat().st_size==entry["bytes"] and digest(path)==entry["sha256"],"payload "+name)
        seen.add(name)
    actual={path.relative_to(ROOT).as_posix() for path in ROOT.rglob("*") if path.is_file() and path.name not in {"MANIFEST.json","MANIFEST.sha256","PACKAGE-SEAL.json"}}
    req(seen==EXPECTED==actual,"inventory")
    seal=obj("PACKAGE-SEAL.json")
    req(seal.get("package_id")==PACKAGE_ID and seal.get("manifest_sha256")==manifest_hash and seal.get("payload_count")==len(seen),"seal")
    req((ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()==[manifest_hash,"MANIFEST.json"],"sidecar")
    contract=obj("NATIVE-CONDITION.json"); req(contract.get("production_allowed") is False,"production")
    edges=contract.get("edge_sequence",[]); req(len(edges)==3,"edge count")
    req(edges[0]["condition"]=="param_10 == 0x15" and edges[0]["write"]=="parent+0x1832c = U32(1)","conditional write")
    req(edges[1]["operation"]=="copies parent+0x1832c to constructed state+0x0c","state initializer")
    req(edges[2]["conditional_effect"]=="value 1 stores state+0x24c = 1 after current ID14 writer","next value")
    req(edges[2]["same_generation_target_join"]=="UNBOUND","generation")
    ref=contract["reference_contract"]; req(ref["downmix_index"]==1 and ref["joc_channel_count"]==7 and ref["joc_object_count"]==15,"reference contract")
    source=obj("SOURCE-RECEIPTS.json"); req(source["source_package"]["manifest_sha256"]=="63e3610e75ad6850ebbd65498e976693e44e0018e8a1bd517d42153c6ea3538f","source bind")
    print("PACKAGE_CHECK=PASS")
    print("ID14_INDEX1_STATIC_CONDITION=BOUND")
    print("CURRENT_GENERATION=UNBOUND")
    print("PRODUCTION_ALLOWED=NO")
if __name__=="__main__": main()
