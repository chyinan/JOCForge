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
require(sha256(manifest_path) == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest digest")
require(manifest.get("package_id") == "ATMOS-ROW-HANDOFF-SOURCE-INDEX-ERRATUM-V1", "package id")

payload_entries = manifest.get("files")
require(isinstance(payload_entries, list) and len(payload_entries) == 3, "payload inventory")
actual_names = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
require(actual_names == {entry.get("file") for entry in payload_entries}, "payload file set")
for entry in payload_entries:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"hash {entry['file']}")

seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
sealed_files = seal.get("files")
require(isinstance(sealed_files, list) and seal.get("file_count") == len(sealed_files), "seal count")
require({entry.get("file") for entry in sealed_files} == actual_names | {"MANIFEST.json", "MANIFEST.sha256"}, "sealed file set")
for entry in sealed_files:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"sealed size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"sealed hash {entry['file']}")

erratum = json.loads((ROOT / "ERRATUM.json").read_text(encoding="utf-8"))
audit = erratum.get("reference_audit", {})
mismatch = audit.get("mismatch", {})
require(audit.get("references_checked") == 24, "reference audit count")
require(audit.get("matching_references") == 23 and audit.get("mismatching_references") == 1, "reference audit result")
require(mismatch.get("source_index_value") != mismatch.get("verified_file_sha256"), "transcription discrepancy")
require(mismatch.get("verified_file_sha256") == mismatch.get("source_package_seal_value"), "source package seal corroboration")
require(erratum.get("source_package_validation", {}).get("normal_checker") == "PASS", "normal source checker")
require(erratum.get("source_package_validation", {}).get("optimized_checker") == "PASS", "optimized source checker")
require(erratum.get("impact", {}).get("audio_rule_changed") is False, "audio rule unchanged")
require(erratum.get("impact", {}).get("private_key_or_license_material_included") is False, "private material excluded")

print("PASS: hash-bound source-index erratum; 23/24 references match; source TRACE digest corroborated by its seal")
