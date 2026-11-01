from __future__ import annotations
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
MAP={0:1,1:2,2:3,3:4,4:11,5:12,6:5,7:6,8:7,9:8,12:9,13:10}
def require(ok,msg):
    if not ok: raise SystemExit(msg)
def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def evaluate(fields):
    count=fields["row_count"]
    if count<1 or count>128: return "RETURN_5",None
    rows=fields["rows"]
    if len(rows)!=count: return "INVALID_VECTOR",None
    out=[]; run_count=1; last_u16=None
    for index,row in enumerate(rows):
        tag=row["byte_plus2_signed"]
        if tag==-1:
            out.append(0x8000|(index&0xff))
            continue
        if tag not in MAP: return "RETURN_7",None
        if last_u16 is not None and row["u16_plus4"]!=last_u16: run_count+=1
        out.append(((MAP[tag]&0x7f)<<8)|(run_count&0xff))
        last_u16=row["u16_plus4"]
    prior=fields.get("prior_state_count")
    if fields.get("param1_state_is_null") is False and prior is not None and prior>count:
        for index in range(count,prior): out.append(0x8000|(index&0xff))
    return "RETURN_0",[f"0x{value:04x}" for value in out]
def main():
    mp=ROOT/"MANIFEST.json"; m=json.loads(mp.read_text(encoding="utf-8"))
    require(m.get("package_id")=="id14_row_normalizer_static_formula_v1","package id")
    require(m.get("implementation_delivery")=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","delivery")
    src=json.loads((ROOT/"SOURCE-BINDING.json").read_text(encoding="utf-8"))
    require(src.get("incoming_manifest_sha256")=="4f4df835bb762faaf1c4d46361dc32c06c58a87149e64a0f4b56105a70fc1159","source manifest")
    vectors=json.loads((ROOT/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(len(vectors.get("vectors",[]))==29,"vector count")
    require(vectors.get("vector_execution")=="NOT_RUN","vector execution")
    for case in vectors["vectors"]:
        status,out=evaluate(case["input_fields"])
        require(status==case["expected_status"],"status "+case["id"])
        expected=case["expected_output"]
        if expected!="NOT_SUPPLIED": require(out==expected,"formula "+case["id"])
    e=json.loads((ROOT/"EVIDENCE.json").read_text(encoding="utf-8"))
    f=e["ghidra_formula"]
    require(f.get("classifier_input_offset")==2 and f.get("classifier_input_signedness")=="SIGNED_INT8","tag field")
    require(f.get("byte_plus3_read_by_normalizer") is False,"row+3")
    require(e.get("sample_row10",{}).get("byte_plus2")=="NOT_CAPTURED","sample tag")
    require(e.get("sample_row10",{}).get("semantic_field")=="UNBOUND","semantic")
    for row in m.get("payloads",[]):
        name=row["name"]; p=(ROOT/name).resolve()
        require(p.parent==ROOT.resolve() and p.is_file(),"payload path")
        require(p.stat().st_size==row["bytes"] and digest(p)==row["sha256"],"payload "+name)
    side=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()
    require(side and side[0]==digest(mp),"manifest seal")
    v=json.loads((ROOT/"VERIFICATION.json").read_text(encoding="utf-8"))
    require(v.get("package_integrity")=="PASS_PACKAGE_INTEGRITY_ONLY","verification")
    print("PASS: 29 static-formula vectors, correction, hashes, and UNBOUND boundaries verified")
if __name__=="__main__": main()
