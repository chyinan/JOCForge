from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-36-DBMD-ROW-COUNT-WASH-V1"
PAYLOADS = [
    "REPORT.md",
    "EVIDENCE.json",
    "INPUT-ONLY-VECTOR.json",
    "SOURCE-MANIFESTS.json",
    "verify_package.py",
    "seal_package.py",
]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(block)
    return hasher.hexdigest()


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    manifest_hash = digest(manifest_path)
    checksum = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(checksum == [manifest_hash, "MANIFEST.json"], "manifest checksum")
    require(manifest.get("package_id") == PACKAGE_ID == seal.get("package_id"), "package id")
    require(manifest.get("files") == seal.get("files"), "manifest/seal file list")
    require([item["name"] for item in manifest["files"]] == PAYLOADS, "payload list")
    require(seal.get("manifest_sha256") == manifest_hash, "seal manifest hash")
    for item in manifest["files"]:
        path = ROOT / item["name"]
        require(path.stat().st_size == item["bytes"], f"size {item['name']}")
        require(digest(path) == item["sha256"], f"hash {item['name']}")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["container"]["channels"] == 36, "fmt channel count")
    require(evidence["chna"]["track_count"] == evidence["chna"]["uid_count"] == 36, "CHNA dimensions")
    require(evidence["dbmd"]["track_record_count"] == 36, "DBMD record count")
    require(evidence["dbmd"]["section_lengths_and_checksums_valid"] is True, "DBMD section validation")
    require(evidence["execution_scope"]["pcm_bytes_read"] == 0, "PCM read boundary")
    require(evidence["execution_scope"]["same_prepared_job_input_join_proven"] is False, "join boundary")
    require(evidence["execution_scope"]["production_allowed"] is False, "metadata-only scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS dimensions=fmt36/chna36/dbmd36 pcm_bytes_read=0")


if __name__ == "__main__":
    main()
