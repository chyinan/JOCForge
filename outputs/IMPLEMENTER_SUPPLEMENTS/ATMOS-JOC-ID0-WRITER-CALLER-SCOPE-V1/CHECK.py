from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ATMOS-JOC-ID0-WRITER-CALLER-SCOPE-V1"
SOURCE_MANIFEST_SHA256 = "c922dc158862f8a6861aff64d6703b10e96cee59010eb3aed30c7c1ae4fafa00"
SOURCE_SEAL_SHA256 = "39fe1c285aea209710907ed42efa59e1dd224ab6b3eca8a7517039fdb73ff9c0"
EXPECTED_STATUS = "IN_IMAGE_BASE_WRITER_CALLER_SCOPE_BOUND; ID0_WRITER_AND_RUN_VALUE_UNBOUND"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {message}")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> int:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = load("MANIFEST.json")
    manifest_hash = digest(manifest_path)
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    require(manifest.get("status") == EXPECTED_STATUS, "manifest status")
    require(manifest.get("source_reverse_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest binding")
    require(manifest.get("source_reverse_package_seal_sha256") == SOURCE_SEAL_SHA256, "source seal binding")

    payload = {entry["file"]: entry for entry in manifest["files"]}
    actual_payload = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
    require(actual_payload == set(payload), "payload file set")
    for name, entry in payload.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"payload size {name}")
        require(digest(path) == entry["sha256"], f"payload hash {name}")

    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal.get("package_id") == PACKAGE_ID, "seal package id")
    require(seal.get("file_count") == len(sealed), "seal file count")
    require(set(sealed) == set(payload) | {"MANIFEST.json", "MANIFEST.sha256"}, "sealed file set")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(digest(path) == entry["sha256"], f"sealed hash {name}")

    evidence = load("EVIDENCE.json")
    require(evidence.get("status") == EXPECTED_STATUS, "evidence status")
    require(evidence["selected_id0_lookup"]["lookup_requests_type_id"] == 0, "selected ID")
    require(evidence["selected_id0_lookup"]["selector_getter"].endswith("vtable+0x38"), "getter slot")
    require(evidence["base_selector_field"]["literal_default_assigned"] is False, "no literal default")
    callers = evidence["in_image_direct_constructor_callers"]
    require(len(callers) == 1 and callers[0]["visible_type_ids"] == [3, 4, 5], "scoped direct caller")
    require(callers[0]["selected_by_id0_lookup"] is False, "Rerender candidate exclusion")
    scope = evidence["scope_limit"]
    require(scope["id0_concrete_writer_established"] is False, "ID0 writer remains open")
    require(scope["external_module_construction_excluded"] is False, "external boundary retained")
    require(scope["per_run_selector_value"] == "UNBOUND/NO", "run value boundary")
    require(evidence["runtime_contract"]["preserve_conditional"] is True, "conditional contract")
    require(evidence["observation_scope"]["dee_run"] == "NOT_RUN", "DEE status")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(vector.get("execution") == "NOT_RUN", "input-only status")

    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print(f"PASS status={EXPECTED_STATUS}")
    print("PASS scoped constructor caller, excluded IDs 3/4/5, ID-0 writer/value remain UNBOUND")
    print("PASS input-only vector remains NOT_RUN; no private material or media payload")
    return 0


if __name__ == "__main__":
    sys.exit(main())
