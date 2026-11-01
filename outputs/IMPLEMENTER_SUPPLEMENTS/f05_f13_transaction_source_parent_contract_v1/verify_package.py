import hashlib,json
from pathlib import Path
def require(ok,msg):
    if not ok: raise SystemExit("FAIL: "+msg)
def sha(p):
    h=hashlib.sha256()
    with p.open("rb") as f:
        for b in iter(lambda:f.read(1048576),b""): h.update(b)
    return h.hexdigest()
def classify(c,s):
    a=c.get("actual_producer_transaction_id")
    if a is None: return "REJECT_MISSING_PROVENANCE"
    if c.get("producer_generation")!=s or c.get("writer_generation")!=s or c.get("dispatch")!=s+1: return "REJECT_CROSS_GENERATION"
    if a!=c.get("expected_producer_transaction_id"): return "REJECT_LABEL_MISMATCH"
    return "ACCEPT"
def main():
    p=Path(__file__).resolve().parent;mp=p/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((p/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="f05_f13_transaction_source_parent_contract_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status")
    require(e["observation"]["actual_producer_transaction_id"] is None,"runtime actual must remain unbound")
    require(e["observation"]["failing_executable_sha256"] is None,"binary identity must remain unbound")
    r=e["static_projection"];require((r["callback_index"],r["source_index"],r["dispatch"])==(2,0,1),"clock")
    require(r["parent_callback_transaction_id"]!=r["expected_producer_transaction_id"],"distinct scopes")
    require(r["current_source_route_comparison"]=="MATCH","source route")
    require(e["open_edges"]["failed_run_expected_actual_values"]=="UNBOUND","open runtime edge")
    require(v["payload_or_audio_values"] is False,"input-only")
    for c in v["vectors"]: require(classify(c,v["clock_projection"]["source_index"])==c["expected_classification"],"vector "+c["name"])
    expected={"REPORT.md","EVIDENCE.json","INPUT-ONLY-VECTORS.json","SOURCE-BINDING.json","verify_package.py"}
    require({x["name"] for x in m["payloads"]}==expected,"payload set")
    for x in m["payloads"]:
        q=p/x["name"];require(q.is_file() and q.stat().st_size==x["bytes"] and sha(q)==x["sha256"],"payload "+x["name"])
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: static transaction scopes; failed-run actual remains UNBOUND")
if __name__=="__main__": main()
