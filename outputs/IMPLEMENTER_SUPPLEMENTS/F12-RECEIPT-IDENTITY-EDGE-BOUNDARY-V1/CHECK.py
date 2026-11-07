"""Validate the sanitized, evidence-only F12 receipt-edge supplement."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
EXPECTED_SOURCE_MANIFEST = "b0e79e0fd8f0214d8a29d7ee7254ab8ea93291d7110143885e3cba8148520099"
EXPECTED_SOURCE_SEAL = "b2d1092b37355b2c62cf968fd4031ffc96c992b74bf683a98925889f0f8508b5"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))

    require(manifest.get("package_id") == "F12-RECEIPT-IDENTITY-EDGE-BOUNDARY-V1", "package id")
    require(manifest.get("status") == "EVIDENCE_ONLY_UNBOUND", "manifest status")
    require(manifest.get("implementation_allowed") is False, "implementation gate")
    require(manifest.get("contains_native_or_research_source_code") is False, "research source exclusion")
    require(manifest.get("contains_package_checker_code") is True, "checker provenance")
    require(manifest.get("source_package_manifest_sha256") == EXPECTED_SOURCE_MANIFEST, "source manifest binding")
    require(manifest.get("source_package_seal_sha256") == EXPECTED_SOURCE_SEAL, "source seal binding")
    require(evidence.get("status") == "NOT_READY", "source status preservation")
    require(evidence.get("receipt_edge_status") == "UNBOUND", "receipt edge preservation")
    require(evidence["model_boundary"].get("independent_model_review") == "NOT_RUN", "model review state")
    require(evidence["implementation_effect"].get("blocks_supported_audio_core") is False, "audio-core scope")
    require(len(vectors.get("vectors", [])) == 2, "input-only vector count")

    listed = manifest.get("files", [])
    require({entry.get("path") for entry in listed} == {
        "EVIDENCE.json", "SOURCE-MANIFESTS.json", "INPUT-ONLY-VECTORS.json", "WASH-REVIEW.md", "CHECK.py"
    }, "manifest payload set")
    for entry in listed:
        path = ROOT / entry["path"]
        require(path.is_file(), f"missing payload {entry['path']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch {entry['path']}")
        require(sha256(path) == entry.get("sha256"), f"hash mismatch {entry['path']}")

    print("PASS package=F12-RECEIPT-IDENTITY-EDGE-BOUNDARY-V1 status=NOT_READY receipt_edge=UNBOUND audio_core_blocked=false")


if __name__ == "__main__":
    main()
