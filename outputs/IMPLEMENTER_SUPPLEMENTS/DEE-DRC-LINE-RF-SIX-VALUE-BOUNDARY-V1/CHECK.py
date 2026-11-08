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
need(len(e["line_profile"]["candidate_values"])==6 and len(e["rf_profile"]["candidate_values"])==6,"six token counts")
need(all(x["selected_encoder_join"]=="UNBOUND" for x in e["line_profile"]["candidate_values"]),"line join")
need(all(x["xml_to_handler"]=="UNBOUND" for x in e["rf_profile"]["candidate_values"]),"RF join")
need(e["next_minimal_line_pair"]["status"]=="PROPOSED_NOT_RUN" and v["not_a_full_six_by_six_matrix"],"experiment scope")
need(e["conclusion"]["audio_gain_or_listening_effect"]=="UNBOUND","audio scope")
print(json.dumps({"status":"PASS","line_tokens":6,"rf_tokens":6,"next_line_pair":"PROPOSED_NOT_RUN"}))
