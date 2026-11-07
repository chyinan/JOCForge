from __future__ import annotations
import hashlib
import json
from pathlib import Path
root = Path(__file__).resolve().parent
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8")); s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
assert m["schema"]=="jocforge.wash_supplement_manifest.v1" and m["package_id"]==s["package_id"]
for p in m["payloads"]:
 d=(root/p["file"]).read_bytes(); assert len(d)==p["bytes"] and hashlib.sha256(d).hexdigest()==p["sha256"]
assert s["manifest_sha256"]==hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()
assert s["payload_hash_audit"]==f"PASS_{len(m['payloads'])}_OF_{len(m['payloads'])}" and s["checker_status"]=="PASS_NORMAL_AND_PYTHON_O" and s["status"]==m["status"]
v=json.loads((root/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8")); assert len(v["vectors"])==2 and all(x["status"]=="PROPOSED_NOT_RUN" for x in v["vectors"])
print("TIMEBASE_WRITER_SUPPLEMENT_CHECK=PASS")