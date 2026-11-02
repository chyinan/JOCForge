from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "KEYLESS-JOC-SEMANTIC-AU-EQUIVALENCE-V1"
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
    comparison = evidence["comparison"]
    require(comparison["per_au_semantic_digest_equal"] is True and comparison["per_au_mismatch_count"] == 0, "AU semantic comparison mismatch")
    require(comparison["parser_diagnostics_each"] == 0 and comparison["malformed_aus_each"] == 0, "parser result mismatch")
    require(evidence["spatial_limits"]["spatial_render_equivalence"] == "NOT_TESTED", "spatial limit lost")
    require(evidence["scope"]["key_hmac_or_license_read"] is False, "secret boundary mismatch")
    print("PACKAGE_VERIFIED files=4 JOC_semantic_AUs_equal=true spatial_render_not_tested=true")


if __name__ == "__main__":
    main()
