"""Verify the sanitized StageRunner AtmosMezz route supplement."""

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
        == "c4698113b15fa1524e7161c79ce8ffd841e73db6eb268c627fb66bbd2e6be2e2",
        "unexpected upstream manifest identity",
    )
    require(manifest.get("files") == seal.get("files"), "manifest/seal payload list mismatch")
    for entry in manifest["files"]:
        path = root / entry["name"]
        require(path.is_file(), f"missing payload: {entry['name']}")
        require(path.stat().st_size == entry["bytes"], f"payload size mismatch: {entry['name']}")
        require(sha256(path) == entry["sha256"], f"payload hash mismatch: {entry['name']}")
    evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["selected_job_types"]["input_type"] == "atmos_mezz", "wrong selected input type")
    require(evidence["non_equivalent_raw_pcm_route"]["raw_pcm_max_8_applies_to_selected_atmos_mezz_job"] == "NOT_BOUND", "must not conflate raw PCM route")
    require(evidence["unbound_edges"][2].startswith("first 36/38 count gate"), "count gate must remain UNBOUND")
    print(f"PASS package={manifest['package_id']}")
    print(f"PASS manifest_sha256={digest}")
    print(f"PASS payload_count={len(manifest['files'])}")
    print("PASS selected_input=atmos_mezz raw_pcm_gate_not_applied without join")


if __name__ == "__main__":
    main()
