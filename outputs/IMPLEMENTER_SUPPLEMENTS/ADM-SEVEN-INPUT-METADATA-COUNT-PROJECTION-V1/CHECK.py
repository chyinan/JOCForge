"""Validate the sanitized, evidence-only seven-input metadata count projection."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE_MANIFEST_LIST = "5a8d953b5be8c0e4e23a57c7974d8ebe633ed8856314407e59b911c5d24e36a8"
IDENTITY_RECEIPT = "19b134d2d3409d48429bdbf9d0779452fd26099c2ebb64ae39a47bb11a6f4e11"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == "ADM-SEVEN-INPUT-METADATA-COUNT-PROJECTION-V1", "package id")
    require(manifest.get("status") == "EVIDENCE_ONLY", "evidence-only status")
    require(manifest.get("implementation_allowed") is False, "implementation gate")
    require(manifest.get("source_manifest_list_sha256") == SOURCE_MANIFEST_LIST, "source manifest list")
    require(manifest.get("authorized_input_identity_receipt_sha256") == IDENTITY_RECEIPT, "identity receipt")
    require(evidence["status"]["native_K_marker_count"] == "NOT_EVALUATED", "native K not upgraded")
    require(evidence["status"]["source_media_reopened"] is False, "no source media reads")
    require(evidence["cross_sample_shape"]["sample_count"] == 7, "sample count")
    require(evidence["cross_sample_shape"]["n36_samples"] == 6 and evidence["cross_sample_shape"]["n38_samples"] == 1, "36/38 split")
    require(len(vectors.get("vectors", [])) == 3, "vector count")
    expected = {"EVIDENCE.json", "SOURCE-MANIFESTS.json", "INPUT-ONLY-VECTORS.json", "WASH-REVIEW.md", "CHECK.py"}
    listed = manifest.get("files", [])
    require({entry.get("path") for entry in listed} == expected, "manifest payload set")
    for entry in listed:
        path = ROOT / entry["path"]
        require(path.is_file(), f"missing payload {entry['path']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch {entry['path']}")
        require(sha256(path) == entry.get("sha256"), f"hash mismatch {entry['path']}")
    print("PASS package=ADM-SEVEN-INPUT-METADATA-COUNT-PROJECTION-V1 samples=7 native_K=NOT_EVALUATED")


if __name__ == "__main__":
    main()
