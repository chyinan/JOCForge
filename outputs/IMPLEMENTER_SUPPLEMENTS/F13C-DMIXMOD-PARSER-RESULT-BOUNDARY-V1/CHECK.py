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
PACKAGE_ID = "F13C-DMIXMOD-PARSER-RESULT-BOUNDARY-V1"
INCOMING_MANIFEST = "0473987f247572cd78d164afd901e2f7495c60fd8aab765167e18ad710e3c15f"
WASH_V2 = "7dabde419bf0a5e378d94613393686298c2baff4053f015d9d09222e4e9fad4e"


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
        "status": "PARSER_TO_GENERIC_RESULT_HANDOFF_BOUND; NATIVE_DEFAULT_AND_AUDIO_CONSUMER_UNBOUND",
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
    require(evidence["parser"]["function"] == "FUN_1404ecea0"
            and evidence["parser"]["fields"] == ["lorocmixlev", "lorosurmixlev", "dmixmod", "ltrtcmixlev", "ltrtsurmixlev"]
            and evidence["parser"]["dmixmod_converter"] == "FUN_1404dddd0"
            and evidence["parser"]["dmixmod_slot"] == "0x11", "parser field mapping")
    require(evidence["parser"]["native_value_after_omission"] == "UNBOUND"
            and evidence["parser"]["initializing_write_located"] is False, "omitted-field boundary")
    require(evidence["call_chain"][0]["callsite"] == "0x1403f50e5"
            and evidence["call_chain"][1]["callsite"] == "0x14048054e"
            and evidence["call_chain"][2]["callsite"] == "0x1404805d8"
            and evidence["call_chain"][2]["callee"] == "FUN_1404807d0", "call chain")
    require(evidence["generic_result_handoff"]["explicit_downmix_key_lookup_in_targeted_body"] == "NOT_FOUND"
            and evidence["generic_result_handoff"]["result_record_identity_to_slot_0x11"] == "UNBOUND", "generic result limit")
    for key in ["slot_0x11_to_ddp_joc_ec3_frame_writer", "slot_0x11_to_stereo_postmix",
                "slot_0x11_to_target_5_1_joc_processing_or_signaling", "preferred_downmix_mode_to_native_dmixmod",
                "native_value_when_dmixmod_omitted"]:
        require(evidence["consumer_status"][key] == "UNBOUND", f"consumer edge {key}")
    require(evidence["front_end_boundary"]["preferred_stereo_downmix_display_value" if "preferred_stereo_downmix_display_value" in evidence["front_end_boundary"] else "display_value"] == "Stereo"
            and evidence["front_end_boundary"]["ui_value_is_native_default"] is False, "front-end default boundary")
    require(evidence["execution_scope"]["encoder_launched"] is False
            and evidence["execution_scope"]["pcm_read"] is False
            and evidence["execution_scope"]["encoded_payload_read"] is False
            and evidence["execution_scope"]["private_license_hmac_key_content_read_or_exported"] is False,
            "execution and secret boundary")

    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["related_wash_v2"]["manifest_sha256"] == WASH_V2,
            "source binding")
    require(sources["incoming_package"]["incoming_checker"] == "NOT_PROVIDED; Wash performed manual inventory/hash checks",
            "incoming checker status")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST
            and verification["incoming_checker"] == "NOT_PROVIDED"
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "dmixmod_slot": "0x11", "audio_consumer": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
