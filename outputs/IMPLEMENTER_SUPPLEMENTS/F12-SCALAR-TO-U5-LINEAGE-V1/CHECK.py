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
require(manifest.get("package_id") == "F12-SCALAR-TO-U5-LINEAGE-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "2f9936de80f09cca10f3c0763f0a615e495fc88508f72554a4e1fcaefc20013a", "source package hash")

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
observations = evidence.get("same_run_observations", [])
require(len(observations) == 3, "three observed same-run samples")
require(all(item.get("same_run_bound") is True and item.get("u5_observed") == item.get("u5_predicted") for item in observations), "candidate matches observed U5")
require(evidence.get("scalar_only", {}).get("counted_as_observation") is False, "scalar-only sample boundary")
require(evidence.get("ghidra_facts", {}).get("normalizer_direct_bsi_write") is False, "native join limit")
require(evidence.get("candidate_rule", {}).get("mvp_blocking") is False, "scoped trial disposition")
require(evidence.get("payload_policy", {}).get("private_key_or_license_material_included") is False, "private material exclusion")

print("PASS: manifest/payload hashes, F32 U5 candidate, three same-run matches, scalar-only limit, and exclusions")
