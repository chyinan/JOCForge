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
    evidence = read_json("EVIDENCE.json")
    require(evidence["selected_clean_rule"]["writes_drc_control_fifo"] is False, "selected FIFO rule mismatch")
    require(evidence["native_selected_process_boundary"]["category5_or_InputControlEvent_parameter"] is False, "process boundary mismatch")
    require(evidence["unbound_edges"], "generic event edges were incorrectly closed")
    print(f"PASS_F06ND_CATEGORY5_SELECTED_STATUS files={len(entries)} generic_edges=UNBOUND")


if __name__ == "__main__":
    main()
