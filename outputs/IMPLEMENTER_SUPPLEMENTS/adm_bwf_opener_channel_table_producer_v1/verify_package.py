from __future__ import annotations
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def req(c,m):
    if not c: raise SystemExit(m)
raw=(ROOT/"MANIFEST.json").read_bytes()
expected=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
req(hashlib.sha256(raw).hexdigest()==expected,"manifest hash mismatch")
m=json.loads(raw)
req(m.get("schema")=="washed-adm-bwf-opener-channel-table-producer.v1","schema mismatch")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve(); req(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad payload")
 b=p.read_bytes(); req(len(b)==e["bytes"] and hashlib.sha256(b).hexdigest()==e["sha256"],"payload mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
req(o["producer"]["row_selection"]=="TrackIndex-1 selects zero-based physical output row","row selection mismatch")
req("not that U16 payload" in o["correction"],"U16 identity correction missing")
req(o["remaining_join"]["same_generation"]=="UNBOUND","generation join overclaimed")
for n in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/n).read_text(encoding="utf-8")
 req("FUN_180" not in t and re.search(r"\b0x[0-9a-fA-F]{6,}\b",t) is None,"raw coordinates leaked")
 req("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
print("WASHED_OPENER_PRODUCER=PASS")
print("OPENER_TO_SETUP_JOIN=UNBOUND")
