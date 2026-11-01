import hashlib,json
from pathlib import Path
def require(ok,msg):
    if not ok: raise SystemExit("FAIL: "+msg)
def sha(p):
    h=hashlib.sha256()
    with p.open("rb") as f:
        for b in iter(lambda:f.read(1048576),b""): h.update(b)
    return h.hexdigest()
def table_rows():
    rows=[]
    for code in range(32):
        ends=[i+1 for i in range(5) if code & (1 << (4-i))]+[6]
        row=[0]*6;start=0
        for end in ends:
            length=end-start;row[start]=3 if length==1 else 2 if length<=3 else 1;start=end
        rows.append(row)
    return rows
def strategies(markers,force):
    row=[0]*6;distance=0
    for block in range(5,-1,-1):
        distance+=1
        if markers[block]:
            row[block]=3 if distance==1 else 2 if distance<=3 else 1
            distance=0
        if force[block] and row[block]!=3: row[block]=3
    return row
def main():
    p=Path(__file__).resolve().parent;mp=p/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((p/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"));o=json.loads((p/"OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="f09_4_u5_no_row_cold_warm_boundary_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status binding")
    rows=table_rows();table={tuple(r) for r in rows};require(len(table)==32,"table rows")
    cold=0
    for mask in range(32):
        markers=[1]+[(mask>>(4-i))&1 for i in range(5)]
        if tuple(strategies(markers,[0]*6)) in table:cold+=1
    require(cold==32,"cold marker enumeration")
    matches=0
    for mask in range(32):
        markers=[1]+[(mask>>(4-i))&1 for i in range(5)]
        for force_mask in range(64):
            force=[(force_mask>>(5-i))&1 for i in range(6)]
            if tuple(strategies(markers,force)) in table:matches+=1
    require(matches==553,"synthetic force sweep match count")
    require(2048-matches==1495,"synthetic force sweep no-row count")
    witness=strategies([1,0,0,0,0,0],[1,0,0,0,0,0])
    require(witness==[3,0,0,0,0,0] and tuple(witness) not in table,"minimal witness")
    require(e["native_boundary"]["later_nonzero_source_writer"]=="UNBOUND","native reachability boundary")
    require(e["selector"]["caller_checks_return"] is False and e["selector"]["caller_sets_ready_flag"] is True,"caller no-row handling")
    require(v["audio_or_payload_values"] is False and v["native_force_reachability"]=="UNBOUND","input-only status")
    require(o["package_id"]==m["package_id"],"open-edge package id")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","INPUT-ONLY-VECTORS.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        q=p/x["name"];require(q.is_file() and q.stat().st_size==x["bytes"] and sha(q)==x["sha256"],"payload "+x["name"])
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: cold no-row excluded; synthetic warm force no-row possible; native reachability remains UNBOUND")
if __name__=="__main__": main()
