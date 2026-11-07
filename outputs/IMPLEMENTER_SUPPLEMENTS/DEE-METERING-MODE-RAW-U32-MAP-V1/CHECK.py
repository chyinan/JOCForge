from __future__ import annotations
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent
m=json.loads((root/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((root/"package_seal.json").read_text(encoding="utf-8"))
assert m["schema"]=="jocforge.wash_supplement_manifest.v1" and m["package_id"]==s["package_id"]
for p in m["payloads"]:
 d=(root/p["file"]).read_bytes();assert len(d)==p["bytes"] and hashlib.sha256(d).hexdigest()==p["sha256"]
assert s["manifest_sha256"]==hashlib.sha256((root/"MANIFEST.json").read_bytes()).hexdigest()
assert s["payload_file_count"]==len(m["payloads"]) and s["status"]==m["status"]
e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((root/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
assert [x["raw_u32"] for x in e["token_map"]]==[0,1,2,3,4] and [x["static_raw_u32"] for x in v["vectors"]]==[0,1,2,3,4]
assert all(x["effect"]=="NOT_RUN" for x in v["vectors"])
print("METERING_RAW_MAP_SUPPLEMENT_CHECK=PASS")