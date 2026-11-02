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
require(manifest.get("package_id") == "DEE-NATIVE-JOC-DELTA-WRITER-BOUNDARY-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "f4971adde8991da614db1abb21bbed661ad536d794e836c7f1eac8622d4a3998", "source manifest hash")
require(manifest.get("status") == "CANDIDATE_RULE_READY_FOR_FOCUSED_IMPLEMENTATION_TRIAL; ROOT_CAUSE_NOT_ESTABLISHED", "status")
require(manifest.get("implementation_trial_ready") is True, "trial disposition")
require(manifest.get("production_conformance_established") is False, "no production overclaim")

files = manifest.get("files")
require(isinstance(files, list) and files, "file inventory")
sealed_names = {entry.get("file") for entry in files}
actual_names = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256"}}
require(sealed_names == actual_names, "sealed file inventory")
for entry in files:
    path = ROOT / entry["file"]
    require(path.is_file(), f"missing {entry['file']}")
    require(path.stat().st_size == entry["bytes"], f"size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"hash {entry['file']}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vector = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
require(evidence.get("scope") == "NATIVE ATMOS DDP JOC COEFFICIENT PAYLOAD ONLY", "audio scope")
require(evidence.get("confidence", {}).get("candidate_caused_user_reported_output_failure") == "LOW", "root-cause confidence")
require(evidence.get("unbound", {}).get("producer_packer_pointer_alias") is True, "pointer alias boundary")
require(evidence.get("audio_core_boundary", {}).get("explains_exponent_minus_one_error") is False, "core exponent exclusion")
require(vector.get("expected_outputs_included") is False and vector.get("executed") is False, "input-only vector")
require(vector.get("native_delta_transform", {}).get("expected_output_included") is False, "no expected delta output")
require(vector.get("native_bit_writer_interface", {}).get("writer_bytes_included") is False, "no writer bytes")
require(sources.get("source_files_copied") is False and sources.get("source_binaries_copied") is False, "no source copied")
require(sources.get("private_key_or_license_material_copied") is False, "no private material")

print("PASS: manifest and payload hashes, exact JOC payload scope, candidate confidence, input-only vector, and exclusions")
