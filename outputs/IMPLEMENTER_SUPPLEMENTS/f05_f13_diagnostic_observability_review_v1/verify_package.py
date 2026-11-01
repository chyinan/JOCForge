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
    root=Path(__file__).resolve().parent;mp=root/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((root/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"));o=json.loads((root/"OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="f05_f13_diagnostic_observability_review_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status")
    require(e["rust_error"]["kind"]=="unit_variant_no_fields" and e["rust_error"]["error_values_serialized"] is False,"error observability")
    require(e["open_edges"]["attempt03_compare_values"]=="UNBOUND","runtime boundary")
    require(e["prior_runner_review"]["current_runner_claim"]=="NOT_EVALUATED_BY_THIS_PACKAGE","runner scope")
    require(v["audio_or_payload_values"] is False and len(v["vectors"])==5,"synthetic vector scope")
    require(o["package_id"]==m["package_id"],"open edges id")
    require({x["name"] for x in m["payloads"]}=={"REPORT.md","EVIDENCE.json","INPUT-ONLY-VECTORS.json","SOURCE-BINDING.json","OPEN-EDGES.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        p=root/x["name"];require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["name"])
    seal=(root/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: diagnostic contract review; current runner and runtime values remain separate")
if __name__=="__main__":main()
