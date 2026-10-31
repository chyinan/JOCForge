#!/usr/bin/env python3
"""Verify the bounded third-scalar owner follow-up package."""

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
    require(manifest.get("package_id") == "carrier_block_third_scalar_owner_followup_wash_v1", "package id mismatch")
    require(manifest.get("implementation_ready") == "NO", "implementation readiness must remain NO")
    payloads = verify_inventory(manifest.get("payloads"), "payload")
    sealed = verify_inventory(seal.get("files"), "seal")
    require(seal.get("seal_file_excluded_from_own_hash_list") is True, "seal policy mismatch")
    require(seal.get("file_count") == len(sealed), "seal count mismatch")
    require(sealed == payloads | {"MANIFEST.json"}, "sealed inventory mismatch")

    finding = load_json("FINDING.json")
    require(finding.get("status") == "THIRD_U32_READ_EDGE_BOUND; FIRST_WRITER_AND_SEMANTIC_OWNER_UNBOUND", "finding status mismatch")
    scope = finding.get("selected_scope", {})
    require(scope.get("carrier_block") == 2, "selected carrier block mismatch")
    require(scope.get("complete_emdf_container_max_bytes") == 511, "complete container cap mismatch")
    require(scope.get("single_fragment_required") is True, "single-fragment scope mismatch")
    require(scope.get("arbitrary_0_to_5_placement_required") is False, "unapproved arbitrary placement expansion")
    require(scope.get("fragmentation_over_511_required") is False, "unapproved fragmentation expansion")
    require(scope.get("approved_product_policy_modified") is False, "approved policy must remain unchanged")

    scalar = finding.get("third_scalar", {})
    require(scalar.get("type") == "u32" and scalar.get("plan_argument_position") == 3, "third scalar type or argument position mismatch")
    require(scalar.get("first_writer") == "UNBOUND", "third scalar first writer must remain UNBOUND")
    require(scalar.get("semantic_owner") == "UNBOUND", "third scalar semantic owner must remain UNBOUND")
    require(scalar.get("approved_carrier_block_join") == "UNBOUND", "third scalar must not be named carrier_block")
    require(scalar.get("direct_store_at_selected_frame_displacement") == "NOT_FOUND_IN_GHIDRA_OPERAND_SURVEY", "static field survey status mismatch")
    require(scalar.get("bulk_initialization_or_indirect_write") == "NOT_EXCLUDED", "bulk or indirect initialization must remain open")
    require(scalar.get("same_numeric_displacement_on_other_base") == "NOT_JOINED", "same-offset object must remain unjoined")

    native = finding.get("native_owner_generation", {})
    require(native.get("plan_to_bsi_owner_generation") == "BOUND_WITHIN_EXISTING_SAMPLE_SPECIFIC_CAPTURE", "existing native chain scope mismatch")
    require(native.get("approved_profile_owner_to_native_frame_descriptor") == "UNBOUND", "profile owner join must remain open")
    require(native.get("native_capture_this_followup") == "NOT_RUN", "no new native job may be claimed")

    reused = finding.get("reused_scalar_observation", {})
    require(reused.get("selected_scope_scalar_observation") == "NOT_AVAILABLE", "unjoined scalar observation cannot be promoted")
    require(reused.get("no_cross_sample_join") is True, "cross-sample join must remain false")

    plan = finding.get("minimum_next_observation", {})
    require(plan.get("status") == "PENDING_COORDINATION_AFTER_CURRENT_AUTHORIZED_WIDTH5_OBSERVATION", "coordination gate mismatch")
    require("a second native job before the current width-5 job completes" in plan.get("avoid", []), "parallel native job must stay blocked")
    require("recapturing the closed BSI presence chain" in plan.get("avoid", []), "closed BSI chain must be reused")

    sources = load_json("SOURCES.json")
    require(sources.get("ghidra_static_identity", {}).get("program_executable_sha256") == "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2", "Ghidra program identity mismatch")
    require(sources.get("reused_carrier_lineage", {}).get("bsi_presence_chain_recaptured") is False, "closed presence chain must not be recaptured")
    require(sources.get("authorized_input_boundary", {}).get("prior_capture_joined_to_authorized_bwf_inputs") is False, "prior sample must remain separate")

    for name in ("README.md", "FINDING.json", "SOURCES.json"):
        text = (ROOT / name).read_text(encoding="utf-8")
        require(re.search(r"\b0x[0-9a-f]{3,}\b", text, re.IGNORECASE) is None, f"raw Ghidra address in {name}")
        require("function_body" not in text.lower() and "decompile" not in text.lower(), f"raw function content marker in {name}")

    print(f"PASS: {len(payloads)} payloads; {len(sealed)} sealed files; third U32 first writer/owner UNBOUND; selected scope preserved; no new native job")


if __name__ == "__main__":
    main()
