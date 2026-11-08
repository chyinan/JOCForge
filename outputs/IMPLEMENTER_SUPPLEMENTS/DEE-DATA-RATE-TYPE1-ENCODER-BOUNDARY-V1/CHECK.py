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
need([x["enum_code"] for x in e["static_mapping"]]==[0,1,2,3,4,5],"enum map")
need(all(x["core_field_id"]==3 for x in e["static_mapping"]),"field id")
need(e["carrier_boundary"]["field_id_3_to_bsi_or_frame_size_writer"]=="UNBOUND","carrier writer")
need(e["conclusion"]["runtime_output_rate"]=="NOT_RUN","runtime status")
need(e["minimum_contrast"]["status"]=="PROPOSED_NOT_RUN" and v["not_a_pass_record"],"vector status")
need(e["exclusions"]["ffmpeg_or_other_encoder_used"] is False and e["exclusions"]["key_license_read"] is False,"exclusions")
print(json.dumps({"status":"PASS","tokens":6,"field_id":3,"carrier_writer":"UNBOUND","runtime":"NOT_RUN"}))
