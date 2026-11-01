import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "F06-BSI-OWNER-TO-FINAL-AU-BOUNDARY-V1"
SOURCE_MANIFESTS = {
    "native-f06-bsi-final-au-writer-generation-addendum": "5d8af71b76ae1b0d56affbdbaa0679c143c31ff5e939727df657656507017647",
    "native-f06-bsi-owner-lifetime-default-addendum": "dc9a0d3c3363b4fc2c41b05f89e11b982beb48d01662d89c3d2ca00355aa184d",
    "native-f06-bsi-u16-standard-header-generation-boundary-addendum": "034cc25f8b12e3794bc5580468a0cf7aa54928f13613496352acb46422aef104",
    "native-f06-bsi-u16-value-lineage-addendum": "33cb42032997d5f79ddaf50b6e0b7eeebaefac77e7aea4863d18377ccac49d89",
    "native-f06-source-vector-bsi-owner-generation-addendum": "3d03c7894b14dead56c2dea63be9b5b5d50e4914cab751d5906c4ca6a5f6ad66",
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
        "status": "SAME_GENERATION_OWNER_TO_FINAL_EC3_FRAME_BOUND_FOR_ONE_RUN; U16_TO_HEADER_U5_UNBOUND",
        "implementation_ready": False,
        "source_manifest_sha256s": SOURCE_MANIFESTS,
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
            and manifest["source_manifest_sha256s"] == SOURCE_MANIFESTS, "manifest identity/readiness")
    rows = {row["file"]: row for row in manifest["files"]}
    require(len(rows) == len(manifest["files"]) and set(rows) == PAYLOADS, "manifest payload inventory")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"], f"payload hash: {name}")

    evidence = read_json("EVIDENCE.json")
    require(evidence["package_id"] == PACKAGE_ID and evidence["implementation_ready"] is False, "evidence identity")
    writer = evidence["full_run_writer_observation"]
    require(writer["dee_exit_code"] == 0 and writer["frame_index"] == 3481
            and writer["frame_bytes"] == 4096 and writer["header_probe_bytes"] == 7
            and writer["header_result"]["dialnorm_u5"] == 17 and writer["payload_bytes_read"] == 0,
            "full-run writer/header scope")
    default = evidence["typed_owner_default_path"]
    require(default["owner_field_offset"] == "0x34" and default["owner_field_type"] == "signed U16"
            and default["explicit_constructor_value"] == 0, "owner default scope")
    short = evidence["separate_short_run_observation"]
    require(short["owner_u16"] == 0 and short["first_filewriter_length"] == 0
            and short["join_to_first_nonempty_writer_frame"] == "UNBOUND", "separate short-run scope")
    fields = evidence["field_relationships"]
    require(fields["owner_u16_plus_0x34_to_standard_header_u5"] == "UNBOUND/NOT_PROVEN"
            and fields["full_run_owner_u16_value"] == "NOT_REREAD", "U16/U5 relation")
    require(evidence["scope_deviations"]["earlier_short_run_prefix_bytes_read"] == 16
            and evidence["scope_deviations"]["later_hash_stream_bytes"] == 262144
            and evidence["scope_deviations"]["raw_bytes_retained"] is False, "scope deviation")
    require(evidence["policy"]["private_key_license_or_hmac_content_read"] is False
            and evidence["policy"]["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE", "policy")
    sources = read_json("SOURCE-MANIFESTS.json")
    require({row["package_id"]: row["manifest_sha256"] for row in sources["source_packages"]} == SOURCE_MANIFESTS,
            "source bindings")
    verify = read_json("WASH-VERIFICATION.json")
    require(verify["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "delivery scope")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "frame_index": writer["frame_index"], "u5": writer["header_result"]["dialnorm_u5"],
                      "u16_to_u5": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
