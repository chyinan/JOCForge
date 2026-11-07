from __future__ import annotations
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
PID="DEE-RF-F98-CANDIDATE-FIRST-READER-OWNER-BOUNDARY-V1"
SM="04c765fa9f638feabf07cc1993ee3db96bcf21d0380bf3f627997127952fee41"
SS="2169f77752a5f2709678ca849f05baec2b299495265db053fc571ab3689dfc3d"
def require(ok,label):
    if not ok: raise RuntimeError(f"FAIL {PID}: {label}")
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
mp=ROOT/"MANIFEST.json"; m=json.loads(mp.read_text(encoding="utf-8")); s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8")); e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
require(m.get("package_id")==PID and m.get("source_manifest_sha256")==SM and m.get("source_package_seal_sha256")==SS,"identity")
require(s.get("manifest_sha256")==sha(mp),"seal")
expected={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"}
require({x.get("name") for x in m.get("payloads",[])}==expected,"inventory")
for x in m["payloads"]:
 p=ROOT/x["name"]; require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["name"])
require(e["candidate_observation"]["entry_read_before_next_helper"]=="NOT_OBSERVED" and e["candidate_observation"]["candidate_argument_to_producer_owner_join"]=="UNBOUND","candidate boundary")
joined=json.dumps(e,ensure_ascii=True)+(ROOT/"REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\",joined) is None and re.search(r"0x[0-9a-fA-F]{6,}",joined) is None,"leak check")
require(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False,"content flags")
print(f"PASS package={PID} producer_owner_join=UNBOUND")
