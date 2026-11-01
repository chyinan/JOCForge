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
    require(m["package_id"]=="pcm24_ddp_run150_stream_object_query_boundary_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","delivery scope")
    require(e["status"]==m["status"],"status")
    require(e["run150"]["host_stream_object_callback_returned_true"] is False and e["run150"]["stream_object_output_pointer_null"] is True,"null stream object")
    require(e["run150"]["host_runinput_param2_token_equals_ddp_source_receiver"] is False,"host/DDP object identity")
    require(e["run150"]["ddp_row_table_token_equals_descriptor_offset0"] is True,"descriptor offset 0")
    require(e["run149_converter_identity"]["mapped_image_sha256"]=="UNBOUND/NO","mapped image hash")
    require(e["converter_disk_versions"]["current_e_installed_sha256"]!=e["converter_disk_versions"]["r4_declared_disk_sha256"],"converter hash discrepancy")
    require(e["open_edges"]["selected_adm_master_to_ddp_internal_receiver"]=="UNBOUND/NO","selected input boundary")
    require(e["output"]["bitstream_parse"]=="NOT_RUN" and e["output"]["playback"]=="NOT_RUN","output verification boundary")
    require(e["exclusions"]["private_key_or_license_contents_read"] is False and e["exclusions"]["media_pcm_payload_or_bitstream_read"] is False,"secret/media boundary")
    require(o["package_id"]==m["package_id"],"open edge package id")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        p=root/x["name"];require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["name"])
    seal=(root/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: Run150 stream query and DDP descriptor; selected input and converter image remain unbound")
if __name__=="__main__":main()
