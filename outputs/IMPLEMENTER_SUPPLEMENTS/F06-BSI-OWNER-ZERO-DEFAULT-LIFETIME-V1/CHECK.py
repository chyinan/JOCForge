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
PACKAGE_ID = "F06-BSI-OWNER-ZERO-DEFAULT-LIFETIME-V1"
INCOMING_MANIFEST = "dc9a0d3c3363b4fc2c41b05f89e11b982beb48d01662d89c3d2ca00355aa184d"
IMAGE_SHA256 = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"


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
        "status": "ACTIVE_TYPED_OWNER_ZERO_DEFAULT_BOUND; ALTERNATE_WRITERS_NOT_EXHAUSTIVE; U16_TO_STANDARD_DIALNORM_UNBOUND",
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
    require(evidence["status"].startswith("ACTIVE_TYPED_OWNER_ZERO_DEFAULT_BOUND;"),
            "owner default status mismatch")
    require(evidence["target_image_sha256"] == IMAGE_SHA256, "target image identity mismatch")
    owner = evidence["owner_construction"]
    require(owner["owner_field_type"] == "SIGNED_U16"
            and owner["owner_field_offset_hex"] == "0x34"
            and owner["block_clear_covers_owner_plus_0x34"] is True
            and owner["typed_initializer_store_width_bits"] == 32
            and owner["typed_initializer_store_value"] == 0
            and owner["typed_initializer_store_targets_owner_plus_0x34"] is True
            and owner["constructor_default_before_conditional_updates"] == 0,
            "typed owner zero-default path mismatch")
    runtime = evidence["parent_runtime_identity"]
    require(runtime["current_run_bsi_u16_value"] == "NOT_CAPTURED_PER_NO_REPEAT"
            and runtime["header_u5_observed"] == 17,
            "runtime/default and header-U5 boundary mismatch")
    require(evidence["unbound"]["owner_u16_to_standard_ec3_dialnorm_numeric_mapping"] is True
            and evidence["unbound"]["alternate_writers_across_all_dependencies_and_lifetimes"] is True,
            "open mapping/alternate-writer scope was promoted")
    require(evidence["access_boundary"]["new_encode_run"] == "NOT_RUN"
            and evidence["access_boundary"]["pcm_body_bytes_read"] == 0
            and evidence["access_boundary"]["ec3_body_bytes_read"] == 0
            and evidence["access_boundary"]["private_key_or_license_contents_read"] is False,
            "access boundary mismatch")

    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["evidence_class"] == "STATIC_MODEL_ONLY_NOT_CURRENT_RUN_OBSERVATION"
            and len(vectors["cases"]) == 2,
            "vector evidence class/count mismatch")
    default_case = vectors["cases"][0]
    value = default_case["input"]["initializer_store_value"] & 0xFFFF
    require(value == default_case["expected_constructor_default_u16"] == 0,
            "constructor default vector mismatch")
    require(default_case["current_run_owner_u16"] == "NOT_CAPTURED_PER_NO_REPEAT"
            and default_case["expected_standard_ec3_dialnorm"] == "NOT_SUPPLIED",
            "current-run/mapping boundary mismatch")
    header_case = vectors["cases"][1]
    require(header_case["expected_numeric_mapping"] == "NOT_SUPPLIED"
            and header_case["equality_claim"] == "PROHIBITED",
            "header U5 equality was incorrectly inferred")

    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_addendum"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_addendum"]["checker_normal"] == "PASS"
            and sources["incoming_addendum"]["checker_python_optimized"] == "PASS",
            "incoming addendum provenance mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST,
            "Wash verification source mismatch")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "constructor_default_u16": 0,
                      "current_run_u16": "NOT_CAPTURED_PER_NO_REPEAT",
                      "dialnorm_numeric_mapping": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
