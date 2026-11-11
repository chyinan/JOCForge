from __future__ import annotations
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(65536), b""):
            h.update(block)
    return h.hexdigest()

manifest = json.loads((root / "MANIFEST.json").read_text(encoding="utf-8"))
seal = json.loads((root / "package_seal.json").read_text(encoding="utf-8"))
payloads = manifest["payloads"]
expected = {item["file"] for item in payloads} | {"MANIFEST.json", "package_seal.json"}
actual = {item.name for item in root.iterdir() if item.is_file()}
assert actual == expected
for item in payloads:
    path = root / item["file"]
    assert path.stat().st_size == item["bytes"]
    assert sha(path) == item["sha256"]
manifest_hash = sha(root / "MANIFEST.json")
assert seal["manifest_sha256"] == manifest_hash
seal_files = {item["file"]: item for item in seal["files"]}
assert set(seal_files) == {item["file"] for item in payloads} | {"MANIFEST.json"}
for name, item in seal_files.items():
    path = root / name
    assert path.stat().st_size == item["bytes"]
    assert sha(path) == item["sha256"]
evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
vector = json.loads((root / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
assert evidence["synthetic_c511"]["parser_result"] == "Parsed"
assert evidence["synthetic_c511"]["bytes_consumed"] == 511
assert evidence["separate_boundaries"]["joc_payload_semantics"] == "NOT_TESTED"
assert evidence["separate_boundaries"]["production_allowed"] is False
assert vector["raw_bytes_included"] is False
assert vector["cases"][1]["disposition"] == "REJECTED_BEFORE_FRAME_BUILD"
print("PASS: synthetic exact-C511 parser receipt; raw bytes excluded; production gate preserved")
