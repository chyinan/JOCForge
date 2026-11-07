from __future__ import annotations
import hashlib,json,re
from pathlib import Path
R=Path(__file__).resolve().parent;PID="DEE-TIME-BASE-SELECTED-PROXY-SETTER-CHAIN-BOUNDARY-V1";SM="172f69569673e605bb1d0077317cc4c62e7082d19fbee81e56006cdd7c947717";SUM="c6ff39336cdd9d5a802062e142a37795b6021caa04577c76d22d1d312922a121"
def req(v,n):
 if not v:raise RuntimeError(f"FAIL {PID}: {n}")
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
mp=R/"MANIFEST.json";m=json.loads(mp.read_text(encoding="utf-8"));s=json.loads((R/"package_seal.json").read_text(encoding="utf-8"));e=json.loads((R/"EVIDENCE.json").read_text(encoding="utf-8"))
req(m.get("package_id")==PID and m.get("source_manifest_sha256")==SM and m.get("source_sha256sums_file_sha256")==SUM,"identity");req(s.get("manifest_sha256")==h(mp),"seal")
want={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"};req({x.get("name") for x in m.get("payloads",[])}==want,"inventory")
for x in m["payloads"]:
 p=R/x["name"];req(p.is_file() and p.stat().st_size==x["bytes"] and h(p)==x["sha256"],"payload")
req(e["static_chain"]["slot_to_nested_stage_setter"]=="BOUND_STATIC" and e["time_base_field"]["direct_operand_in_selected_setter_body"]=="NOT_OBSERVED" and e["time_base_field"]["xml_value_or_omission_to_descriptor"]=="UNBOUND","boundary")
j=json.dumps(e,ensure_ascii=True)+(R/"REPORT.md").read_text(encoding="utf-8");req(re.search(r"[A-Za-z]:\\",j) is None and re.search(r"0x[0-9a-fA-F]{6,}",j) is None,"leak")
req(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False,"content flags")
print(f"PASS package={PID} setter_chain=BOUND_STATIC xml_value=UNBOUND")
