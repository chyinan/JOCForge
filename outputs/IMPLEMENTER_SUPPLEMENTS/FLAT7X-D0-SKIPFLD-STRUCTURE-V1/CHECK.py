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
expected_files = {item["file"] for item in manifest["payloads"]} | {"MANIFEST.json", "package_seal.json"}
actual_files = {item.name for item in root.iterdir() if item.is_file()}
assert actual_files == expected_files, (sorted(actual_files), sorted(expected_files))
for item in manifest["payloads"]:
    path = root / item["file"]
    assert path.stat().st_size == item["bytes"]
    assert sha(path) == item["sha256"]
manifest_hash = sha(root / "MANIFEST.json")
assert seal["manifest_sha256"] == manifest_hash
seal_files = {item["file"]: item for item in seal["files"]}
assert set(seal_files) == {item["file"] for item in manifest["payloads"]} | {"MANIFEST.json"}
for name, item in seal_files.items():
    path = root / name
    assert path.stat().st_size == item["bytes"]
    assert sha(path) == item["sha256"]
evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
vector = json.loads((root / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
sources = json.loads((root / "SOURCE-PACKAGE.json").read_text(encoding="utf-8"))
assert manifest["production_allowed"] is False
assert evidence["implementation_contract"]["production_allowed"] is False
assert evidence["separate_statuses"]["openjoc_exact_c511_container_acceptance"] == "NOT_RUN"
assert evidence["separate_statuses"]["audio_payload_generation"] == "UNBOUND"
assert vector["raw_payload_bytes_included"] is False
assert sources["source_packages"][0]["source_file_sha256"] == "f976aba7a4750be80b4373a5d7fabf2ad101e0c227fc4bd77716b76879d546cd"
assert sources["source_packages"][1]["manifest_sha256"] == "2c1b9327c914b0e0dc8543e89e8f7ec64f1b56c11e80e9cde9711ec9f5d3ab69"
print("PASS: sealed Flat7x D0 skipfld structural contract; source hashes bound; payload excluded; production gate preserved")
