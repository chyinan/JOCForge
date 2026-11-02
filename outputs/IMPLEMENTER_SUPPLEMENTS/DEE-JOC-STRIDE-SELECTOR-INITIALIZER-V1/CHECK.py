from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
sidecar = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(sha256(manifest_path) == sidecar, "manifest sidecar")
require(manifest.get("package_id") == "DEE-JOC-STRIDE-SELECTOR-INITIALIZER-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "2e28a33cc6f9930e54ad9b99922d08a681d67d30b43aa95e48bfec36ba74d214", "source package hash")

files = manifest.get("files")
require(isinstance(files, list) and bool(files), "file inventory")
names = {entry.get("file") for entry in files}
actual = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256"}}
require(names == actual, "sealed file inventory")
for entry in files:
    path = ROOT / entry["file"]
    require(path.is_file(), f"missing {entry['file']}")
    require(path.stat().st_size == entry["bytes"], f"size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"hash {entry['file']}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
mapping = evidence.get("selector_mapping", {})
require(mapping.get("known_default_rows") == {"0":0,"1":0}, "known zero rows")
require(mapping.get("rows_2_to_5_initial_values") == "UNBOUND", "later row boundary")
require(evidence.get("constructor", {}).get("copies_selector_from_second_argument") is False, "no copy inference")
require(vector.get("executed") is False, "input-only vector")
require(evidence.get("payload_policy", {}).get("private_key_or_license_material_included") is False, "private material boundary")

print("PASS: manifest/payload hashes, row0/row1 initialization, later-row boundary, and exclusions")
