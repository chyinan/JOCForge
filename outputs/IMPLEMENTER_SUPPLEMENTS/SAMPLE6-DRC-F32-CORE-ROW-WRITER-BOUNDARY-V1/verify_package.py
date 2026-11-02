"""Verify the sanitized Wash supplement manifest and payload hashes."""

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
    manifest_digest = sha256(manifest_path)
    declared_digest = (root / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]

    require(manifest_digest == declared_digest, "MANIFEST.sha256 mismatch")
    require(manifest_digest == seal.get("manifest_sha256"), "package seal manifest mismatch")
    require(manifest.get("package_id") == seal.get("package_id"), "package id mismatch")
    require(
        manifest.get("upstream_manifest_sha256")
        == "e33a3254d5877b6cce046cb39447e30c47dc88807663e5c716548e1d4e3665f9",
        "unexpected upstream manifest identity",
    )
    require(manifest.get("files") == seal.get("files"), "manifest/seal file list mismatch")

    for entry in manifest["files"]:
        path = root / entry["name"]
        require(path.is_file(), f"missing payload: {entry['name']}")
        require(path.stat().st_size == entry["bytes"], f"payload size mismatch: {entry['name']}")
        require(sha256(path) == entry["sha256"], f"payload hash mismatch: {entry['name']}")

    print(f"PASS package={manifest['package_id']}")
    print(f"PASS manifest_sha256={manifest_digest}")
    print(f"PASS payload_count={len(manifest['files'])}")


if __name__ == "__main__":
    main()
