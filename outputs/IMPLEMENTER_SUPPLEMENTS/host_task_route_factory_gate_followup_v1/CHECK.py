from __future__ import annotations

import hashlib
import json
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent


def require(condition: bool, code: str) -> None:
    if not condition:
        raise SystemExit(f"CHECK_FAILED:{code}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def verify_manifest() -> str:
    path = PACKAGE / "MANIFEST.json"
    require(path.is_file(), "MANIFEST_MISSING")
    content = path.read_bytes()
    manifest_hash = hashlib.sha256(content).hexdigest()
    seal = (PACKAGE / "MANIFEST.sha256").read_text(encoding="utf-8").split()
    require(bool(seal) and seal[0] == manifest_hash, "MANIFEST_SEAL")
    manifest = json.loads(content.decode("utf-8"))
    require(manifest["package_id"] == "HOST-TASK-ROUTE-FACTORY-GATE-FOLLOWUP-V1", "PACKAGE_ID")
    expected = {
        "CHECK.py",
        "CONTROLLED-VECTORS.json",
        "EVIDENCE.json",
        "REPORT.md",
        "SOURCE-IDENTITIES.json",
    }
    entries = manifest["payloads"]
    require({entry["file"] for entry in entries} == expected, "SANITIZED_PAYLOAD_SET")
    for entry in entries:
        payload = PACKAGE / entry["file"]
        require(payload.is_file(), f"PAYLOAD_MISSING:{entry['file']}")
        require(payload.stat().st_size == entry["bytes"], f"PAYLOAD_SIZE:{entry['file']}")
        require(sha256(payload) == entry["sha256"], f"PAYLOAD_HASH:{entry['file']}")
    return manifest_hash


def verify_sources(source_ids: dict, evidence: dict) -> None:
    require(source_ids["analysis"] == {
        "tool": "Ghidra",
        "version": "12.2 PUBLIC",
        "read_only_project": True,
        "ida_used": False,
    }, "ANALYSIS_IDENTITY")
    expected = {
        "dee.exe": ("65485b6d35fa534b25109ab6f238971aaded73fc99c7f7fb119d1a5d5299234e", 4233728),
        "dee_audio_filter_ddp_atmos.dll": ("3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2", 10942464),
    }
    require(len(source_ids["sources"]) == 2, "SOURCE_COUNT")
    for source in source_ids["sources"]:
        name = source["file_name"]
        require(name in expected, f"SOURCE_NAME:{name}")
        require((source["sha256"], source["bytes"]) == expected[name], f"SOURCE_HASH:{name}")
    require(source_ids["sources"] == evidence["source_modules"], "EVIDENCE_SOURCE_BINDING")


def verify_vectors(vectors: dict, evidence: dict) -> None:
    require(vectors["classification"] == "STATIC_REGISTRATION_AND_CALLSITE_CROSSWALK_NOT_EXECUTED", "VECTOR_CLASSIFICATION")
    require(vectors["native_execution"] == "NOT_RUN", "VECTOR_NATIVE")
    descriptors = {entry["index"]: entry["descriptor_name"] for entry in evidence["dll_factory_registration"]["entries"]}
    require(descriptors == {
        0: "encode_to_atmos_ddp",
        1: "atmos_mezz_to_atmos_ddp",
        2: "pcm_to_atmos_ddp",
    }, "DLL_DESCRIPTOR_INDEX_MAP")
    require(len(vectors["vectors"]) == 4, "VECTOR_COUNT")
    seen: set[str] = set()
    for vector in vectors["vectors"]:
        case_id = vector["case_id"]
        require(case_id not in seen, f"VECTOR_DUPLICATE:{case_id}")
        seen.add(case_id)
        inputs = vector["input"]
        expected = vector["expected_static"]
        if case_id.startswith("dll_index_"):
            index = inputs["factory_index"]
            require(index in descriptors, f"FACTORY_INDEX:{case_id}")
            require(expected["descriptor_name"] == descriptors[index], f"FACTORY_DESCRIPTOR:{case_id}")
            if index == 0:
                require(expected["host_selection"] == "UNBOUND", "INDEX_ZERO_SELECTION_BOUNDARY")
        elif case_id == "host_route_profile_lookup":
            require(inputs["route_key"] == "encode_to_atmos_ddp", "ROUTE_KEY")
            require(expected == {
                "lookup_result_domain": "profile/config option values",
                "factory_index_transfer": "NONE_OBSERVED",
                "factory_index_status": "UNBOUND",
            }, "ROUTE_PROFILE_VECTOR_RESULT")
        else:
            raise SystemExit(f"CHECK_FAILED:UNKNOWN_VECTOR:{case_id}")
    require(seen == {
        "dll_index_zero_candidate",
        "dll_index_one_registration",
        "dll_index_two_registration",
        "host_route_profile_lookup",
    }, "VECTOR_CASE_SET")


def verify_evidence(evidence: dict) -> None:
    require(
        evidence["result"]
        == "HOST_ROUTE_TO_PROFILE_CONFIG_LOOKUP_CLOSED; HOST_ROUTE_TO_FACTORY_INDEX_UNBOUND; PACKET_IDENTITY_AND_GENERATION_UNBOUND; NATIVE_OBSERVATION_NOT_RUN",
        "RESULT",
    )
    require(evidence["evidence_tier"] == "GHIDRA_STATIC", "EVIDENCE_TIER")
    prior = evidence["prior_sanitized_crosswalk"]
    require(prior["package_id"] == "HOST-TASK-ROUTE-FACTORY-SELECTION-FOLLOWUP-V1", "PRIOR_PACKAGE_ID")
    require(prior["manifest_sha256"] == "d15e3c71a31bf0943a9c62ccfab00b7f5fdb214e0eaa916ce93c6ecc7765ec0b", "PRIOR_PACKAGE_HASH")

    gate = evidence["host_route_profile_gate"]
    require(gate["route_key"] == "encode_to_atmos_ddp", "HOST_ROUTE")
    require(gate["lookup_edge_status"] == "STATIC_ROUTE_TO_PROFILE_CONFIG_LOOKUP_CLOSED", "PROFILE_LOOKUP")
    require(gate["lookup_result_domain"] == "profile/config option values", "LOOKUP_DOMAIN")
    require(gate["factory_index_transfer_at_callsite"] == "NONE_OBSERVED", "NO_FACTORY_TRANSFER")
    require(gate["route_to_factory_index_status"] == "UNBOUND", "ROUTE_FACTORY_UNBOUND")
    require(gate["same_name_candidate_index"] == 0, "CANDIDATE_INDEX")
    require(gate["same_name_candidate_status"] == "CANDIDATE_ONLY_NOT_SELECTION_EVIDENCE", "CANDIDATE_LIMIT")

    boundary = evidence["remaining_identity_boundaries"]
    require(boundary["runtime_task_selected"] == "NOT_OBSERVED", "RUNTIME_TASK")
    require(boundary["AtmosMezz_Run_GetTargets_to_plugin_process_packet_same_object"] == "UNBOUND", "PACKET_IDENTITY")
    require(boundary["same_parent_or_source_generation"] == "UNBOUND", "GENERATION")
    require(boundary["native_execution"] == "NOT_RUN", "NATIVE_STATUS")
    require(boundary["pcm_or_bitstream_read_or_saved"] is False, "PAYLOAD_BOUNDARY")
    require(boundary["key_license_or_authentication_accessed"] is False, "SECRET_BOUNDARY")

    report = (PACKAGE / "REPORT.md").read_text(encoding="utf-8").lower()
    require("task-profile option lookup" in report and "factory index" in report, "REPORT_EDGE")
    require("unbound" in report and "not_run" in report, "REPORT_BOUNDARIES")
    require("0x" not in report and "fun_" not in report, "REPORT_SANITIZED")


def main() -> None:
    manifest_hash = verify_manifest()
    evidence = read_json(PACKAGE / "EVIDENCE.json")
    source_ids = read_json(PACKAGE / "SOURCE-IDENTITIES.json")
    vectors = read_json(PACKAGE / "CONTROLLED-VECTORS.json")
    verify_sources(source_ids, evidence)
    verify_vectors(vectors, evidence)
    verify_evidence(evidence)
    manifest = read_json(PACKAGE / "MANIFEST.json")
    print(
        "CHECK_OK",
        f"manifest_sha256={manifest_hash}",
        f"payloads={len(manifest['payloads'])}",
        f"vectors={len(vectors['vectors'])}",
        "route_profile_lookup=STATIC_CLOSED",
        "route_factory_index=UNBOUND",
        "packet_generation=UNBOUND",
        "native_observation=NOT_RUN",
    )


if __name__ == "__main__":
    main()
