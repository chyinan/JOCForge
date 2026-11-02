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
require(manifest.get("package_id") == "ADM-CHNA-PHYSICAL-ROW-ORDER-MVP-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "7fcc5d6b2486c2c882c618843e19ca259e447f2c3ca502eb0688eaec3b5528b0", "source manifest hash")
require(manifest.get("status") == "BEST_SUPPORTED_MVP_RULE; EXACT_RUNTIME_OWNER_GENERATION_JOIN_UNBOUND", "status")
require(manifest.get("implementation_trial_ready") is True, "MVP disposition")

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
vector = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
sources = json.loads((ROOT / "SOURCE-MANIFESTS.json").read_text(encoding="utf-8"))
cross = evidence.get("cross_sample_evidence", {})
require(cross.get("distinct_authorized_inputs") == 7 and cross.get("chna_records_checked") == 254, "sample counts")
require(cross.get("track_index_minus_one_to_interleaved_slot_mismatches") == 0, "cross-sample mismatches")
require(evidence.get("best_supported_mvp_rule", {}).get("chna_track_index_mapping") == "zero_based_physical_slot = track_index_1based - 1", "MVP rule")
require(evidence.get("unbound_or_not_run", {}).get("exact_runtime_owner_generation_row_pointer_join") == "UNBOUND", "exact join boundary")
require(evidence.get("unbound_or_not_run", {}).get("dee_or_converter_runtime") == "NOT_RUN_SAFETY_GATE_UNPROVEN", "runtime status")
require(vector.get("expected_output_audio_included") is False and vector.get("runtime_pointer_values_included") is False, "input-only vector")
require(vector.get("aggregate", {}).get("executed") is False, "vector remains unexecuted")
require(sources.get("implementation_source_or_binary_copied") is False, "no implementation source copied")
require(sources.get("private_key_or_license_material_copied") is False, "no private material copied")

print("PASS: manifest and payload hashes, seven-input metadata rule, scoped UNBOUND, input-only vectors, and exclusions")
