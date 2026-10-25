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
    expected = {entry["path"] for entry in entries} | {"MANIFEST.json", "DELIVERY-ENVELOPE.json"}
    actual = {path.name for path in ROOT.iterdir() if path.is_file()}
    require(actual == expected, "package file set mismatch")
    for entry in entries:
        path = ROOT / entry["path"]
        require(path.stat().st_size == entry["bytes"], f"size mismatch: {entry['path']}")
        require(hashlib.sha256(path.read_bytes()).hexdigest() == entry["sha256"], f"hash mismatch: {entry['path']}")
    envelope = read_json("DELIVERY-ENVELOPE.json")
    require(envelope["manifest_sha256"] == hashlib.sha256(manifest_bytes).hexdigest(), "manifest seal mismatch")
    chain = read_json("OWNER-CHAIN.json")
    require(chain["status"].startswith("WRITER_STATE_OWNER_INITIALIZER_STATIC_CLOSED"), "owner status mismatch")
    require(chain["static_owner_edges"][2]["status"] == "UNBOUND", "allocator extent overstated")
    require(chain["static_owner_edges"][3]["status"] == "UNBOUND_NOT_ASSUMED_SAME", "IDA/Ghidra identity overstated")
    print(f"PASS_F13C_WRITER_PHYSICAL_EXTENT_V2 files={len(entries)} capacity=UNBOUND")


if __name__ == "__main__":
    main()
