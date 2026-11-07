from __future__ import annotations
import hashlib,json,re
from pathlib import Path
R=Path(__file__).resolve().parent;PID="DEE-ADM-INDIVIDUAL-RECORD-ACCESSOR-NO-LOCAL-BOUNDS-BOUNDARY-V2";SM="3656d7ad09f5c8647b239ba931872a58d094de1644ba93cc2f102c9f3d93209b";SS="79fa02f48aa7e8575f4693b618766348b193ebfc3597384a3b35c00c94e53b36"
def req(v,n):
 if not v:raise RuntimeError(f"FAIL {PID}: {n}")
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
mp=R/"MANIFEST.json";m=json.loads(mp.read_text(encoding="utf-8"));s=json.loads((R/"package_seal.json").read_text(encoding="utf-8"));e=json.loads((R/"EVIDENCE.json").read_text(encoding="utf-8"))
req(m.get("package_id")==PID and m.get("source_manifest_sha256")==SM and m.get("source_package_seal_sha256")==SS,"identity");req(s.get("manifest_sha256")==h(mp),"seal")
want={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"};req({x.get("name") for x in m.get("payloads",[])}==want,"inventory")
for x in m["payloads"]:
 p=R/x["name"];req(p.is_file() and p.stat().st_size==x["bytes"] and h(p)==x["sha256"],"payload")
req(e["local_bounds_in_selected_accessor"] is False and e["accessor_reads_k_count"] is False and e["remaining_boundaries"]["physical_slot_less_than_k_count"]=="UNBOUND","bounded result")
j=json.dumps(e,ensure_ascii=True)+(R/"REPORT.md").read_text(encoding="utf-8");req(re.search(r"[A-Za-z]:\\",j) is None and re.search(r"0x[0-9a-fA-F]{6,}",j) is None,"leak")
req(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False,"content flags")
print(f"PASS package={PID} local_slot_bounds=NO runtime_oob=NOT_OBSERVED")
