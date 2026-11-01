from __future__ import annotations
import hashlib, json
from pathlib import Path
def require(ok, message):
    if not ok:
        raise SystemExit("FAIL: " + message)
def sha(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1048576), b""):
            h.update(block)
    return h.hexdigest()
def main():
    p = Path(__file__).resolve().parent
    mp = p / "MANIFEST.json"
    m = json.loads(mp.read_text(encoding="utf-8"))
    e = json.loads((p / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(m.get("package_id") == "payload11_group_presence_branch_candidates_v1", "package id")
    require(m.get("implementation_delivery") == "PROHIBITED", "delivery boundary")
    require(e.get("run145", {}).get("producer_join") == "UNBOUND", "run join")
    require(e.get("typed_type11_path", {}).get("group_id_present_default") == 0, "typed default")
    require(e.get("alternate_same_layout_path", {}).get("presence_one_condition") ==
            {"mode": 0, "selector": 0}, "alternate branch")
    require(e.get("alternate_same_layout_path", {}).get("type11_association") == "UNBOUND",
            "alternate type association")
    entries = m.get("payloads")
    require(isinstance(entries, list) and
            {x.get("name") for x in entries} == {"README.md", "EVIDENCE.json", "verify_package.py"},
            "payload inventory")
    for item in entries:
        path = p / item["name"]
        require(Path(item["name"]).name == item["name"], "flat payload")
        require(path.is_file() and path.stat().st_size == item.get("bytes") and
                sha(path) == item.get("sha256"), "payload hash")
    seal = (p / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal) == 2 and seal[1] == "MANIFEST.json" and sha(mp) == seal[0],
            "manifest seal")
    print("PASS: P11 branch candidate scope and hashes")
if __name__ == "__main__":
    main()
