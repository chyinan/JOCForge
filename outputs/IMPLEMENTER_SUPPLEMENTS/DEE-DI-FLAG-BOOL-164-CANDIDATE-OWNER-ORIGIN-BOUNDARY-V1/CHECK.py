from __future__ import annotations
import hashlib, json
from pathlib import Path

ROOT=Path(__file__).resolve().parent
PID="DEE-DI-FLAG-BOOL-164-CANDIDATE-OWNER-ORIGIN-BOUNDARY-V1"
SOURCE_ID="di-flag-bool-empire-loudness-subcontext-164-candidate-owner-origin-boundary-addendum-wash-v1"
SOURCE_MANIFEST="5d9d090826c2e686f830ee3807555ecf0469bc2a08e11669130cf87ef735d752"
SOURCE_SEAL="6c33751c17d636c1a4a1a9208141d83a75be95c2c85ea5d5bbc8ade4017e036a"
STATUS="CANDIDATE_CHILD_OWNER_BOUND; PLUS_0XDF0_POINTER_TARGET_OPAQUE; DLL_SUBCONTEXT_JOIN_UNBOUND"
PAYLOADS={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"}
def require(value,message):
    if not value: raise SystemExit("FAIL "+message)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def load(name):
    value=json.loads((ROOT/name).read_text(encoding="utf-8")); require(isinstance(value,dict),"json:"+name); return value
def main():
    m=load("MANIFEST.json"); s=load("package_seal.json"); e=load("EVIDENCE.json"); src=load("SOURCE-MANIFESTS.json"); v=load("INPUT-ONLY-VECTORS.json")
    require(m.get("package_id")==PID and m.get("source_package_id")==SOURCE_ID,"identity")
    require(m.get("source_manifest_sha256")==SOURCE_MANIFEST and m.get("source_package_seal_sha256")==SOURCE_SEAL,"source-binding")
    require(s.get("manifest_sha256")==sha(ROOT/"MANIFEST.json") and s.get("checker_status")=="PASS_NORMAL_AND_PYTHON_O","seal")
    rows=m.get("payloads",[]); require({x.get("name") for x in rows}==PAYLOADS,"inventory")
    for row in rows:
        p=ROOT/row["name"]; require(p.is_file() and p.stat().st_size==row.get("bytes") and sha(p)==row["sha256"],"payload:"+row["name"])
    require(e.get("status")==STATUS and e["owner_path"]["candidate_param_1_source"]=="*(undefined8 *)(lVar6+0xdf0)","edge")
    require(e["owner_path"]["nearest_typed_owner"]=="codec-parent child record at lVar6" and e["owner_path"]["pointer_target"]=="OPAQUE","owner")
    require(e["comparison_target"]["typed_owner_generation_join"]=="UNBOUND" and e["result_causality"]=="UNBOUND","joins")
    require(e["global_absence_claim"] is False and e["execution_scope"]["dee"]=="NOT_RUN","scope")
    require(len(src.get("parent_packages",[]))==2 and src.get("raw_trace_or_script_included") is False,"sources")
    require(v["vectors"][0]["expected_disposition"]["typed_owner_generation_join"]=="UNBOUND","vector")
    require(m.get("contains_native_source_or_decompilation") is False and m.get("contains_private_key_or_license") is False and m.get("contains_pcm_or_media") is False,"exclusions")
    print("PASS package="+PID+"; pointer_target=OPAQUE; join=UNBOUND; runtime=NOT_RUN")
if __name__=="__main__": main()
