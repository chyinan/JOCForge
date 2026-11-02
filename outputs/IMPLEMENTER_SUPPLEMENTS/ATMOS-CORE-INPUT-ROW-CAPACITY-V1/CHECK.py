from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-CORE-INPUT-ROW-CAPACITY-V1"
SOURCE_MANIFEST = "f8df33ceb4eecf306b3e2356ad00ba12f2f103efab1375d822b6c336f4e4320a"
STATUS = "READER_TO_CORE_DYNAMIC_ROW_CAPACITY_BOUND; HUMAN_CAT_RUNTIME_COUNT_UNBOUND"


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
    require(manifest["package_id"] == PACKAGE_ID and manifest["status"] == STATUS, "manifest id/status")
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
    rule = evidence["reader_to_core_rule"]
    require(rule["reader_row_count_type"] == "U16" and rule["row_width_bytes"] == 6, "reader contract")
    require(rule["core_owned_copy"] == "allocate 6*count bytes; copy rows index 0..count-1 in original order", "copy rule")
    require(rule["fixed_36_channel_cap_in_traced_path"] == "NONE_OBSERVED", "36 cap scope")
    sample = evidence["human_cat_metadata_candidate"]
    require(sample["chna_tracks"] == 38 and sample["bed_tracks"] + sample["object_tracks"] == 38, "metadata context")
    require(sample["candidate_allocation_bytes"] == 38 * 6 == 228, "candidate allocation")
    require(sample["runtime_reader_u16"] == "UNBOUND", "runtime count boundary")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(vector["expected_static_representation"]["allocation_bytes"] == 228, "vector allocation")
    require(vector["target_sample_context"]["runtime_u16"] == "UNBOUND", "vector runtime boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS dynamic 6-byte row capacity, 38-row candidate, and UNBOUND runtime U16")


if __name__ == "__main__":
    main()
