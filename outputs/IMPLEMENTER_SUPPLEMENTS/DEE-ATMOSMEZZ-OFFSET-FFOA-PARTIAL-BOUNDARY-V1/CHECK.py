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
need(e["properties"]["offset"]["first_ddp_consumer"]=="FUN_18014fd90","offset consumer")
need(e["properties"]["ffoa"]["first_ddp_consumer"]=="FUN_18014fd90","ffoa consumer")
need(e["combined_dependency"]["adm_bwf_source_chunk_for_named_fields"]=="UNBOUND","source chunk")
need(e["combined_dependency"]["reader_or_frame_alignment_consumer"]=="UNBOUND","reader consumer")
need(v["status"]=="PROPOSED_NOT_RUN" and v["not_a_pass_record"],"vector status")
need(e["execution"]["dee_run"]=="NOT_RUN","runtime status")
need(e["conclusion"]["global_absence_claim"] is False,"absence scope")
print(json.dumps({"status":"PASS","auto_rule":"STATIC_RULE_BOUND","explicit_values":"UNBOUND","vectors":"NOT_RUN"}))
