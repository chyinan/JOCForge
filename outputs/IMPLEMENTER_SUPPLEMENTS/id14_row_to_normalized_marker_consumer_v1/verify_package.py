from __future__ import annotations
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(ok,msg):
    if not ok: raise SystemExit(msg)
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    mb=(ROOT / "MANIFEST.json").read_bytes(); m=json.loads(mb)
    require(m.get("package_id")=="id14_row_to_normalized_marker_consumer_v1","package id")
    require(m.get("implementation_delivery")=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","delivery scope")
    e=json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    row=e.get("one_observed_row",{})
    require(row.get("raw_byte_plus3")=="0xBA" and row.get("copied_byte_plus3")=="0xBA","row byte observation")
    require(row.get("raw_u16_plus4")=="0x0000" and row.get("copied_u16_plus4")=="0x0000","row U16 observation")
    require(row.get("normalized_u16_consumed")=="0x800A" and row.get("bit15_test_taken") is True,"normalized consumer observation")
    require(row.get("semantic_name")=="UNBOUND" and row.get("causal_claim_about_u16_plus4")=="NONE","semantic overclaim")
    require(e.get("count_context_only",{}).get("writer_instruction_executed") is False,"writer boundary")
    require(e.get("limits",{}).get("bitstream_emitted_or_parsed")=="NOT_RUN","bitstream boundary")
    vectors=json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(all(x.get("expected_output")=="NOT_SUPPLIED" for x in vectors.get("vectors",[])),"vector prediction added")
    for row in m.get("payloads",[]):
        name=row.get("name"); require(isinstance(name,str) and Path(name).name==name,"unsafe payload name")
        p=(ROOT/name).resolve(); require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row.get("bytes") and digest(p)==row.get("sha256"),"payload hash")
    side=(ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(ROOT / "MANIFEST.json"),"manifest seal")
    v=json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity") in {"PENDING_PACKAGE_CHECK","PASS_PACKAGE_INTEGRITY_ONLY"},"verification state")
    print("PASS: row copy, normalized consumer, hash binding and open semantics verified")
if __name__=="__main__": main()
