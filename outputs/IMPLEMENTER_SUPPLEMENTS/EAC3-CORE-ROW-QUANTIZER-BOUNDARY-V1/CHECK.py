from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "EAC3-CORE-ROW-QUANTIZER-BOUNDARY-V1"
SOURCE_MANIFEST = "aff6340a983d0e47645df8d6fc0c74b4410425d7287df642f0abc0683cb933d8"
STATUS = "STATIC_ROW_POINTER_PATH_BOUND; SYNTHETIC_FORMULA_MATCH; SAMPLE_CAUSE_UNBOUND"


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
    require(evidence["current_provider_state"]["provider_pin_matches_json"] is True, "provider pin")
    require(evidence["current_provider_state"]["current_window_words_match_dee"] is True, "window equality")
    require(evidence["native_row_path"]["row_pointer_identity"] == "STATIC_TYPED_POINTER_BASE_AND_INDEX_MATCH", "row pointer path")
    require(evidence["native_row_path"]["BAP_value_producer"] == "UNBOUND", "BAP source boundary")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(vector["expected"]["row_power_f32_bits_native_and_rust"] == "3ea00000", "power vector")
    require(vector["expected"]["scalar_code_native_and_rust"] == 1, "quantizer vector")
    require(vector["execution"] == "SYNTHETIC_MODEL_BOUNDARY_ONLY; NATIVE_EMULATOR_NOT_RUN", "vector execution")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS static row pointer path, synthetic power/BAP-1 formula match, and UNBOUND sample rows")


if __name__ == "__main__":
    main()
