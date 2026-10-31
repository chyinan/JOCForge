"""Validate the sanitized writer-field producer-gap package."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Any


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"expected JSON object: {path.name}")
    return value


def main() -> None:
    root = Path(__file__).resolve().parent
    manifest = read_json(root / "MANIFEST.json")
    evidence = read_json(root / "EVIDENCE.json")

    require(manifest.get("schema") == "wash-evidence-manifest.v1", "manifest schema mismatch")
    require(manifest.get("status") == "FOR_WASH_REVIEW", "status must remain FOR_WASH_REVIEW")
    require(manifest.get("package_id") == "native-measurement-dialnorm-writer-producer-gap-wash-v3", "package id mismatch")
    require(manifest.get("recipient_role") == "wash-agent", "recipient must remain Wash")
    require(manifest.get("implementation_agent_contacted") is False, "implementation contact must remain false")
    for key in ("contains_raw_addresses", "contains_binary_or_decompilation_excerpt", "contains_pcm_or_bitstream", "contains_private_auth_or_key", "contains_local_absolute_paths"):
        require(manifest.get(key) is False, f"sanitization flag changed: {key}")

    listed: set[str] = set()
    for entry in manifest.get("files", []):
        name = entry["name"]
        listed.add(name)
        path = root / name
        require(path.is_file(), f"missing package file: {name}")
        require(sha256(path) == entry["sha256"], f"SHA-256 mismatch: {name}")
        require(path.stat().st_size == entry["bytes"], f"byte count mismatch: {name}")
        require(path.suffix.lower() not in {".wav", ".bw64", ".pcm", ".ec3", ".dll", ".exe", ".xml"}, f"forbidden payload/source type: {name}")
    expected = {"REPORT.md", "EVIDENCE.json", "NEXT-OBSERVATION-PLAN.md", "validate_writer_producer_gap_package.py"}
    require(listed == expected, "package file set changed")

    for path in [root / "MANIFEST.json", *(root / name for name in listed)]:
        content = path.read_text(encoding="utf-8")
        require(not re.search(r"(?i)\b[A-Z]:[\\/]", content), f"local absolute path found: {path.name}")
        require(not re.search(r"(?i)0x[0-9a-f]{6,}", content), f"raw address found: {path.name}")
        require("DECOMPILED" + "_BEGIN" not in content, f"decompilation excerpt found: {path.name}")

    hit = evidence.get("runtime_helper_call", {})
    run11_scope = evidence.get("sample_scope", {})
    require(run11_scope.get("run11_execution_config_sha256") == "NOT_CAPTURED", "run11 config hash must remain NOT_CAPTURED")
    require(run11_scope.get("run11_input_type") == "NOT_CAPTURED_FROM_RETAINED_RUN_METADATA", "run11 input type must remain NOT_CAPTURED")
    require(run11_scope.get("run11_filter_or_route_key") == "NOT_CAPTURED", "run11 filter/route key must remain NOT_CAPTURED")
    require(run11_scope.get("run11_factory_index") == "NOT_CAPTURED", "run11 factory index must remain NOT_CAPTURED")
    require(run11_scope.get("run11_standalone_manifest") == "NOT_CREATED", "run11 standalone manifest status changed")
    require(hit.get("helper_argument_shape") == ["field_pointer", "element_count", "width_bits"], "helper argument shape changed")
    require(hit.get("element_count") == 1, "element count changed")
    require(hit.get("width_bits") == 5, "width changed")
    require(hit.get("member_scalar_value") == "NOT_CAPTURED", "member scalar must remain uncaptured")
    run11_scope = evidence.get("sample_scope", {})
    require(run11_scope.get("run11_execution_config_sha256") == "NOT_CAPTURED", "run11 config hash must remain NOT_CAPTURED")
    require(run11_scope.get("run11_input_type") == "NOT_CAPTURED_FROM_RETAINED_RUN_METADATA", "run11 input type must remain NOT_CAPTURED")
    require(run11_scope.get("run11_filter_or_route_key") == "NOT_CAPTURED", "run11 filter/route key must remain NOT_CAPTURED")
    require(run11_scope.get("run11_factory_index") == "NOT_CAPTURED", "run11 factory index must remain NOT_CAPTURED")
    require(run11_scope.get("run11_standalone_manifest") == "NOT_CREATED", "run11 standalone manifest status changed")
    require(evidence.get("static_field_lineage", {}).get("field_semantic_tag") == "UNBOUND", "field tag must remain unbound")
    require(evidence.get("static_field_lineage", {}).get("typed_producer") == "UNBOUND", "typed producer must remain unbound")
    require(evidence.get("static_field_lineage", {}).get("owner_and_generation") == "UNBOUND", "owner/generation must remain unbound")
    require(evidence.get("measurement_and_output_relations", {}).get("measurement_to_f06nd_drc") == "UNBOUND", "measurement/DRC join must remain unbound")
    require(evidence.get("measurement_and_output_relations", {}).get("f06nd_drc_to_f12_bsi_dialnorm") == "UNBOUND", "DRC/BSI join must remain unbound")
    require(evidence.get("next_observation_plan", {}).get("status") == "PROPOSED_NOT_RUN_PENDING_WASH_COORDINATION", "next plan must remain unrun")
    require(evidence.get("next_observation_plan", {}).get("raw_pointer_persisted") is False, "raw pointer persistence must remain false")
    require(evidence.get("next_observation_plan", {}).get("repeat_if_no_hit") is False, "plan must not repeat if not reached")

    print("package_files=HASH_AND_SIZE_MATCH")
    print("runtime_helper=FIELD_POINTER; COUNT_1; WIDTH_5; FIELD_VALUE_NOT_CAPTURED")
    print("field_producer_and_measurement_dialnorm_join=UNBOUND")
    print("next_observation=PROPOSED_NOT_RUN_PENDING_WASH_COORDINATION")


if __name__ == "__main__":
    main()
