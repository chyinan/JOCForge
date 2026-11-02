from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-READER-CONFIG-U16-PRODUCER-V1"
SOURCE_MANIFEST = "c8451947f82cae591b9112adb6b9e7f562b332772aeaf6c401dca614b2e3223d"
STATUS = "READER_CONFIG_U16_PRODUCER_BOUND; HUMAN_CAT_RUNTIME_U16_UNBOUND"


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
    owner = evidence["typed_owner_and_descriptor"]
    require(owner["descriptor_count_source"] == "U16 reader config+0x0A", "descriptor count source")
    require(owner["descriptor_count_destination"] == "U64 descriptor+0x08, zero-extended", "descriptor count destination")
    producer = evidence["config_producer"]
    require(producer["count_source_write"] == "U16 at CompositeMaster+0x1A", "count producer")
    require(producer["fixed_36_limit_or_default_in_traced_writer"] == "NONE_OBSERVED", "36-cap boundary")
    sample = evidence["human_cat_context"]
    require(sample["bed_tracks"] + sample["object_tracks"] == sample["chna_tracks"] == 38, "metadata candidate")
    require(sample["same_run_reader_u16"] == "UNBOUND", "runtime boundary")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(vector["static_rule"]["candidate_core_map_bytes_if_u16_is_38"] == 228, "candidate allocation")
    require(vector["static_rule"]["runtime_u16"] == "UNBOUND", "vector runtime boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS reader U16 producer/descriptor copy, 38-row candidate, and UNBOUND runtime value")


if __name__ == "__main__":
    main()
