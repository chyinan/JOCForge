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
    require(m.get("package_id") == "dee_ddp_plugin_factory_measurement_boundary_v1", "package id")
    require(m.get("implementation_delivery") == "PROHIBITED", "delivery boundary")
    require(e.get("source_package", {}).get("source_hash_count") == 27 and
            e.get("source_package", {}).get("source_hash_mismatches") == 0, "source inventory")
    require(e.get("ddp_plugin_factory_chain") == "STATICALLY_CONFIRMED", "factory chain")
    require(e.get("conditional_measurement_path") == "STATICALLY_CONFIRMED_BRANCH_NOT_OBSERVED",
            "measurement branch")
    require(e.get("host_callback_join") == "UNBOUND/NO" and
            e.get("descriptor_offset0_owner_generation") == "UNBOUND/NO", "open edges")
    entries = m.get("payloads")
    require(isinstance(entries, list) and
            {x.get("name") for x in entries} == {"README.md", "EVIDENCE.json", "OPEN-EDGES.json", "verify_package.py"},
            "payload inventory")
    for item in entries:
        name = item["name"]
        require(Path(name).name == name, "flat payload")
        path = p / name
        require(path.is_file() and path.stat().st_size == item.get("bytes") and
                sha(path) == item.get("sha256"), "payload hash")
    seal = (p / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal) == 2 and seal[1] == "MANIFEST.json" and sha(mp) == seal[0], "manifest seal")
    print("PASS: DDP plugin factory/measurement scope and hashes")
if __name__ == "__main__":
    main()
