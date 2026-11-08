from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def need(ok,msg):
 if not ok:raise SystemExit("FAIL: "+msg)
m=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8"))
e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((ROOT/"INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
for x in m["payloads"]:
 p=ROOT/x["file"];need(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["file"])
sealed={x["file"]:x for x in s["files"]}
for x in m["payloads"]:need(x["file"] in sealed and sealed[x["file"]]["sha256"]==x["sha256"],"seal "+x["file"])
need(sha(ROOT/"MANIFEST.json")==sealed["MANIFEST.json"]["sha256"],"manifest seal")
need(e["construction"]["same_path_owner_join"].endswith("UNBOUND"),"same-path descriptor join")
need(e["callsite"]["target"].endswith("UNBOUND"),"slot target")
need(e["first_file_path_action"]=="UNBOUND" and e["cleanup_delete_effect"]=="UNBOUND","cleanup effect")
need(e["runtime_owner_generation"]=="NOT_CAPTURED","runtime generation")
need(v["status"]=="STATIC_EDGE_ONLY_NOT_RUN" and v["not_a_pass_record"],"vector")
need(e["exclusions"]["media_pcm_payload_key_license"] is False,"payload policy")
print(json.dumps({"status":"PASS","slot58":"UNBOUND","owner_join":"UNBOUND","cleanup":"UNBOUND"}))
