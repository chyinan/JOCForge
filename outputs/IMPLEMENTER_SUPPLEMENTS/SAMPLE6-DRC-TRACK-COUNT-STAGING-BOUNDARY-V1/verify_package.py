"""Verify the sanitized native track-count staging boundary supplement."""

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
    require(
        manifest.get("upstream_manifest_sha256")
        == "613d934c86bb9a3bf21049fed7a5bc92f6650d11fadddcf79167533adef9b066",
        "unexpected upstream manifest identity",
    )
    require(manifest.get("files") == seal.get("files"), "manifest/seal payload list mismatch")
    require(manifest.get("runtime_observation") == "NOT_RUN", "runtime status must remain NOT_RUN")
    for entry in manifest["files"]:
        path = root / entry["name"]
        require(path.is_file(), f"missing payload: {entry['name']}")
        require(path.stat().st_size == entry["bytes"], f"payload size mismatch: {entry['name']}")
        require(sha256(path) == entry["sha256"], f"payload hash mismatch: {entry['name']}")

    vector = json.loads((root / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
    require(vector.get("execution") == "NOT_RUN", "symbolic vectors must remain unexecuted")
    require(len(vector.get("vectors", [])) == 4, "expected four symbolic capacity cases")
    print(f"PASS package={manifest['package_id']}")
    print(f"PASS manifest_sha256={digest}")
    print(f"PASS payload_count={len(manifest['files'])}")
    print("PASS runtime_observation=NOT_RUN; symbolic vectors only")


if __name__ == "__main__":
    main()
