from __future__ import annotations
import hashlib, json, re
from pathlib import Path
ROOT = Path(__file__).resolve().parent
PID = "DEE-ADM-ATTEMPT12-LOADER-ERROR-1114-BOUNDARY-V1"
SM = "edb59767188568dc798a130057f70c7a1b255d6283c7694fcd5f837f72e1bea7"
SS = "0267e5719a5661322473da189872d39325ead424e75ec8c595c996367bb17168"
def require(ok, label):
    if not ok: raise RuntimeError(f"FAIL {PID}: {label}")
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
m_path=ROOT/"MANIFEST.json"; m=json.loads(m_path.read_text(encoding="utf-8")); s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8")); e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
require(m.get("package_id")==PID, "package id")
require(m.get("source_manifest_sha256")==SM and m.get("source_package_seal_sha256")==SS, "source hashes")
require(s.get("manifest_sha256")==sha(m_path) and s.get("status")=="SEALED_FOR_CHECK", "seal")
expected={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"}
require({x.get("name") for x in m.get("payloads",[])}==expected, "inventory")
for x in m["payloads"]:
    p=ROOT/x["name"]; require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"], "payload " + x["name"])
l=e["observed_loader_error"]
require(l["error_code_values"]==[1114] and l["wrapper_group_counts"]=={"wrapper_a":40,"wrapper_b":4}, "observed values")
require(l["exact_failing_initializer"]=="UNBOUND" and e["run_result"]["u_k_b_d_observations"]==0, "scope boundary")
joined=json.dumps(e,ensure_ascii=True)+(ROOT/"REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\",joined) is None and re.search(r"0x[0-9a-fA-F]{6,}",joined) is None, "path/address leak")
require(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False, "content flags")
print(f"PASS package={PID} observed_error=1114 failing_initializer=UNBOUND")
