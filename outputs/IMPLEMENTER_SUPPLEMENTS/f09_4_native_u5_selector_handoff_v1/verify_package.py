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
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"));o=json.loads((p/"OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="f09_4_native_u5_selector_handoff_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","delivery scope")
    require(e["status"]==m["status"],"status binding")
    require(e["target_image"]["sha256"]=="1767070831dc0d5925a1fc401684ae1447cef2d6bc59659f8a25f53c95f482e9","target identity")
    require(e["table_selector"]["native_projection_sha256"]=="acfcaf98c12fa9688ef75943537a2f2f0a97a41ab75aa342468bd649b783d6bc","approved table projection")
    require(e["table_selector"]["input_only_vector"]["code"]==5,"discriminating vector")
    require(e["strategy_generation"]["update_order"].startswith("distance_next = distance + 1 before marker test"),"distance order")
    require(e["strategy_generation"]["force_d45_override"]["first_writer_of_owner_0x135c0"]=="NOT_LOCATED","D45 source boundary")
    require(e["no_match"]["status"]=="NO_MATCH_INVARIANT_OR_ERROR_PROPAGATION_UNPROVEN","no-match boundary")
    require(e["open_edges"]["dynamic_input_vectors"]=="NOT_RUN","dynamic scope")
    require(e["exclusions"]["raw_decompilation_copied"] is False and e["exclusions"]["private_key_or_license_contents_read"] is False,"exclusions")
    require(o["package_id"]==m["package_id"],"open-edge package id")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        q=p/x["name"];require(q.is_file() and q.stat().st_size==x["bytes"] and sha(q)==x["sha256"],"payload "+x["name"])
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: native U5 selector/table/stuffer scope; state and no-row edges remain open")
if __name__=="__main__": main()
