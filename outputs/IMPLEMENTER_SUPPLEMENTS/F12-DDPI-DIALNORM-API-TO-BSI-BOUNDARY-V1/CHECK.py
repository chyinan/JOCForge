import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "EVIDENCE.json", "REPORT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "F12-DDPI-DIALNORM-API-TO-BSI-BOUNDARY-V1"
INCOMING = "ee62a344ae5c5e9680aecc6002c37fecbc6efdaae6d1e12a0504c68ac7b5a0ba"


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
        "status": "DDPI_ID_0x12_VALIDATION_BOUND; FINAL_BSI_U5_MAPPING_NOT_PROVEN",
        "implementation_ready": False,
        "source_reverse_manifest_sha256": INCOMING,
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
            and manifest["source_reverse_manifest_sha256"] == INCOMING, "manifest identity/readiness")
    rows = {row["file"]: row for row in manifest["files"]}
    require(len(rows) == len(manifest["files"]) and set(rows) == PAYLOADS, "manifest payload inventory")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"], f"payload hash: {name}")
    evidence = read_json("EVIDENCE.json")
    require(evidence["package_id"] == PACKAGE_ID and evidence["implementation_ready"] is False, "evidence identity")
    require(evidence["api_parameter"]["parameter_id"] == "0x12"
            and evidence["api_parameter"]["valid_integer_range"] == [1, 31]
            and evidence["stream_field"]["destination"] == "stream+0x9C", "API parameter field")
    require(evidence["downstream"]["first_converter"]["actual_stream_offset"] == "+0xA8"
            and evidence["downstream"]["first_converter"]["equals_validated_stream_plus_0x9C"] is False
            and evidence["downstream"]["standard_header_u5_bit_offset"] == "NOT_PROVEN", "writer offset boundary")
    require(evidence["measurement_join"]["measured_float_to_parameter_id_0x12"] == "NOT_PROVEN"
            and evidence["measurement_join"]["native_measurement_unit"] == "NOT_STATED_IN_NATIVE_LOG", "measurement boundary")
    require(evidence["policy"]["other_encoder_or_ffmpeg_used"] is False
            and evidence["policy"]["pcm_payload_bitstream_output_or_key_material_read"] is False
            and evidence["policy"]["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE", "policy")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING
            and sources["incoming_package"]["manifest_seal_matches"] is True, "source identity")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "api_id": "0x12", "final_u5": "NOT_PROVEN"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
