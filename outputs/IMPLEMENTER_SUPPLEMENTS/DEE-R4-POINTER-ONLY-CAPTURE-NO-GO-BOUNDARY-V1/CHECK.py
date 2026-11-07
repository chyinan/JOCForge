from __future__ import annotations
import hashlib,json,re
from pathlib import Path
R=Path(__file__).resolve().parent;PID="DEE-R4-POINTER-ONLY-CAPTURE-NO-GO-BOUNDARY-V1";SM="245f5446b8805f41d43c3d2c000a79afd879254066e5db588d94a337961dd285";SS="98c7593c1819154fd4d37e425f53d5e2f77907f9cd84553786b73d8cb3f3c64d"
def req(v,n):
 if not v:raise RuntimeError(f"FAIL {PID}: {n}")
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
mp=R/"MANIFEST.json";m=json.loads(mp.read_text(encoding="utf-8"));s=json.loads((R/"package_seal.json").read_text(encoding="utf-8"));e=json.loads((R/"EVIDENCE.json").read_text(encoding="utf-8"))
req(m.get("package_id")==PID and m.get("source_manifest_sha256")==SM and m.get("source_package_seal_sha256")==SS,"identity");req(s.get("manifest_sha256")==h(mp),"seal")
want={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"};req({x.get("name") for x in m.get("payloads",[])}==want,"inventory")
for x in m["payloads"]:
 p=R/x["name"];req(p.is_file() and p.stat().st_size==x["bytes"] and h(p)==x["sha256"],"payload")
req(e["preflight"]["pointer_capture"]=="NOT_RUN" and e["recent_sample_receipts"]["eligible_normal_no_key_job"] is False and e["disposition"]["terminal_drc_effect"]=="UNBOUND_NO","scope")
j=json.dumps(e,ensure_ascii=True)+(R/"REPORT.md").read_text(encoding="utf-8");req(re.search(r"[A-Za-z]:\\",j) is None and re.search(r"0x[0-9a-fA-F]{6,}",j) is None,"leak")
req(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False,"content flags")
print(f"PASS package={PID} pointer_capture=NOT_RUN terminal_drc_effect=UNBOUND")
