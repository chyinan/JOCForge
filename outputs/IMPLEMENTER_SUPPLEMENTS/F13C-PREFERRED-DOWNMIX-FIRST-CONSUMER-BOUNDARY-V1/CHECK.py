"""Validate the sanitized, evidence-only preferred-downmix first-break supplement."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE_MANIFEST = "a2ed71f829bda692bd8662c9186ff0b33625703a529d0f4ab1e24e9797ea5383"
SOURCE_SUMS = "0030a09cfd24f227a05c384f3bf2a68302897b488c228bbc3897f63538fde753"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == "F13C-PREFERRED-DOWNMIX-FIRST-CONSUMER-BOUNDARY-V1", "package id")
    require(manifest.get("status") == "EVIDENCE_ONLY", "evidence-only status")
    require(manifest.get("implementation_allowed") is False, "implementation gate")
    require(manifest.get("source_package_manifest_sha256") == SOURCE_MANIFEST, "source manifest")
    require(manifest.get("source_package_sha256sums_sha256") == SOURCE_SUMS, "source checksum file")
    require(evidence["status"]["xml_value_to_native_typed_dmixmod_owner"] == "UNBOUND", "typed owner")
    require(evidence["status"]["candidate_pair_runtime"] == "NOT_RUN", "runtime")
    require(evidence["limits"]["global_absence_claim"] is False, "global scope")
    require(len(vectors.get("vectors", [])) == 2, "candidate vector count")
    expected = {"EVIDENCE.json", "SOURCE-MANIFESTS.json", "INPUT-ONLY-VECTORS.json", "WASH-REVIEW.md", "CHECK.py"}
    listed = manifest.get("files", [])
    require({entry.get("path") for entry in listed} == expected, "manifest payload set")
    for entry in listed:
        path = ROOT / entry["path"]
        require(path.is_file(), f"missing payload {entry['path']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch {entry['path']}")
        require(sha256(path) == entry.get("sha256"), f"hash mismatch {entry['path']}")
    print("PASS package=F13C-PREFERRED-DOWNMIX-FIRST-CONSUMER-BOUNDARY-V1 mode_owner=UNBOUND runtime=NOT_RUN")


if __name__ == "__main__":
    main()
