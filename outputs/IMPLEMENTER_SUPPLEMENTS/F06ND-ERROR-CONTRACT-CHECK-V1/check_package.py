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
    errors = read_json("ERROR-MAP.json")
    require("first-error precedence for simultaneous-invalid gates" in errors["unbound_outcomes"], "precedence overstated")
    require(errors["declared_outcomes"]["intermediate_nonfinite_or_overflow"] == "NUMERIC_FAILURE", "numeric outcome mismatch")
    require(read_json("VECTOR-COVERAGE.json")["invalid_case_vectors_for_unbound_edges"] == 0, "unbound vectors overstated")
    print(f"PASS_F06ND_ERROR_CONTRACT files={len(entries)} unbound_edges={len(errors['unbound_outcomes'])}")


if __name__ == "__main__":
    main()
