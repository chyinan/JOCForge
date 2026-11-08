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
need(e["explicit_values"]["runtime_xml_acceptance"]=="NOT_RUN","runtime")
need(e["source_fields"]["exact_xml_element_or_attribute"]=="UNBOUND","XML source")
need(e["source_fields"]["ffoa_source_writer_to_descriptor+0x58"]=="UNBOUND","FFOA writer")
need(e["execution"]["dee"]=="NOT_RUN","DEE status")
need(v["status"]=="PROPOSED_INPUT_ONLY_NOT_RUN" and v["not_a_pass_record"],"vector")
print(json.dumps({"status":"PASS","conversion":"STATIC_ONLY","source_fields":"UNBOUND","runtime":"NOT_RUN"}))
