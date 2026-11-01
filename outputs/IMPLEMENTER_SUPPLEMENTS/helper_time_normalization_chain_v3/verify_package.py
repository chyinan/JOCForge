from __future__ import annotations
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def req(c,m):
 if not c: raise SystemExit(m)
raw=(ROOT/"MANIFEST.json").read_bytes(); exp=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
req(hashlib.sha256(raw).hexdigest()==exp,"manifest hash mismatch")
m=json.loads(raw)
req(m.get("schema")=="washed-helper-time-normalization-chain.v3","schema mismatch")
req(m.get("status")=="SOURCE_INFO_FIELDS_AND_FINAL_FFOA_NORMALIZATION_STATIC_BOUND; SOURCE_INFO_FIELD_ORIGIN_UNBOUND","status mismatch")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve();req(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad payload")
 b=p.read_bytes();req(len(b)==e["bytes"] and hashlib.sha256(b).hexdigest()==e["sha256"],"payload mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
req(o["source_info"]["start_time"]["constructor_default"]==0.0,"start default mismatch")
req(o["source_info"]["absolute_ffoa"]["constructor_default"]==-1.0,"ffoa sentinel mismatch")
req(o["final_rule"]["if_absolute_ffoa_nonnegative"]=="relative FFOA = max(0, absolute FFOA - start_time)","normalization mismatch")
req("source-info start_time/absolute FFOA first writer and its AXML/BEXT/DBMD origin" in o["unbound"],"field origin must remain open")
for n in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/n).read_text(encoding="utf-8")
 req("FUN_180" not in t and re.search(r"\b0x[0-9a-fA-F]{6,}\b",t) is None,"raw coordinates leaked")
 req("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
print("WASHED_HELPER_TIME_CHAIN_V3=PASS")
print("SOURCE_INFO_FIELD_ORIGIN=UNBOUND")
