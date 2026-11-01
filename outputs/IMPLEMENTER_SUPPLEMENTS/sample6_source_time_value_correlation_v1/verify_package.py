from __future__ import annotations
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(value,message):
    if not value: raise SystemExit(message)
raw=(ROOT/"MANIFEST.json").read_bytes()
expected=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(raw).hexdigest()==expected,"manifest hash mismatch")
m=json.loads(raw)
require(m.get("schema")=="washed-sample6-source-time-value-correlation.v1","schema mismatch")
require(m.get("status")=="SAME_INPUT_AND_SCALAR_VALUES_BOUND; AXML_TO_PROPERTY_WRITER_UNBOUND","status mismatch")
for e in m.get("payloads",[]):
    p=(ROOT/e["file"]).resolve()
    require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad payload path")
    data=p.read_bytes()
    require(len(data)==e["bytes"] and hashlib.sha256(data).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["source"]["full_file_sha256"]=="7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1","input identity mismatch")
require(o["same_hash_native_reference"]["offset_seconds"]==o["same_hash_native_reference"]["ffoa_seconds"]==3610.0,"scalar values mismatch")
require(o["conditional_range"]["sample6_result"]==0,"sample6 derived start mismatch")
require(o["limits"]["axml_start_to_offset_writer"]=="UNBOUND","writer boundary overclaimed")
require(o["disposition"]["general_auto_resolver_approved"] is False,"general resolver must remain unapproved")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
    text=(ROOT/name).read_text(encoding="utf-8")
    require("FUN_180" not in text and "0x" not in text,"raw reverse coordinates leaked")
    require("D:\\" not in text and "C:\\" not in text,"absolute path leaked")
print("WASHED_SAMPLE6_SOURCE_TIME=PASS")
print("GENERAL_AUTO_RESOLVER=UNBOUND")
