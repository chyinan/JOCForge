import hashlib,json
from pathlib import Path
def require(ok,msg):
    if not ok: raise SystemExit("FAIL: "+msg)
def sha(p):
    h=hashlib.sha256()
    with p.open("rb") as f:
        for b in iter(lambda:f.read(1048576),b""):h.update(b)
    return h.hexdigest()
def main():
    root=Path(__file__).resolve().parent;mp=root/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((root/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"));o=json.loads((root/"OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="sample6_attempt03_terminal_status_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status")
    require(e["source_record"]["run_id"]=="sample6-keyless-full-20261101-03" and e["source_record"]["attempt_02_reused"] is False,"run isolation")
    require(e["process"]["exe_sha256_observed_while_pid_alive"]=="a080ea176f8e418898973ac87c1e1aad47c2ed9c07435d2093f8f2598299fe6e","process image path hash")
    require(e["terminal"]["exit_code"]==2 and e["terminal"]["target_ec3_present"] is False and e["terminal"]["same_name_partial_present"] is False,"terminal status")
    require(e["terminal"]["compare_point_filter_matches"]==0 and e["terminal"]["exact_error_message"]=="UNBOUND_NOT_CAPTURED","cause boundary")
    require(all(v=="UNBOUND" for v in e["unbound_runtime_fields"].values()),"runtime transaction fields")
    require(e["isolation"]["attempt_02_failed_executable_hash"]=="UNBOUND" and e["isolation"]["attempt_03_values_not_backfilled_to_attempt_02"] is True,"old run boundary")
    require(e["exclusions"]["private_key_or_license_read"] is False and e["exclusions"]["media_pcm_or_payload_read_by_wash"] is False,"data boundary")
    require(v["audio_or_payload_values"] is False and o["package_id"]==m["package_id"],"input-only/open-edge binding")
    require({x["name"] for x in m["payloads"]}=={"REPORT.md","EVIDENCE.json","INPUT-ONLY-VECTORS.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        p=root/x["name"];require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["name"])
    seal=(root/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: attempt-03 terminal evidence; failure cause remains UNBOUND")
if __name__=="__main__":main()
