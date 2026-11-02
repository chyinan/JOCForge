"""Verify the sanitized selected StageRunner route supplement."""

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
    require(manifest.get("upstream_manifest_sha256") == "49ca286cf86d221d42413bd5dced050e2413651c88dcef1d453f85e3b028d8b8", "upstream manifest mismatch")
    for entry in manifest["files"]:
        path = root / entry["name"]
        require(path.is_file(), f"missing payload: {entry['name']}")
        require(path.stat().st_size == entry["bytes"], f"payload size mismatch: {entry['name']}")
        require(sha256(path) == entry["sha256"], f"payload hash mismatch: {entry['name']}")
    evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["selected_job_types"]["selected_factory_module"] == "dee_audio_filter_ddp_atmos.dll", "wrong selected module")
    require(evidence["count_boundary"]["reader_to_process_count_transfer"] == "UNBOUND_R2_NOT_EVALUATED", "count transfer must remain unbound")
    require(evidence["count_boundary"]["raw_pcm_pcm_to_ddp_max8_gate_applies"] == "NOT_BOUND", "raw PCM gate must not be conflated")
    print(f"PASS package={manifest['package_id']}")
    print(f"PASS manifest_sha256={digest}")
    print(f"PASS payload_count={len(manifest['files'])}")
    print("PASS selected_atmos_factory; count and raw-PCM gate boundaries preserved")


if __name__ == "__main__":
    main()
