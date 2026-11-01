import hashlib,json
from pathlib import Path
def require(ok,msg):
    if not ok: raise SystemExit("FAIL: "+msg)
def sha(path):
    h=hashlib.sha256()
    with path.open("rb") as f:
        for b in iter(lambda:f.read(1048576),b""): h.update(b)
    return h.hexdigest()
def main():
    root=Path(__file__).resolve().parent;mp=root/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"))
    e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"))
    o=json.loads((root/"OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="pcm24_ddp_receiver_run149_owner_boundary_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status binding")
    ptr=e["run149_pointer_relations"]
    require(ptr["host_runinput_param2_equals_ddp_source_receiver"] is False,"object identity distinction")
    require(ptr["ddp_row_table_equals_descriptor_offset0"] is True,"descriptor equality")
    require(ptr["raw_pointer_tokens_copied"] is False,"pointer token boundary")
    require(e["static_ddp_receiver"]["typed_vtable"]=="atmos_storage_composite_master::vftable","DDP receiver type")
    require(e["sources"]["converter"]["hash_relation"].startswith("MISMATCH"),"converter version boundary")
    require(e["open_edges"]["actual_converter_module_hash_for_run149"]=="UNBOUND","run module hash")
    require(e["open_edges"]["selected_adm_master_to_ddp_internal_master"]=="UNBOUND/NO","selected reader boundary")
    require(e["run149_output"]["bitstream_read_or_parsed"] is False and e["run149_output"]["playback"]=="NOT_RUN","output scope")
    require(e["exclusions"]["private_key_or_license_contents_read"] is False,"secret boundary")
    require(o["package_id"]==m["package_id"],"open-edge package id")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        q=root/x["name"]
        require(q.is_file() and q.stat().st_size==x["bytes"] and sha(q)==x["sha256"],"payload "+x["name"])
    seal=(root/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: DDP internal receiver/descriptor boundary and selected-reader gaps")
if __name__=="__main__":main()
