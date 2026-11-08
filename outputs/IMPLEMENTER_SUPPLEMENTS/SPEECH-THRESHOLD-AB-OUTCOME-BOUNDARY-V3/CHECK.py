from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def need(ok,msg):
 if not ok:raise SystemExit("FAIL: "+msg)
m=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8"))
e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
for x in m["payloads"]:
 p=ROOT/x["file"];need(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["file"])
sealed={x["file"]:x for x in s["files"]}
for x in m["payloads"]:need(x["file"] in sealed and sealed[x["file"]]["sha256"]==x["sha256"],"seal "+x["file"])
need(sha(ROOT/"MANIFEST.json")==sealed["MANIFEST.json"]["sha256"],"manifest seal")
need(e["implementation_ready"] is False,"readiness")
need(len(e["arms"])==2,"arm count")
for a in e["arms"]:
 need(a["exit_code"]==10 and a["natural_exit"] and a["observer_event_count"]==0,"arm outcome")
 need(a["raw_module_base_value"]=="OMITTED","raw address")
 need(not a["output_file_exists_now"] and a["output_directory_empty_now"] and a["temp_directory_empty_now"],"output/temp status")
 need(not a["payload_read_or_retained"] and not a["original_receipts_modified"],"receipt/payload policy")
 need("run_receipt_path" not in a and "status_receipt_path" not in a,"external path")
need(e["conclusion"]["threshold_effect"]=="UNBOUND_NOT_CAPTURED" and e["conclusion"]["exit_cause"]=="UNBOUND","conclusion")
need(e["redaction"]["raw_module_base_value_included"] is False,"redaction")
need(e["redaction"]["key_license_read"] is False and e["redaction"]["payload_retained"] is False,"excluded material")
need("path" not in e["source_package"] and "path" not in e["preflight_parent"],"research path leakage")
print(json.dumps({"status":"PASS","arms":2,"effect":"UNBOUND_NOT_CAPTURED","external_paths_read":False}))
