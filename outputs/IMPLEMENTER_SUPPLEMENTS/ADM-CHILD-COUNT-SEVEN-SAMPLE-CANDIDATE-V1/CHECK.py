from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-CHILD-COUNT-SEVEN-SAMPLE-CANDIDATE-V1"
SOURCE_MANIFEST_SHA256 = "38f49cb279c2a2e68c4adac89d42ddb0da1133ff7c177ecd397c43ec8a3918b7"
STATUS = "METADATA_CANDIDATE_MATCHES_CHNA_7_OF_7; NATIVE_RUNTIME_CHILD_U16_UNBOUND"


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
    require(len(evidence["samples"]) == 7, "sample count")
    require(all(sample["bed_sum"] == 10 and sample["candidate"] == sample["chna_tracks"] for sample in evidence["samples"]), "per-sample count match")
    require(sum(sample["candidate"] == 36 for sample in evidence["samples"]) == 6, "six 36-track samples")
    require(sum(sample["candidate"] == 38 for sample in evidence["samples"]) == 1, "one 38-track sample")
    require(evidence["summary"]["native_bed_vector_identity"] == "UNBOUND/NO", "native identity boundary")
    require(evidence["summary"]["runtime_child_u16"] == "UNBOUND/NO", "runtime U16 boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS metadata candidate matches CHNA for all seven authorized ADM receipts")
    print("PASS runtime child U16 remains UNBOUND; no media/payload/key included")
    return 0


if __name__ == "__main__":
    sys.exit(main())
