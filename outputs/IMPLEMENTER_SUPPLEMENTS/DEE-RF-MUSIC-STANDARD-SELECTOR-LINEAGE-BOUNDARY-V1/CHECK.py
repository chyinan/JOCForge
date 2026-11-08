from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
def require(ok,msg):
 if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","schema")
require(m["package_id"]==s["package_id"]=="DEE-RF-MUSIC-STANDARD-SELECTOR-LINEAGE-BOUNDARY-V1","identity")
require(m["status"]==s["status"],"status")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest")
require(s["payload_file_count"]==len(m["payloads"])==5 and s["payload_hash_audit"]=="PASS_5_OF_5","payload audit")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload set")
for x in m["payloads"]:
 b=(root/x["file"]).read_bytes()
 require(len(b)==x["bytes"] and hashlib.sha256(b).hexdigest()==x["sha256"],"payload "+x["file"])
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(e["rf_input"]["token"]=="music_standard" and e["rf_input"]["static_map_raw_u32"]==3,"input map")
require(e["selector"]["field"]=="Priv+0xfc8","selector field")
require(e["selector"]["parsewav_writer"]["rule"]=="2 if channels < 10 else 1","parseWav rule")
require(e["selector"]["raw_music_standard_3_to_selector"]=="UNBOUND","raw-to-selector join")
require(e["effect"]["bsi_writer"]=="UNBOUND" and e["effect"]["dee_run"]=="NOT_RUN","effect status")
print("DEE-RF-MUSIC-STANDARD-SELECTOR-LINEAGE-BOUNDARY-V1_CHECK=PASS")
