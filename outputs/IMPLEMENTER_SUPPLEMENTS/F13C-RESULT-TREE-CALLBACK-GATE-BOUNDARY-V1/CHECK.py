import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "F13C-RESULT-TREE-CALLBACK-GATE-BOUNDARY-V1"
INCOMING_MANIFEST = "b42be389f86010d6d4d62a7b726c550f0eaef2487ec57966c57fd8da25baa1ab"
INCOMING_CHECKSUMS = "6b604c5146ee7e511d27a945adf9663ad708c7013a6362e0639370c5b41cce84"
PARENT_V5 = "03d3355763663e2dd6dc5202c87d1af1c1bcdf4d272670765908c3b07585445f"
GHIDRA_LOG_HASHES = {
    "work/ghidra_event_context_assignments_script.log": "428ee3eab6d82d7ede32fc3979e65934bdfe506cabe56cbb72a0079e4b3e9538",
    "work/ghidra_event_factory_machine_script.log": "3ef688ed9f53d3510a344c18a27272e5f1e432f5635dd7baeadf2d4a57044248",
    "work/ghidra_event_owner_factory_headless.log": "52aa31f3002c04ba9e7cd655c24ccb8b339f95d0505bf5f274a54dd537485f7b",
    "work/ghidra_event_context_callers2_script.log": "b9dff0657ee4314af3aefb10b91cad0c83601039532f2cecd6b18538ad26751b",
    "work/ghidra_event_parent_constructors_headless.log": "902c4639b639363f02fece2db96adfc1427569839185a01a63f2bc7f7f634784",
    "work/ghidra_event_owner_creators_headless.log": "ef2bacd3894dab31be77d0ba37fcd8d2deb25b2b8399401176bf90fb88466655",
    "work/ghidra_event_outer_caller_script.log": "e33feba4e454de8394dee44dee5cc73e2d44b89efb51de305f4fbffd217ab2cd",
    "work/ghidra_atmos_event_state_script.log": "ff4888d49fb050c99ed9b752dedea1d69b250f750f0989d2a995e0a9e42dc25c",
}


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
        "status": "ATMOSMEZZ_RESULT_CALLBACK_GATED_FOR_THIS_DESCRIPTOR; F13C_WRITER_UNBOUND",
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
    require(manifest["schema"] == "jocforge-washed-implementer-supplement.v1"
            and manifest["package_id"] == PACKAGE_ID
            and manifest["implementation_ready"] is False
            and manifest["source_reverse_manifest_sha256"] == INCOMING_MANIFEST, "manifest identity/readiness")
    rows = {row["file"]: row for row in manifest["files"]}
    require(len(rows) == len(manifest["files"]) and set(rows) == PAYLOADS, "manifest payload inventory")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"], f"payload hash: {name}")

    evidence = read_json("EVIDENCE.json")
    require(evidence["package_id"] == PACKAGE_ID and evidence["implementation_ready"] is False, "evidence identity")
    require(evidence["owner"]["type"] == "AtmosMasterFileStorageDesc registration"
            and evidence["owner"]["context_member"] == "owner+0x10"
            and evidence["owner"]["context_address_in_imported_image"] == "0x141038a80", "owner identity")
    require(evidence["owner"]["gate_offset"] == "+0x1D0" and evidence["owner"]["gate_value"] == 0,
            "owner gate state")
    require([row["event_code"] for row in evidence["result_adapters"]] == [2, 3], "event adapter codes")
    require(evidence["dispatcher"]["outcome_for_this_descriptor"] == "INDIRECT_CALLBACK_SKIPPED_GATE_ZERO",
            "dispatcher branch")
    require(evidence["dispatcher"]["payload_type"] == "generic result-record payload; not a typed dmixmod value",
            "payload type boundary")
    for key in ["typed_ddp_dmixmod_edge", "selected_ddp_joc_5_1_stage", "f13c_frame_writer",
                "preferred_downmix_mode_to_native_dmixmod"]:
        require(evidence["unbound"][key] == "UNBOUND", f"unbound edge {key}")
    require(evidence["execution_scope"]["encoder_launched"] is False
            and evidence["execution_scope"]["pcm_read"] is False
            and evidence["execution_scope"]["encoded_payload_read"] is False
            and evidence["execution_scope"]["private_license_hmac_key_content_read_or_exported"] is False,
            "execution boundary")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST, "source binding")
    require(sources["incoming_package"]["sha256sums_file_sha256"] == INCOMING_CHECKSUMS, "checksum file identity")
    require({row["path"]: row["sha256"] for row in sources["recomputed_ghidra_log_sources"]} == GHIDRA_LOG_HASHES,
            "Ghidra log source bindings")
    require(sources["parent_source_package"]["manifest_sha256"] == PARENT_V5, "parent source manifest")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "gate": "ZERO_FOR_THIS_DESCRIPTOR", "f13c_writer": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
