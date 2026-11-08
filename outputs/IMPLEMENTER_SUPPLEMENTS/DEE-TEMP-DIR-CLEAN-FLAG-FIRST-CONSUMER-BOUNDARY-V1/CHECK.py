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
need(e["config_binding"]["static_object_join"]=="BOUND_TO_STAGE_RUNNER_TEMP_PATH_OBJECT","writer owner scope")
need(e["first_consumer"]["cleanup_or_delete_effect"]=="UNBOUND","cleanup effect")
need(e["first_consumer"]["slot_target_and_object_type"]=="UNBOUND","provider target")
need(e["execution"]["dee"]=="NOT_RUN" and e["execution"]["file_deletion_performed"] is False,"runtime status")
need(v["status"]=="INPUT_ONLY_STATIC_NOT_RUN" and v["not_a_pass_record"],"vector status")
need(e["exclusions"]["media_pcm_payload_key_license_read"] is False,"payload policy")
print(json.dumps({"status":"PASS","field_read":"BOUND_STATIC_PATH_ONLY","cleanup":"UNBOUND","runtime":"NOT_RUN"}))
