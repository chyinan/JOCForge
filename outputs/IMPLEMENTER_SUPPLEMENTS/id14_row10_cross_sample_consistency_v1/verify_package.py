from __future__ import annotations
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(ok,msg):
    if not ok: raise SystemExit(msg)
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    manifest=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id")=="id14_row10_cross_sample_consistency_v1","package id")
    evidence=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
    samples=evidence.get("samples",[])
    require(len(samples)==2 and samples[0]["input_sha256"]!=samples[1]["input_sha256"],"distinct inputs")
    require(all(x["row_count"] in (36,38) and x["row_index"]==10 for x in samples),"row scope")
    require(all(x["byte_plus3"]=="0xBA" and x["u16_plus4"]=="0x0000" and x["normalized_u16"]=="0x800A" and x["bit15"] is True for x in samples),"observed row values")
    require(evidence.get("comparison",{}).get("semantic_identity_from_value_equality")=="NOT_INFERRED","identity inference")
    require(evidence.get("latest_run",{}).get("writer_instruction_executed") is False,"writer status")
    require(evidence.get("latest_run",{}).get("bitstream")=="NOT_RUN","bitstream status")
    source=json.loads((ROOT/"SOURCE-BINDING.json").read_text(encoding="utf-8"))
    require(source.get("incoming_manifest_sha256")=="dd60231ca4e14737cde054b8f5bdc7173d7a0a3a958c49f0580433d80c59dd23","source manifest")
    require(source.get("parent_row_consumer_manifest_sha256")=="3d5df8de998d0de67405ecc36b1b31b4c9a7bddc95b4b6bd185ab04eae67a3f4","parent package")
    vectors=json.loads((ROOT/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(all(x.get("expected_output")=="NOT_SUPPLIED" for x in vectors.get("vectors",[])),"vector expectation")
    for row in manifest.get("payloads",[]):
        name=row.get("name")
        require(isinstance(name,str) and Path(name).name==name,"unsafe payload")
        p=(ROOT/name).resolve()
        require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    seal=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()
    require(seal and seal[0]==digest(ROOT/"MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT/"VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification")
    print("PASS: two-input row10 consistency and semantic UNBOUND verified")
if __name__=="__main__": main()
