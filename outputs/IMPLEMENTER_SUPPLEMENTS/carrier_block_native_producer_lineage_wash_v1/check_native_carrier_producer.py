#!/usr/bin/env python3
"""Verify the scoped carrier-to-native-producer Wash addendum."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def load_json(name: str) -> object:
    try:
        return json.loads((ROOT / name).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise RuntimeError(f"invalid JSON {name}: {exc}") from exc


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_inventory(entries: object, label: str) -> set[str]:
    require(isinstance(entries, list), f"{label} inventory missing")
    names: set[str] = set()
    for entry in entries:
        require(isinstance(entry, dict), f"invalid {label} entry")
        name = entry.get("file")
        require(isinstance(name, str) and Path(name).name == name, f"unsafe {label} path")
        require(name not in names, f"duplicate {label} file")
        names.add(name)
        path = ROOT / name
        require(path.is_file(), f"missing {label} file: {name}")
        require(path.stat().st_size == entry.get("bytes"), f"size mismatch: {name}")
        require(sha256(path) == entry.get("sha256"), f"SHA-256 mismatch: {name}")
        if path.suffix.lower() == ".json":
            load_json(name)
    return names


def main() -> None:
    manifest = load_json("MANIFEST.json")
    seal = load_json("package_seal.json")
    require(manifest.get("package_id") == "carrier_block_native_producer_lineage_wash_v1", "package id mismatch")
    require(manifest.get("implementation_ready") == "NO", "implementation readiness must remain NO")
    payloads = verify_inventory(manifest.get("payloads"), "payload")
    sealed = verify_inventory(seal.get("files"), "seal")
    require(seal.get("seal_file_excluded_from_own_hash_list") is True, "seal policy mismatch")
    require(seal.get("file_count") == len(sealed), "seal count mismatch")
    require(sealed == payloads | {"MANIFEST.json"}, "sealed inventory mismatch")

    finding = load_json("FINDING.json")
    require(finding.get("status") == "NATIVE_REQUEST_LENGTH_TO_PLAN_BOUND; APPROVED_PROFILE_TO_NATIVE_SEMANTIC_CARRIER_UNBOUND", "finding status mismatch")
    scope = finding.get("selected_scope", {})
    require(scope.get("approved_carrier_block") == 2, "selected block must remain 2")
    require(scope.get("complete_emdf_container_max_bytes") == 511, "container cap mismatch")
    require(scope.get("single_fragment_required") is True, "selected scope must remain single-fragment")
    require(scope.get("arbitrary_0_to_5_placement_required") is False, "unapproved placement expansion")
    require(scope.get("fragmentation_over_511_required") is False, "unapproved fragmentation expansion")
    require(scope.get("product_policy_modified") is False, "approved policy must remain unchanged")
    require(scope.get("whole_f13_readiness") == "NOT_EVALUATED", "whole F13 readiness must remain not evaluated")
    require(finding.get("implementation_ready") == "NO", "whole readiness must remain NO")

    model = finding.get("frozen_model_contract", {})
    require(model.get("carrier_block_type") == "usize", "model field type mismatch")
    require(model.get("domain") == [0, 1, 2, 3, 4, 5], "model domain mismatch")
    require(model.get("selected_value") == 2, "model selected value mismatch")
    require(model.get("native_owner_join") == "UNBOUND", "model/native owner join must remain UNBOUND")

    native = finding.get("native_static_edge", {})
    require(native.get("request_type") == "u32 byte count", "native request type mismatch")
    require(native.get("plan_output_type") == "u32 first_block in the mutable transport plan", "native plan type mismatch")
    require(native.get("positive_request_rule") == [
        {"request_bytes_min": 1, "request_bytes_max": 2040, "first_block": 2},
        {"request_bytes_min": 2041, "request_bytes_max": 2551, "first_block": 1},
        {"request_bytes_min": 2552, "request_bytes_max": None, "first_block": 0},
    ], "length-to-plan thresholds mismatch")
    require(native.get("zero_request_rule", "").startswith("The positive-request branch does not assign"), "zero request boundary mismatch")
    third = native.get("separate_third_scalar", {})
    require(third.get("type") == "u32", "third scalar type mismatch")
    require(third.get("profile_carrier_block_join") == "UNBOUND", "third scalar must not be named carrier_block")
    require(third.get("first_writer_and_semantic_owner") == "UNBOUND", "third scalar source must remain open")
    require(native.get("approved_profile_to_native_plan_owner") == "UNBOUND", "profile/native producer edge must remain UNBOUND")
    require(native.get("native_plan_to_bsi_consumer", "").startswith("BOUND within"), "existing consumer join must be reused")
    require(native.get("native_capture_this_followup") == "NOT_RUN", "no new native capture is claimed")

    compatibility = finding.get("selected_scope_compatibility", {})
    require(compatibility.get("existing_container_relation") == "C=B+4", "container/body unit relation mismatch")
    require(compatibility.get("complete_container_cap_bytes") == 511, "container cap mismatch")
    require(compatibility.get("implied_positive_body_max_bytes") == 507, "body cap mismatch")
    require(compatibility.get("typed_profile_field_transfer_proven") is False, "typed profile transfer must remain unbound")

    owner = finding.get("owner_generation_boundary", {})
    require(owner.get("native_plan_and_bsi_join") == "BOUND_WITHIN_SAMPLE_SPECIFIC_PRIOR_CAPTURE", "native owner/generation scope mismatch")
    require(owner.get("approved_profile_owner_to_native_owner") == "UNBOUND", "profile owner join must remain open")
    require(owner.get("native_capture_joined_to_current_authorized_bwf_inputs") is False, "cross-sample join forbidden")
    require(owner.get("model_frame_generation_joined_to_native_generation") is False, "cross-clock generation join forbidden")

    sources = load_json("SOURCES.json")
    identity = sources.get("native_static_identity", {})
    require(identity.get("ghidra_program_executable_sha256") == "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2", "native program identity mismatch")
    require(identity.get("raw_ghidra_output_in_package") is False, "raw Ghidra output must remain excluded")
    reused = sources.get("reused_native_capture", {})
    require(reused.get("recaptured") is False, "presence/consumer chain must not be recaptured")
    require(reused.get("cross_join_to_seven_authorized_bwf_inputs") is False, "sample identity boundary mismatch")

    for name in ("README.md", "FINDING.json", "SOURCES.json"):
        text = (ROOT / name).read_text(encoding="utf-8")
        require(re.search(r"\b0x[0-9a-f]{3,}\b", text, re.IGNORECASE) is None, f"raw Ghidra address in {name}")
        require("function_body" not in text.lower() and "decompile" not in text.lower(), f"raw function content marker in {name}")

    print(f"PASS: {len(payloads)} payloads; {len(sealed)} sealed files; selected block2/C<=511 scope; length-to-plan bound; profile-to-native edge UNBOUND; prior BSI join reused")


if __name__ == "__main__":
    main()
