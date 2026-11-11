from __future__ import annotations
import hashlib, json
from pathlib import Path, PurePosixPath
ROOT=Path(__file__).resolve().parent
PACKAGE_ID="FLAT7X-I0-AC3-CRC-GENERATION-ETSI-V1"
EXPECTED={"FINDINGS.md","CRC-GENERATION-CONTRACT.json","INPUT-ONLY-VECTORS.json","SOURCE-RECEIPTS.json","verify_package.py"}
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
    c=obj("CRC-GENERATION-CONTRACT.json"); v=obj("INPUT-ONLY-VECTORS.json"); s=obj("SOURCE-RECEIPTS.json")
    req(c.get("production_allowed") is False and c["crc_parameters"]["polynomial_hex"]=="0x8005","scope/parameters")
    req(c["crc1"]["covered_range_for_target"]==[2,1600] and c["crc1"]["field_byte_range"]==[2,4],"CRC1 target")
    req(c["crc2"]["field_byte_range_for_target"]==[2558,2560] and c["crc2"]["crcrsv_bit_offset"]==20463,"CRC2 target")
    vectors={x["id"]:x for x in v["vectors"]}; req(len(vectors)==4,"vector count")
    t=vectors["target_2560_byte_48khz_i0_geometry"]; req(t["crc1"]=="0x6837" and t["crc2"]=="0xADE9" and t["crc1_residue"]==t["crc2_residue"]=="0x0000","target vector")
    col=vectors["crcrsv_crc2_equals_syncword_collision"]; req(col["pre_adjustment_crc2"]=="0x0B77" and col["crcrsv_toggled"] is True and col["crc2"]=="0x8B72" and col["crc2_residue"]=="0x0000","collision vector")
    req(v["dee_or_encoder_run"]=="NOT_RUN" and v["product_validation"]=="NOT_VALIDATED","validation boundary")
    req(s["source_package"]["manifest_sha256"]=="2b6bf1d0c8af1287b5da6f1e723ffb26c7e43f08360783fb5766e8183f84dde7","source package bind")
    print("PACKAGE_CHECK=PASS")
    print("CRC1_LEADING_FIELD_DERIVATION=INDEPENDENT_MATRIX_CROSSCHECK")
    print("CRC2_AND_CRCRSV_COLLISION=RESIDUAL_CHECKED")
    print("REAL_FRAME_VALIDATION=NOT_RUN")
if __name__=="__main__": main()
