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
    vectors = read_json("INPUT-ONLY-VECTORS.json")
    for vector in vectors["vectors"]:
        source = vector["source_u8_at_offset_36"]
        expected_bytes = [1 if source == 0 else 0, 0, 0, 0]
        require(vector["row_bytes_0_to_3_after_writer"] == expected_bytes, f"DWORD result mismatch: {vector['id']}")
        value = sum(byte << (8 * index) for index, byte in enumerate(expected_bytes))
        require(value == vector["derived_full_u32"], f"LE value mismatch: {vector['id']}")
        require(vector["bytes_1_to_3_zero"] is True, f"padding status mismatch: {vector['id']}")
    status = read_json("EVIDENCE.json")
    require(status["claims"]["row_to_formatter_to_selected_gate_owner_generation"] == "UNBOUND", "gate join overstated")
    require(status["claims"]["selector_zero_selected_callback_reachability"] == "NOT_OBSERVED", "branch reachability overstated")
    print(f"PASS_CONTROL52_PADDING_WRITE_SET files={len(entries)} vectors={len(vectors['vectors'])} gate_join=UNBOUND")


if __name__ == "__main__":
    main()
