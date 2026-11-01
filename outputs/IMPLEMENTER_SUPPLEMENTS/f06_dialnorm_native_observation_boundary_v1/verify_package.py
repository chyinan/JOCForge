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
    m=json.loads(mp.read_text(encoding="utf-8"));e=json.loads((p/"EVIDENCE.json").read_text(encoding="utf-8"))
    require(m["package_id"]=="f06_dialnorm_native_observation_boundary_v1","package id")
    require(m["implementation_delivery"]=="PROHIBITED","delivery boundary")
    require(e["input"]["authorized_input_sha256"]==e["input"]["task_alias_sha256"],"input identity")
    require(e["loaded_module"]["sha256"]==e["loaded_module"]["expected_sha256"] and e["loaded_module"]["same_process_pid"]==e["process"]["pid"],"module/process join")
    require(e["observation"]["dialnorm_u5"]==31 and e["observation"]["prefix_bytes_read"]==7 and e["observation"]["payload_body_bytes_read"]==0,"prefix boundary")
    require(e["formula_comparison"]["comparison"]=="LOWER_CLAMP_SAMPLE_ONLY" and e["formula_comparison"]["same_owner_generation"]=="UNBOUND","formula boundary")
    entries=m["payloads"];require({x["name"] for x in entries}=={"README.md","EVIDENCE.json","MODEL-CROSSCHECK.json","verify_package.py"},"payload set")
    for x in entries:
        f=p/x["name"];require(f.is_file() and f.stat().st_size==x["bytes"] and sha(f)==x["sha256"],"payload hash")
    seal=(p/"MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal)==2 and seal[1]=="MANIFEST.json" and sha(mp)==seal[0],"manifest seal")
    print("PASS: F06 DialNorm observation scope and hashes")
if __name__=="__main__":main()
