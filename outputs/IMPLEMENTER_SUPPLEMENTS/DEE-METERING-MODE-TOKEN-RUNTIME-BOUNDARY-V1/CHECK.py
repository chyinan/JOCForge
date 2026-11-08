from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def need(ok,msg):
 if not ok:raise SystemExit("FAIL: "+msg)
m=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8"));e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((ROOT/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
for x in m["payloads"]:
 p=ROOT/x["file"];need(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["file"])
sealed={x["file"]:x for x in s["files"]}
for x in m["payloads"]:need(x["file"] in sealed and sealed[x["file"]]["sha256"]==x["sha256"],"seal "+x["file"])
need(sha(ROOT/"MANIFEST.json")==sealed["MANIFEST.json"]["sha256"],"manifest seal")
need([x["raw_u32_candidate"] for x in e["static_candidate_map"]]==[0,1,2,3,4],"candidate map")
need(e["mapping_boundary"]["first_effective_algorithm_consumer"]=="UNBOUND","consumer boundary")
need(e["experiment"]["exit_code"]==10 and not e["experiment"]["measurement_scalar_captured"] and not e["experiment"]["output_file_created"],"run result")
need(e["experiment"]["failure_cause"]=="UNBOUND" and e["experiment"]["retry"]=="NOT_RUN","failure status")
need(e["prior_measurement_observation"]["new_run"] is False,"prior observation scope")
need(v["vector_is_not_a_pass_record"] is True,"vector status")
need(e["payload_policy"]["pcm_read"] is False and e["payload_policy"]["key_license_read"] is False,"payload policy")
print(json.dumps({"status":"PASS","tokens":5,"first_effective_consumer":"UNBOUND","new_1770_3":"EXIT10_NO_SCALAR"}))
