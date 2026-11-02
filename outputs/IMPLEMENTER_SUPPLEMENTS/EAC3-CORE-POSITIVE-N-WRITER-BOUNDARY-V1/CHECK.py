from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "EAC3-CORE-POSITIVE-N-WRITER-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "15278b6103e7f4b2abe9018d6cb406f4f881383ca36641d7ef1b4c4cc980ac68"
STATUS = "N_CONSUMER_AND_ZERO_RESET_BOUND; POSITIVE_N_WRITER_UNBOUND"


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
    require(evidence["positive_write_disposition"]["positive_N_write_found"] is False, "positive N remains unbound")
    require(evidence["positive_write_disposition"]["adjacent_positive_write"]["copy_into_D_or_N_found"] is False, "adjacent field not joined")
    require(evidence["best_supported_candidate"]["classification"] == "implementation inference only", "inference grade")
    require(evidence["open_edges"]["JOC_FUN_180_path_relation"] == "NOT_ESTABLISHED", "core/JOC separation")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS E-AC-3 core N consumer/reset facts and positive-writer boundary")
    print("PASS candidate N inference retained; no JOC extrapolation")
    return 0


if __name__ == "__main__":
    main()
