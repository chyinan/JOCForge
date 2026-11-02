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
require(manifest.get("package_id") == "DEE-NATIVE-JOC-DELTA-WRITER-BOUNDARY-V2", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "49b64ec5541beeecb472ac33beb617e5f0a03dc4ed103ab470c17153e052a459", "source manifest hash")
require(manifest.get("status") == "CANDIDATE_RULE_READY_FOR_FOCUSED_IMPLEMENTATION_TRIAL; CURRENT_PROFILE_SELECTOR_AND_ROOT_CAUSE_UNBOUND", "status")
require(manifest.get("implementation_trial_ready") is True, "trial disposition")

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
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
selector = evidence.get("stride_selector", {})
require(selector.get("first_four_table_values") == [0, 1, 2, 4], "stride table entries")
require(selector.get("mapping") == {"0":"disabled","1":"stride 1","2":"stride 2","3":"stride 4"}, "selector map")
require(selector.get("current_1024kbps_51_selector_value") == "UNBOUND", "profile selector boundary")
require(evidence.get("core_audio_boundary", {}).get("explains_exponent_minus_one_error") is False, "core exponent boundary")
require(vector.get("expected_outputs_included") is False and vector.get("executed") is False, "input-only vector")
require(sources.get("source_code_included") is False and sources.get("private_key_or_license_material_copied") is False, "exclusions")

print("PASS: manifest and payload hashes, per-block stride map, JOC-only scope, input-only vector, and exclusions")
