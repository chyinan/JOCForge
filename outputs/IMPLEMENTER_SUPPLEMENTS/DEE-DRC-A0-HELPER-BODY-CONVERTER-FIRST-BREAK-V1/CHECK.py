from __future__ import annotations
import hashlib, json, re
from pathlib import Path
ROOT = Path(__file__).resolve().parent
PID = 'DEE-DRC-A0-HELPER-BODY-CONVERTER-FIRST-BREAK-V1'
SOURCE_MANIFEST = 'dac89b72631ffa611946b16be457658990b4c4e7b17a17ff2d41983247d12438'
SOURCE_SEAL = 'c2bce0a8f0fc424b4183fd8117476d9272825aee18180a0b8d5febb17d47dd43'
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
