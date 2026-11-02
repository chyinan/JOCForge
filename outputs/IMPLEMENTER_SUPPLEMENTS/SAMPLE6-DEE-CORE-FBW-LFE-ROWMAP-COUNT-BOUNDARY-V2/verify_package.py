"""Verify the table-derived row-map count producer supplement."""

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
    require(manifest.get("upstream_manifest_sha256") == "569a1c870494b4697c3343daed4e6fcdc4c2b81e120273d76c93e1c076db24cb", "upstream manifest mismatch")
    for entry in manifest["files"]:
        path = root / entry["name"]
        require(path.is_file(), f"missing payload: {entry['name']}")
        require(path.stat().st_size == entry["bytes"], f"payload size mismatch: {entry['name']}")
        require(sha256(path) == entry["sha256"], f"payload hash mismatch: {entry['name']}")
    evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["domain_separation"]["adm_track_count_to_slot_count"] == "UNBOUND", "ADM-to-slot join must remain UNBOUND")
    require(evidence["domain_separation"]["runtime_values"] == "NOT_RUN", "runtime must remain NOT_RUN")
    vector = json.loads((root / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
    require(len(vector["selectors"]) == 8 and vector["execution"] == "NOT_RUN", "symbolic selector vector mismatch")
    print(f"PASS package={manifest['package_id']}")
    print(f"PASS manifest_sha256={digest}")
    print(f"PASS payload_count={len(manifest['files'])}")
    print("PASS ADM_count_join=UNBOUND runtime=NOT_RUN")


if __name__ == "__main__":
    main()
