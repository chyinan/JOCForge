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
    require(manifest["package_id"] == "HOST-ATMOSMEZZ-PROVIDER-TO-PROCESS-IDENTITY-FOLLOWUP-V1", "PACKAGE_ID")
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
    require(source_ids["sources"][2]["sha256"] == evidence["existing_run_applicability"]["execution_config_sha256"], "RUN_CONFIG_BINDING")


def verify_vectors(vectors: dict) -> None:
    require(vectors["classification"] == "STATIC_VALUE_DOMAIN_AND_RUN_APPLICABILITY_NOT_NEW_CAPTURE", "VECTOR_CLASSIFICATION")
    require(vectors["native_execution"] == "NOT_RUN", "VECTOR_NATIVE")
    require(len(vectors["vectors"]) == 5, "VECTOR_COUNT")
    by_id = {vector["case_id"]: vector for vector in vectors["vectors"]}
    require(by_id["run_return_not_process_packet"]["expected_static"]["identity"] == "NOT_SAME", "RUN_NOT_SAME")
    require(by_id["get_targets_return_not_process_packet"]["expected_static"]["identity"] == "NOT_SAME", "TARGETS_NOT_SAME")
    require(by_id["storage_provider_to_process_packet"]["expected_static"] == {
        "identity": "UNBOUND",
        "same_task_generation": "NOT_OBSERVED",
    }, "PROVIDER_PACKET_UNBOUND")
    require(by_id["r138_not_atmosmezz_input"]["expected_static"]["reuse_for_provider_identity"] == "NOT_COMPATIBLE", "R138_APPLICABILITY")
    require(by_id["process_frame_size_domain"]["expected_static"]["accepted_frame_size_units"] == [1536, 0], "PROCESS_FRAME_DOMAIN")


def verify_evidence(evidence: dict) -> None:
    require(
        evidence["result"]
        == "RUN_AND_GETTARGETS_RETURN_VALUES_NOT_SAME_AS_PROCESS_PACKET; STORAGE_PROVIDER_TO_PROCESS_PACKET_UNBOUND; SAME_TASK_GENERATION_NOT_OBSERVED",
        "RESULT",
    )
    require(evidence["evidence_tier"] == "GHIDRA_STATIC_PLUS_HASH_BOUND_CONFIG_APPLICABILITY", "EVIDENCE_TIER")
    run = evidence["existing_run_applicability"]
    require(run["route_key"] == "encode_to_atmos_ddp", "R138_ROUTE")
    require(run["input_type"] == "audio.adm" and run["AtmosMezz_input_used"] is False, "R138_INPUT_TYPE")
    require(run["current_process_or_debugger_session"] == "NONE", "R138_SESSION")
    host = evidence["host_value_domains"]
    require(host["Run_return"]["relation_to_plugin_packet"] == "NOT_SAME", "RUN_STATUS_IDENTITY")
    require(host["GetTargets_return"]["relation_to_plugin_packet"] == "NOT_SAME", "TARGET_LIST_IDENTITY")
    require(host["nested_provider_callback_result"]["relation_to_plugin_packet"] == "NOT_SAME", "CALLBACK_VALUE_IDENTITY")
    process = evidence["plugin_process_packet"]
    require(process["accepted_frame_size_units"] == [1536, 0], "PROCESS_PACKET_SHAPE")
    status = evidence["identity_status"]
    require(status["provider_to_plugin_process_packet"] == "UNBOUND", "PROVIDER_PACKET_EDGE")
    require(status["same_task_activation"] == "NOT_OBSERVED", "TASK_ACTIVATION")
    require(status["same_parent_or_source_generation"] == "NOT_OBSERVED", "GENERATION")
    require(status["target_boundary_reached_in_existing_runs"] == "NOT_REACHED", "NOT_REACHED")
    require(status["new_native_run"] == "AWAITING_WASH_COORDINATION", "NEW_RUN_GATE")
    require(status["pcm_or_bitstream_read_or_saved"] is False, "PAYLOAD_BOUNDARY")
    require(status["key_license_or_authentication_accessed"] is False, "SECRET_BOUNDARY")
    report = (PACKAGE / "REPORT.md").read_text(encoding="utf-8").lower()
    require("not_same" in report and "unbound" in report and "not_reached" in report, "REPORT_STATUSES")
    require("audio.adm" in report and "atmos_mezz" in report, "REPORT_INPUT_DOMAIN")
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
        "r138_applicability=NOT_COMPATIBLE",
        "provider_packet_join=UNBOUND",
        "generation=NOT_OBSERVED",
        "new_capture=AWAITING_WASH_COORDINATION",
    )


if __name__ == "__main__":
    main()
