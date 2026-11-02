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
require(manifest.get("package_id") == "SAMPLE6-KEYLESS-DEE-REFERENCE-GATE-V4", "package id")
payload = manifest.get("files")
require(isinstance(payload, list) and len(payload) == 3, "payload inventory")
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
require(evidence["source_gates"]["bounded_dee_dialnorm_package"]["time_window"] == {"time_base": "file_position", "start": "2160", "end": "2208"}, "bounded DEE window")
require(evidence["dialnorm_scope"]["same_full_file_scope"] is False, "scope mismatch")
require(evidence["dialnorm_scope"]["mismatch_claim_disposition"] == "DO_NOT_LABEL AS A FULL-FILE SAME-SCOPE U5 MISMATCH", "no false mismatch claim")
require(evidence["fresh_dee_comparison"]["full_file_audio_essence_comparison"] == "NOT_RUN", "fresh audio comparison remains NOT_RUN")
require(evidence["fresh_dee_comparison"]["audio_equivalence"] == "NOT_ESTABLISHED", "no audio equivalence claim")
require(evidence["bounded_raw_header_check"]["bytes_read"] == 10 and evidence["bounded_raw_header_check"]["raw_bytes_emitted_or_retained"] is False, "bounded header read")
require(evidence["constraints"]["private_key_or_license_accessed"] is False and evidence["constraints"]["pcm_persisted"] is False, "key/PCM boundary")

print("PASS: full output parse/decode, bounded-window U5 scope correction, BSI discrepancy, and fresh DEE NOT_RUN")
