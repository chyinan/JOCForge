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
    require(m.get("package_id") == "id14_complexity_carrier_lineage_boundary_v1", "package id")
    require(m.get("implementation_delivery") == "PROHIBITED", "delivery boundary")
    require(e["model"]["selected_complexity"] == 16 and
            e["model"]["complexity_rule"] == "object_count_plus_one", "model complexity")
    require(e["native"]["id14_body_storage"] == "OPAQUE_BYTES" and
            e["native"]["complexity_to_id14_same_generation"] == "UNBOUND", "native complexity boundary")
    require(e["carrier"]["selected_container_compatibility"]["selected_length_compatible_with_block_2"] is True and
            e["carrier"]["typed_carrier_block_transfer"] == "UNBOUND", "carrier boundary")
    require(e["unresolved"]["native_id14_parser_global_absence_claimed"] is False,
            "parser scope")
    entries = m.get("payloads")
    require(isinstance(entries, list) and
            {x.get("name") for x in entries} == {"README.md", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "verify_package.py"},
            "payload inventory")
    for item in entries:
        name = item["name"]
        require(Path(name).name == name, "flat payload")
        path = p / name
        require(path.is_file() and path.stat().st_size == item.get("bytes") and
                sha(path) == item.get("sha256"), "payload hash")
    seal = (p / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal) == 2 and seal[1] == "MANIFEST.json" and sha(mp) == seal[0], "manifest seal")
    print("PASS: ID14 model/native boundary and hashes")
if __name__ == "__main__":
    main()
