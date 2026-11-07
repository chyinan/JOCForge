from __future__ import annotations
import hashlib, json, re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
PID="DEE-DI-EMPIRE-BOOL-RESULT-FIRST-BREAK-BOUNDARY-V1"
SM="240abb2fe291a7316e3a28bf1932f73a489ac07a07d6e8aefa724cc597d5850e"
SS="549b93c3c4ec348850a35144e3e604775efd54dbfbbc690dad7d62e19dc48a2b"
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
require(e["selected_consumer"]["reads_or_forwards_0x164"] is False,"selected edge")
require(e["result"]["bool_to_float32_causality"]=="UNBOUND","result boundary")
joined=json.dumps(e,ensure_ascii=True)+(ROOT/"REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\",joined) is None and re.search(r"0x[0-9a-fA-F]{6,}",joined) is None,"leak check")
require(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False,"content flags")
print(f"PASS package={PID} result_causality=UNBOUND")
