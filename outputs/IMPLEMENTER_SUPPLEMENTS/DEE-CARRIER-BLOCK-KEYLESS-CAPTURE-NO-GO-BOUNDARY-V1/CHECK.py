from __future__ import annotations
import hashlib,json,re
from pathlib import Path
R=Path(__file__).resolve().parent;PID="DEE-CARRIER-BLOCK-KEYLESS-CAPTURE-NO-GO-BOUNDARY-V1";SM="9466a1a59ffef87bcedad71e7a789d3a719ef0a315145adfe597540904af46ea";SS="5cdff0a74a2b52daebb41285a8ba3f724cbeefe0403252dd55b7637ea6c97718"
def req(v,n):
 if not v:raise RuntimeError(f"FAIL {PID}: {n}")
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
mp=R/"MANIFEST.json";m=json.loads(mp.read_text(encoding="utf-8"));s=json.loads((R/"package_seal.json").read_text(encoding="utf-8"));e=json.loads((R/"EVIDENCE.json").read_text(encoding="utf-8"))
req(m.get("package_id")==PID and m.get("source_manifest_sha256")==SM and m.get("source_package_seal_sha256")==SS,"identity");req(s.get("manifest_sha256")==h(mp),"seal")
want={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"};req({x.get("name") for x in m.get("payloads",[])}==want,"inventory")
for x in m["payloads"]:
 p=R/x["name"];req(p.is_file() and p.stat().st_size==x["bytes"] and h(p)==x["sha256"],"payload")
req(e["decision"]["attempt"]=="NO_GO/BLOCKED_BEFORE_LAUNCH" and e["decision"]["capture"]=="NOT_RUN","decision")
req(e["receipt_scope"]["license_or_key_content_read"] is False and e["retained_boundaries"]["third_u32_semantic_owner"]=="UNBOUND","scope")
j=json.dumps(e,ensure_ascii=True)+(R/"REPORT.md").read_text(encoding="utf-8");req(re.search(r"[A-Za-z]:\\",j) is None and re.search(r"0x[0-9a-fA-F]{6,}",j) is None,"leak")
req(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False,"content flags")
print(f"PASS package={PID} launch=NO_GO capture=NOT_RUN")
