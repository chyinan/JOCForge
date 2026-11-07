"""Validate the sanitized, evidence-only time endpoint metadata boundary."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE_MANIFEST = "17d122408803bed5e3026cfe7dc42bbadd5d1728fb9c4fda0c8040e16f623db1"
SOURCE_SEAL = "7ff573ef2be516f152d9e39440cee25675fb3391b4bb22fee6756c896be5ae73"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == "DEE-TIME-ENDPOINT-METADATA-INDEX-BOUNDARY-V1", "package id")
    require(manifest.get("status") == "EVIDENCE_ONLY", "evidence-only status")
    require(manifest.get("implementation_allowed") is False, "implementation gate")
    require(manifest.get("source_package_manifest_sha256") == SOURCE_MANIFEST, "source manifest")
    require(manifest.get("source_package_seal_sha256") == SOURCE_SEAL, "source seal")
    require(evidence["status"]["endpoints_to_actual_input_frame_clip"] == "UNBOUND", "clip boundary")
    require(evidence["status"]["inclusive_exclusive_comparison"] == "NOT_CAPTURED", "inclusive/exclusive")
    require(evidence["status"]["final_audio_effect"] == "NOT_RUN", "audio effect")
    require(len(vectors.get("vectors", [])) == 1, "vector count")
    expected = {"EVIDENCE.json", "SOURCE-MANIFESTS.json", "INPUT-ONLY-VECTORS.json", "WASH-REVIEW.md", "CHECK.py"}
    listed = manifest.get("files", [])
    require({entry.get("path") for entry in listed} == expected, "manifest payload set")
    for entry in listed:
        path = ROOT / entry["path"]
        require(path.is_file(), f"missing payload {entry['path']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch {entry['path']}")
        require(sha256(path) == entry.get("sha256"), f"hash mismatch {entry['path']}")
    print("PASS package=DEE-TIME-ENDPOINT-METADATA-INDEX-BOUNDARY-V1 metadata_index=BOUND frame_clip=UNBOUND")


if __name__ == "__main__":
    main()
