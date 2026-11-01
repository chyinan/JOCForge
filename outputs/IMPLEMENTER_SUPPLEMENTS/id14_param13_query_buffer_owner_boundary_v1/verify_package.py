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
    require(m["package_id"]=="id14_param13_query_buffer_owner_boundary_v1","package id")
    require(m["implementation_delivery"]=="PROHIBITED","delivery boundary")
    root=e["query_buffer_root"]
    require(root["callee_param3_index1_is_plugin_scratch_root"] is True,"scratch root")
    require(root["field_c170"]["semantic_role"]=="UNBOUND" and root["field_c174"]["semantic_role"]=="UNBOUND","field semantics")
    require(root["first_writer"]=="UNBOUND" and root["bulk_zero_coverage"]=="NOT_PROVEN","root initializer boundary")
    require(e["capture"]["status"]=="NOT_RUN_KEYLESS_PROFILE_NOT_DEMONSTRATED","capture status")
    entries=m["payloads"];require({x["name"] for x in entries}=={"README.md","EVIDENCE.json","OPEN-EDGES.json","verify_package.py"},"payload set")
    for x in entries:
        f=p/x["name"];require(f.is_file() and f.stat().st_size==x["bytes"] and sha(f)==x["sha256"],"payload hash")
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: ID14 query-buffer root boundary and hashes")
if __name__=="__main__":main()
