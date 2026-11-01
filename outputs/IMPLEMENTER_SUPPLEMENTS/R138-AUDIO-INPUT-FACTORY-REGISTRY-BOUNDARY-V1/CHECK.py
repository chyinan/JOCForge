import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {
    "CHECK.py",
    "EVIDENCE.json",
    "REPORT.md",
    "SOURCE-MANIFESTS.json",
    "WASH-VERIFICATION.json",
}
PACKAGE_ID = "R138-AUDIO-INPUT-FACTORY-REGISTRY-BOUNDARY-V1"
INCOMING_MANIFEST = "9fa06b6a89970328b3c9654125a04d9f44193bd1ec4624d46a6a30841c110060"
INCOMING_SEAL = "2b4f96b4c5cf648c34769fdbc008639147368e8aab2e91eb57c4ea67a9315d0a"


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
        "status": "GENERIC_HOST_AUDIO_INPUT_REGISTRY_BOUND; R138_FACTORY_AND_TYPED_READER_UNBOUND",
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
            and manifest["source_reverse_manifest_sha256"] == INCOMING_MANIFEST,
            "manifest identity/readiness mismatch")
    rows = {row["file"]: row for row in manifest["files"]}
    require(len(rows) == len(manifest["files"]) and set(rows) == PAYLOADS, "manifest payload inventory mismatch")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"], f"payload hash mismatch: {name}")

    evidence = read_json("EVIDENCE.json")
    require(evidence["package_id"] == PACKAGE_ID and evidence["implementation_ready"] is False, "evidence identity")
    route = {row["function"]: row for row in evidence["generic_host_route"]}
    require(route["FUN_14016AF20"]["finding"].startswith("maps audio_input category"), "category getter mapping")
    require(route["FUN_14016A1C0"]["rva"] == "0x16A1C0", "LoadOne callback path")
    require(route["FUN_140049120"]["finding"].startswith("enumerates candidate configs"), "candidate enumeration scope")
    scan = evidence["candidate_getter_scan"]
    require(scan["exact_export"] == "dlb_phoenix_get_audio_input_handle"
            and scan["hash_bound_candidate_images"] == 7
            and scan["exact_match_count_each"] == 0
            and scan["inventory_is_complete_successful_r138_module_receipt"] is False, "candidate scan limit")
    for key, value in {
        "r138_selected_factory_module_and_image_hash": "UNBOUND/NO",
        "r138_selected_factory_local_callback_slot": "UNBOUND/NO",
        "r138_stage_runner_selected_candidate_index": "UNBOUND/NO",
        "typed_audio_reader_owner_and_vtable": "UNBOUND/NO",
        "effective_plugin_roots_and_successful_run_module_set": "UNBOUND/NO",
    }.items():
        require(evidence["unbound"][key] == value, f"unbound status {key}")
    require(evidence["excluded_string_reference"]["owner"].startswith("FUN_140035700")
            and evidence["excluded_string_reference"]["used_as_input_factory_evidence"] is False, "output-side string exclusion")
    require(evidence["minimum_next_observation"]["status"] == "NOT_RUN"
            and evidence["minimum_next_observation"]["authorization_from_this_package"] is False, "next observation status")
    require(evidence["policy"]["new_dee_launch"] is False
            and evidence["policy"]["license_content_read"] is False
            and evidence["policy"]["pcm_or_payload_read"] is False
            and evidence["policy"]["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE", "policy boundary")

    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["seal_sha256"] == INCOMING_SEAL, "source identity")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST
            and verification["incoming_seal_sha256"] == INCOMING_SEAL
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "candidate_images": 7, "r138_factory": "UNBOUND/NO"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
