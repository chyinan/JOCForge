"""Validate the sanitized, evidence-only F06ND generic storage-event boundary."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE_MANIFEST = "3d0462cb661b40efbac0b228f0a507de5644241b0e9f82de2996195ccc851dba"
SOURCE_SEAL = "4585eaa30e542b798d828854c633541637ec8d420b8cd8ad867923c62657ff47"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == "F06ND-GENERIC-TYPED-STORAGE-EVENT-EDGE-BOUNDARY-V1", "package id")
    require(manifest.get("status") == "EVIDENCE_ONLY", "evidence-only status")
    require(manifest.get("implementation_allowed") is False, "implementation gate")
    require(manifest.get("source_package_manifest_sha256") == SOURCE_MANIFEST, "source manifest")
    require(manifest.get("source_package_seal_sha256") == SOURCE_SEAL, "source seal")
    require(evidence["status"]["selected_category5_owner_generation_join"] == "NO_SELECTED_JOIN_ESTABLISHED", "category5 join")
    require(evidence["status"]["drc_bsi_output_effect"] == "UNBOUND_NO", "effect boundary")
    require(evidence["status"]["runtime"] == "NOT_RUN", "runtime")
    require(len(vectors.get("vectors", [])) == 1, "vector count")
    expected = {"EVIDENCE.json", "SOURCE-MANIFESTS.json", "INPUT-ONLY-VECTORS.json", "WASH-REVIEW.md", "CHECK.py"}
    listed = manifest.get("files", [])
    require({entry.get("path") for entry in listed} == expected, "manifest payload set")
    for entry in listed:
        path = ROOT / entry["path"]
        require(path.is_file(), f"missing payload {entry['path']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch {entry['path']}")
        require(sha256(path) == entry.get("sha256"), f"hash mismatch {entry['path']}")
    print("PASS package=F06ND-GENERIC-TYPED-STORAGE-EVENT-EDGE-BOUNDARY-V1 category5_join=NO_SELECTED_JOIN_ESTABLISHED runtime=NOT_RUN")


if __name__ == "__main__":
    main()
