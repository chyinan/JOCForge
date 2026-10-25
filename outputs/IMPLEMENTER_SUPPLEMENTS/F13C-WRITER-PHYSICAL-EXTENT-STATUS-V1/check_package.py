from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def read_json(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> None:
    manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
    manifest = json.loads(manifest_bytes.decode("utf-8"))
    entries = manifest["files"]
    expected = {item["path"] for item in entries} | {"MANIFEST.json", "DELIVERY-ENVELOPE.json"}
    actual = {p.name for p in ROOT.iterdir() if p.is_file()}
    require(actual == expected, "package file set mismatch")
    for item in entries:
        path = ROOT / item["path"]
        require(path.stat().st_size == item["bytes"], f"size mismatch: {item['path']}")
        require(hashlib.sha256(path.read_bytes()).hexdigest() == item["sha256"], f"hash mismatch: {item['path']}")
    envelope = read_json("DELIVERY-ENVELOPE.json")
    require(envelope["manifest_sha256"] == hashlib.sha256(manifest_bytes).hexdigest(), "manifest seal mismatch")
    evidence = read_json("EVIDENCE.json")
    require(evidence["identity_edges"]["prior_native_pointer_expression_to_Ghidra_writer_argument"] == "UNBOUND", "pointer join overstated")
    require(evidence["identity_edges"]["allocator_to_physical_byte_capacity"] == "UNBOUND", "physical capacity overstated")
    print(f"PASS_F13C_WRITER_PHYSICAL_EXTENT files={len(entries)} capacity=UNBOUND")


if __name__ == "__main__":
    main()
