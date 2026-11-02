from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-NATIVE-F32-TO-BSI-U5-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "7ca61364dae1601e090bc78f18d40ea075c23cacbc9eadf6fe469c9cf135424b"
STATUS = "RUST_DIALNORM_RULE_BOUND; DEE_F32_PRODUCER_AND_U5_WRITER_UNJOINED"


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
    require(evidence["dee_measurement_path"]["aggregate_join_to_EmpireEnc_open_config"] == "UNBOUND", "aggregate/open-config boundary")
    require(evidence["dee_measurement_path"]["result_written_to_BSI"] == "NOT_FOUND_IN_APPROVED_TRACE", "no BSI write claim")
    require(evidence["separate_ddpi_path"]["join_to_DEE_F32_path"] == "UNBOUND", "DDPi/F32 boundary")
    require(evidence["rust_run"]["selected_lufs_word_in_receipt"] is None, "Rust scalar remains absent")
    require(evidence["bounded_reference"]["scope"] == "bounded reference window, not Rust full-file", "scope distinction")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS Rust selected word, DEE F32, DDPi ID0x12, and BSI U5 remain distinct until a typed join")
    return 0


if __name__ == "__main__":
    main()
