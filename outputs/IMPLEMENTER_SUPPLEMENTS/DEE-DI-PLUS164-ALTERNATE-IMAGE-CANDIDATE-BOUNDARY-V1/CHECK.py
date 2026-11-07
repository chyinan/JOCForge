from __future__ import annotations
import hashlib,json,re
from pathlib import Path
R=Path(__file__).resolve().parent;PID="DEE-DI-PLUS164-ALTERNATE-IMAGE-CANDIDATE-BOUNDARY-V1";SM="c0e8a2510244e073c4a93bcc089f256b780e680acb61f4b03cfaa055c7535a70";SS="2742cb6e72fd832b820b2adada0f8772dfa5e952cd6f74c639e11546968cb3a2"
def req(v,n):
 if not v:raise RuntimeError(f"FAIL {PID}: {n}")
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
mp=R/"MANIFEST.json";m=json.loads(mp.read_text(encoding="utf-8"));s=json.loads((R/"package_seal.json").read_text(encoding="utf-8"));e=json.loads((R/"EVIDENCE.json").read_text(encoding="utf-8"))
req(m.get("package_id")==PID and m.get("source_manifest_sha256")==SM and m.get("source_package_seal_sha256")==SS,"identity");req(s.get("manifest_sha256")==h(mp),"seal")
want={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"};req({x.get("name") for x in m.get("payloads",[])}==want,"inventory")
for x in m["payloads"]:
 p=R/x["name"];req(p.is_file() and p.stat().st_size==x["bytes"] and h(p)==x["sha256"],"payload")
req(e["candidate"]["reads_plus_0x164"] is True and e["identity_boundary"]["candidate_child_record_to_target_dll_subcontext"]=="UNBOUND","owner boundary")
j=json.dumps(e,ensure_ascii=True)+(R/"REPORT.md").read_text(encoding="utf-8");req(re.search(r"[A-Za-z]:\\",j) is None and re.search(r"0x[0-9a-fA-F]{6,}",j) is None,"leak")
req(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False,"content flags")
print(f"PASS package={PID} cross_image_owner_join=UNBOUND")
