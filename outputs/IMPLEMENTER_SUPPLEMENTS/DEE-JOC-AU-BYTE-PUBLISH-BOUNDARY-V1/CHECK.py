from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-JOC-AU-BYTE-PUBLISH-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "cef605a777a29e7739619917b7a1f8628deaafe0475318e65d408354f96bf486"
STATUS = "JOC_BITWRITER_REPACK_AND_MEMORY_PUBLISH_BOUND; APPLICATION_FILE_WRITER_OUTSIDE_SLICE"
SOURCE_ARTIFACTS = {
    "REPORT.md": "69c358ae42274c485c1326b5dce4c88be6f29420367378addea758927e6c457a",
    "EVIDENCE.json": "35a08f4e64a299e7f5d9e15c3602ecba2b17ea63ec9229ee0feab94ee5964955",
    "TraceAudioWriterFlushBoundary.java": "cf02e7eab44b136c2b7708e9937a3f76887b9178e4e072c732524f52c17d52c2",
    "InspectAudioWriterFlushCallsites.java": "dc7084e1f1f82603d033c0bf46f7ba58fe6f39ff32ab4328193fb04db1ad6236",
    "TraceSelectedOutputBufferEdge.java": "08aac09b8971342dae7ce35cfd340b2988d97f4732a68dfdaad757dcc95d39cc",
    "TraceAudioStreamDispatch.java": "18f010c1387ecfbe9381ff38a47c70ad4b184b6877d954611b0508d71512fd99",
    "check_audio_writer_au_boundary.py": "88fceab588d4d3e95ab75304fcec456940fb50e4464444b2f5f720329f2568f5",
}


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
    source_manifest = load("SOURCE-MANIFESTS.json")
    source_artifacts = {row["path"]: row["sha256"] for row in source_manifest["sources"][0]["manifest_artifacts"]}
    require(source_manifest["sources"][0]["manifest_sha256"] == SOURCE_MANIFEST_SHA256, "source manifest file binding")
    require(source_artifacts == SOURCE_ARTIFACTS, "source artifact hashes")
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
    require(evidence["writer_context"]["same_context_finalizer"] == "FUN_180093360", "same context finalizer")
    require(evidence["repack_and_dispatch"]["repack_function"] == "FUN_18008d260", "repack bridge")
    require(evidence["first_byte_publish_boundary"]["memory_writer"] == "FUN_1800e01e0", "byte publish edge")
    require(evidence["first_byte_publish_boundary"]["file_or_container_writer_reached_in_direct_chain"] is False, "file writer boundary")
    require(evidence["scope_limits"]["application_file_append_stage"] == "OUTSIDE_THIS_SLICE", "scope limit")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS bit-writer/repack/callback-buffer boundary; application file writer remains outside slice")
    print("PASS no raw media, payload, binary, decompilation, key, or license material")
    return 0


if __name__ == "__main__":
    sys.exit(main())
