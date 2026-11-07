"""Validate the sanitized, evidence-only F06ND Category5 owner first-break."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE_MANIFEST = "ca8ef796a3547b469c43e37266bc65aeff2d8b86beeda5af87e1896575c9a606"
SOURCE_SEAL = "6093b4cffadded1b575425b56576c3fde065aac275995c7be24b71a5f298ecbd"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    manifest = json.loads((ROOT / "MANIFEST.json").read_text(encoding="utf-8"))
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == "F06ND-CATEGORY5-EVENT-OWNER-FIRST-BREAK-BOUNDARY-V1", "package id")
    require(manifest.get("status") == "EVIDENCE_ONLY", "evidence-only status")
    require(manifest.get("implementation_allowed") is False, "implementation gate")
    require(manifest.get("source_package_manifest_sha256") == SOURCE_MANIFEST, "source manifest")
    require(manifest.get("source_package_seal_sha256") == SOURCE_SEAL, "source seal")
    require(evidence["status"]["category5_to_native_typed_application"] == "UNBOUND_NO", "typed application")
    require(evidence["status"]["selected_category5_to_generic_event_vector"] == "NO_SELECTED_JOIN_ESTABLISHED", "typed event-vector join")
    require(evidence["status"]["selected_category5_to_text_event_collector"] == "NO_SELECTED_JOIN_ESTABLISHED", "text event join")
    require(evidence["status"]["owner_generation"] == "UNBOUND_NO", "owner generation")
    require(evidence["status"]["runtime"] == "NOT_RUN", "runtime state")
    require(evidence["limits"]["global_absence_claim"] is False, "global scope")
    require(evidence["limits"]["audio_core_blocked"] is False, "audio-core scope")
    require(len(vectors.get("vectors", [])) == 1, "vector count")
    expected = {"EVIDENCE.json", "SOURCE-MANIFESTS.json", "INPUT-ONLY-VECTORS.json", "WASH-REVIEW.md", "CHECK.py"}
    listed = manifest.get("files", [])
    require({entry.get("path") for entry in listed} == expected, "manifest payload set")
    for entry in listed:
        path = ROOT / entry["path"]
        require(path.is_file(), f"missing payload {entry['path']}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch {entry['path']}")
        require(sha256(path) == entry.get("sha256"), f"hash mismatch {entry['path']}")
    print("PASS package=F06ND-CATEGORY5-EVENT-OWNER-FIRST-BREAK-BOUNDARY-V1 join=NO_SELECTED_JOIN_ESTABLISHED runtime=NOT_RUN")


if __name__ == "__main__":
    main()
