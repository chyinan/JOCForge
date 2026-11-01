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
req(m.get("schema")=="washed-custom-dialnorm-option-boundary.v1","schema mismatch")
req(m.get("status")=="OPTION_ID_AND_DEFAULT_STATIC; HOST_INSTANCE_TRANSFER_AND_U16_EDGE_UNBOUND","status mismatch")
for e in m.get("payloads",[]):
    p=(ROOT/e["file"]).resolve();req(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad payload")
    b=p.read_bytes();req(len(b)==e["bytes"] and hashlib.sha256(b).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
req(o["option"]["xml_key"]=="custom_dialnorm" and o["option"]["default"]==0,"option/default mismatch")
req(o["run144"]["task_xml_custom_dialnorm"]=="ABSENT","run config mismatch")
req(o["unbound"] and o["implementation_disposition"]["production_dialnorm_formula_approved"] is False,"open edge missing")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
    t=(ROOT/name).read_text(encoding="utf-8")
    req("FUN_180" not in t and re.search(r"\b0x[0-9a-fA-F]{6,}\b",t) is None,"raw function addresses leaked")
    req("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
print("WASHED_CUSTOM_DIALNORM_OPTION=PASS")
print("MEASUREMENT_TO_U16_FORMULA=UNBOUND")
