from __future__ import annotations
import hashlib, json
from pathlib import Path, PurePosixPath
ROOT=Path(__file__).resolve().parent
PACKAGE_ID="FLAT7X-ID14-INDEX-OBJECT-HEADER-EDGES-V1"
EXPECTED={"FINDINGS.md","ID14-HEADER-EDGES.json","SOURCE-RECEIPTS.json","verify_package.py"}
def req(value,message):
    if not value: raise SystemExit("PACKAGE_CHECK=FAIL "+message)
def digest(path):
    h=hashlib.sha256()
    with path.open("rb") as f:
        for b in iter(lambda:f.read(1024*1024),b""): h.update(b)
    return h.hexdigest()
def obj(name): return json.loads((ROOT/name).read_text(encoding="utf-8"))
def main():
    raw=(ROOT/"MANIFEST.json").read_bytes(); m=json.loads(raw)
    req(m.get("schema")=="jocforge.wash_implementer_supplement.v1","manifest schema")
    req(m.get("package_id")==PACKAGE_ID,"manifest id")
    files=m.get("files",[]); seen=set()
    for e in files:
        n=e["path"]; p=PurePosixPath(n)
        req(n not in seen and not p.is_absolute() and all(x not in {"",".",".."} for x in p.parts),"path")
        f=ROOT.joinpath(*p.parts); req(f.is_file() and f.stat().st_size==e["bytes"] and digest(f)==e["sha256"],"payload "+n); seen.add(n)
    actual={p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name not in {"MANIFEST.json","MANIFEST.sha256","PACKAGE-SEAL.json"}}
    req(seen==EXPECTED==actual,"inventory")
    seal=obj("PACKAGE-SEAL.json")
    req(seal.get("package_id")==PACKAGE_ID and seal.get("manifest_sha256")==hashlib.sha256(raw).hexdigest(),"seal")
    req((ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()==[hashlib.sha256(raw).hexdigest(),"MANIFEST.json"],"sidecar")
    c=obj("ID14-HEADER-EDGES.json"); req(c.get("production_allowed") is False,"production gate")
    ref=c["official_demo_reference"]["contract"]; req(ref["downmix_index"]==1 and ref["joc_channel_count"]==7 and ref["object_count"]==15,"official contract")
    n=c["native_edges"]["downmix_index"]; req(n["writer"].endswith("U3") and n["current_target_generation"]=="UNBOUND","index edge")
    o=c["native_edges"]["object_count"]; req(o["producer"]=="code13-code14" and o["target_encoded_u6"]==14,"object edge")
    req(c["native_edges"]["transport"]["typed_carrier_block_transfer"]=="UNBOUND","transport join")
    s=obj("SOURCE-RECEIPTS.json"); req(s["source_package"]["manifest_sha256"]=="29841b2ee5553fdb71500ac4cca2f9599587480a00e61e3c655176c81e219729","source binding")
    req(s["access"]["raw_MKA_EC3_payload_PCM_included"] is False,"payload boundary")
    print("PACKAGE_CHECK=PASS")
    print("ID14_INDEX1_U3_AND_WIDTH7=BOUND_STATIC")
    print("OBJECT_COUNT15_TO_U6_14=BOUND_STATIC")
    print("CURRENT_GENERATION_AND_PRODUCTION=UNBOUND_NO")
if __name__=="__main__": main()
