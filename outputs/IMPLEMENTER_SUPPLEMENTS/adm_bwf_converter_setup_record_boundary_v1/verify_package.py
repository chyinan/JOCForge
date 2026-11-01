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
req(m.get("schema")=="washed-adm-bwf-converter-setup-record-boundary.v1","schema mismatch")
req(m.get("status")=="ADM_SELECTOR3_AND_SETUP_CONSUMER_BOUND; PARAM4_RECORD_PRODUCER_TO_CHNA_UNBOUND","status mismatch")
for e in m.get("payloads",[]):
    p=(ROOT/e["file"]).resolve()
    req(p.is_relative_to(ROOT.resolve()) and p.is_file(),"bad payload")
    b=p.read_bytes()
    req(len(b)==e["bytes"] and hashlib.sha256(b).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
req(o["registration"]["adm_bwf_route_selector"]==3 and o["registration"]["damf_route_selector"]==2,"route selector mismatch")
req(o["open"]["param4_record_table_producer"]=="UNBOUND","record producer overclaimed")
req(o["implementation_disposition"]["new_row_permutation_approved"] is False,"row mapping overclaimed")
for n in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
    t=(ROOT/n).read_text(encoding="utf-8")
    req("FUN_180" not in t and "0x" not in t,"reverse coordinates leaked")
    req("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
print("WASHED_ADM_SETUP_RECORD_BOUNDARY=PASS")
print("PARAM4_TO_CHNA=UNBOUND")
