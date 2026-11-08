from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
SRC=Path(r"C:\Users\chyinan\Documents\Codex\2026-10-23\jocforge-reverse-agent-3\outputs\speech_threshold_ab_run_summary_wash_v1")
PRE=Path(r"C:\Users\chyinan\Documents\Codex\2026-10-23\jocforge-reverse-agent-3\outputs\speech_threshold_ab_prelaunch_wash_v1")
def sha(p):
 h=hashlib.sha256()
 with Path(p).open("rb") as f:
  for b in iter(lambda:f.read(1048576),b""):h.update(b)
 return h.hexdigest()
def need(ok,msg):
 if not ok:raise SystemExit("FAIL: "+msg)
m=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8"));e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
for x in m["payloads"]:
 p=ROOT/x["file"];need(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["file"])
seal={x["file"]:x for x in s["files"]}
for x in m["payloads"]:need(x["file"] in seal and seal[x["file"]]["sha256"]==x["sha256"],"seal "+x["file"])
need(sha(ROOT/"MANIFEST.json")==seal["MANIFEST.json"]["sha256"],"manifest seal")
need(sha(SRC/"evidence_manifest.json")==e["source_package"]["manifest_sha256"],"source manifest")
need(sha(SRC/"package_seal.json")==e["source_package"]["package_seal_sha256"],"source seal")
need(sha(SRC/"RUN-SUMMARY.json")==e["source_package"]["run_summary_sha256"],"source summary")
need(sha(PRE/"evidence_manifest.json")==e["preflight_parent"]["manifest_sha256"],"preflight manifest")
need(sha(PRE/"package_seal.json")==e["preflight_parent"]["package_seal_sha256"],"preflight seal")
src=json.loads((SRC/"RUN-SUMMARY.json").read_text(encoding="utf-8"));need(len(src["arms"])==2,"arm count")
for a in src["arms"]:
 need(a["exit_code"]==10 and a["observer_event_count"]==0 and a["natural_exit"],"arm result")
 need(a["module_base_value"]=="OMITTED","raw address")
 need(not a["output_file_exists_now"] and a["output_directory_empty_now"] and a["temp_directory_empty_now"],"output/temp")
 need(sha(a["run_receipt_path"])==a["run_receipt_sha256"],"run receipt")
 need(sha(a["status_receipt_path"])==a["status_receipt_sha256"],"status receipt")
need(e["conclusion"]["threshold_effect"]=="UNBOUND_NOT_CAPTURED","effect")
need(e["conclusion"]["exit_cause"]=="UNBOUND","exit cause")
need(not e["redaction"]["raw_module_base_value_included"],"redaction")
print(json.dumps({"status":"PASS","arms":2,"threshold_effect":"UNBOUND_NOT_CAPTURED","exit_cause":"UNBOUND"}))
