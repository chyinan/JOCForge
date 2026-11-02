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
require(manifest.get("package_id") == "F06-R138-OUTPUTWRAPPER-POSTFILTER-ROW-ORDER-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "8b35e3671e2e7abd1e143944b8f77875f2b800f7bb29ca03b8b468f0f6be8cd6", "source manifest")

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
require(evidence.get("host_local_facts", {}).get("postfilter_wrapper_order_preserved") is True, "postfilter order")
require(evidence.get("host_local_facts", {}).get("match_used_prepass") == "BOUND_STATIC; can filter/omit entries", "filtering boundary")
require(evidence.get("identity_boundaries", {}).get("wrapper_source_row_equals_converter_source_vector_row") == "UNBOUND", "cross-vector identity")
require(vector.get("expected_output_included") is False and vector.get("executed") is False, "input-only vector")
require(evidence.get("payload_policy", {}).get("private_key_or_license_material_included") is False, "private material boundary")

print("PASS: manifest/payload hashes, postfilter row order, cross-vector boundary, and exclusions")
