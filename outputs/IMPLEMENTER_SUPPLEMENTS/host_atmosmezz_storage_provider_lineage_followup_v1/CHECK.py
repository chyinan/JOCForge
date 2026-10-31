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
    require(manifest["package_id"] == "HOST-ATMOSMEZZ-STORAGE-PROVIDER-LINEAGE-FOLLOWUP-V1", "PACKAGE_ID")
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
    require(vectors["classification"] == "STATIC_CONDITIONAL_EXPECTATIONS_NOT_EXECUTED", "VECTOR_CLASSIFICATION")
    require(vectors["native_execution"] == "NOT_RUN", "VECTOR_NATIVE")
    require(len(vectors["vectors"]) == 5, "VECTOR_COUNT")
    seen: set[str] = set()
    for vector in vectors["vectors"]:
        case_id = vector["case_id"]
        require(case_id not in seen, f"VECTOR_DUPLICATE:{case_id}")
        seen.add(case_id)
        inputs = vector["input"]
        result = vector["expected_static"]
        if case_id == "storage_property_writes_provider_member":
            require(inputs["property_key"] == "storage", "STORAGE_KEY")
            require(result == {"destination_role": "AtmosMezz storage-provider member", "status": "STATIC_MEMBER_TRANSFER_CLOSED"}, "STORAGE_MEMBER_RESULT")
        elif case_id == "misc_container_property_uses_separate_member":
            require(inputs["property_key"] == "dee_misc_container", "CONTAINER_KEY")
            require(result["aliases_storage_provider"] is False, "PROPERTY_FIELD_SEPARATION")
        elif case_id == "run_returns_status_and_metadata":
            require(inputs["method_role"] == "Run", "RUN_METHOD")
            require(result["return_role"] == "completion status scalar" and result["process_packet_returned"] is False, "RUN_NOT_PACKET")
        elif case_id == "get_targets_returns_names":
            require(inputs["method_role"] == "GetTargets", "GETTARGETS_METHOD")
            require(result["return_role"] == "target-label string vector" and result["process_packet_returned"] is False, "TARGETS_NOT_PACKET")
        elif case_id == "provider_to_plugin_process_packet_join":
            require(result == {"same_object": "UNBOUND", "same_task_invocation": "UNBOUND", "same_parent_generation": "UNBOUND"}, "PROVIDER_JOIN_BOUNDARY")
        else:
            raise SystemExit(f"CHECK_FAILED:UNKNOWN_VECTOR:{case_id}")
    require(seen == {
        "storage_property_writes_provider_member",
        "misc_container_property_uses_separate_member",
        "run_returns_status_and_metadata",
        "get_targets_returns_names",
        "provider_to_plugin_process_packet_join",
    }, "VECTOR_CASE_SET")


def verify_evidence(evidence: dict) -> None:
    require(
        evidence["result"]
        == "HOST_STORAGE_PROPERTY_TO_RUN_AND_GETTARGETS_PROVIDER_FIELD_CLOSED; PROVIDER_TO_PLUGIN_PROCESS_PACKET_IDENTITY_AND_GENERATION_UNBOUND; NATIVE_OBSERVATION_NOT_RUN",
        "RESULT",
    )
    require(evidence["evidence_tier"] == "GHIDRA_STATIC", "EVIDENCE_TIER")
    provider = evidence["host_storage_provider_lineage"]
    require(provider["storage_property_key"] == "storage", "STORAGE_PROPERTY")
    require("stored in the AtmosMezz reader storage-provider member" in provider["storage_value_transfer"], "STORAGE_TRANSFER")
    require(provider["separate_property_key"] == "dee_misc_container", "SEPARATE_PROPERTY")
    require(provider["provider_field_lineage_status"] == "STATIC_MEMBER_TRANSFER_CLOSED", "PROVIDER_FIELD_STATUS")
    require(provider["consumers"] == ["Init", "Run", "GetTargets"], "PROVIDER_CONSUMERS")
    outputs = evidence["run_and_targets_outputs"]
    require(outputs["run_return_role"] == "completion status scalar", "RUN_RETURN")
    require(outputs["get_targets_return_role"] == "target-label string vector", "GETTARGETS_RETURN")
    require(outputs["neither_method_returns_plugin_process_packet"] is True, "NO_PACKET_RETURN")
    boundary = evidence["cross_module_packet_identity"]
    require(boundary["storage_provider_to_plugin_process_packet"] == "UNBOUND", "PROVIDER_PROCESS_PACKET")
    require(boundary["AtmosMezz_Run_GetTargets_to_plugin_process_packet_same_object"] == "UNBOUND", "RUN_PACKET")
    require(boundary["same_parent_or_source_generation"] == "UNBOUND", "GENERATION")
    require(boundary["native_execution"] == "NOT_RUN", "NATIVE_STATUS")
    require(boundary["pcm_or_bitstream_read_or_saved"] is False, "PAYLOAD_BOUNDARY")
    require(boundary["key_license_or_authentication_accessed"] is False, "SECRET_BOUNDARY")
    report = (PACKAGE / "REPORT.md").read_text(encoding="utf-8").lower()
    require("storage-provider member" in report and "process packet" in report, "REPORT_EDGE")
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
        "provider_field=STATIC_CLOSED",
        "process_packet_join=UNBOUND",
        "generation=UNBOUND",
        "native_observation=NOT_RUN",
    )


if __name__ == "__main__":
    main()
