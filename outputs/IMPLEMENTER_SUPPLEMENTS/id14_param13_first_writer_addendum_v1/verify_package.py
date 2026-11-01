import hashlib,json
from pathlib import Path
def require(ok,msg):
    if not ok: raise SystemExit("FAIL: "+msg)
def sha(p):
    h=hashlib.sha256()
    with p.open("rb") as f:
        for b in iter(lambda:f.read(1048576),b""): h.update(b)
    return h.hexdigest()
def main():
    p=Path(__file__).resolve().parent;mp=p/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="id14_param13_first_writer_addendum_v1","package id")
    require(m["implementation_delivery"]=="PROHIBITED","delivery boundary")
    require(e["id14_count_path"]["wire"]=="U6(count-1)","ID14 wire")
    require(e["id14_count_path"]["root_field_first_writer"]=="UNBOUND","root owner")
    require(e["parameter13_path"]["wire"]=="additional_BSI_U8","parameter13 wire")
    require(e["same_worker_relation"]["static"] is True and e["same_worker_relation"]["same_parent_generation"]=="UNBOUND","generation boundary")
    require(e["capture"]["status"]=="NOT_RUN_KEYLESS_PROFILE_NOT_DEMONSTRATED","capture status")
    entries=m["payloads"];require({x["name"] for x in entries}=={"README.md","EVIDENCE.json","OPEN-EDGES.json","verify_package.py"},"payload set")
    for x in entries:
        f=p/x["name"];require(f.is_file() and f.stat().st_size==x["bytes"] and sha(f)==x["sha256"],"payload hash")
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: ID14/parameter13 writer paths and open joins")
if __name__=="__main__": main()
