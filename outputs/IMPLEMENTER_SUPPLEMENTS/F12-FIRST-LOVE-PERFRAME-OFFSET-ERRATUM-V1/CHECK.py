import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {
    "CHECK.py",
    "EVIDENCE.json",
    "INPUT-ONLY-VECTORS.json",
    "REPORT.md",
    "SOURCE-MANIFESTS.json",
    "WASH-VERIFICATION.json",
}
PACKAGE_ID = "F12-FIRST-LOVE-PERFRAME-OFFSET-ERRATUM-V1"
INCOMING_MANIFEST = "36f97eb3422cab3b3fedb7c94f22dcfca3b750f94b1cd6fa6702d48a64d89612"
CLOSURE_JSON = "f3f9f657c3fc004df5f8d820e3a15a61381a0c183713add811edb0cd0203d7a1"
OLD_WASH_MANIFEST = "e3bfcb7bde5fb217071c605c122cd4bfeeabaa8763df013e3c37e5289d047fcc"
CORRECTED_BYTE_OFFSET = 0x3C * 4


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
        "status": "PER_FRAME_DESTINATION_OFFSET_CORRECTED; PER_FRAME_VALUE_SEMANTICS_AND_FINAL_BSI_FIELD_OPEN",
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
    offset = evidence["corrected_type_arithmetic"]
    require(offset["param_1_native_type"] == "int*"
            and offset["expression"] == "param_1 + 0x3C"
            and offset["pointer_element_size_bytes"] == 4
            and int(offset["computed_destination_byte_offset"], 16) == CORRECTED_BYTE_OFFSET == 0xF0,
            "typed pointer offset arithmetic mismatch")
    require(offset["scalar_bearing_empireenc_byte_offset"] == "0x3C"
            and offset["same_destination"] is False
            and offset["param_4_10_semantic_source_join"] == "UNBOUND",
            "distinct field/source boundary mismatch")
    unchanged = evidence["unchanged_f12_boundary"]
    require(unchanged["measurement_scalar"] == -21.010569
            and unchanged["native_unit"] == "NOT_STATED_IN_NATIVE_LOG"
            and unchanged["final_bsi_dialnorm_field"] == "NOT_PROVEN",
            "unchanged native scalar boundary mismatch")
    require(evidence["access_boundary"]["new_measurement_for_erratum"] == "NOT_RUN"
            and evidence["access_boundary"]["pcm_payload_bitstream_or_output_read"] is False
            and evidence["access_boundary"]["private_key_hmac_or_license_contents_read"] is False,
            "erratum access boundary mismatch")

    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["evidence_class"] == "STATIC_TYPE_ARITHMETIC_ONLY" and len(vectors["cases"]) == 2,
            "type arithmetic vector scope mismatch")
    first, second = vectors["cases"]
    require(int(first["expected_byte_offset_hex"], 16)
            == int(first["element_offset_hex"], 16) * first["pointer_element_size_bytes"]
            == CORRECTED_BYTE_OFFSET,
            "int* vector expected offset mismatch")
    require(int(second["per_frame_write_byte_offset_hex"], 16)
            != int(second["scalar_config_field_byte_offset_hex"], 16)
            and second["expected_same_destination"] is False,
            "same-field claim remains in vector")

    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["updated_consumer_closure_json_sha256"] == CLOSURE_JSON
            and sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS"
            and sources["corrected_measurement_supplement"]["manifest_sha256"] == OLD_WASH_MANIFEST,
            "incoming/superseded package binding mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST
            and verification["incoming_consumer_closure_json_sha256"] == CLOSURE_JSON,
            "verification provenance mismatch")

    print(json.dumps({"status": "VALID_CORRECTION", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "param1_plus_0x3c_byte_offset": hex(CORRECTED_BYTE_OFFSET),
                      "same_as_empireenc_scalar_field": False,
                      "final_bsi_dialnorm": "NOT_PROVEN"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
