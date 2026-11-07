from __future__ import annotations
import hashlib, json, re
from pathlib import Path
ROOT = Path(__file__).resolve().parent
PID = 'DEE-ADM-INDIVIDUAL-ROW-SLOT-K-CAPACITY-BOUNDARY-V1'
SOURCE_MANIFEST = '29e43456134526e279963af0ee329e05fd55a4665afdbeaf9c3976d489078482'
SOURCE_SEAL = 'd5f254fdfed630ce9cae7c5d00746b94c3bbd6f5ea60c99351d21d60fd718982'
PAYLOADS = {"EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json"}
def require(ok, label):
    if not ok: raise RuntimeError(f"FAIL {PID}: {label}")
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
mp=ROOT/"MANIFEST.json"; m=json.loads(mp.read_text(encoding="utf-8")); seal=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8")); e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
require(m.get("package_id")==PID, "id")
require(m.get("source_manifest_sha256")==SOURCE_MANIFEST, "source manifest")
require(m.get("source_package_seal_sha256")==SOURCE_SEAL, "source seal")
require(seal.get("manifest_sha256")==sha(mp), "manifest digest")
require({x.get("name") for x in m.get("payloads",[])}==PAYLOADS|{"CHECK.py"}, "inventory")
for x in m["payloads"]:
    p=ROOT/x["name"]; require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"], "payload "+x["name"])
joined=json.dumps(e,ensure_ascii=True)+(ROOT/"REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\",joined) is None, "path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}",joined) is None, "raw address leak")
require(m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False and m.get("contains_native_source_or_decompilation") is False, "content flags")
