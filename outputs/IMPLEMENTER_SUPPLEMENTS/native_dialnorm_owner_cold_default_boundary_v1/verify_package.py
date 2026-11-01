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
req(m.get("schema")=="washed-native-dialnorm-owner-cold-default-boundary.v1","schema mismatch")
req(m.get("status")=="COLD_U16_DEFAULT_AND_CONSUMER_OWNER_BOUND; POST_INIT_VALUE_WRITER_UNBOUND","status mismatch")
for e in m.get("payloads",[]):
 p=(ROOT/e["file"]).resolve();req(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad payload")
 b=p.read_bytes();req(len(b)==e["bytes"] and hashlib.sha256(b).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
req(o["cold_state"]["constructor_and_reset_default"]==27,"cold value mismatch")
req(o["later_writer_boundary"]["post_init_writer_for_28"]=="NOT_CAPTURED","post-init gap missing")
req(o["implementation_disposition"]["conversion_formula_approved"] is False,"formula must remain open")
for n in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/n).read_text(encoding="utf-8")
 req("FUN_180" not in t and re.search(r"\b0x[0-9a-fA-F]{6,}\b",t) is None,"raw coordinates leaked")
 req("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
print("WASHED_DIALNORM_COLD_OWNER=PASS")
print("POST_INIT_DIALNORM_WRITER=UNBOUND")
