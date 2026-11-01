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
req(m["schema"]=="washed-helper-time-normalization-chain.v2","schema mismatch")
req(m["status"]=="SOURCE_INFO_NORMALIZATION_STATIC_BOUND; SOURCE_INFO_START_WRITER_UNBOUND","status mismatch")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve();req(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad path")
 b=p.read_bytes();req(len(b)==e["bytes"] and hashlib.sha256(b).hexdigest()==e["sha256"],"payload mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
req(o["tuple_intermediate"]["role"].startswith("intermediate"),"tuple rule status mismatch")
req(o["source_info_contract"]["relative_when_absolute_ffoa_nonnegative"]=="max(0, absolute_ffoa - start_time)","normalization mismatch")
req("source-info start_time first writer" in o["unbound"],"start writer boundary missing")
for n in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/n).read_text(encoding="utf-8")
 req("FUN_180" not in t and re.search(r"\b0x[0-9a-fA-F]{6,}\b",t) is None,"raw coordinates leaked")
 req("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
print("WASHED_HELPER_TIME_V2=PASS")
print("START_TIME_WRITER=UNBOUND")
