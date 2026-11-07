from __future__ import annotations
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
assert m["schema"]=="jocforge.wash_supplement_manifest.v1" and m["package_id"]==s["package_id"]
for p in m["payloads"]:
 d=(root/p["file"]).read_bytes();assert len(d)==p["bytes"] and hashlib.sha256(d).hexdigest()==p["sha256"]
assert s["manifest_sha256"]==hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest() and s["status"]==m["status"]
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"));assert e["configured_value"]=="file_position" and e["dee_exit_code"]==0 and e["what_this_does_not_prove"][0].startswith("XML value reached")
print("TIMEBASE_ATTEMPT13_SUPPLEMENT_CHECK=PASS")