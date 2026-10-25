from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


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
    envelope = json.loads((ROOT / "DELIVERY-ENVELOPE.json").read_text(encoding="utf-8"))
    require(envelope["manifest_sha256"] == hashlib.sha256(manifest_bytes).hexdigest(), "manifest seal mismatch")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["constructor_region"]["sha256"] == "53adf8a9e03c120dc6340f3e8502ee9c6379f956a9bd17a5c2b33b3703396dfd", "constructor region mismatch")
    require(evidence["unchanged_open_boundaries"][0].endswith("NOT_OBSERVED"), "branch status overstated")
    require(evidence["unchanged_open_boundaries"][1].endswith("UNBOUND"), "gate join overstated")
    print(f"PASS_CONTROL52_CONSTRUCTOR_REGION_ERRATUM files={len(entries)} gate_join=UNBOUND")


if __name__ == "__main__":
    main()
