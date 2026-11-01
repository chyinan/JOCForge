import hashlib,json,math,struct
from pathlib import Path
def require(ok,msg):
    if not ok: raise SystemExit("FAIL: "+msg)
def sha(p):
    h=hashlib.sha256()
    with p.open("rb") as f:
        for b in iter(lambda:f.read(1048576),b""):h.update(b)
    return h.hexdigest()
def r32(x):
    return struct.unpack("<f",struct.pack("<f",x))[0]
def predict(x):
    t=math.trunc(r32(x-0.5))
    return -max(-31,min(-1,t))
def main():
    root=Path(__file__).resolve().parent;mp=root/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((root/"EVIDENCE.json").read_text(encoding="utf-8"));v=json.loads((root/"INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"));o=json.loads((root/"OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="multi_adm_dialnorm_three_sample_addendum_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status binding")
    require(len(e["samples"])==3 and len({x["input_sha256"] for x in e["samples"]})==3,"three distinct samples")
    for x in e["samples"]:
        require(x["current_authorized_file_hash"]=="MATCH" and x["ascii_alias_same_bytes"] is True,"input identity")
        require(x["target_module_sha256"]=="3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2","target module")
        require(x["prefix_bytes_read"]==7 and x["payload_bytes_read"]==0 and x["raw_prefix_retained"] is False,"prefix boundary")
        require(x["measurement_unit"]=="NOT_STATED_IN_NATIVE_LOG","measurement unit")
        require(predict(x["measurement_scalar"])==x["first_au_u5"],"F12 conditional calculation")
    require(e["formula"]["assumption"]=="native measured scalar is selected_lufs","conditional assumption")
    parser=e["parser_identity"]
    require(parser["custom_helper_source_rehashed"]=="MATCH" and parser["custom_helper_binary_rehashed"]=="MATCH","helper identity")
    require(parser["openjoc_eac3_version"]=="0.20.1" and parser["openjoc_bitio_version"]=="0.20.1","helper crate versions")
    cli=e["official_cli"]
    require(cli["binary_sha256"]=="2e84afcfa5e197332a84f9386ffa4c69f0c12e8ca5cf95c23690af320cfa1a39" and cli["invoked"] is False,"official CLI boundary")
    require(e["open_edges"]["general_native_rounding_rule"]=="NOT_CLOSED" and e["open_edges"]["measurement_to_BSI_owner_generation_join"]=="NOT_CLOSED","open native rules")
    require(e["inherited_failed_capture"]["prefix_bytes_read"]==0 and e["inherited_failed_capture"]["u5_observation"] is False,"failed capture not counted")
    require(v["audio_payload_values"] is False and o["package_id"]==m["package_id"],"input-only scope")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","INPUT-ONLY-VECTORS.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        p=root/x["name"];require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["name"])
    seal=(root/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: three distinct input matches under explicit selected_lufs assumption; native unit and owner join remain open")
if __name__=="__main__":main()
