from __future__ import annotations
import hashlib, json
from pathlib import Path, PurePosixPath
ROOT=Path(__file__).resolve().parent
PACKAGE_ID="FLAT7X-MAIN-ARCHITECTURE-REUSE-MATRIX-V1"
EXPECTED={"FINDINGS.md","REUSE-MATRIX.json","SOURCE-RECEIPTS.json","verify_package.py"}
def req(v,m):
    if not v: raise SystemExit("PACKAGE_CHECK=FAIL "+m)
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def obj(n): return json.loads((ROOT/n).read_text(encoding="utf-8"))
def main():
    raw=(ROOT/"MANIFEST.json").read_bytes(); m=json.loads(raw); h=hashlib.sha256(raw).hexdigest()
    req(m.get("schema")=="jocforge.wash_implementer_supplement.v1" and m.get("package_id")==PACKAGE_ID,"manifest")
    seen=set()
    for e in m.get("files",[]):
        n=e["path"]; rel=PurePosixPath(n)
        req(n not in seen and not rel.is_absolute() and all(p not in {"",".",".."} for p in rel.parts),"path")
        f=ROOT.joinpath(*rel.parts); req(f.is_file() and f.stat().st_size==e["bytes"] and digest(f)==e["sha256"],"payload "+n); seen.add(n)
    actual={p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name not in {"MANIFEST.json","MANIFEST.sha256","PACKAGE-SEAL.json"}}
    req(seen==EXPECTED==actual,"inventory")
    seal=obj("PACKAGE-SEAL.json"); req(seal.get("package_id")==PACKAGE_ID and seal.get("manifest_sha256")==h and seal.get("payload_count")==len(seen),"seal")
    req((ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()==[h,"MANIFEST.json"],"sidecar")
    matrix=obj("REUSE-MATRIX.json"); req(matrix.get("production_allowed") is False,"production gate")
    req(matrix["target"]["i0_kbps"]==640 and matrix["target"]["d0_kbps"]==1024 and matrix["target"]["d0_chanmap"]=="0x1A00","target contract")
    req(matrix["target"]["joc_index"]==1 and matrix["target"]["joc_coefficient_channels"]==7 and matrix["target"]["joc_objects"]==15,"Flat7x dimensions")
    old=matrix["reuse_decisions"][3]; req("legacy constants are incompatible" in old["action"],"legacy mismatch")
    src=obj("SOURCE-RECEIPTS.json"); req(src["main_reference_package"]["manifest_sha256"]=="845e66208f997e14741b54f9fbc0458dd3bee0d7c97372fbc1b5c863e740647c","main receipt")
    req(src["current_isolated_snapshot"]["head_and_remote"]=="9456d4ca2c105ba2c799bf24a1545af2eb3e1d48","current head receipt")
    req(src["access"]["main_source_code_copied"] is False and src["access"]["old_audio_algorithm_included"] is False,"clean-room boundary")
    print("PACKAGE_CHECK=PASS")
    print("REUSE_SCOPE=ORDERED_ROLES_AND_OWNERSHIP_ARCHITECTURE_ONLY")
    print("LEGACY_VALUES_AND_AUDIO_PATH=NOT_REUSED")
    print("PRODUCTION_ALLOWED=NO")
if __name__=="__main__": main()
