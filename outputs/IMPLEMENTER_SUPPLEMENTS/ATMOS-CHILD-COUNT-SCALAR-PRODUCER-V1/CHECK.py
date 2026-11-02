from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-CHILD-COUNT-SCALAR-PRODUCER-V1"
SOURCE_MANIFEST_SHA256 = "c65ff6716eaa61ee5679717362677c721712a46f80342dbe51255c1b718a5e46"
STATUS = "NATIVE_CHILD_COUNT_PRODUCER_FORMULA_BOUND; SAMPLE_RUNTIME_VECTOR_AND_U16_UNBOUND"


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
    require(evidence["ordinary_getter_path"]["getter_reads_vector_directly"] is False, "getter scalar boundary")
    require(evidence["scalar_producer_path"]["formula"] == "Object source count + sum of per-bed-row channel-vector lengths", "producer formula")
    candidate = evidence["authorized_sample_metadata_candidate"]
    require(candidate["object_track_count"] + candidate["bed_vector_sum"] == candidate["candidate_scalar"] == 38, "metadata candidate")
    require(candidate["native_row_identity"] == "UNBOUND/NO" and candidate["runtime_child_u16"] == "UNBOUND/NO", "runtime boundary")
    require(evidence["alternative_count_mode"]["selected_for_authorized_run"] == "UNBOUND/NO", "mode boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS native scalar producer/getter formula and separate metadata candidate")
    print("PASS runtime row identity, selected mode, and child U16 remain UNBOUND")
    return 0


if __name__ == "__main__":
    sys.exit(main())
