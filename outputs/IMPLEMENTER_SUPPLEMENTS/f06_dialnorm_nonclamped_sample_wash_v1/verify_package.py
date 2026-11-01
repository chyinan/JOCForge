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
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"));o=json.loads((p/"OPEN-EDGES.json").read_text(encoding="utf-8"));v=json.loads((p/"MODEL-CROSSCHECK.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="f06_dialnorm_nonclamped_sample_wash_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","delivery scope")
    require(e["status"]==m["status"],"status")
    require(e["source"]["input_sha256"]=="7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1","authorized input identity")
    require(e["source"]["target_module_sha256"]=="3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2","target image identity")
    a=e["nonclamped_observation"]["attempt1"];b=e["nonclamped_observation"]["attempt2"]
    require(a["prefix_bytes_read"]==0 and "u5_observation" in a,"failed capture not promoted")
    require(b["prefix_bytes_read"]==7 and b["payload_body_bytes_read"]==0 and b["raw_prefix_retained"] is False,"bounded read")
    require(b["measured_scalar"]==-20.665958 and b["dialnorm_u5"]==21,"native observation")
    require(e["formula_comparison"]["conditional_assumption"].startswith("native measured scalar"),"conditional basis")
    require(e["formula_comparison"]["comparison"]=="MATCHES_ON_THIS_NONCLAMPED_SAMPLE_ONLY","scope limit")
    require(e["open_edges"]["general_rounding_rule"]=="UNBOUND" and e["open_edges"]["measurement_to_bsi_owner_same_parent_generation"]=="UNBOUND","open algorithm edges")
    require(o["package_id"]==m["package_id"] and o["status"]["additional_adm_vectors"]=="NOT_RUN","open-edge status")
    require(v["input_only"] is True and v["body_bytes_read"]==0 and v["raw_prefix_retained"] is False,"input-only vector boundary")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","MODEL-CROSSCHECK.json","SOURCE-BINDING.json","OPEN-EDGES.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        q=p/x["name"];require(q.is_file() and q.stat().st_size==x["bytes"] and sha(q)==x["sha256"],"payload "+x["name"])
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: one conditional nonclamped sample; general native rule and owner join remain open")
if __name__=="__main__": main()
