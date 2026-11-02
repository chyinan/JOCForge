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
require(sha256(manifest_path) == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
require(manifest.get("package_id") == "SAMPLE6-RUST-U5-RECEIPT-SCALAR-GAP-V1", "package id")
payload = manifest.get("files")
require(isinstance(payload, list) and len(payload) == 4, "payload inventory")
actual_names = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
require(actual_names == {entry.get("file") for entry in payload}, "payload file set")
for entry in payload:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"hash {entry['file']}")

seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
sealed = seal.get("files")
require(isinstance(sealed, list) and seal.get("file_count") == len(sealed), "seal inventory")
require({entry.get("file") for entry in sealed} == actual_names | {"MANIFEST.json", "MANIFEST.sha256"}, "sealed file set")
for entry in sealed:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"sealed size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"sealed hash {entry['file']}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
require(evidence["source_package"]["manifest_sha256"] == "f867629d04c3eb684ea6599bb9efa90cdb14334b422ed6b1e229c02658d6941b", "source manifest binding")
require(evidence["current_run"]["output_u5"]["value"] == 17, "current output U5")
require(evidence["current_run"]["selected_lufs_f32_word_in_receipt_or_logs"] is False, "missing scalar input")
require(evidence["dee_reference_scope"]["file_position_window"]["full_file"] is False, "bounded DEE window")
require(evidence["dee_reference_scope"]["comparison_disposition"].startswith("NOT_SAME_SCOPE"), "scope-correct comparison")
require(evidence["future_receipt_requirement"]["sensitive"] is False, "nonsecret receipt field")
require(vector["valid_same_scope_mismatch_claim"] is False and vector["new_dee_run"] == "NOT_RUN", "no false mismatch")
require(evidence["execution_boundary"]["private_key_or_license_read"] is False, "private material boundary")

print("PASS: current U5 receipt gap and bounded-window DEE scope preserved")
