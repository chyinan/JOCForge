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
    require(manifest["package_id"] == "HOST-ENCODE-ROUTE-PROFILE-FACTORY-SELECTION-FOLLOWUP-V1", "PACKAGE_ID")
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
        "native-reference-execution.xml": ("fa65143f0bd7de52b36367137e61d681baa4c17824aded10be26317e7d1018a4", 2473),
    }
    require(len(source_ids["sources"]) == 3, "SOURCE_COUNT")
    for source in source_ids["sources"]:
        name = source["file_name"]
        require(name in expected, f"SOURCE_NAME:{name}")
        require((source["sha256"], source["bytes"]) == expected[name], f"SOURCE_HASH:{name}")
    require(source_ids["sources"][:2] == evidence["source_modules"], "MODULE_SOURCE_BINDING")
    require(source_ids["sources"][2]["sha256"] == evidence["existing_profile_record"]["execution_config_sha256"], "PROFILE_CONFIG_BINDING")


def verify_vectors(vectors: dict) -> None:
    require(vectors["classification"] == "STATIC_AND_HASH_BOUND_PROFILE_CROSSWALK_NOT_FACTORY_SELECTION", "VECTOR_CLASSIFICATION")
    require(vectors["native_execution"] == "NOT_RUN", "VECTOR_NATIVE")
    require(len(vectors["vectors"]) == 4, "VECTOR_COUNT")
    by_id = {vector["case_id"]: vector for vector in vectors["vectors"]}
    route = by_id["r138_profile_route_identity"]
    require(route["input"] == {"route_key": "encode_to_atmos_ddp", "input_type": "audio.adm", "data_rate": 1024}, "R138_PROFILE_INPUT")
    require(route["expected_static"] == {"profile_lookup": "STATIC_CLOSED", "factory_index_recorded": False}, "R138_PROFILE_RESULT")
    table = by_id["audio_filter_indices_enumerated"]
    require(table["expected_static"] == {"registered_indices": [0, 1, 2], "index_source": "enumeration counter"}, "INDEX_ENUMERATION")
    crosswalk = by_id["dll_registration_index_crosswalk"]
    require(crosswalk["expected_static"]["descriptor_names"] == ["encode_to_atmos_ddp", "atmos_mezz_to_atmos_ddp", "pcm_to_atmos_ddp"], "DLL_CROSSWALK")
    selection = by_id["host_route_to_selected_factory_index"]
    require(selection["expected_static"]["selected_factory_index"] is None, "INDEX_MUST_REMAIN_NULL")
    require(selection["expected_static"]["status"] == "UNBOUND", "INDEX_STATUS")


def verify_evidence(evidence: dict) -> None:
    require(
        evidence["result"]
        == "R138_ROUTE_AND_PROFILE_CONFIG_HASH_BOUND; DLL_DESCRIPTOR_TO_FACTORY_INDEX_CLOSED; HOST_ROUTE_TO_SELECTED_FACTORY_INDEX_UNBOUND",
        "RESULT",
    )
    require(evidence["evidence_tier"] == "GHIDRA_STATIC_PLUS_HASH_BOUND_RUN_RECORD", "EVIDENCE_TIER")
    profile = evidence["existing_profile_record"]
    require(profile["run_id"] == "run-138", "RUN_ID")
    require(profile["route_key"] == "encode_to_atmos_ddp" and profile["input_type"] == "audio.adm", "RUN_PROFILE")
    require(profile["selected_audio_filter_factory_index"] is None, "INDEX_NOT_CAPTURED")
    require(profile["factory_selection_evidence"] == "NOT_CAPTURED", "FACTORY_SELECTION_NOT_CAPTURED")

    lookup = evidence["host_route_profile_lookup"]
    require(lookup["route_key"] == "encode_to_atmos_ddp", "ROUTE_KEY")
    require(lookup["lookup_output_domain"] == "profile/config option values", "PROFILE_OUTPUT_DOMAIN")
    require(lookup["profile_output_to_index_argument"] == "NONE_OBSERVED", "NO_TYPED_INDEX_TRANSFER")

    loader = evidence["host_audio_filter_index_registration"]
    require(loader["category"] == "audio_filter" and loader["table_count"] == 3, "AUDIO_FILTER_TABLE")
    require(loader["indices_registered"] == [0, 1, 2], "CATEGORY_INDICES")
    require(loader["selected_route_index"] is None, "ROUTE_INDEX_UNBOUND")

    crosswalk = evidence["dll_factory_registration"]
    require(crosswalk["descriptor_to_index_status"] == "STATIC_CLOSED", "DESCRIPTOR_CROSSWALK")
    require(crosswalk["index_zero_route_match"] == "CANDIDATE_ONLY_NOT_SELECTION_EVIDENCE", "NAME_MATCH_LIMIT")
    edge = evidence["unclosed_selection_edge"]
    require(edge["route_profile_output_to_factory_index"] == "UNBOUND", "SELECTION_EDGE")
    require(edge["same_invocation_or_owner_transfer"] == "NOT_OBSERVED", "INVOCATION_OWNER")
    require(edge["actual_factory_dispatch_callsite"] == "NOT_IDENTIFIED", "DISPATCH_CALLSITE")
    require(edge["runtime_task_parent_token"] == "NOT_OBSERVED", "PARENT_TOKEN")
    require(edge["native_observation"] == "NOT_RUN", "NATIVE_STATUS")

    report = (PACKAGE / "REPORT.md").read_text(encoding="utf-8").lower()
    require("same-name index-0 entry is a registration candidate only" in report, "REPORT_NAME_LIMIT")
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
        "profile_config=HASH_BOUND",
        "dll_descriptor_index=CLOSED",
        "host_selected_factory_index=UNBOUND",
        "native_observation=NOT_RUN",
    )


if __name__ == "__main__":
    main()
