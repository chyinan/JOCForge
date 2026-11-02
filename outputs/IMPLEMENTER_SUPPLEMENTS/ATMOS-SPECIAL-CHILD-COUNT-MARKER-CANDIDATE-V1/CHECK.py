from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-SPECIAL-CHILD-COUNT-MARKER-CANDIDATE-V1"
SOURCE_MANIFEST = "e8e1b6fea8bc7cd69dbedcd51b0e6eb6701ed61e7561ba2084a5c9b1e62d2ed6"
STATUS = (
    "SPECIAL_MARKER_PREDICATE_AND_WRITER_BOUND; SEVEN_SAMPLE_METADATA_CANDIDATE_SUPPORTED; "
    "RUNTIME_CHILD_PARTITION_AND_U16_UNBOUND"
)


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> None:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID, "package id")
    require(manifest["status"] == STATUS, "status")
    require(manifest["source_reverse_manifest_sha256"] == SOURCE_MANIFEST, "source manifest")
    entries = {entry["file"]: entry for entry in manifest["files"]}
    actual = {
        path.name for path in ROOT.iterdir()
        if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
    }
    require(actual == set(entries), "payload inventory")
    for name, entry in entries.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID, "seal id")
    require(set(sealed) == set(entries) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")

    evidence = load("EVIDENCE.json")
    require(evidence["status"] == STATUS, "evidence status")
    rule = evidence["native_rule"]
    require(rule["output_formula"] == "10 + max(marked_records_per_child)", "native formula")
    require(rule["consumer_predicate"].endswith("+2 equals 0xFF"), "tag predicate")
    require(rule["runtime_mode_byte"] == "NOT_CAPTURED", "mode boundary")
    require(rule["runtime_child_u16"] == "UNBOUND", "runtime U16 boundary")
    require(evidence["implementation_trial"]["ready"] is True, "candidate trial readiness")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(len(vector["samples"]) == 7, "sample count")
    for sample in vector["samples"]:
        require(sample["candidate"] == 10 + sample["chna_objects"], "candidate formula")
        require(sample["candidate"] == sample["chna_tracks"], "CHNA comparison")
        require(sample["axml_object_nodes_including_bed"] == sample["chna_objects"] + 1, "AXML distinction")
    require(vector["runtime_partition"] == "UNBOUND", "partition boundary")
    require(vector["checks"]["candidate_is_not_runtime_u16"] is True, "candidate scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS native marker formula, seven conditional candidates, and UNBOUND runtime partition/U16")


if __name__ == "__main__":
    main()
