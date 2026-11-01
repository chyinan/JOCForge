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
    require(m.get("package_id") == "f06nd_category5_selected_process_boundary_v1", "package id")
    require(m.get("implementation_delivery") == "PROHIBITED", "delivery boundary")
    require(e.get("separate_generic_storage_surface", {}).get("selected_category5_join") == "UNBOUND",
            "generic edge")
    require(e.get("native_process_observation", {}).get("selected_process_byte_layout") == "NOT_APPLICABLE",
            "selected path layout")
    require(e.get("logical_contract", {}).get("selected_fifo_rule_claim", {}).get("writes_drc_control_fifo") is False,
            "selected FIFO result")
    require(e.get("logical_contract", {}).get("spec_source_status") ==
            "DECLARED_HASH_SOURCE_NOT_REHASHED", "spec source scope")
    entries = m.get("payloads")
    require(isinstance(entries, list) and
            {x.get("name") for x in entries} == {"README.md", "EVIDENCE.json", "DECISION-CASES.json", "verify_package.py"},
            "payload inventory")
    for item in entries:
        name = item["name"]
        require(Path(name).name == name, "flat payload")
        path = p / name
        require(path.is_file() and path.stat().st_size == item.get("bytes") and
                sha(path) == item.get("sha256"), "payload hash")
    seal = (p / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal) == 2 and seal[1] == "MANIFEST.json" and sha(mp) == seal[0], "manifest seal")
    print("PASS: F06ND selected-process scope and hashes")
if __name__ == "__main__":
    main()
