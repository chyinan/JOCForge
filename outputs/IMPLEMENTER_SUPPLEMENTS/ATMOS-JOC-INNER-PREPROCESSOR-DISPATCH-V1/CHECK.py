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
require(manifest.get("package_id") == "ATMOS-JOC-INNER-PREPROCESSOR-DISPATCH-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "d926de87e639bb9e9201ece1f5f7ec70e943e7f03e2167b5c582aad9bc7cfffe", "source manifest")

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
require(evidence.get("dispatch", {}).get("selector_per_run_value") == "UNBOUND", "per-run selector boundary")
require(evidence.get("selector_zero_branch", {}).get("plus_0x28_role") == "state/control update, not sample transform", "zero branch role")
require(evidence.get("selector_zero_branch", {}).get("sample_process_slot") == "+0x08", "sample process slot")
require(evidence.get("selector_nonzero_branch", {}).get("actual_indirect_target") == "UNBOUND/NO", "bypass target boundary")
require(vector.get("execution") == "NOT_RUN" and vector.get("pcm_values_included") is False, "input-only vector")
require(evidence.get("payload_policy", {}).get("private_key_or_license_material_included") is False, "private material exclusion")

print("PASS: manifest/payload hashes, selector branch classification, input-only vector, and exclusions")
