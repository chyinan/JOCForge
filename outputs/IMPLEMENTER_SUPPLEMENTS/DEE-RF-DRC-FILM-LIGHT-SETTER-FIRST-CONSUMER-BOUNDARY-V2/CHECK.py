from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
def require(ok,msg):
    if not ok: raise SystemExit(msg)
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"))
s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
require(m["schema"]=="jocforge.wash_supplement_manifest.v1","manifest schema")
require(m["package_id"]==s["package_id"]=="DEE-RF-DRC-FILM-LIGHT-SETTER-FIRST-CONSUMER-BOUNDARY-V2","package identity")
require(m["status"]==s["status"],"status mismatch")
require(hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()==s["manifest_sha256"],"manifest hash")
require(s["payload_file_count"]==len(m["payloads"])==5,"payload count")
require(s["payload_hash_audit"]=="PASS_5_OF_5","payload audit")
require(s["checker_status"] in {"PENDING","PASS_NORMAL_AND_PYTHON_O"},"checker status")
expected={"REPORT.md","EVIDENCE.json","SOURCE-MANIFESTS.json","INPUT-ONLY-VECTORS.json","CHECK.py"}
require({x["file"] for x in m["payloads"]}==expected,"payload set")
for item in m["payloads"]:
    data=(root/item["file"]).read_bytes()
    require(len(data)==item["bytes"],"size mismatch "+item["file"])
    require(hashlib.sha256(data).hexdigest()==item["sha256"],"hash mismatch "+item["file"])
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
require(e["source_package"]["manifest_sha256"] == "8626e8b262736eb7ba9ac73a62f488bf33217793a4d058ef6345dea35d6231bd", "source_package.manifest_sha256 mismatch")
require(e["field"]["input_token"] == "film_light", "field.input_token mismatch")
require(e["xml_runtime_join"]["same_owner_write_to_value_plus_0x08"] == "UNBOUND", "xml_runtime_join.same_owner_write_to_value_plus_0x08 mismatch")
print("DEE-RF-DRC-FILM-LIGHT-SETTER-FIRST-CONSUMER-BOUNDARY-V2_CHECK=PASS")
