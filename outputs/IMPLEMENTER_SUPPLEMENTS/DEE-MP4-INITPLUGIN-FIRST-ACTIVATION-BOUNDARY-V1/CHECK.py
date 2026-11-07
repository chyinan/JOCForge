from __future__ import annotations
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
assert m["schema"]=="jocforge.wash_supplement_manifest.v1" and m["package_id"]==s["package_id"]
for p in m["payloads"]:
 d=(root/p["file"]).read_bytes();assert len(d)==p["bytes"] and hashlib.sha256(d).hexdigest()==p["sha256"]
assert s["manifest_sha256"]==hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest() and s["status"]==m["status"]
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"));assert e["first_activation"]["callsite"]=="0x140148c7d" and e["unbound"][0].startswith("indirect target")
print("MP4_ACTIVATION_SUPPLEMENT_CHECK=PASS")