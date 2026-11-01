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
    require(manifest["package_id"] == "HOST-ATMOSMEZZ-R138-REUSE-BOUNDARY-FOLLOWUP-V1", "PACKAGE_ID")
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
    config = source_ids["sources"][2]
    require((config["sha256"], config["bytes"]) == (
        evidence["r138_existing_run"]["execution_config_sha256"],
        2473,
    ), "EXECUTION_CONFIG_BINDING")


def verify_vectors(vectors: dict) -> None:
    require(vectors["classification"] == "HASH_BOUND_EXECUTION_CONFIG_APPLICABILITY_NOT_NEW_CAPTURE", "VECTOR_CLASSIFICATION")
    require(vectors["native_execution"] == "NOT_RUN", "VECTOR_NATIVE")
    require(len(vectors["vectors"]) == 4, "VECTOR_COUNT")
    expected_cases = {
        "r138_route_matches_input_type_does_not",
        "r138_session_not_live",
        "r4_reuse_requires_route_input_check",
        "provider_to_process_packet_edge",
    }
    seen = {vector["case_id"] for vector in vectors["vectors"]}
    require(seen == expected_cases, "VECTOR_CASE_SET")
    for vector in vectors["vectors"]:
        case_id = vector["case_id"]
        if case_id == "r138_route_matches_input_type_does_not":
            require(vector["input"] == {"route_key": "encode_to_atmos_ddp", "input_type": "audio.adm"}, "R138_INPUT")
            require(vector["expected_static"]["atmosmezz_reader_used"] is False, "R138_NOT_ATMOSMEZZ")
            require(vector["expected_static"]["reuse_for_provider_edge"] == "NOT_COMPATIBLE", "R138_NOT_REUSABLE")
        elif case_id == "r138_session_not_live":
            require(vector["expected_static"] == {"live_pid_or_windbg_session": "NONE", "new_capture": "NOT_RUN"}, "R138_SESSION")
        elif case_id == "r4_reuse_requires_route_input_check":
            require(vector["expected_static"] == {"reuse_status": "AWAITING_WASH", "cross_thread_attach": False}, "R4_REUSE_GATE")
        else:
            require(vector["expected_static"] == {"status": "UNBOUND", "same_parent_generation": "NOT_OBSERVED"}, "PACKET_EDGE")


def verify_evidence(evidence: dict) -> None:
    require(
        evidence["result"]
        == "R138_ROUTE_MATCHES_BUT_INPUT_IS_ADM_NOT_ATMOSMEZZ; PROVIDER_PACKET_EDGE_NOT_REACHED_IN_R138; NEW_NATIVE_CAPTURE_NOT_RUN",
        "RESULT",
    )
    run = evidence["r138_existing_run"]
    require(run["route_key"] == "encode_to_atmos_ddp", "R138_ROUTE")
    require(run["input_type"] == "audio.adm" and run["atmosmezz_reader_path_used"] is False, "R138_INPUT_SCOPE")
    require(run["live_process_or_windbg_session"] == "NONE", "R138_NO_LIVE_SESSION")
    require(run["reuse_for_atmosmezz_provider_capture"] == "NOT_COMPATIBLE", "R138_REUSE")
    r4 = evidence["other_active_run"]
    require(r4["task_thread_identity"] == "DIFFERENT_TASK_THREAD", "R4_TASK_SCOPE")
    require(r4["reuse_status"] == "AWAITING_WASH", "R4_REUSE_CONFIRMATION")
    status = evidence["packet_identity_status"]
    require(status["R138_atmosmezz_provider_boundary"] == "NOT_REACHED", "R138_NOT_REACHED")
    require(status["host_storage_provider_to_plugin_process_packet"] == "UNBOUND", "PROCESS_PACKET_EDGE")
    require(status["same_parent_or_source_generation"] == "NOT_OBSERVED", "GENERATION")
    require(status["new_native_capture"] == "NOT_RUN", "NO_NEW_CAPTURE")
    require(status["pcm_or_bitstream_read_or_saved"] is False, "PAYLOAD_BOUNDARY")
    require(status["key_license_or_authentication_accessed"] is False, "SECRET_BOUNDARY")
    report = (PACKAGE / "REPORT.md").read_text(encoding="utf-8").lower()
    require("audio.adm" in report and "atmos_mezz" in report, "REPORT_INPUT_DISTINCTION")
    require("unbound" in report and "not_reached" in report and "not_run" in report, "REPORT_STATUSES")
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
        "r138_reuse_for_atmosmezz=NOT_COMPATIBLE",
        "r4_reuse=AWAITING_WASH",
        "provider_process_edge=UNBOUND",
        "new_capture=NOT_RUN",
    )


if __name__ == "__main__":
    main()
