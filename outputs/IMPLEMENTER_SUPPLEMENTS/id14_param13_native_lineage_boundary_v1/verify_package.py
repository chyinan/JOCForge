from __future__ import annotations
import hashlib, json
from pathlib import Path
def require(ok, message):
    if not ok:
        raise SystemExit("FAIL: " + message)
def sha(path):
    h=hashlib.sha256()
    with path.open("rb") as f:
        for b in iter(lambda:f.read(1048576),b""): h.update(b)
    return h.hexdigest()
def main():
    p=Path(__file__).resolve().parent
    mp=p/"MANIFEST.json"; m=json.loads(mp.read_text(encoding="utf-8"))
    e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"))
    require(m.get("package_id")=="id14_param13_native_lineage_boundary_v1","package id")
    require(m.get("implementation_delivery")=="PROHIBITED","delivery boundary")
    id14=e["native_paths"]["id14_count"]
    p13=e["native_paths"]["parameter13"]
    require(id14["input_root_fields"]==["+0x0c170","+0x0c174"] and
            id14["operation"]=="U32 minuend minus U32 subtrahend","ID14 count source")
    require(id14["wire_encoding"]=="U6(count-1)" and p13["wire_encoding"]=="additional_BSI_U8","wire fields")
    require(e["relation"]["same_child_worker_static"] is True and e["relation"]["same_parent_generation"]=="UNBOUND","lineage boundary")
    require(e["capture"]["status"]=="NOT_RUN_KEYLESS_PROFILE_NOT_DEMONSTRATED","capture status")
    entries=m.get("payloads")
    require(isinstance(entries,list) and {x.get("name") for x in entries}=={"README.md","EVIDENCE.json","INPUT-ONLY-VECTORS.json","verify_package.py"},"payload inventory")
    for x in entries:
        n=x["name"]; require(Path(n).name==n,"flat payload"); f=p/n
        require(f.is_file() and f.stat().st_size==x["bytes"] and sha(f)==x["sha256"],"payload hash")
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: ID14/parameter-13 producer lineage boundary and hashes")
if __name__=="__main__": main()
