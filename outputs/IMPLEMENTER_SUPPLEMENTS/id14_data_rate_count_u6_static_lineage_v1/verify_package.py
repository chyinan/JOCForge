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
    p=Path(__file__).resolve().parent;mp=p/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"));o=json.loads((p/"OPEN-EDGES.json").read_text(encoding="utf-8"));v=json.loads((p/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="id14_data_rate_count_u6_static_lineage_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status binding")
    rates={x[0]:x[1] for x in e["plugin_selector"]["rate_map"]}
    require(rates["1024"]==5 and rates["384"]==0,"data_rate map")
    require(e["plugin_selector"]["selected_enum_to_query_slot"].endswith("accepts enum 0..5"),"accepted enum range")
    require(e["normal_id14_path"]["code13"]=="base + marker_count" and e["normal_id14_path"]["code14"]=="marker_count","normal property formula")
    require(e["normal_id14_path"]["count_difference"]=="code13 - code14 = base","count difference")
    require(e["normal_id14_path"]["id14_u6_writer"].endswith("serializes count-1 as U6"),"U6 writer")
    require(e["plugin_selector"]["host_task_value_to_selected_priv_instance"]=="UNBOUND","host value join")
    require(e["frozen_model_boundary"]["relationship_to_native_param3_and_priv"]=="UNBOUND; no same-file/job token","model join")
    require(e["exclusions"]["private_key_or_license_contents_read"] is False,"secret boundary")
    require(v["input_only"] is True and all(row["classification"]=="STATIC_MODEL_VECTOR" for row in v["vectors"]),"vector classification")
    require(o["status"]["host_run"]=="UNBOUND","runtime boundary")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","INPUT-ONLY-VECTORS.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        q=p/x["name"];require(q.is_file() and q.stat().st_size==x["bytes"] and sha(q)==x["sha256"],"payload "+x["name"])
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: static data_rate to native count/U6 path; host run and model EMDF joins remain open")
if __name__=="__main__":main()
