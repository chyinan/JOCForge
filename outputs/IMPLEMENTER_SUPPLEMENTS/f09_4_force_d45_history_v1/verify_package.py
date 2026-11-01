import hashlib,json
from pathlib import Path
def require(ok,msg):
    if not ok: raise SystemExit("FAIL: "+msg)
def sha(p):
    h=hashlib.sha256()
    with p.open("rb") as f:
        for b in iter(lambda:f.read(1048576),b""): h.update(b)
    return h.hexdigest()
def main():
    p=Path(__file__).resolve().parent;mp=p/"MANIFEST.json"
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"));o=json.loads((p/"OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="f09_4_force_d45_history_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","delivery scope")
    require(e["status"]==m["status"],"status binding")
    require(e["source_state"]["initial_value"]==0 and e["source_state"]["explicit_constructor_clear"] is True,"cold zero")
    require(e["copy_path"]["copy"]=="first U16 copied unchanged","copy semantics")
    require(e["copy_path"]["block_ordinal"].startswith("low 16-bit block ordinal starts at 0"),"block order")
    require(e["consumer"]["physical_unit"]=="none established; consumer tests zero/nonzero","value unit")
    require(e["source_writer_boundary"]["later_direct_writer_in_single_pass_image"]=="NOT_FOUND","unresolved update")
    require(e["source_writer_boundary"]["indirect_or_external_writer"]=="NOT_EXCLUDED","external mutation boundary")
    require(e["source_writer_boundary"]["runtime_generation_id"]=="NOT_CAPTURED","runtime boundary")
    require(o["package_id"]==m["package_id"] and o["status"]["later_update"]=="UNBOUND","open edge status")
    require(e["exclusions"]["raw_decompilation_included"] is False and e["exclusions"]["private_key_or_license_contents_read"] is False,"exclusions")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        q=p/x["name"];require(q.is_file() and q.stat().st_size==x["bytes"] and sha(q)==x["sha256"],"payload "+x["name"])
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: cold-zero and force-D45 copy path; later source mutation remains unbound")
if __name__=="__main__": main()
