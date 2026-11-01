from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "KEYLESS-PROFILE-PROBE-RUST-CONTAINER-PROFILE-DIFF-V1"
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
    require(evidence["source"]["source_manifest_sha256"] == "a2e687ad59e311151a4d6dc4c4febbd8d1a2e8e2e85fce2c0466e79ea7eca498", "source manifest mismatch")
    require(evidence["profile_matrix"]["emdf_protection"]["status"] == "SOURCE_PROFILE_DIFFERENCE", "EMDF boundary lost")
    require(evidence["profile_matrix"]["eac3_crc2"]["status"] == "SOURCE_ALGORITHM_MATCH", "CRC2 comparison changed")
    require(evidence["profile_matrix"]["joc_matrix_and_coefficient_generation"]["status"] == "DIFFERENT_LAYER", "prototype scope expanded")
    require(evidence["interpretation"]["container_profile_difference_is_causal_root"] == "NOT_ESTABLISHED", "root cause promoted")
    require(evidence["scope"]["private_key_hmac_or_license_read"] is False, "secret boundary mismatch")
    require(evidence["scope"]["ec3_or_wav_bytes_opened_or_hashed"] is False, "media boundary mismatch")
    print("PACKAGE_VERIFIED files=4 source_profile_diff_only=true implementation_ready=false")


if __name__ == "__main__":
    main()
