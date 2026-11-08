from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def need(ok,msg):
 if not ok:raise SystemExit("FAIL: "+msg)
m=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8"))
e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((ROOT/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
for x in m["payloads"]:
 p=ROOT/x["file"];need(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["file"])
sealed={x["file"]:x for x in s["files"]}
for x in m["payloads"]:need(x["file"] in sealed and sealed[x["file"]]["sha256"]==x["sha256"],"seal "+x["file"])
need(sha(ROOT/"MANIFEST.json")==sealed["MANIFEST.json"]["sha256"],"manifest seal")
need(e["host_to_child"]["dme_state_to_clientjob_json"]=="STATIC_SOURCE_BOUND","host JSON edge")
need(e["host_to_child"]["clientjob_json_to_target_ddp_xml_leaf"]=="UNBOUND","xml writer")
need(e["mode"]["effective_stored_value"].startswith("UNBOUND"),"mode stored value")
need(all(x["typed_consumer"]=="UNBOUND" for x in e["mix_levels"]),"level consumers")
need(v["status"]=="NOT_RUN" and v["not_a_pass_record"],"vectors")
need(e["execution"]["dee_run"]=="NOT_RUN" and e["execution"]["implementation_contact"] is False,"execution")
print(json.dumps({"status":"PASS","dme_xml_writer":"UNBOUND","mode_storage":"UNBOUND","levels":"UNBOUND"}))
