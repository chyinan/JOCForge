from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-JOC-ID0-SELECTED-OBJECT-PROVENANCE-V1"
SOURCE_MANIFEST_SHA256 = "bcd366ee63916707421278c34ba430b8b35bede053e65f6f8de13963b35ce8bc"
SOURCE_SEAL_SHA256 = "960fafcb65249bb846a7dbd46e7dd0aad966a04b848f9078cdf4fb361ee94645"
EXPECTED_STATUS = "ID0_LOOKUP_AND_GETTER_EDGE_BOUND; ID0_CONCRETE_TYPE_AND_RUN_VALUE_UNBOUND"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> int:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = read_json("MANIFEST.json")
    manifest_hash = sha256(manifest_path)
    expected_manifest_hash = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
    require(manifest_hash == expected_manifest_hash, "manifest hash")
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    require(manifest.get("status") == EXPECTED_STATUS, "status")
    require(manifest.get("source_reverse_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest binding")
    require(manifest.get("source_reverse_package_seal_sha256") == SOURCE_SEAL_SHA256, "source seal binding")

    seal = read_json("package_seal.json")
    expected_files = {entry["file"]: entry for entry in manifest["files"]}
    require(seal.get("package_id") == PACKAGE_ID, "seal package id")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal.get("file_count") == len(sealed), "seal file count")
    for name, entry in expected_files.items():
        path = ROOT / name
        require(path.is_file(), f"missing {name}")
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    require(set(sealed) == set(expected_files) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal file set")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.is_file(), f"missing sealed {name}")
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")

    evidence = read_json("EVIDENCE.json")
    require(evidence.get("status") == EXPECTED_STATUS, "evidence status")
    edge = evidence["selected_id0_edge"]
    require(edge["lookup_requests_type_id"] == 0, "ID-0 lookup")
    require(edge["selector_getter_slot"] == "+0x38", "selector getter slot")
    require(edge["factory_argument"] == 5, "factory argument")
    plugin = evidence["adjacent_plugin_config_path"]
    require(plugin["typed_renderbus_vector_identity_established"] is False, "plugin vector identity boundary")
    require(plugin["constructs_selected_id0_object"] is False, "plugin ID-0 construction boundary")
    require(plugin["writes_selected_id0_plus_0x58"] is False, "plugin selector storage boundary")
    require(evidence["excluded_rerender_candidate"]["applicable_to_selected_id0"] is False, "Rerender candidate exclusion")
    runtime = evidence["best_supported_runtime_contract"]
    require(runtime["per_run_id0_class"] == "UNBOUND/NO", "ID-0 class boundary")
    require(runtime["per_run_selector_value"] == "UNBOUND/NO", "run value boundary")
    require(runtime["hardcoded_default_allowed"] is False, "no invented default")
    require(evidence["run_scope"]["new_dee_run"] == "NOT_RUN", "DEE run status")

    vector = read_json("INPUT-ONLY-VECTOR.json")
    require(vector.get("execution") == "NOT_RUN", "input vector execution status")
    require([case["expected_existing_branch"] for case in vector["cases"]] == ["SpatialCoderPlatform", "PreProcessorBypass"], "conditional cases")
    require(vector["fixed"]["selected_config_type_id"] == 0, "fixed selected type")

    allowed = set(sealed) | {"package_seal.json"}
    actual = {path.name for path in ROOT.iterdir() if path.is_file()}
    require(actual == allowed, "unexpected package file")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print(f"PASS status={EXPECTED_STATUS}")
    print("PASS source hashes, selected-ID0 edge, candidate exclusion, and UNBOUND boundaries")
    print("PASS no private material, raw media, PCM, binary, or decompilation payload")
    return 0


if __name__ == "__main__":
    sys.exit(main())
