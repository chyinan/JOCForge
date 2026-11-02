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
require(manifest.get("package_id") == "DEE-JOC-STRIDE-SELECTOR-SOURCE-BOUNDARY-V2", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "bd4284314b12586aa9cce3e927d897a647ee9ccb8acaaac141ddc0df75b77f1b", "source package hash")

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
require(evidence.get("direct_scalar_store_scan", {}).get("candidate_store_count") == 5, "direct store count")
require(evidence.get("direct_scalar_store_scan", {}).get("target_selector_store_joined") is False, "no target writer join")
require(evidence.get("selector_consumer", {}).get("first_four_values") == [0,1,2,4], "selector table")
require(evidence.get("selector_consumer", {}).get("current_profile_selector") == "UNBOUND", "profile boundary")
require(evidence.get("payload_policy", {}).get("private_key_or_license_material_included") is False, "private material boundary")

print("PASS: manifest/payload hashes, full direct-store scan scope, selector rule, and exclusions")
