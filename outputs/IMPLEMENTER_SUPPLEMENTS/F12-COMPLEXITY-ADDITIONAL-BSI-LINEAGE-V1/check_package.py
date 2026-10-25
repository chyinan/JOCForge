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
    lineage = read_json("FIELD-LINEAGE.json")
    require(lineage["model_field_link"] == "STATIC_CLOSED", "model field link status mismatch")
    require(lineage["native_source_status"]["native_object_count_plus_one_to_bsi_owner_generation_join"] == "UNBOUND", "native identity overstated")
    vector = read_json("INPUT-ONLY-VECTORS.json")["vectors"][0]
    require(vector["derived_model_value"]["complexity_u8"] == vector["input"]["joc_object_count"] + 1, "complexity formula mismatch")
    require(vector["derived_model_value"]["wire_start"] == 105 and vector["derived_model_value"]["wire_width"] == 8, "F12 field position mismatch")
    print(f"PASS_F12_COMPLEXITY_FIELD_LINEAGE files={len(entries)} model_link=STATIC_CLOSED native_join=UNBOUND")


if __name__ == "__main__":
    main()
