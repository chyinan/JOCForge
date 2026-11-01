import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {
    "CHECK.py",
    "EVIDENCE.json",
    "OBSERVATION-PLAN.json",
    "REPORT.md",
    "SOURCE-MANIFESTS.json",
    "WASH-VERIFICATION.json",
}
PACKAGE_ID = "R138-INPUT-OBJECT-RUN-CALLSITE-BOUNDARY-V1"
INCOMING_MANIFEST = "0c79d0e16be7f07a559bfc9a57eee6c821156713c9898844a4258789e16b3352"
PARENT_MANIFEST = "164409eeda49b71e52a9800feaa25c91aae0f45580f258c78054e5533c1de9d9"


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(name):
    return json.loads((PACKAGE / name).read_text(encoding="utf-8"))


def build_manifest():
    actual = {path.name for path in PACKAGE.iterdir() if path.is_file()} - {"MANIFEST.json"}
    require(actual == PAYLOADS, f"payload inventory mismatch: {sorted(actual ^ PAYLOADS)}")
    files = [
        {"file": name, "bytes": (PACKAGE / name).stat().st_size, "sha256": sha256(PACKAGE / name)}
        for name in sorted(PAYLOADS)
    ]
    manifest = {
        "schema": "jocforge-washed-implementer-supplement.v1",
        "package_id": PACKAGE_ID,
        "status": "STATIC_CALLSITE_BOUND; R138_SELECTION_AND_OWNER_JOIN_UNBOUND",
        "implementation_ready": False,
        "source_reverse_manifest_sha256": INCOMING_MANIFEST,
        "files": files,
    }
    (PACKAGE / "MANIFEST.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def verify():
    expected = PAYLOADS | {"MANIFEST.json"}
    actual = {path.name for path in PACKAGE.iterdir() if path.is_file()}
    require(actual == expected, f"file inventory mismatch: {sorted(actual ^ expected)}")
    manifest = read_json("MANIFEST.json")
    require(
        manifest["schema"] == "jocforge-washed-implementer-supplement.v1"
        and manifest["package_id"] == PACKAGE_ID
        and manifest["implementation_ready"] is False
        and manifest["source_reverse_manifest_sha256"] == INCOMING_MANIFEST
        and manifest["status"] == "STATIC_CALLSITE_BOUND; R138_SELECTION_AND_OWNER_JOIN_UNBOUND",
        "manifest identity/readiness mismatch",
    )
    rows = {row["file"]: row for row in manifest["files"]}
    require(len(rows) == len(manifest["files"]) and set(rows) == PAYLOADS, "manifest payload inventory mismatch")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"], f"payload hash mismatch: {name}")

    evidence = read_json("EVIDENCE.json")
    require(evidence["package_id"] == PACKAGE_ID and evidence["implementation_ready"] is False, "evidence identity/readiness")
    sequence = evidence["static_callsite"]["sequence"]
    require([row["rva"] for row in sequence] == ["0xCA9F2", "0xCA9F9", "0xCA9FC"], "callsite sequence")
    require(sequence[0]["instruction"] == "MOV RCX, qword ptr [RSI+0x1F0]", "reader field load")
    require(sequence[2]["instruction"] == "CALL qword ptr [RAX+0x30]", "virtual Run call")
    require(evidence["static_callsite"]["recommended_stop"] == "dee.exe+0xCA9FC", "stop boundary")
    require(evidence["static_callsite"]["r138_runtime_capture"] == "NOT_CAPTURED", "runtime boundary")
    require(evidence["observed_schema_activation"]["task_profile_supplied"] is False, "schema profile")
    require(evidence["observed_schema_activation"]["audio_input_supplied"] is False, "schema audio")
    require(evidence["observed_schema_activation"]["route_selected"] is False, "schema route")
    require(evidence["observed_schema_activation"]["full_encode"] == "NOT_RUN", "schema encode")
    for key, value in {
        "audio_adm_owner_to_selected_reader": "UNBOUND",
        "reader_factory_table_count_slot": "UNBOUND",
        "reader_field_writer_or_constructor_identity": "UNBOUND",
        "parent_generation": "UNBOUND",
    }.items():
        require(evidence["edges"][key] == value, f"edge status: {key}")
    require(evidence["policy"]["new_encode_started"] is False and evidence["policy"]["r138_xml_read"] is False,
            "run/XML policy")
    require(evidence["policy"]["row_or_table_bytes_read"] is False and evidence["policy"]["pcm_read"] is False,
            "sample-read policy")
    require(evidence["policy"]["payload_or_encoded_output_read"] is False
            and evidence["policy"]["key_or_license_content_read"] is False,
            "payload/key policy")
    require(evidence["policy"]["implementation_behavior_changed"] is False, "implementation boundary")

    plan = read_json("OBSERVATION-PLAN.json")
    require(plan["execution_status"] == "NOT_RUN; SPECIFICATION_ONLY" and plan["encode_authorized_by_this_plan"] is False,
            "observation plan status")
    require(plan["stop"]["rva"] == "0xCA9FC" and plan["stop"]["metadata_only"] is True, "plan stop")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST, "incoming provenance")
    require(sources["upstream_parent"]["manifest_sha256"] == PARENT_MANIFEST, "parent provenance")
    require(sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS", "incoming checker status")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST, "verification provenance")
    require(verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "delivery scope")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "static_callsite": "dee.exe+0xCA9FC", "r138_reader": "UNBOUND",
                      "observation_plan": "NOT_RUN"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
