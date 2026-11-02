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
require(manifest.get("package_id") == "DEE-JOC-STRIDE-SELECTOR-SOURCE-BOUNDARY-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "ddce8bd663b4dd28323a2f6e1624c7fc83f5997361843c6f18534816a244deef", "source package hash")

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
selector = evidence.get("selector", {})
require(selector.get("first_four_values") == [0,1,2,4], "selector table")
require(selector.get("current_1024kbps_51_value") == "UNBOUND", "profile value boundary")
require(evidence.get("narrow_write_scan", {}).get("write_to_selector_field_found") is False, "no writer boundary")
require(vector.get("executed") is False and vector.get("expected_payload_bytes_included") is False, "input-only vector")
require(evidence.get("boundaries", {}).get("eac3_core_exponent_path_covered") is False, "E-AC3 scope")

print("PASS: manifest/payload hashes, selector read/table, bounded write scan, and exclusions")
