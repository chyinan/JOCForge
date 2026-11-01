from __future__ import annotations
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def req(c,m):
    if not c: raise SystemExit(m)
raw=(ROOT/"MANIFEST.json").read_bytes()
expected=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
req(hashlib.sha256(raw).hexdigest()==expected,"manifest hash mismatch")
m=json.loads(raw)
req(m.get("schema")=="washed-helper-ffoa-offset-auto-normalization.v1","schema mismatch")
req(m.get("status")=="RELATIVE_FFOA_NORMALIZATION_STATIC; START_TIME_PRODUCER_UNBOUND","status mismatch")
for e in m.get("payloads",[]):
    p=(ROOT/e["file"]).resolve()
    req(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad payload")
    b=p.read_bytes()
    req(len(b)==e["bytes"] and hashlib.sha256(b).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
req(o["helper_time_fields"]["relative_ffoa_default_sentinel"]==-1.0,"FFOA sentinel mismatch")
req(o["helper_time_fields"]["host_output_property_mapping"]["Start time (seconds)"]=="offset","offset mapping mismatch")
req(o["relative_ffoa"]["normalization"]=="relative < 0 becomes 0","negative normalization mismatch")
req(any("master start_time first writer" in item for item in o["unbound"]),"start-time source must remain open")
for n in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
    t=(ROOT/n).read_text(encoding="utf-8")
    req("FUN_180" not in t and "0x" not in t,"raw reverse coordinates leaked")
    req("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
print("WASHED_HELPER_FFOA_NORMALIZATION=PASS")
print("START_TIME_PRODUCER=UNBOUND")
