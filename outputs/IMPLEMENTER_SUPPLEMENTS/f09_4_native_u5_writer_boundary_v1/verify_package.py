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
    require(m.get("package_id") == "f09_4_native_u5_writer_boundary_v1", "package id")
    require(m.get("implementation_delivery") == "PROHIBITED", "delivery boundary")
    require(e.get("native_u5_producer") == "NOT_CLOSED", "native status")
    require(e.get("static_contract", {}).get("f10_segment_grouping") == "STATIC_ONLY_NOT_U5",
            "F10 separation")
    require(e.get("approved_model_vector", {}).get("table_e1_9_code") == 5, "model vector")
    require(e.get("implementation_code_inspected") is False, "implementation boundary")
    entries = m.get("payloads")
    require(isinstance(entries, list) and
            {x.get("name") for x in entries} == {"README.md", "EVIDENCE.json", "MODEL-VECTOR.json", "verify_package.py"},
            "payload inventory")
    for item in entries:
        name = item["name"]
        require(Path(name).name == name, "flat payload")
        path = p / name
        require(path.is_file() and path.stat().st_size == item.get("bytes") and
                sha(path) == item.get("sha256"), "payload hash")
    seal = (p / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal) == 2 and seal[1] == "MANIFEST.json" and sha(mp) == seal[0], "manifest seal")
    print("PASS: F09.4 native/model boundary and hashes")
if __name__ == "__main__":
    main()
