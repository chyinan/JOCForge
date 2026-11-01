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
    p=Path(__file__).resolve().parent;mp=p/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"));o=json.loads((p/"OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="dee_pcm_to_ddp_provider_inventory_scope_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","delivery scope")
    require(e["status"]==m["status"],"status binding")
    require(e["source_inventory"]["dll_count"]==6 and e["source_inventory"]["exact_pcm_to_ddp_provider_count"]==1,"inventory scope")
    matches=[(mod,n) for mod in e["modules"] for n in mod["local_stage_names"] if n=="pcm_to_ddp"]
    require(len(matches)==1 and matches[0][0]["name"]=="dee_audio_filter_ddp.dll","exact provider uniqueness")
    require(e["selected_provider"]["local_index"]==0 and e["selected_provider"]["callback_rva"]=="0x1d1540","selected local slot")
    require(e["host_loader"]["same_manager_dispatch"].startswith("StageRunner"),"host dispatch scope")
    require(e["open_edges"]["run_loaded_module_set"]=="NOT_RUN/UNBOUND","runtime boundary")
    require(e["open_edges"]["absolute_host_registry_ordinal"]=="ORDER_DEPENDENT/NOT_BOUND","ordinal boundary")
    require(e["exclusions"]["raw_decompilation_included"] is False and e["exclusions"]["private_key_or_license_contents_read"] is False,"exclusions")
    require(o["package_id"]==m["package_id"],"open edge package id")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        q=p/x["name"];require(q.is_file() and q.stat().st_size==x["bytes"] and sha(q)==x["sha256"],"payload "+x["name"])
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: unique exact provider in bounded six-DLL inventory; runtime load set and ordinal remain open")
if __name__=="__main__":main()
