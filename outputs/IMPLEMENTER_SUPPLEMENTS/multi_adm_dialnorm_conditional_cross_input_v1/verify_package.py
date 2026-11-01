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
    require(m["package_id"]=="multi_adm_dialnorm_conditional_cross_input_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status")
    require(len(e["attempts"])==2 and len({x["sha256"] for x in e["attempts"]})==2,"two distinct inputs")
    for x in e["attempts"]:
        require(x["source_hash_rechecked"]=="MATCH" and x["ascii_alias_hash"]==x["sha256"],"input identity")
        require(x["module_sha256"]=="3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2","target module")
        require(x["prefix_bytes"]==7 and x["payload_bytes"]==0,"bounded parse")
    require(e["parser"]["source_rehashed_by_wash"] is True and e["parser"]["binary_rehashed_by_wash"] is True,"parser provenance")
    require(e["parser"]["crate_versions"]=={"openjoc_eac3":"0.20.1","openjoc_bitio":"0.20.1"},"parser crate versions")
    require(e["parser"]["official_cli_used"] is False,"official CLI distinction")
    require(e["formula"]["comparisons"][0]["predicted_u5"]==21 and e["formula"]["comparisons"][0]["observed_u5"]==21,"sample A comparison")
    require(e["formula"]["comparisons"][1]["predicted_u5"]==14 and e["formula"]["comparisons"][1]["observed_u5"]==14,"sample B comparison")
    require(e["formula"]["assumption"]=="native measured scalar is selected_lufs","conditional assumption")
    require(e["open_edges"]["native_measurement_unit"]=="NOT_STATED_IN_NATIVE_LOG" and e["open_edges"]["general_native_rounding_rule"]=="NOT_CLOSED","formula boundary")
    require(e["open_edges"]["F06_measurement_to_BSI_owner_generation"]=="NOT_CLOSED","owner join boundary")
    require(e["inherited_failed_capture"]["prefix_bytes_read"]==0 and e["inherited_failed_capture"]["u5_observation"] is False,"failed capture not counted")
    require(e["cleanup"]["license_or_authorization_contents_read"] is False,"secret boundary")
    require(v["classification"]=="INPUT_ONLY_CROSS_INPUT_OBSERVATIONS_NO_MEDIA_BYTES" and o["package_id"]==m["package_id"],"evidence scope")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","INPUT-ONLY-VECTORS.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        p=root/x["name"];require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["name"])
    seal=(root/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: two distinct input matches under explicit selected_lufs assumption; native general rule remains open")
if __name__=="__main__":main()
