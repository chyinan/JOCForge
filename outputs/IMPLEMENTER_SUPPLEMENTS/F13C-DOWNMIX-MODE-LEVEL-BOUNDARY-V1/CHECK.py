from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def need(ok,msg):
 if not ok:raise SystemExit("FAIL: "+msg)
m=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"))
s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8"))
scope=json.loads((ROOT/"SUPPLEMENT-SCOPE.json").read_text(encoding="utf-8"))
src=json.loads((ROOT/"SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
for x in m["payloads"]:
 p=ROOT/x["file"]
 need(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["file"])
sealed={x["file"]:x for x in s["files"]}
for x in m["payloads"]:need(x["file"] in sealed and sealed[x["file"]]["sha256"]==x["sha256"],"seal "+x["file"])
need(sha(ROOT/"MANIFEST.json")==sealed["MANIFEST.json"]["sha256"],"manifest seal")
need("path" not in scope["source_package"],"external path present")
need(src["raw_source_paths_or_codec_source_included"] is False,"source scope")
need(scope["claims"]["dee_native_effect"]=="UNBOUND","DEE effect")
need(scope["claims"]["mode_and_level_experiments"]=="NOT_RUN","vector status")
need(scope["implementation_ready"] is False,"implementation readiness")
v=json.loads((ROOT/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
need(v["status"]=="INPUT_ONLY_NOT_RUN","input vector status")
print(json.dumps({"status":"PASS","source_payloads":src["payloads_verified"],"native_effect":"UNBOUND","vectors":"NOT_RUN"}))
