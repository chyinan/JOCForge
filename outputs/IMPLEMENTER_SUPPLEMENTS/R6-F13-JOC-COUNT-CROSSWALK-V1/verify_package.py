"""Verify the washed R6 F13/JOC count crosswalk supplement."""

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
    require(manifest.get("upstream_manifest_sha256") == "3135b65e1113b013f6bc4a364c9483bb9c7bde353bc819bb07bc1bbce6c79b43", "upstream manifest mismatch")
    for entry in manifest["files"]:
        path = root / entry["name"]
        require(path.is_file(), f"missing payload: {entry['name']}")
        require(path.stat().st_size == entry["bytes"], f"payload size mismatch: {entry['name']}")
        require(sha256(path) == entry["sha256"], f"payload hash mismatch: {entry['name']}")
    evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["native_count_chain"]["slot_extent"]["producer"] == "FUN_180099830", "slot extent producer mismatch")
    require(evidence["native_count_chain"]["matched_mode_and_slot_value"] == "UNBOUND", "matched slot value must remain UNBOUND")
    require(evidence["execution"]["dee_run"] is False, "DEE runtime must remain NOT_RUN")
    print(f"PASS package={manifest['package_id']}")
    print(f"PASS manifest_sha256={digest}")
    print(f"PASS payload_count={len(manifest['files'])}")
    print("PASS source_count_to_slot_count=UNBOUND runtime=NOT_RUN")


if __name__ == "__main__":
    main()
