"""Verify the metadata-only 36/38 ADM comparison with inherited input identities."""

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
    require(manifest.get("files") == seal.get("files"), "manifest/seal file list mismatch")
    for entry in manifest["files"]:
        path = root / entry["name"]
        require(path.is_file(), f"missing payload: {entry['name']}")
        require(path.stat().st_size == entry["bytes"], f"payload size mismatch: {entry['name']}")
        require(sha256(path) == entry["sha256"], f"payload hash mismatch: {entry['name']}")

    evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
    inputs = evidence["inputs"]
    require([x["fmt"]["channels"] for x in inputs] == [36, 38], "expected 36/38 channel counts")
    require([x["chna"]["track_count"] for x in inputs] == [36, 38], "expected 36/38 CHNA counts")
    require(all(x["chna"]["records_size_valid"] for x in inputs), "invalid CHNA record area")
    require(all(x["fmt"]["sample_rate_hz"] == 48000 and x["fmt"]["bits_per_sample"] == 24 for x in inputs), "format mismatch")
    require(all(x["inherited_file_sha256"] for x in inputs), "inherited file identities missing")
    require("seek over payload" in evidence["metadata_scan"]["data_chunk_policy"], "data chunk skip policy missing")
    require(evidence["no_pcm_or_codec_claim"]["pcm_bytes_read"] == 0, "PCM bytes must remain unread")
    print(f"PASS package={manifest['package_id']}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS 36/38 metadata and inherited identities; PCM bytes read=0")


if __name__ == "__main__":
    main()
