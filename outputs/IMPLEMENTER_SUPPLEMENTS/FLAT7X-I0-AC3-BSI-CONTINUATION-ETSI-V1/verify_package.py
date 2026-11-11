from __future__ import annotations
import hashlib, json
from pathlib import Path, PurePosixPath
ROOT=Path(__file__).resolve().parent
PACKAGE_ID="FLAT7X-I0-AC3-BSI-CONTINUATION-ETSI-V1"
EXPECTED={"FINDINGS.md","I0-AC3-BSI-CONTINUATION.json","SOURCE-RECEIPTS.json","verify_package.py"}
def req(v,m):
    if not v: raise SystemExit("PACKAGE_CHECK=FAIL "+m)
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def obj(n): return json.loads((ROOT/n).read_text(encoding="utf-8"))
def main():
    raw=(ROOT/"MANIFEST.json").read_bytes(); m=json.loads(raw); h=hashlib.sha256(raw).hexdigest()
    req(m.get("schema")=="jocforge.wash_implementer_supplement.v1" and m.get("package_id")==PACKAGE_ID,"manifest")
    seen=set()
    for e in m.get("files",[]):
        n=e["path"]; p=PurePosixPath(n); req(n not in seen and not p.is_absolute() and all(x not in {"",".",".."} for x in p.parts),"path")
        f=ROOT.joinpath(*p.parts); req(f.is_file() and f.stat().st_size==e["bytes"] and digest(f)==e["sha256"],"payload "+n); seen.add(n)
    actual={p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name not in {"MANIFEST.json","MANIFEST.sha256","PACKAGE-SEAL.json"}}
    req(seen==EXPECTED==actual,"inventory")
    seal=obj("PACKAGE-SEAL.json"); req(seal.get("package_id")==PACKAGE_ID and seal.get("manifest_sha256")==h and seal.get("payload_count")==len(seen),"seal")
    req((ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()==[h,"MANIFEST.json"],"sidecar")
    c=obj("I0-AC3-BSI-CONTINUATION.json")
    req(c.get("production_allowed") is False and c["target"]["frame_size_bytes"]==2560,"target")
    req(c["target"]["header_end_bits"]=={"without_compr":62,"with_compr":70},"header cursor")
    req(c["frame_part_order"]==["syncinfo","bsi","audblk[0]","audblk[1]","audblk[2]","audblk[3]","audblk[4]","audblk[5]","auxdata","errorcheck"],"frame order")
    req(c["bsi_tail_after_compr"][2]["target_acmod7"]=="OMITTED","acmod7 dual mono")
    req(c["crc_contract"]["crc1"]["coverage_end_bytes_for_2560B_frame"]==1600 and c["crc_contract"]["crc1"]["generated_value"]=="CALLER_OWNED_NOT_VALIDATED_BY_THIS_CONTRACT","crc1")
    req(c["crc_contract"]["crc2"]["field_start_bit"] == 20464 and c["crc_contract"]["crc2"]["field_width_bits"] == 16,"crc2")
    s=obj("SOURCE-RECEIPTS.json"); req(s["primary_standard"]["title"]=="ETSI TS 102 366 V1.4.1 (2017-09)","standard receipt")
    req(s["access"]["media_pcm_payload_key_license_access"] is False,"access boundary")
    print("PACKAGE_CHECK=PASS")
    print("I0_BSI_CONTINUATION_ORDER=BOUND_BY_ETSI")
    print("I0_AUDIO_BLOCKS_AND_CRC_VALUES=CALLER_OWNED")
    print("PRODUCTION_ALLOWED=NO")
if __name__=="__main__": main()
