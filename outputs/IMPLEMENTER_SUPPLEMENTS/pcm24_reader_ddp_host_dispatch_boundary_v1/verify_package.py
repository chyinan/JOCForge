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
    p=Path(__file__).resolve().parent; mp=p/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8")); e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="pcm24_reader_ddp_host_dispatch_boundary_v1","package id")
    require(m["implementation_delivery"]=="PROHIBITED","delivery boundary")
    require(e["reader_to_descriptor_offset0"]=="UNBOUND/NO" and e["same_owner_generation"]=="UNBOUND/NO","open edge")
    require(e["execution"]["runtime_capture"]=="NOT_RUN" and e["execution"]["license_contents_read"] is False,"runtime/key boundary")
    entries=m["payloads"]; require({x["name"] for x in entries}=={"README.md","EVIDENCE.json","OPEN-EDGE.json","verify_package.py"},"payload set")
    for x in entries:
        f=p/x["name"]; require(f.is_file() and f.stat().st_size==x["bytes"] and sha(f)==x["sha256"],"payload hash")
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: reader/DDP host-dispatch boundary and hashes")
if __name__=="__main__": main()
