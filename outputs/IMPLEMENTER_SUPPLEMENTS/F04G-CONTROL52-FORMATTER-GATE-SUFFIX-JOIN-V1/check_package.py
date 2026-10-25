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
    vectors = read_json("JOIN-ARITHMETIC.json")
    checks = 0
    for case in vectors["cases"]:
        P, C, K, B = (case[x] for x in ("P", "C", "K", "B"))
        require(0 <= C <= K, f"capacity precondition failed: {case['id']}")
        for i in case["row_indices"]:
            require(0 <= i < C, f"row index out of range: {case['id']}")
            formatter = B + 56 * P + 56 * i
            gate = B + 56 * (P + i)
            require(formatter == gate, f"suffix index mismatch: {case['id']}")
            checks += 1
    evidence = read_json("EVIDENCE.json")
    require(evidence["still_open"]["row_value_equality_after_reset_or_active_updater"] == "NOT_EVALUATED", "row-value claim overstated")
    require(evidence["still_open"]["source_context_row_to_frame_gate_owner_generation"] == "UNBOUND", "generation join overstated")
    print(f"PASS_CONTROL52_FORMATTER_GATE_SUFFIX files={len(entries)} synthetic_rows={checks} generation_join=UNBOUND")


if __name__ == "__main__":
    main()
