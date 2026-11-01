import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "PCM24-DDP-TYPED-SOURCE-CONFIG-TO-PARAM3-V1"
INCOMING_MANIFEST = "07c85b08a09508d059a620f10c84d24dd5002340977486e62ab9a1ed37109ecc"
POINTER_CAPTURE_MANIFEST = "93bd92a3a6004ea5efbdc54f94c558ab0058573db1b6792fb2c35fa936e22bc4"


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
    files = [{"file": name, "bytes": (PACKAGE / name).stat().st_size, "sha256": sha256(PACKAGE / name)}
             for name in sorted(PAYLOADS)]
    manifest = {
        "schema": "jocforge-washed-implementer-supplement.v1",
        "package_id": PACKAGE_ID,
        "status": "TYPED_CONFIG_TO_DDP_PARAM3_SAME_ACTIVATION_BOUND; HOST_ADM_OWNER_GENERATION_UNBOUND",
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
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"],
                f"payload hash mismatch: {name}")

    evidence = read_json("EVIDENCE.json")
    require(evidence["config_dispatch"]["method"] == "FUN_18013E9D0"
            and evidence["config_dispatch"]["source_option"]["key"] == "dee_audio_filter_source"
            and evidence["config_dispatch"]["source_option"]["value_type"]
            == "SafeType<std::vector<std::map<std::string,std::string>>>",
            "typed source option contract mismatch")
    capture = evidence["source_map_to_record"]["runtime_pointer_only_capture"]
    require(capture["package_manifest_sha256"] == POINTER_CAPTURE_MANIFEST
            and capture["same_ddp_owner_token_at_builder_and_writer"] is True
            and capture["same_FUN_18014ABA0_measurement_activation"] is True
            and capture["writer_body_executed"] is False,
            "same-activation DDP pointer capture boundary mismatch")
    identity = evidence["identity_boundary"]
    require(identity["host_selected_adm_owner_generation_to_ddp_param_3"] == "UNBOUND/NO"
            and identity["path_string_equality_as_identity"] is False
            and identity["shared_outer_stage_runner_frame_as_identity"] is False,
            "host owner/generation boundary mismatch")
    require(evidence["access_boundary"]["new_encoder_run"] == "NOT_RUN"
            and evidence["access_boundary"]["row_contents_or_pcm_read"] is False
            and evidence["access_boundary"]["payload_or_bitstream_read"] is False
            and evidence["access_boundary"]["private_key_auth_or_license_contents_read"] is False,
            "access boundary mismatch")

    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS"
            and sources["pointer_capture_parent"]["manifest_sha256"] == POINTER_CAPTURE_MANIFEST,
            "incoming package binding mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST,
            "Wash verification provenance mismatch")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "config_to_param3": "BOUND_WITHIN_ONE_DDP_ACTIVATION",
                      "host_adm_owner_generation": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
