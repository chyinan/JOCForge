from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATTEMPT10-FIRST-AU-EXPONENT-SYNTAX-BOUNDARY-V1"
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
    require(evidence["source_package"]["manifest_sha256"] == "6a219a9454316e75b5d92dc77eabf3907f2779a510356558f81a6db134c42902", "source manifest mismatch")
    require(evidence["attempt10_diagnostic_from_prior_wash"]["reported_decoded_exponent"] == -1, "diagnostic mismatch")
    require(evidence["attempt10_diagnostic_from_prior_wash"]["lane_block_and_bit_interval"] == "UNBOUND", "causal boundary promoted")
    require(evidence["id14_and_complexity_boundary"]["direct_id14_body_causality_for_first_diagnostic"] == "NOT_SUPPORTED_BY_EVIDENCE", "ID14 causality promoted")
    require(evidence["execution_scope"]["private_key_hmac_or_license_read"] is False, "secret boundary mismatch")
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    underflow = vectors["vectors"][1]
    require(underflow["first_update"] == "1 + (0 - 2) = -1", "underflow vector mismatch")
    require(underflow["does_not_identify_attempt10_lane_block_or_bit_offset"] is True, "vector scope mismatch")
    print("PACKAGE_VERIFIED files=4 parser_boundary_only=true implementation_ready=false")


if __name__ == "__main__":
    main()
