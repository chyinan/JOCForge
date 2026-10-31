#!/usr/bin/env python3
"""Verify the carrier-block time-base applicability boundary."""

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
    require(manifest.get("package_id") == "carrier_block_timebase_applicability_wash_v1", "package id mismatch")
    require(manifest.get("implementation_ready") == "NO", "implementation readiness must remain NO")
    payloads = verify_inventory(manifest.get("payloads"), "payload")
    sealed = verify_inventory(seal.get("files"), "seal")
    require(seal.get("seal_file_excluded_from_own_hash_list") is True, "seal policy mismatch")
    require(seal.get("file_count") == len(sealed), "seal count mismatch")
    require(sealed == payloads | {"MANIFEST.json"}, "sealed inventory mismatch")

    applicability = load_json("APPLICABILITY.json")
    require(applicability.get("status") == "R138_CAPTURE_SPEC_APPLICABILITY_NOT_EVALUATED", "applicability status mismatch")
    config = applicability.get("r138_configuration", {})
    require(config.get("time_base") == "embedded_timecode", "R138 time base mismatch")
    require(config.get("wash_reported_manifest_sha256_prefix") == "fa6514", "R138 hash prefix mismatch")
    require(config.get("hash_identity_status") == "WASH_REPORTED_HASH_BOUND_PREFIX; FULL_HASH_NOT_RECHECKED_IN_THIS_ADDENDUM", "R138 identity limit mismatch")
    require(config.get("file_position_assumption") == "NOT_APPLICABLE_TO_THIS_HASH_BOUND_CONFIGURATION", "old time-base assumption must be excluded")
    decision = applicability.get("applicability", {})
    require(decision.get("third_u32_capture_spec_to_r138_profile") == "NOT_EVALUATED", "capture applicability must remain NOT_EVALUATED")
    require(decision.get("plan_call_path_to_r138_profile") == "NOT_EVALUATED", "call-path applicability must remain NOT_EVALUATED")
    require(decision.get("profile_carrier_to_native_owner") == "UNBOUND", "profile/native owner must remain UNBOUND")
    require(decision.get("approved_carrier_block") == 2, "selected carrier block mismatch")
    require(decision.get("complete_emdf_container_max_bytes") == 511, "container cap mismatch")
    require(decision.get("single_fragment_required") is True, "single-fragment scope mismatch")
    require(decision.get("approved_scope_changed") is False, "approved scope must remain unchanged")
    coordination = applicability.get("coordination", {})
    require(coordination.get("native_job_authorized_by_this_addendum") is False, "this addendum cannot authorize a job")
    require(coordination.get("combined_run_decision") == "PENDING_WASH_RECONCILIATION_OF_TIME_BASE_AND_CALL_PATH", "coordination status mismatch")
    require(coordination.get("presence_bsi_chain_recaptured") is False, "closed BSI chain must remain reused")
    require(coordination.get("cross_sample_join_performed") is False, "cross-sample join forbidden")

    print(f"PASS: {len(payloads)} payloads; {len(sealed)} sealed files; R138 embedded_timecode applicability NOT_EVALUATED; no native job authorized")


if __name__ == "__main__":
    main()
