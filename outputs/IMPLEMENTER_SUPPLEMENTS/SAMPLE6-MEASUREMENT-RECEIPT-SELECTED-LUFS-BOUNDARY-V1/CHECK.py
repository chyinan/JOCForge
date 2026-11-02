from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-MEASUREMENT-RECEIPT-SELECTED-LUFS-BOUNDARY-V1"
SOURCE_MANIFEST_SHA256 = "779264b7c2fc3baabcaf7bc7d88bd46da1cc3c66a9b089a09b3f915eed1e5a2b"
STATUS = "SELECTED_LUFS_WORD_PRESENT_IN_TYPED_RECEIPT; OMITTED_FROM_MANAGED_RUN_JSON"


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
    require(evidence["typed_receipt"]["selected_lufs_word_exists"] is True, "typed receipt field")
    require(evidence["typed_receipt"]["serde_serialize_derive"] is False, "non-serialized typed field")
    require(evidence["serialization_boundary"]["run_json_selected_lufs_word"] is False, "run JSON omission")
    require(evidence["minimum_non_sensitive_receipt_change"]["derive_from_U5_or_digest"] is False, "no inverse inference")
    require(evidence["run"]["selected_lufs_word_in_run_json"] is None, "run value remains absent")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS typed selected-LUFS receipt field and managed run JSON loss boundary")
    print("PASS no new DEE/audio payload/key access; no scalar inferred")
    return 0


if __name__ == "__main__":
    main()
