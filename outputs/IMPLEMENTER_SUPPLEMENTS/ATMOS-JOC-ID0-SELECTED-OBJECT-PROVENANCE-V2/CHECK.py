from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-JOC-ID0-SELECTED-OBJECT-PROVENANCE-V2"
EXPECTED_STATUS = "ID0_LOOKUP_AND_SELECTOR_CONTRACT_BOUND; LOCAL_OPAQUE_VECTOR_NOT_JOINED; ID0_PRODUCER_AND_RUN_VALUE_UNBOUND"
SOURCE_BINDINGS = {
    "atmos_joc_selector_id0_config_wash_v1": (
        "bcd366ee63916707421278c34ba430b8b35bede053e65f6f8de13963b35ce8bc",
        "960fafcb65249bb846a7dbd46e7dd0aad966a04b848f9078cdf4fb361ee94645",
    ),
    "atmos_joc_id0_selector_source_wash_v1": (
        "c922dc158862f8a6861aff64d6703b10e96cee59010eb3aed30c7c1ae4fafa00",
        "39fe1c285aea209710907ed42efa59e1dd224ab6b3eca8a7517039fdb73ff9c0",
    ),
    "atmos_joc_id0_local_factory_origin_wash_v1": (
        "a40ce4c4669aa9b2a16ec60f32996ad45350586ac266b82f2fd178aebb7779be4",
        "a857dbe13ab2ed6a7f7ee692fb45e8e97bdbf9d3770df7695a1c639b563dcf30",
    ),
}


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> int:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(manifest_path)
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    require(manifest.get("status") == EXPECTED_STATUS, "manifest status")
    source_map = {row["package_id"]: row for row in manifest["source_packages"]}
    require(set(source_map) == set(SOURCE_BINDINGS), "source package set")
    for package_id, (manifest_hash_expected, seal_hash_expected) in SOURCE_BINDINGS.items():
        row = source_map[package_id]
        require(row["manifest_sha256"] == manifest_hash_expected, f"source manifest {package_id}")
        require(row["package_seal_sha256"] == seal_hash_expected, f"source seal {package_id}")

    payload = {entry["file"]: entry for entry in manifest["files"]}
    actual_payload = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
    require(actual_payload == set(payload), "payload file set")
    for name, entry in payload.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"payload size {name}")
        require(sha256(path) == entry["sha256"], f"payload hash {name}")

    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal.get("package_id") == PACKAGE_ID, "seal package id")
    require(seal.get("file_count") == len(sealed), "seal count")
    require(set(sealed) == set(payload) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal file set")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")

    evidence = load("EVIDENCE.json")
    require(evidence["status"] == EXPECTED_STATUS, "evidence status")
    require(evidence["renderer_selected_id0_chain"]["lookup_type_id"] == 0, "ID-0 lookup")
    require(evidence["renderer_selected_id0_chain"]["selector_getter_slot"] == "+0x38", "selector slot")
    local = evidence["local_plugin_factory_chain"]
    require(local["element_stride_bytes"] == 32, "opaque vector stride")
    require(local["opaque_vector_typed_as_renderbus_config"] == "UNBOUND/NO", "opaque type boundary")
    require(local["opaque_vector_joined_to_renderer_renderbus_vector"] == "UNBOUND/NO", "vector identity boundary")
    require(local["constructs_selected_id0_object"] is False, "local constructor boundary")
    field = evidence["base_selector_field_contract"]
    require(field["literal_default_assigned"] is False, "no literal default")
    require(field["visible_type_ids_of_that_caller"] == [3, 4, 5], "Rerender caller IDs")
    require(field["caller_is_selected_by_id0_lookup"] is False, "Rerender exclusion")
    contract = evidence["runtime_contract"]
    require(contract["id0_concrete_producer_or_writer"] == "UNBOUND/NO", "ID-0 producer remains open")
    require(contract["per_run_selector_value"] == "UNBOUND/NO", "per-run value remains open")
    require(contract["preserve_typed_conditional"] is True, "conditional preserved")
    require(evidence["observation_scope"]["dee_run"] == "NOT_RUN", "DEE run state")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(vector.get("execution") == "NOT_RUN", "input-only status")

    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print(f"PASS status={EXPECTED_STATUS}")
    print("PASS three hash-bound sources; plugin vector stays distinct and opaque")
    print("PASS ID-0 producer/value UNBOUND; no default inferred; no payload/key material")
    return 0


if __name__ == "__main__":
    sys.exit(main())
