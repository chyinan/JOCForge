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
    require(m["package_id"]=="f09_4_force_d45_cross_image_scan_v1","package id")
    require(m["implementation_delivery"]=="ALLOWED_WITHIN_DECLARED_SCOPE_ONLY","scope")
    require(e["status"]==m["status"],"status")
    require(e["reference_attempt"]["dee_job_attempted_once"] is True and e["reference_attempt"]["breakpoints_resolved"] is False,"runtime attempt")
    require(e["reference_attempt"]["owner_fields_captured"] is False and e["reference_attempt"]["retry"]=="NOT_RUN","capture boundary")
    require(len(e["target_modules"])==7 and len({x["name"] for x in e["target_modules"]})==7,"image inventory")
    require(e["inference_boundary"]["direct_writer_in_scanned_images"]=="NOT_FOUND","direct writer scope")
    require(e["inference_boundary"]["indirect_writer"]=="NOT_EXCLUDED" and e["inference_boundary"]["outside_scanned_images"]=="NOT_EXCLUDED","external writer boundary")
    require(e["exclusions"]["private_key_or_license_contents_read"] is False and e["exclusions"]["media_pcm_payload_or_bitstream_read"] is False,"data boundary")
    require(o["package_id"]==m["package_id"] and o["status"]["later_source_writer"]=="UNBOUND","open edges")
    require({x["name"] for x in m["payloads"]}=={"README.md","EVIDENCE.json","OPEN-EDGES.json","SOURCE-BINDING.json","verify_package.py"},"payload set")
    for x in m["payloads"]:
        p=root/x["name"];require(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["name"])
    seal=(root/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: seven-image direct-write scan; later nonzero reachability remains UNBOUND")
if __name__=="__main__":main()
