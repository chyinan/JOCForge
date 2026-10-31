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
    manifest_path = PACKAGE / "MANIFEST.json"
    require(manifest_path.is_file(), "MANIFEST_MISSING")
    payload = manifest_path.read_bytes()
    manifest_hash = hashlib.sha256(payload).hexdigest()
    sidecar = (PACKAGE / "MANIFEST.sha256").read_text(encoding="utf-8").split()
    require(bool(sidecar) and sidecar[0] == manifest_hash, "MANIFEST_SIDECAR")
    manifest = json.loads(payload.decode("utf-8"))
    require(manifest["package_id"] == "HOST-TASK-ROUTE-FACTORY-SELECTION-FOLLOWUP-V1", "PACKAGE_ID")
    entries = manifest["payloads"]
    expected_names = {
        "CHECK.py",
        "CONTROLLED-VECTORS.json",
        "EVIDENCE.json",
        "REPORT.md",
        "SOURCE-IDENTITIES.json",
    }
    require({entry["file"] for entry in entries} == expected_names, "SANITIZED_PAYLOAD_SET")
    for entry in entries:
        path = PACKAGE / entry["file"]
        require(path.is_file(), f"PAYLOAD_MISSING:{entry['file']}")
        require(path.stat().st_size == entry["bytes"], f"PAYLOAD_SIZE:{entry['file']}")
        require(sha256(path) == entry["sha256"], f"PAYLOAD_HASH:{entry['file']}")
    return manifest, manifest_hash


def verify_source_identities(source_ids: dict, evidence: dict) -> None:
    require(source_ids["analysis"]["tool"] == "Ghidra", "TOOL")
    require(source_ids["analysis"]["version"] == "12.2 PUBLIC", "GHIDRA_VERSION")
    require(source_ids["analysis"]["read_only_project"] is True, "READ_ONLY")
    require(source_ids["analysis"]["ida_used"] is False, "IDA_BOUNDARY")
    expected = {
        "dee.exe": ("65485b6d35fa534b25109ab6f238971aaded73fc99c7f7fb119d1a5d5299234e", 4233728),
        "dee_audio_filter_ddp_atmos.dll": ("3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2", 10942464),
    }
    require(len(source_ids["sources"]) == 2, "SOURCE_COUNT")
    for source in source_ids["sources"]:
        name = source["file_name"]
        require(name in expected, f"SOURCE_NAME:{name}")
        require((source["sha256"], source["bytes"]) == expected[name], f"SOURCE_HASH:{name}")
    require(evidence["source_modules"] == source_ids["sources"], "EVIDENCE_SOURCE_BINDING")


def verify_vectors(vectors: dict, evidence: dict) -> None:
    require(vectors["classification"] == "STATIC_REGISTRATION_CROSSWALK_ONLY_NOT_TASK_SELECTION", "VECTOR_CLASSIFICATION")
    require(vectors["native_execution"] == "NOT_RUN", "VECTOR_NATIVE_STATUS")
    entries = evidence["dll_factory_registration"]["entries"]
    by_index = {entry["index"]: entry["descriptor_name"] for entry in entries}
    require(by_index == {
        0: "encode_to_atmos_ddp",
        1: "atmos_mezz_to_atmos_ddp",
        2: "pcm_to_atmos_ddp",
    }, "FACTORY_INDEX_DESCRIPTOR_MAP")
    require(len(vectors["vectors"]) == 4, "VECTOR_COUNT")
    seen: set[str] = set()
    for vector in vectors["vectors"]:
        case_id = vector["case_id"]
        require(case_id not in seen, f"DUPLICATE_CASE:{case_id}")
        seen.add(case_id)
        inputs = vector["input"]
        expected = vector["expected_static"]
        if case_id.startswith("audio_filter_factory_index_"):
            index = inputs["factory_index"]
            require(inputs["category"] == "audio_filter", f"VECTOR_CATEGORY:{case_id}")
            require(index in by_index, f"VECTOR_INDEX:{case_id}")
            require(expected == {"descriptor_name": by_index[index]}, f"VECTOR_RESULT:{case_id}")
        elif case_id == "host_encode_route_factory_selection":
            require(inputs["host_route_key"] == "encode_to_atmos_ddp", "HOST_ROUTE_KEY")
            require(expected == {"selected_factory_index": None, "status": "UNBOUND"}, "HOST_ROUTE_UNBOUND_VECTOR")
        else:
            raise SystemExit(f"CHECK_FAILED:UNKNOWN_VECTOR:{case_id}")
    require(seen == {
        "audio_filter_factory_index_zero",
        "audio_filter_factory_index_one",
        "audio_filter_factory_index_two",
        "host_encode_route_factory_selection",
    }, "VECTOR_CASE_SET")


def verify_evidence(evidence: dict) -> None:
    require(
        evidence["result"]
        == "DLL_REGISTRATION_NAME_TO_FACTORY_INDEX_CLOSED; HOST_TASK_ROUTE_TO_FACTORY_INDEX_UNBOUND; HOST_PACKET_IDENTITY_AND_GENERATION_UNBOUND; NATIVE_OBSERVATION_NOT_RUN",
        "RESULT",
    )
    require(evidence["evidence_tier"] == "GHIDRA_STATIC", "EVIDENCE_TIER")
    host = evidence["host_route"]
    require(host["route_key"] == "encode_to_atmos_ddp", "HOST_ROUTE")
    require(host["runtime_selected_route"] == "NOT_OBSERVED", "RUNTIME_ROUTE")
    require(host["selected_audio_filter_factory_index"] is None, "SELECTED_INDEX_MUST_REMAIN_UNBOUND")
    require(host["route_to_factory_index_status"] == "UNBOUND", "ROUTE_FACTORY_BOUNDARY")
    require("no typed value transfer" in host["reason"], "ROUTE_FACTORY_GAP_REASON")

    loader = evidence["host_audio_filter_loader"]
    require(loader["category"] == "audio_filter", "LOADER_CATEGORY")
    require(loader["loader_resolves_category_export"] is True, "LOADER_EXPORT_RESOLUTION")
    require(loader["factory_table_count"] == 3, "LOADER_FACTORY_COUNT")
    require(loader["registered_indices"] == [0, 1, 2], "LOADER_INDEX_REGISTRATION")

    registration = evidence["dll_factory_registration"]
    require(registration["mapping_status"] == "STATIC_DESCRIPTOR_TO_FACTORY_INDEX_CLOSED", "DLL_MAP_STATUS")
    require(len(registration["entries"]) == 3, "DLL_ENTRY_COUNT")
    require(registration["host_route_identity_join"].startswith("UNBOUND"), "DLL_HOST_JOIN")

    identity = evidence["identity_boundaries"]
    require(identity["runtime_parent_task_token"] == "NOT_OBSERVED", "PARENT_TASK_TOKEN")
    require(identity["AtmosMezz_Run_GetTargets_to_plugin_process_packet_same_object"] == "UNBOUND", "PACKET_IDENTITY")
    require(identity["same_parent_or_source_generation"] == "UNBOUND", "GENERATION")
    require(identity["native_execution"] == "NOT_RUN", "NATIVE_STATUS")
    require(identity["pcm_or_bitstream_read_or_saved"] is False, "PAYLOAD_BOUNDARY")
    require(identity["key_license_or_authentication_accessed"] is False, "SECRET_BOUNDARY")

    report = (PACKAGE / "REPORT.md").read_text(encoding="utf-8").lower()
    require("unbound" in report and "not_run" in report, "REPORT_BOUNDARIES")
    require("0x" not in report and "fun_" not in report, "REPORT_SANITIZED")


def main() -> None:
    manifest, manifest_hash = verify_manifest()
    evidence = read_json(PACKAGE / "EVIDENCE.json")
    source_ids = read_json(PACKAGE / "SOURCE-IDENTITIES.json")
    vectors = read_json(PACKAGE / "CONTROLLED-VECTORS.json")
    verify_source_identities(source_ids, evidence)
    verify_vectors(vectors, evidence)
    verify_evidence(evidence)
    print(
        "CHECK_OK",
        f"manifest_sha256={manifest_hash}",
        f"payloads={len(manifest['payloads'])}",
        f"vectors={len(vectors['vectors'])}",
        "host_task_route_factory_index=UNBOUND",
        "packet_generation=UNBOUND",
        "native_observation=NOT_RUN",
    )


if __name__ == "__main__":
    main()
