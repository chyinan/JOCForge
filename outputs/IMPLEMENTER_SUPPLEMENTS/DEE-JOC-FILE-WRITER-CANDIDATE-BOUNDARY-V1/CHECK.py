from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-JOC-FILE-WRITER-CANDIDATE-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "24fbae9b8d0f16c490ca0f7cdd6943bbed38dd444cde6d0384638488f825619c"
STATUS = "PER_UNIT_MEMORY_PUBLISH_BOUND; FILE_WRITER_CANDIDATE_NOT_JOINED"


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> int:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID, "package id")
    require(manifest["status"] == STATUS, "status")
    require(manifest["source_reverse_manifest_sha256"] == SOURCE_MANIFEST_SHA256, "source manifest binding")
    payload = {entry["file"]: entry for entry in manifest["files"]}
    actual = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
    require(actual == set(payload), "payload inventory")
    for name, entry in payload.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID and seal["file_count"] == len(sealed), "seal identity")
    require(set(sealed) == set(payload) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")
    evidence = load("EVIDENCE.json")
    require(evidence["status"] == STATUS, "evidence status")
    require(evidence["per_unit_publish"]["memory_writer"] == "FUN_1800e01e0", "memory boundary")
    require(evidence["file_writer_candidate"]["implementation"] == "FUN_180189c80", "file writer candidate")
    require(evidence["file_writer_candidate"]["per_unit_buffer_join_to_param2"] == "UNBOUND/NO", "buffer/file join")
    require(evidence["file_writer_candidate"]["FILE_pointer_join_to_ec3_output"] == "UNBOUND/NO", "output FILE join")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS caller memory publish and separate unjoined fwrite candidate")
    return 0


if __name__ == "__main__":
    main()
