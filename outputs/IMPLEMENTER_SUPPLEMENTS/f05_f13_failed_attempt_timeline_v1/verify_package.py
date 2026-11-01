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
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((p/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="f05_f13_failed_attempt_timeline_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status binding")
    require(e["source_record"]["output_truncated"] is False,"source transcript scope")
    require(e["input"]["sha256"]=="7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1","input identity")
    require(e["timeline"][1]["callback_transaction_id"]=="jocforge-main-callback-g0","g0 transaction")
    require(e["timeline"][2]["callback_transaction_id"]=="jocforge-main-callback-g1","g1 transaction")
    require(e["timeline"][3]["error"]=="Joc(ProducerTransactionMismatch)" and e["timeline"][3]["inner_process_exit_code"]==2,"failure event")
    require(e["missing_fields"]["expected_producer_transaction_id"]=="UNBOUND" and e["missing_fields"]["actual_producer_transaction_id"]=="UNBOUND","transaction actual remains unbound")
    require(e["missing_fields"]["gtc_owner_id"]=="UNBOUND" and e["missing_fields"]["joc_owner_generation"]=="UNBOUND","owner fields remain unbound")
    require(e["current_rebuild_boundary"]["classification"]=="CURRENT_REBUILT_FILE_ONLY_NOT_RUN_02_IDENTITY","build boundary")
    require(e["exclusions"]["private_key_or_license_read"] is False and e["exclusions"]["pcm_or_media_read_by_wash"] is False,"data boundary")
    require(v["audio_or_payload_values"] is False,"synthetic vector boundary")
    require(len(v["cases"])==2,"vector count")
    require({x["name"] for x in m["payloads"]}=={"REPORT.md","EVIDENCE.json","INPUT-ONLY-VECTORS.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        q=p/x["name"];require(q.is_file() and q.stat().st_size==x["bytes"] and sha(q)==x["sha256"],"payload "+x["name"])
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: g0/g1 timeline captured; failed transaction values and build identity remain UNBOUND")
if __name__=="__main__": main()
