from __future__ import annotations
import hashlib, json, re
from pathlib import Path
ROOT = Path(__file__).resolve().parent
PID = 'DEE-CARRIER-R141-R1R4-THIRD-U32-COMPATIBILITY-BOUNDARY-V1'
SOURCE_MANIFEST = '1b4db8524ac9f9e1c6dd1992218e42f25efb6382c2c8950d79d08347fa8e782c'
SOURCE_SEAL = '67fd7d40c612be04b764847427ad7dd84f283d586de773c6167b02d87780a79a'
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
