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
need(e["implementation_ready"] is False and e["xml_candidate"]["runtime_acceptance"]=="NOT_RUN","readiness/run")
need(e["static_route"]["setter_receiver_to_run_receiver"]=="UNBOUND","same-instance")
need(e["static_route"]["activation_target_abi_registration_owner"]=="UNBOUND","activation target")
need(e["static_route"]["selected_muxer_or_writer"]=="UNBOUND","muxer")
need(e["minimum_runnable_closure"]["status"]=="BLOCKED_ON_INSTALLATION_BOUND_XML_AND_SAME_OBJECT_SELECTION_JOIN","minimum closure")
need(v["status"]=="NOT_RUN" and len(v["vectors"])==2,"vectors")
need(e["exclusions"]["pcm_read"] is False and e["exclusions"]["key_license_read"] is False,"exclusions")
print(json.dumps({"status":"PASS","muxer":"UNBOUND","runtime":"NOT_RUN","implementation_ready":False}))
