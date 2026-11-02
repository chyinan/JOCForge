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
require(manifest.get("package_id") == "F12-HANMAO-SAME-RUN-U5-CLOSURE-V1", "package id")
require(manifest.get("source_reverse_manifest_sha256") == "cafc228c3ede087cc7727d8b0684c89f562a600ae19ed178607ba4967d996bf7", "incoming package hash")
require(manifest.get("status") == "SAME_RUN_INPUT_SCALAR_U5_RECEIPT_CLOSED; SCALAR_UNIT_AND_NATIVE_OWNER_JOIN_OPEN", "status")
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
same = evidence.get("same_run_observation", {})
require(same.get("input_sha256") == "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1", "Hanmao input identity")
require(same.get("attempt_id") == "a-fresh-reference" and same.get("dee_exit_code") == 0, "same successful attempt")
require(same.get("measured_aggregate_scalar") == -20.665958 and same.get("u5_observed") == 21, "same-run scalar/U5")
require(same.get("prefix_bytes_read") == 7 and same.get("payload_body_bytes_read") == 0, "payload boundary")
require(evidence.get("candidate_rule_check", {}).get("predicted_u5") == 21, "candidate value")
require(evidence.get("closure", {}).get("measurement_to_bsi_owner_generation_join") == "NOT_CLOSED", "owner join boundary")
require(evidence.get("provenance_audit", {}).get("parent_manifest_reference_matches_current_manifest") is False, "provenance mismatch preserved")
require(vector.get("expected_output_included") is False and vector.get("executed_by_this_vector") is False, "input-only vector")
require(sources.get("audio_or_pcm_copied") is False and sources.get("private_key_or_license_material_copied") is False, "payload exclusion")

print("PASS: manifest/payload hashes, same-run Hanmao scalar-to-U5 receipt, input-only vector, and provenance mismatch boundary")
