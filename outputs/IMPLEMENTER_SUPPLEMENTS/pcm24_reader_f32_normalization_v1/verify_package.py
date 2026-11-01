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
req(m.get("schema")=="washed-pcm24-reader-f32-normalization.v1","schema mismatch")
req(m.get("status")=="STATIC_PCM24_TO_F32_RULE_CLOSED_FOR_CONTEXT7_BRANCH; END_TO_END_ROW_JOIN_OPEN","status mismatch")
for e in m.get("payloads",[]):
    p=(ROOT/e["file"]).resolve()
    req(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad payload")
    b=p.read_bytes()
    req(len(b)==e["bytes"] and hashlib.sha256(b).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
req(o["applicability"]["reader_context_value"]==7 and o["applicability"]["bytes_per_sample"]==3,"branch scope mismatch")
req(o["algorithm"]["equivalent_formula"]=="binary32(s24 / 2^23)","formula mismatch")
req(o["still_unbound"] and o["applicability"]["native_job_capture"]=="NOT_RUN","runtime boundary missing")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
    t=(ROOT/name).read_text(encoding="utf-8")
    req("FUN_180" not in t and "0x" not in t,"reverse coordinates leaked")
    req("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
print("WASHED_PCM24_TO_F32_RULE=PASS")
print("END_TO_END_ROW_JOIN=UNBOUND")
