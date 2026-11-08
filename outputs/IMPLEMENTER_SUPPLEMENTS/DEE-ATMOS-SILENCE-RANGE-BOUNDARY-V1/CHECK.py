from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def need(ok,msg):
 if not ok:raise SystemExit("FAIL: "+msg)
m=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"))
s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8"))
scope=json.loads((ROOT/"SUPPLEMENT-SCOPE.json").read_text(encoding="utf-8"))
src=json.loads((ROOT/"SOURCE-PACKAGE.json").read_text(encoding="utf-8"))
for x in m["payloads"]:
 p=ROOT/x["file"]
 need(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["file"])
sealed={x["file"]:x for x in s["files"]}
for x in m["payloads"]:need(x["file"] in sealed and sealed[x["file"]]["sha256"]==x["sha256"],"seal "+x["file"])
need(sha(ROOT/"MANIFEST.json")==sealed["MANIFEST.json"]["sha256"],"manifest seal")
need(src["manifest_sha256"]=="f197567214bb57adb0661dc9f078d200b445ccc9b7f04ab897b1f1c4e1115070","source manifest identity")
need(src["package_seal_sha256"]=="40b30304a1654aa12685adf4287814f28b6a2677fb88b5dde1488847a12134d3","source seal identity")
vec=json.loads((ROOT/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
need(vec["status"]=="PROPOSED_INPUTS_NOT_EXECUTED; HISTORICAL_ZERO_CONTROL_METADATA_REUSED","input vector status")
need("source_xml_path" not in vec.get("reused_range_pair",{}),"research path must be omitted")
need(scope["implementation_ready"] is False and scope["dee_run"]=="NOT_RUN","readiness/run status")
need("first actual sample consumer" in scope["unbound"],"open boundary")
need(scope["payload_policy"]["pcm"] is False and scope["payload_policy"]["key_license"] is False,"payload policy")
print(json.dumps({"status":"PASS","static_rules":len(scope["closed_static_rules"]),"open_boundaries":len(scope["unbound"]),"vectors":"PROPOSED_NOT_RUN"}))
