from __future__ import annotations
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
PID="DEE-ADM-READER-N-ARGUMENT-PRODUCER-BOUNDARY-V1"
SM="b037cddeaea066337fd753cc1f73be66b504eeb035982c6b1339779c29f0fae4"
def require(ok,label):
    if not ok: raise RuntimeError(f"FAIL {PID}: {label}")
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
mp=ROOT/"MANIFEST.json"; m=json.loads(mp.read_text(encoding="utf-8")); s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8")); e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
require(m.get("package_id")==PID and m.get("source_manifest_sha256")==SM,"identity")
require(s.get("manifest_sha256")==sha(mp),"seal")
expected={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"}
require({x.get("name") for x in m.get("payloads",[])}==expected,"inventory")
for x in m["payloads"]:
 p=ROOT/x["name"]; require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["name"])
require(e["reader_n"]["source"]=="Reader descriptor U16 +0x0A" and e["sample_count_s"]["forwarded_separately_from_n"] is True,"input domains")
require(e["first_breaks"]["physical_chna_n_to_reader_descriptor"]=="UNBOUND" and e["first_breaks"]["reader_descriptor_to_mode_zero_composite_k_same_owner_generation"]=="UNBOUND","join boundaries")
joined=json.dumps(e,ensure_ascii=True)+(ROOT/"REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\",joined) is None and re.search(r"0x[0-9a-fA-F]{6,}",joined) is None,"leak check")
require(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False,"content flags")
print(f"PASS package={PID} N_and_S_separate=BOUND physical_N_join=UNBOUND")
