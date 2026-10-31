#!/usr/bin/env python3
"""Verify the authorized BWF input-scope addendum and its sealed payloads."""

from __future__ import annotations

import hashlib
import json
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


def verify_files(entries: object, label: str) -> set[str]:
    require(isinstance(entries, list), f"{label} list missing")
    names: set[str] = set()
    for entry in entries:
        require(isinstance(entry, dict), f"invalid {label} entry")
        name = entry.get("file")
        expected_size = entry.get("bytes")
        expected_hash = entry.get("sha256")
        require(isinstance(name, str) and Path(name).name == name, f"unsafe {label} path")
        require(name not in names, f"duplicate {label} entry: {name}")
        names.add(name)
        path = ROOT / name
        require(path.is_file(), f"missing {label} file: {name}")
        require(path.stat().st_size == expected_size, f"size mismatch: {name}")
        require(sha256(path) == expected_hash, f"SHA-256 mismatch: {name}")
        if path.suffix.lower() == ".json":
            load_json(name)
    return names


def main() -> None:
    manifest = load_json("MANIFEST.json")
    seal = load_json("package_seal.json")
    require(isinstance(manifest, dict) and isinstance(seal, dict), "package metadata must be objects")
    require(manifest.get("package_id") == "carrier_block_authorized_input_scope_wash_v2", "package id mismatch")
    require(manifest.get("implementation_ready") == "NO", "whole implementation readiness must remain NO")
    payload_names = verify_files(manifest.get("payloads"), "payload")
    sealed_names = verify_files(seal.get("files"), "seal")
    require(seal.get("seal_file_excluded_from_own_hash_list") is True, "seal policy mismatch")
    require(seal.get("file_count") == len(sealed_names), "seal file count mismatch")
    require(sealed_names == payload_names | {"MANIFEST.json"}, "sealed inventory mismatch")

    identities = load_json("AUTHORIZED-BWF-INPUT-IDENTITIES.json")
    inputs = identities.get("inputs", [])
    require(len(inputs) == 7, "expected seven authorized BWF identities")
    authorized_hashes = {item["sha256"] for item in inputs}
    require(len(authorized_hashes) == 7, "authorized BWF hashes are not unique")
    lineage_ids = {item.get("lineage_id") for item in inputs}
    require(len(lineage_ids) == 7 and None not in lineage_ids, "each authorized BWF must have a separate lineage id")
    require(all(item.get("status", "HASHED") == "HASHED" for item in inputs), "an authorized BWF was not hashed")
    require(identities.get("data_chunk_decoded") is False, "audio data must remain undecoded")
    require(identities.get("original_files_modified") is False, "authorized inputs must remain unmodified")

    prior = load_json("PRIOR-NATIVE-SAMPLE-SEPARATION.json")
    require(prior.get("prior_native_lineage_id") == "historical_r24_native_input", "prior native lineage id mismatch")
    crosswalk = prior.get("authorized_input_crosswalk", [])
    require(len(crosswalk) == 7, "expected one prior-input comparison for each authorized input identity")
    require({entry.get("lineage_id") for entry in crosswalk} == lineage_ids, "crosswalk lineage identities mismatch")
    require({entry.get("input_file_sha256") for entry in crosswalk} == authorized_hashes, "crosswalk input identities mismatch")
    require(all(entry.get("matches_prior_native_input") is False for entry in crosswalk), "prior native sample unexpectedly joined")
    require(prior.get("prior_native_input_file_sha256") not in authorized_hashes, "prior sample hash collides with an authorized identity")
    require(prior.get("matches_any_authorized_input_identity") is False, "prior native input must match none of the authorized identities")
    require(prior.get("unapproved_sample_path_persisted") is False, "prior sample path must not be persisted")
    require(prior.get("unapproved_sample_name_persisted") is False, "prior sample name must not be persisted")
    require(prior.get("unapproved_sample_content_included") is False, "prior sample content must not be included")

    setup = load_json("AUTHORIZED-INPUT-NATIVE-SETUP.json")
    require(setup.get("input_identity_sha256") in authorized_hashes, "setup attempt input is outside authorized set")
    selected_input = next(item for item in inputs if item["lineage_id"] == setup.get("input_lineage_id"))
    require(selected_input["sha256"] == setup.get("input_identity_sha256"), "setup input identity and lineage id mismatch")
    require(setup.get("schema_validation") == "PASS for the new job configuration", "setup job XSD validation mismatch")
    require(setup.get("workflow_built") is False, "setup failure must not claim workflow construction")
    require(setup.get("native_capture") == "NOT_RUN", "native capture must remain NOT_RUN")
    require(setup.get("dee_input_processing") == "NOT_CONFIRMED", "input processing must remain unconfirmed")
    require(setup.get("output_file_created") is False, "setup attempt unexpectedly created output")
    require(setup.get("license_or_key_diagnostics_read") is False, "license/key diagnostics must remain unread")
    require(setup.get("failed_setup_is_no_hit") is False, "setup failure must not be called a no-hit")

    scope = load_json("SELECTED-CARRIER-SCOPE.json")
    require(scope.get("status") == "SELECTED_BLOCKER_REVOCABLE_WITHIN_FROZEN_SCOPE; WHOLE_F13_NOT_EVALUATED", "selected carrier scope status mismatch")
    contract = scope.get("approved_selected_contract", {})
    require(contract.get("carrier_block") == 2, "selected carrier block must remain 2")
    require(contract.get("complete_emdf_container_max_bytes") == 511, "selected complete-container cap mismatch")
    require(contract.get("arbitrary_0_to_5_product_placement_required") is False, "unapproved placement expansion must stay out")
    require(contract.get("fragmentation_over_511_product_requirement") is False, "unapproved fragmentation must stay out")
    require(contract.get("selected_carrier_blocker_revocable") is True, "selected carrier blocker decision mismatch")
    require(contract.get("approved_product_policy_modified") is False, "approved product policy must remain unchanged")
    require(contract.get("whole_product_ready") == "NOT_CLAIMED", "whole product readiness must remain unclaimed")
    prior_scope = scope.get("lineage_boundary", {})
    require(prior_scope.get("source_file_hash_matches_any_authorized_input_identity") is False, "prior native sample must match no authorized identity")
    require(prior_scope.get("native_observations_joined_to_any_authorized_input_identity") is False, "cross-sample native join must remain false")

    print(f"PASS: {len(payload_names)} payloads; {len(sealed_names)} sealed files; 7 separate authorized BWF identities; R24 crosswalk excludes all; selected scope unchanged")


if __name__ == "__main__":
    main()
