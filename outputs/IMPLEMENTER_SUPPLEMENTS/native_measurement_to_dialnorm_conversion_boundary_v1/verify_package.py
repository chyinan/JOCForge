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
req(m.get("schema")=="washed-native-measurement-to-dialnorm-conversion-boundary.v1","schema mismatch")
req(m.get("status")=="F06_FLOAT_TO_PARAMETER_RECORD_STATIC; U16_DIALNORM_SOURCE_JOIN_UNBOUND","status mismatch")
for e in m.get("payloads",[]):
    p=(ROOT/e["file"]).resolve()
    req(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad payload")
    b=p.read_bytes()
    req(len(b)==e["bytes"] and hashlib.sha256(b).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
req(o["implementation_disposition"]["dialnorm_formula_approved"] is False,"formula must remain unapproved")
req(o["cross_chain"]["typed_f06_float_to_ddp_u16_edge"]=="UNBOUND","cross-chain boundary overclaimed")
req(o["one_run_observation_plan"]["use"].startswith("augment the already authorized R3"),"observation scope mismatch")
for n in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
    t=(ROOT/n).read_text(encoding="utf-8")
    req("FUN_180" not in t and "0x" not in t,"raw reverse coordinates leaked")
    req("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
print("WASHED_F06_TYPED_BOUNDARY=PASS")
print("F06_FLOAT_TO_DIALNORM=UNBOUND")
