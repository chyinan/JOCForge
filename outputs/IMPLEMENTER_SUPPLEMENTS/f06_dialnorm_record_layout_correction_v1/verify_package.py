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
    require(m["package_id"]=="f06_dialnorm_record_layout_correction_v1","package id")
    require(m["implementation_delivery"]=="PROHIBITED","delivery")
    layout=e["record_layout"]
    require(layout["per_source_header_shape"]=="THREE_POINTER_VECTOR" and layout["byte_plus_0x34_is_bsi_u16"] is False,"layout correction")
    require(e["bsi_owner_u16"]["same_generation_join"]=="UNBOUND","join status")
    require(e["observed_u5"]["value"]==31 and e["observed_u5"]["class"]=="LOWER_CLAMP_SAMPLE_ONLY","native observation")
    entries=m["payloads"];require({x["name"] for x in entries}=={"README.md","EVIDENCE.json","OPEN-EDGES.json","verify_package.py"},"payload set")
    for x in entries:
        f=p/x["name"];require(f.is_file() and f.stat().st_size==x["bytes"] and sha(f)==x["sha256"],"payload hash")
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: F06 per-source/BSI record layout correction")
if __name__=="__main__":main()
