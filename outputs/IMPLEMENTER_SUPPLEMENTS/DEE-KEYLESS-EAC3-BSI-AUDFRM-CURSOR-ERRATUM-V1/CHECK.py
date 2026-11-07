from __future__ import annotations
import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent
PID="DEE-KEYLESS-EAC3-BSI-AUDFRM-CURSOR-ERRATUM-V1"
SRC="f53822974ccb9482b01caf34b207c03404486eebc25e16fc6c208fc561866572"
def req(v,n):
    if not v: raise SystemExit("FAIL "+n)
def h(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    m=json.loads((R/"MANIFEST.json").read_text(encoding="utf-8")); s=json.loads((R/"package_seal.json").read_text(encoding="utf-8")); e=json.loads((R/"EVIDENCE.json").read_text(encoding="utf-8"))
    req(m["package_id"]==PID and m["source_manifest_sha256"]==SRC,"identity")
    req(s["manifest_sha256"]==h(R/"MANIFEST.json"),"seal")
    req({x["name"] for x in m["payloads"]}=={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json"},"inventory")
    for x in m["payloads"]:
        p=R/x["name"]; req(p.is_file() and p.stat().st_size==x["bytes"] and h(p)==x["sha256"],"payload:"+x["name"])
    o=e["observed"]; req(o["bsi_end_bit_both"]==113 and o["candidate_expstre"]==0 and o["historical_rust_expstre"]==1,"expstre")
    req(o["frmfgaincode_bit"]==121 and o["candidate_frmfgaincode"]==o["historical_rust_frmfgaincode"]==1,"gain")
    req(e["boundaries"]["user_reported_failure_file_identity"]=="UNBOUND" and e["boundaries"]["audition_failure_root_cause"]=="UNBOUND","failure boundary")
    req(e["boundaries"]["audblk_fast_gain_and_converter_snr_cursor"]=="UNBOUND_NOT_PARSED","audblk boundary")
    req(e["access_exclusions"]["raw_ec3_pcm_key_license_or_native_source_included"] is False,"scope")
    print("PASS: cursor correction; historical comparison only; failure root unbound")
if __name__=="__main__": main()
