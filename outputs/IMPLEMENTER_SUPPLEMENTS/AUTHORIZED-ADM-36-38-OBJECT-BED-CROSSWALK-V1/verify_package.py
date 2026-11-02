"""Verify the washed ADM bed/object metadata crosswalk supplement."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def main() -> None:
    root = Path(__file__).resolve().parent
    manifest_path = root / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads((root / "package_seal.json").read_text(encoding="utf-8"))
    digest = sha256(manifest_path)
    require(digest == (root / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "MANIFEST.sha256 mismatch")
    require(digest == seal.get("manifest_sha256"), "package seal manifest mismatch")
    require(manifest.get("package_id") == seal.get("package_id"), "package id mismatch")
    require(manifest.get("files") == seal.get("files"), "manifest/seal payload list mismatch")
    require(manifest.get("source_receipt_manifest_sha256") == "d318321ff05f2db745f94fc1851140ad1e07b6e5aab417167f279fd5d307aba3", "source receipt manifest mismatch")
    for entry in manifest["files"]:
        path = root / entry["name"]
        require(path.is_file(), f"missing payload: {entry['name']}")
        require(path.stat().st_size == entry["bytes"], f"payload size mismatch: {entry['name']}")
        require(sha256(path) == entry["sha256"], f"payload hash mismatch: {entry['name']}")
    evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
    rows = {item["sample_id"]: item for item in evidence["input_counts"]}
    require(rows["potential_ai"]["pcm_channels"] == 36 and rows["potential_ai"]["Objects_Objects_track_rows"] == 26, "36-channel object/bed partition mismatch")
    require(rows["human_cat"]["pcm_channels"] == 38 and rows["human_cat"]["Objects_Objects_track_rows"] == 28, "38-channel object/bed partition mismatch")
    require(evidence["data_handling"]["data_chunk_bytes_read"] == 0, "data chunk must remain unread")
    require(rows["potential_ai"]["prepared_job_identity_join"] == "UNBOUND", "PreparedJobInput identity must remain UNBOUND")
    print(f"PASS package={manifest['package_id']}")
    print(f"PASS manifest_sha256={digest}")
    print(f"PASS 36/38 bed/object crosswalk; data bytes read=0; PreparedJobInput join=UNBOUND")


if __name__ == "__main__":
    main()
