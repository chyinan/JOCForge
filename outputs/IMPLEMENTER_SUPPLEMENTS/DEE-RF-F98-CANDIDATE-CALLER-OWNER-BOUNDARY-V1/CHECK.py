from __future__ import annotations
import hashlib,json,re
from pathlib import Path
R=Path(__file__).resolve().parent; PID="DEE-RF-F98-CANDIDATE-CALLER-OWNER-BOUNDARY-V1"; SM="4e8c7a289319ae3658be00bc8c1a884404eb053d104f4981211da7c2fef4aa61"; SS="fafe32d70b94bac772c22fb5979e77c00573fec9ff6f594c7e045821537c60d0"
def req(v,n):
 if not v: raise RuntimeError(f"FAIL {PID}: {n}")
def h(p): return hashlib.sha256(p.read_bytes()).hexdigest()
mp=R/"MANIFEST.json"; m=json.loads(mp.read_text(encoding="utf-8")); s=json.loads((R/"package_seal.json").read_text(encoding="utf-8")); e=json.loads((R/"EVIDENCE.json").read_text(encoding="utf-8"))
req(m.get("package_id")==PID and m.get("source_manifest_sha256")==SM and m.get("source_package_seal_sha256")==SS,"identity"); req(s.get("manifest_sha256")==h(mp),"seal")
want={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"}; req({x.get("name") for x in m.get("payloads",[])}==want,"inventory")
for x in m["payloads"]:
 p=R/x["name"]; req(p.is_file() and p.stat().st_size==x["bytes"] and h(p)==x["sha256"],"payload")
req(e["direct_call_edge"]["edge"]=="BOUND_STATIC" and e["owner_boundary"]["candidate_param1_to_f98_producer_object"]=="UNBOUND","owner join")
j=json.dumps(e,ensure_ascii=True)+(R/"REPORT.md").read_text(encoding="utf-8"); req(re.search(r"[A-Za-z]:\\",j) is None and re.search(r"0x[0-9a-fA-F]{6,}",j) is None,"leak")
req(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False,"content flags")
print(f"PASS package={PID} producer_owner_join=UNBOUND")
