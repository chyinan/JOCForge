import hashlib
import json
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {
    "BOUNDARY-CANDIDATE.json",
    "CHECK.py",
    "EVIDENCE.json",
    "REPORT.md",
    "SOURCE-MANIFESTS.json",
    "WASH-VERIFICATION.json",
}
PACKAGE_ID = "F12-FIRST-LOVE-NATIVE-SCALAR-ENCODER-CONFIG-BOUNDARY-V1"
INCOMING_MANIFEST = "77755d22ea8cc19b90dfd03c4b27a552b9c807f31a74f548ae6abfc8abd0afd5"
NATIVE_SCALAR = -21.010569


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
        "status": "MEASURE_ONLY_SCALAR_OBSERVED; FLOAT_CONFIG_HANDOFF_BOUND; NATIVE_UNIT_AND_SERIALIZED_BSI_FIELD_OPEN",
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
    native = evidence["native_observation"]
    require(native["result"]["scalar"] == NATIVE_SCALAR
            and native["result"]["unit"] == "NOT_STATED_IN_NATIVE_LOG"
            and native["source_sha_recomputed_in_this_attempt"] is False,
            "native sample/scalar provenance mismatch")
    require(native["measure_only"] is True
            and native["window"] == {"time_base": "file_position", "start": "2160", "end": "2208"}
            and native["loaded_module_pid_join"] is False,
            "measurement profile or loaded-image boundary mismatch")
    require(evidence["metadata_scan"]["data_chunk_bytes_read"] == 0
            and evidence["metadata_scan"]["dbmd_parsed"] is False
            and native["output"]["output_bytes_read"] == 0,
            "media/output access boundary mismatch")
    chain = evidence["static_config_handoff"]["copy_chain"]
    require(len(chain) == 3
            and chain[0]["destination"] == "owner +0xF6C"
            and chain[1]["destination"] == "encode-config +0x114"
            and "EmpireEnc" in chain[2]["destination"],
            "static config copy path mismatch")
    require(evidence["static_config_handoff"]["pre_api_unit_conversion_observed"] is False
            and evidence["static_config_handoff"]["pre_api_rounding_or_quantization_observed"] is False,
            "pre-API transform boundary mismatch")
    require(evidence["open_fields"]["final_bsi_dialnorm_field"] == "NOT_PROVEN"
            and "NOT_PROVEN" in evidence["open_fields"]["per_frame_empireenc_plus_0x3c_write_source_join"],
            "final BSI or per-frame field join was promoted")

    candidate = read_json("BOUNDARY-CANDIDATE.json")
    require(candidate["status"] == "MODEL_CANDIDATE_ONLY"
            and candidate["native_run"] == "NOT_RUN"
            and candidate["expected_serialized_bsi_values"] == "NOT_SUPPLIED",
            "candidate vector was promoted")
    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS",
            "incoming package binding mismatch")
    prior_hashes = {row["sha256"] for row in sources["prior_distinct_input_hashes"]}
    require(len(prior_hashes) == 3
            and sources["new_measurement_input_hash_from_prior_authorized_identity_record"]
            == evidence["native_observation"]["authorized_source_sha256"]
            and evidence["native_observation"]["authorized_source_sha256"] not in prior_hashes,
            "First Love input hash is not a distinct prior-authorized identity")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST,
            "Wash verification provenance mismatch")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "native_scalar": NATIVE_SCALAR,
                      "unit": "NOT_STATED_IN_NATIVE_LOG",
                      "final_bsi_dialnorm": "NOT_PROVEN"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
