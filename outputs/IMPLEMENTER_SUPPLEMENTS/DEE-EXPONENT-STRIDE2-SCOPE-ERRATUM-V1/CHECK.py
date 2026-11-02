from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-EXPONENT-STRIDE2-SCOPE-ERRATUM-V1"
STATUS = "EAC3_CORE_VECTOR_VALID; JOC_CROSS_SCOPE_COMPARISON_WITHDRAWN; JOC_SELECTOR2_NOT_EVALUATED"
SOURCE_PACKAGES = {
    "dee-ddp-exponent-producer-emulator-wash-v7": "6a832b29dc13e629cae3811223ba69e75562d781399b2acc03559b33b172ab73",
    "EAC3-EXPONENT-CODE-PRODUCER-U4-U7-V1": "1ead416c45cac51589738123219f0f2901e90c45ef4bba7e15592672054dc7b1",
    "DEE-NATIVE-JOC-DELTA-WRITER-BOUNDARY-V2": "d5a1e5b458ca00c092212b4027ef4bc0ebbc46d79cd37b13c359242f04df0c04",
    "DEE-GHIDRA-SYNTHETIC-KERNEL-EMULATION-ADDENDUM-V1": "96bb2eb1f5f120e1b8a204dd7e00f50d2a1204b3931942c0e0d28c1191927005",
    "DEE-EXPONENT-STRIDE2-EMULATOR-COMPARISON-V1": "c2dfb9336d20bc000d1610ee6bc5850dae950f834531f3da618fb62db7359485",
}


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
    source_map = {entry["package_id"]: entry for entry in manifest["source_packages"]}
    require(set(source_map) == set(SOURCE_PACKAGES), "source package set")
    for package_id, expected_hash in SOURCE_PACKAGES.items():
        require(source_map[package_id]["manifest_sha256"] == expected_hash, f"source binding {package_id}")
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
    require(evidence["eac3_core_emulator"]["function"] == "FUN_14032d580", "core target")
    require(evidence["eac3_core_emulator"]["observed_core_U7_codes"] == [76, 12], "core vector retained")
    require(evidence["joc_atmos_path"]["transform"] == "FUN_1800a4d70", "JOC transform scope")
    require(evidence["joc_atmos_path"]["joc_selector2_input_output_vector"] == "NOT_EVALUATED", "JOC selector2 remains open")
    require(evidence["withdrawn_cross_scope_comparison"]["valid_joc_mismatch_claim"] is False, "cross-scope claim withdrawn")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS E-AC-3 core and Atmos JOC scopes kept separate")
    print("PASS no JOC selector-2 result inferred from the core vector")
    return 0


if __name__ == "__main__":
    sys.exit(main())
