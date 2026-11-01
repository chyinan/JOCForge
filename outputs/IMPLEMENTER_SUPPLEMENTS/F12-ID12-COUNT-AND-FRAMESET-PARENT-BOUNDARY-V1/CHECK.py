from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "F12-ID12-COUNT-AND-FRAMESET-PARENT-BOUNDARY-V1"
PAYLOADS = (
    "EVIDENCE.json",
    "INPUT-ONLY-VECTORS.json",
    "REPORT.md",
    "SOURCE-MANIFESTS.json",
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    pinned = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
    require(sha256(manifest_path) == pinned, "manifest hash mismatch")
    require(manifest["schema"] == "jocforge-washed-implementer-supplement.v1", "schema mismatch")
    require(manifest["package_id"] == PACKAGE_ID, "package id mismatch")
    require(manifest["implementation_ready"] is False, "scope must remain non-ready")
    require(tuple(item["file"] for item in manifest["files"]) == PAYLOADS, "payload list mismatch")
    for item in manifest["files"]:
        path = ROOT / item["file"]
        require(path.is_file(), f"missing payload: {item['file']}")
        require(path.stat().st_size == item["bytes"], f"size mismatch: {item['file']}")
        require(sha256(path) == item["sha256"], f"payload hash mismatch: {item['file']}")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["source"]["source_manifest_sha256"] == "439df32fce6b2dfe8f20026225536ef1e954784d1e99fd50d94e1a0b27b2ed33", "source manifest mismatch")
    counts = evidence["container"]
    require(counts["setter_count_offset"] == "+0x4290 (dword index 0x10a4)", "setter count mapping mismatch")
    require(counts["output_count_offset"] == "+0x4294 (dword index 0x10a5)", "output count mapping mismatch")
    require(counts["static_count_equality_invariant"] == "NOT_FOUND", "counts promoted to equal")
    require(evidence["frame_set"]["runtime_generation_token"] == "NOT_CAPTURED", "generation boundary promoted")
    require(evidence["status"]["tail_occurrence_in_real_run"] == "UNBOUND", "tail observation promoted")
    print("PACKAGE_VERIFIED files=4 static_count_boundary=true runtime_values_unbound=true")


if __name__ == "__main__":
    main()
