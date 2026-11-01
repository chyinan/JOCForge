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
req(m.get("schema")=="washed-native-eac3-dialnorm-bsi-field-binding.v1","schema mismatch")
req(m.get("status")=="OWNER_U16_FIELD_STATICALLY_BOUND_TO_BSI_DIALNORM; PRODUCER_FORMULA_OPEN","status mismatch")
for e in m.get("payloads",[]):
 p=(ROOT/e["file"]).resolve();req(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad payload")
 b=p.read_bytes();req(len(b)==e["bytes"] and hashlib.sha256(b).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
req(o["bsi_order"]["dialnorm_bits"]==[45,50],"DialNorm bit field mismatch")
req(o["runtime_reference"]["same_owner_captured"] is True,"run reference missing")
req(o["disposition"]["measured_loudness_to_dialnorm_production_rule_approved"] is False,"producer rule must remain unapproved")
for n in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/n).read_text(encoding="utf-8")
 req("FUN_180" not in t and re.search(r"\b0x[0-9a-fA-F]{6,}\b",t) is None,"reverse coordinates leaked")
 req("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
print("WASHED_EAC3_DIALNORM_FIELD=PASS")
print("MEASUREMENT_TO_DIALNORM=UNBOUND")
