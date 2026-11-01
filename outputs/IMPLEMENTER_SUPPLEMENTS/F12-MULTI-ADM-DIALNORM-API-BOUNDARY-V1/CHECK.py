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
PACKAGE_ID = "F12-MULTI-ADM-DIALNORM-API-BOUNDARY-V1"
INCOMING_MANIFEST = "496e2fccadf0fcf1f369f5862d83eb62da339e79e3fbe7d090ca30d7cd536483"


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
        "status": "THREE_CONDITIONAL_SAMPLE_MATCHES; API_BOUNDARY_PARTIAL; NATIVE_UNIT_AND_BSI_MAPPING_UNBOUND",
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
    require(evidence["status"].endswith("NATIVE_UNIT_AND_BSI_MAPPING_UNBOUND"), "evidence status")
    require(len(evidence["cross_input_observations"]) == 3, "cross-input observation count")
    expected = [
        ("authorized_adm_a", "-20.665958", 21),
        ("authorized_adm_b", "-14.418104", 14),
        ("authorized_adm_c", "-17.093924", 17),
    ]
    for observation, (sample_id, scalar, u5) in zip(evidence["cross_input_observations"], expected):
        require(observation["sample_id"] == sample_id and observation["measurement_scalar_decimal"] == scalar,
                f"scalar receipt {sample_id}")
        require(observation["native_unit"] == "NOT_STATED_IN_NATIVE_LOG", f"unit boundary {sample_id}")
        require(observation["u5"] == u5 and observation["payload_bytes_read"] == 0
                and observation["raw_prefix_retained"] is False, f"bounded U5 probe {sample_id}")
        require(observation["conditional_model"] == "MATCH_IF_SCALAR_IS_SELECTED_LUFS", f"conditional match {sample_id}")
        t = int(float(scalar) - 0.5)
        predicted = -max(-31, min(-1, t))
        require(predicted == u5, f"conditional model arithmetic {sample_id}")

    first_love = evidence["additional_scalar_observation"]
    require(first_love["measurement_scalar_decimal"] == "-21.010569"
            and first_love["u5_observation"] == "NOT_RUN"
            and first_love["native_unit"] == "NOT_STATED_IN_NATIVE_LOG", "First Love scope")
    lineage = evidence["static_lineage"]
    require(lineage["unit_conversion_or_rounding_before_encoder_api"] == "NOT_OBSERVED", "unit/rounding scope")
    require(lineage["frame_field_candidate"]["encoder_api"] == "ddpi_enc_setparams parameter ID 0x12"
            and lineage["frame_field_candidate"]["join_to_measurement_scalar"] == "NOT_PROVEN"
            and lineage["frame_field_candidate"]["serialized_bsi_field"] == "NOT_PROVEN", "frame/API boundary")
    require(lineage["final_bsi_writer_or_bit_packer"] == "NOT_TRACED"
            and lineage["same_generation_measurement_to_encoder_api_join"] == "NOT_PROVEN", "final BSI boundary")
    comparison = evidence["comparison_limits"]
    require(comparison["official_openjoc_cli_cross_check"] == "NOT_RUN"
            and comparison["three_matches_prove_a_general_native_rule"] is False, "comparison scope")
    require(evidence["minimum_next_observation"]["status"] == "NOT_RUN"
            and evidence["minimum_next_observation"]["authorization_from_this_package"] is False, "next observation status")
    require(evidence["policy"]["private_key_hmac_or_license_content_read"] is False
            and evidence["policy"]["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE", "policy boundary")

    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["manifest_seal_matches"] is True
            and sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS", "incoming provenance")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST
            and verification["implementation_delivery"] == "REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false",
            "verification scope")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "conditional_matches": 3, "native_unit": "UNBOUND", "final_bsi": "NOT_PROVEN"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
