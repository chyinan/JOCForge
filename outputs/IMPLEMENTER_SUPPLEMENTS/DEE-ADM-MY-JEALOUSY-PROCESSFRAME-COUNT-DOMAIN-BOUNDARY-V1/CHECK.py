"""Validate the sanitized, evidence-only My Jealousy count-domain supplement."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE_MANIFEST = "13eaaae38f784ca2fd38a2a5fdc50110c411d0ce39354a7ba1f03b11f421ff9b"
METADATA_RECEIPT = "eb14dd147019f917b59ee20420ed96e9bb05e8de81d5f9d465c77a58610e9946"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == "DEE-ADM-MY-JEALOUSY-PROCESSFRAME-COUNT-DOMAIN-BOUNDARY-V1", "package id")
    require(manifest.get("status") == "EVIDENCE_ONLY", "evidence-only status")
    require(manifest.get("implementation_allowed") is False, "implementation gate")
    require(manifest.get("source_package_manifest_sha256") == SOURCE_MANIFEST, "source manifest")
    require(evidence.get("metadata_receipt_sha256") == METADATA_RECEIPT, "metadata receipt")
    require(evidence["status"]["actual_mode_and_runtime_N"] == "NOT_CAPTURED", "runtime binding")
    require(evidence["status"]["full_36_track_output"] == "NOT_ESTABLISHED", "E2E status")
    require(evidence["limits"]["codec_output_channel_count_join"] == "UNBOUND", "count domain")
    require(len(vectors.get("vectors", [])) == 2, "input-only vector count")
    expected = {"EVIDENCE.json", "SOURCE-MANIFESTS.json", "INPUT-ONLY-VECTORS.json", "WASH-REVIEW.md", "CHECK.py"}
    listed = manifest.get("files", [])
    require({entry.get("path") for entry in listed} == expected, "manifest payload set")
    for entry in listed:
        path = ROOT / entry["path"]
        require(path.is_file(), f"missing payload {entry['path']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch {entry['path']}")
        require(sha256(path) == entry.get("sha256"), f"hash mismatch {entry['path']}")
    print("PASS package=DEE-ADM-MY-JEALOUSY-PROCESSFRAME-COUNT-DOMAIN-BOUNDARY-V1 N36_static_conditional=BOUND runtime=NOT_CAPTURED")


if __name__ == "__main__":
    main()
