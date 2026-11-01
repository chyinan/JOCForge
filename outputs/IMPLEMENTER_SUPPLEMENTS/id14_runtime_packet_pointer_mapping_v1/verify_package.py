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
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((root/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"));o=json.loads((root/"OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="id14_runtime_packet_pointer_mapping_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status")
    require(e["correction"]["outer_param3_slot2"]=="1536 frame size","outer descriptor correction")
    require(e["correction"]["outer_param3_slot5"]=="inner metadata packet pointer","inner packet pointer")
    require(e["run"]["profile_data_rate_kbps"]==1024 and e["selector"]["runtime_selector_enum"]==5,"same-job rate/selector")
    require((e["runtime_packet"]["inner_packet_record_count_u16"],e["runtime_packet"]["code13"],e["runtime_packet"]["code14"],e["runtime_packet"]["count_difference"])==(38,16,1,15),"runtime count values")
    require(e["runtime_packet"]["id14_u6_value_if_writer_executes"]==14 and e["runtime_packet"]["writer_instruction_executed"] is False,"pending U6 boundary")
    require(e["open_edges"]["host_task_config_object_to_specific_priv_owner"]=="UNBOUND","owner join")
    require(e["open_edges"]["six_byte_row_semantic_names"]=="UNBOUND","row semantics")
    require(e["open_edges"]["actual_u6_emission"]=="NOT_RUN","writer scope")
    require(e["exclusions"]["private_key_or_license_contents_read"] is False and e["exclusions"]["media_pcm_payload_or_bitstream_read"] is False,"secret/media boundary")
    require(v["payload_or_bitstream_values"] is False and o["status"]["writer"]=="NOT_RUN","input-only scope")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","INPUT-ONLY-VECTORS.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        p=root/x["name"];require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["name"])
    seal=(root/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: corrected outer/inner packet path and same-job count values; U6 emission remains NOT_RUN")
if __name__=="__main__":main()
