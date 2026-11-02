from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-JOC-U7-SELECTOR-ROW-PROFILE-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "dc19b6b53cee996264be40ee2ae751f33b521b5790a26d631d19d9f2f1df9229"
STATUS = "CONSUMER_LOOKUP_BOUND; ROWS_0_1_ZERO_DEFAULT; ROWS_2_5_PROFILE_SOURCE_UNBOUND"


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
    require(evidence["selector_consumer"]["lookup_table"] == [0, 1, 2, 4], "JOC selector mapping")
    require(evidence["constructor_defaults"]["bound_selector_rows"] == {"0":0,"1":0}, "row 0/1 defaults")
    require(evidence["constructor_defaults"]["unbound_selector_rows"] == [2, 3, 4, 5], "rows 2-5 unbound")
    require(evidence["profile_copy_boundary"]["writes_inline_selector_rows"] is False, "profile copy boundary")
    require(evidence["selected_profile_disposition"]["rows_2_to_5_values"] == "UNBOUND/NO", "selected profile values")
    require(evidence["execution"]["dee_run"] == "NOT_RUN", "DEE run status")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS per-block JOC lookup and constructor defaults for rows 0/1")
    print("PASS selected profile values for rows 2-5 remain UNBOUND")
    return 0


if __name__ == "__main__":
    main()
