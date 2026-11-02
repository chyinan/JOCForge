from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-EXPONENT-STRIDE2-EMULATOR-COMPARISON-V1"
SOURCE_MANIFEST_SHA256 = "6a832b29dc13e629cae3811223ba69e75562d781399b2acc03559b33b172ab73"
STATUS = "R2_SYNTHETIC_SELECTOR2_VECTOR_WASHED; RUST_SELECTOR2_MATCH_NOT_DEMONSTRATED"


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def candidate_values(coefficients: list[int], stride: int, output_count: int) -> list[int]:
    values = [coefficients[index] - coefficients[index - stride] + 2 for index in range(stride, len(coefficients), stride)]
    return (values + [2] * output_count)[:output_count]


def main() -> int:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID, "package id")
    require(manifest["status"] == STATUS, "status")
    require(manifest["source_manifest_sha256"] == SOURCE_MANIFEST_SHA256, "source manifest binding")
    payload = {entry["file"]: entry for entry in manifest["files"]}
    actual = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
    require(actual == set(payload), "payload inventory")
    for name, entry in payload.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")

    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID, "seal package id")
    require(seal["file_count"] == len(sealed), "seal count")
    require(set(sealed) == set(payload) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")

    evidence = load("EVIDENCE.json")
    native = evidence["observed_native_emulator_result"]
    require(native["biased_differences"] + native["tail_slots"] == [3, 0, 1, 0, 2, 2], "native delta/tail vector")
    require(native["U7_codes"] == [76, 12], "native U7 codes")
    vector = load("INPUT-ONLY-VECTORS.json")
    cases = {case["id"]: case for case in vector["vectors"]}
    raw = cases["r2-native-emulator-selector2"]["raw_row"]
    compact = cases["r2-native-emulator-selector2"]["observed_selected_exponents"]
    require(candidate_values(raw, 2, 6) == [91, 2, 2, 2, 2, 2], "raw row selector-2 calculation")
    require(candidate_values(compact, 2, 6) == [1, -1, 2, 2, 2, 2], "compact selector-2 calculation")
    require(candidate_values(compact, 1, 6) == [3, 0, 1, 0, 2, 2], "compact adjacent calculation")
    require(evidence["rust_comparison"]["implementation_head"] == "798b7667ca39f89aba1ec2cad9e5425d886aa990", "reviewed HEAD")
    require(evidence["rust_comparison"]["matches_R2_selector2_native_delta_tail"] is False, "selector-2 mismatch status")
    require(evidence["rust_comparison"]["HEAD_added_regression_selector"] == 1, "regression selector")
    require(evidence["unresolved"]["native_output_parity"] == "NOT_RUN", "native parity boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS R2 synthetic selector-2 native vector and Rust source calculations remain distinct")
    print("PASS no Rust function run, no DEE process run, no media/key/binary/decompilation payload")
    return 0


if __name__ == "__main__":
    sys.exit(main())
