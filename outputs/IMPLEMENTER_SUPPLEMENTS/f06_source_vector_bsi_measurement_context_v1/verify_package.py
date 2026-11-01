import hashlib,json
from pathlib import Path
def require(ok,msg):
    if not ok: raise SystemExit("FAIL: "+msg)
def sha(p):
    h=hashlib.sha256()
    with p.open("rb") as f:
        for b in iter(lambda:f.read(1048576),b""):h.update(b)
    return h.hexdigest()
def main():
    root=Path(__file__).resolve().parent;mp=root/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"));o=json.loads((root/"OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="f06_source_vector_bsi_measurement_context_v1","id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status")
    require(e["source_vector"]["header_bytes"]==24 and e["source_vector"]["element_stride_bytes"]==8,"vector shape")
    require(e["record_copy"]["element_stride_bytes"]==8 and e["record_copy"]["separate_row_vector"].startswith("record +0x00"),"copy/row distinction")
    require(e["measurement_context"]["bsi_context_owner_equals_serializer_owner"] is True and e["measurement_context"]["bsi_owner_equals_source_record"] is False,"owner relation")
    require(e["measurement_context"]["process_bound_module_identity_receipt"] is False,"module receipt")
    require(e["bsi_owner"]["first_writer"]=="NOT_LOCATED" and e["bsi_owner"]["measurement_update"]=="NOT_BOUND","numeric owner edge")
    require(e["exclusions"]["private_key_license_contents"] is False and e["exclusions"]["media_pcm_payload_bitstream"] is False,"exclusions")
    require(o["package_id"]==m["package_id"] and o["status"]["numeric_lineage"]=="UNBOUND","open edge")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        p=root/x["name"];require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["name"])
    seal=(root/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: qword vector and measurement call context; numeric BSI lineage remains open")
if __name__=="__main__":main()
