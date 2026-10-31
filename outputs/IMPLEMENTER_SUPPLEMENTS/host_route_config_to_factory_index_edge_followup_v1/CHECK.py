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


def verify_manifest() -> tuple[dict, str]:
    path = PACKAGE / "MANIFEST.json"
    require(path.is_file(), "MANIFEST_MISSING")
    content = path.read_bytes()
    digest = hashlib.sha256(content).hexdigest()
    seal = (PACKAGE / "MANIFEST.sha256").read_text(encoding="utf-8").split()
    require(bool(seal) and seal[0] == digest, "MANIFEST_SEAL")
    manifest = json.loads(content.decode("utf-8"))
    require(manifest["package_id"] == "HOST-ROUTE-CONFIG-TO-FACTORY-INDEX-EDGE-FOLLOWUP-V1", "PACKAGE_ID")
    expected = {"CHECK.py", "CONTROLLED-VECTORS.json", "EVIDENCE.json", "REPORT.md", "SOURCE-IDENTITIES.json"}
    entries = manifest["payloads"]
    require({entry["file"] for entry in entries} == expected, "SANITIZED_PAYLOAD_SET")
    for entry in entries:
        payload = PACKAGE / entry["file"]
        require(payload.is_file(), f"PAYLOAD_MISSING:{entry['file']}")
        require(payload.stat().st_size == entry["bytes"], f"PAYLOAD_SIZE:{entry['file']}")
        require(sha256(payload) == entry["sha256"], f"PAYLOAD_HASH:{entry['file']}")
    return manifest, digest


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


def verify_vectors(vectors: dict) -> None:
    require(vectors["classification"] == "STATIC_CALLSITE_AND_REGISTRATION_CROSSWALK_NOT_EXECUTED", "VECTOR_CLASSIFICATION")
    require(vectors["native_execution"] == "NOT_RUN", "VECTOR_NATIVE")
    require(len(vectors["vectors"]) == 4, "VECTOR_COUNT")
    seen: set[str] = set()
    for vector in vectors["vectors"]:
        case_id = vector["case_id"]
        require(case_id not in seen, f"VECTOR_DUPLICATE:{case_id}")
        seen.add(case_id)
        inputs = vector["input"]
        expected = vector["expected_static"]
        if case_id == "host_route_profile_config_output":
            require(inputs == {"route_key": "encode_to_atmos_ddp"}, "ROUTE_INPUT")
            require(expected == {"output_domain": "profile/config option values", "route_to_profile_config": "STATIC_CLOSED"}, "ROUTE_OUTPUT")
        elif case_id == "audio_filter_category_registration_loop":
            require(inputs == {"category": "audio_filter", "table_count": 3}, "INDEX_LOOP_INPUT")
            require(expected == {"registered_indices": [0, 1, 2], "index_source": "table enumeration loop"}, "INDEX_LOOP_OUTPUT")
        elif case_id == "profile_output_to_factory_index_edge":
            require(inputs == {"profile_config_output": "route option values"}, "EDGE_INPUT")
            require(expected == {"factory_index": None, "status": "UNBOUND", "same_invocation_owner_transfer": "NONE_OBSERVED"}, "EDGE_OUTPUT")
        elif case_id == "same_name_index_zero_candidate":
            require(inputs == {"route_key": "encode_to_atmos_ddp", "dll_descriptor_at_index_zero": "encode_to_atmos_ddp"}, "CANDIDATE_INPUT")
            require(expected == {"selected_factory_index": None, "status": "CANDIDATE_ONLY_NOT_SELECTION_EVIDENCE"}, "CANDIDATE_OUTPUT")
        else:
            raise SystemExit(f"CHECK_FAILED:UNKNOWN_VECTOR:{case_id}")
    require(seen == {
        "host_route_profile_config_output",
        "audio_filter_category_registration_loop",
        "profile_output_to_factory_index_edge",
        "same_name_index_zero_candidate",
    }, "VECTOR_CASE_SET")


def verify_evidence(evidence: dict) -> None:
    require(
        evidence["result"]
        == "ROUTE_TO_PROFILE_CONFIG_OUTPUT_CLOSED; CONFIG_OUTPUT_TO_FACTORY_INDEX_UNBOUND_AT_PRECISE_CALLSITES; NATIVE_OBSERVATION_NOT_RUN",
        "RESULT",
    )
    require(evidence["evidence_tier"] == "GHIDRA_STATIC", "EVIDENCE_TIER")
    gate = evidence["route_profile_lookup"]
    require(gate["route_key"] == "encode_to_atmos_ddp", "ROUTE_KEY")
    require(gate["output_domain"] == "profile/config option values", "ROUTE_OUTPUT_DOMAIN")
    require(gate["consumer_role"] == "task-profile option application", "ROUTE_CONSUMER")
    require(gate["route_to_profile_config_status"] == "STATIC_CLOSED", "ROUTE_PROFILE_EDGE")

    registration = evidence["audio_filter_index_registration"]
    require(registration["category"] == "audio_filter", "FACTORY_CATEGORY")
    require(registration["entry_count"] == 3 and registration["registered_indices"] == [0, 1, 2], "FACTORY_INDEX_DOMAIN")
    require(registration["index_argument_source"] == "loop counter over the returned three-entry plugin table", "INDEX_ARGUMENT_SOURCE")
    require(registration["registration_role"].startswith("category index enumeration"), "INDEX_REGISTRATION_ROLE")

    edge = evidence["cross_edge"]
    require(edge["profile_config_output_to_factory_index_argument"] == "UNBOUND", "EDGE_STATUS")
    require(edge["same_invocation_or_owner_transfer"] == "NONE_OBSERVED", "EDGE_TRANSFER")
    require(edge["route_key_to_factory_dispatch_callsite"] == "NOT_IDENTIFIED", "DISPATCH_CALLSITE")
    require(edge["selected_factory_index"] is None, "SELECTED_INDEX")
    require(edge["same_name_index_zero"] == "CANDIDATE_ONLY_NOT_SELECTION_EVIDENCE", "NAME_MATCH_LIMIT")
    require(edge["does_route_gate_profile_config"] is True, "PROFILE_GATE")
    require(edge["does_route_gate_factory_selection"] == "NOT_PROVEN", "FACTORY_GATE")

    boundary = evidence["remaining_identity_boundaries"]
    require(boundary["runtime_task_parent_token"] == "NOT_OBSERVED", "PARENT_TOKEN")
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
    manifest, digest = verify_manifest()
    evidence = read_json(PACKAGE / "EVIDENCE.json")
    source_ids = read_json(PACKAGE / "SOURCE-IDENTITIES.json")
    vectors = read_json(PACKAGE / "CONTROLLED-VECTORS.json")
    verify_sources(source_ids, evidence)
    verify_vectors(vectors)
    verify_evidence(evidence)
    print(
        "CHECK_OK",
        f"manifest_sha256={digest}",
        f"payloads={len(manifest['payloads'])}",
        f"vectors={len(vectors['vectors'])}",
        "route_profile_lookup=STATIC_CLOSED",
        "config_to_factory_index=UNBOUND",
        "packet_generation=UNBOUND",
        "native_observation=NOT_RUN",
    )


if __name__ == "__main__":
    main()
