"""Validate the sanitized, evidence-only F06ND text-event producer boundary."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE_MANIFEST = "9b89074318558ff692631e0b39f8d8699877c714d08fdadbf68b24e1e66342d5"
SOURCE_SEAL = "0ae3cc3d6155422e3bcc799a61ac07b4de461c4f9f3008de621f62e899d43029"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == "F06ND-TEXT-EVENT-PRODUCER-BOUNDARY-V1", "package id")
    require(manifest.get("status") == "EVIDENCE_ONLY", "evidence-only status")
    require(manifest.get("implementation_allowed") is False, "implementation gate")
    require(manifest.get("source_package_manifest_sha256") == SOURCE_MANIFEST, "source manifest")
    require(manifest.get("source_package_seal_sha256") == SOURCE_SEAL, "source seal")
    require(evidence["status"]["selected_category5_owner_generation_join"] == "NO_SELECTED_JOIN_ESTABLISHED", "category5 join")
    require(evidence["status"]["drc_bsi_effect"] == "UNBOUND_NO", "effect boundary")
    require(evidence["status"]["runtime"] == "NOT_RUN", "runtime")
    require(evidence["scope"]["global_no_event_claim"] is False, "global scope")
    require(len(vectors.get("vectors", [])) == 1, "vector count")
    expected = {"EVIDENCE.json", "SOURCE-MANIFESTS.json", "INPUT-ONLY-VECTORS.json", "WASH-REVIEW.md", "CHECK.py"}
    listed = manifest.get("files", [])
    require({entry.get("path") for entry in listed} == expected, "manifest payload set")
    for entry in listed:
        path = ROOT / entry["path"]
        require(path.is_file(), f"missing payload {entry['path']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch {entry['path']}")
        require(sha256(path) == entry.get("sha256"), f"hash mismatch {entry['path']}")
    print("PASS package=F06ND-TEXT-EVENT-PRODUCER-BOUNDARY-V1 category5_join=NO_SELECTED_JOIN_ESTABLISHED runtime=NOT_RUN")


if __name__ == "__main__":
    main()
