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
need(e["static_boundary"]["first_typed_algorithm_use"]=="UNBOUND","algorithm status")
need(e["static_boundary"]["owner_generation_join"]=="UNBOUND","owner status")
need(e["conclusion"]["new_runtime_capture"]=="NOT_RUN","runtime status")
need(e["prior_measurement_observations"]["measurement_unit"]=="NOT_STATED_IN_NATIVE_LOG","unit status")
need(all(x["arms"][0]["new_run"]=="NOT_RUN" and x["arms"][1]["new_run"]=="NOT_RUN" for x in v["pairs"]),"vector status")
need(e["exclusions"]["pcm"] is False and e["exclusions"]["key_license"] is False,"exclusion status")
print(json.dumps({"status":"PASS","DI_consumer":"UNBOUND","new_run":"NOT_RUN"}))
