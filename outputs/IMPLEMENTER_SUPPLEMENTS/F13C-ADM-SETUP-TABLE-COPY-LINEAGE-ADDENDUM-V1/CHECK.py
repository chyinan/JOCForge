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
PACKAGE_ID = "F13C-ADM-SETUP-TABLE-COPY-LINEAGE-ADDENDUM-V1"
INCOMING_MANIFEST = "16d65151a76514fe53a3c8ac8ebb2382e77c943d5226a60fea94aa1e23416821"


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
        "status": "SELECTOR3_OPENER_TO_SETUP_COPY_CHAIN_STATICALLY_BOUND; ACTUAL_DDP_PACKET_OWNER_GENERATION_UNBOUND",
        "implementation_ready": False,
        "source_reverse_manifest_sha256": INCOMING_MANIFEST,
        "files": files,
    }
    (PACKAGE / "MANIFEST.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def copy_record(record_hex):
    data = bytes.fromhex(record_hex)
    require(len(data) == 6, "synthetic record must be six bytes")
    return data.hex().upper()


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
    require(evidence["status"].startswith("SELECTOR3_OPENER_TO_SETUP_COPY_CHAIN_STATICALLY_BOUND;"),
            "copy chain status mismatch")
    require(evidence["selector3_copy_chain"]["source_and_clone_pointer_identity"] is False
            and evidence["selector3_copy_chain"]["static_data_lineage"] == "PROVEN_FOR_SELECTOR_3",
            "pointer/copy distinction mismatch")
    require(evidence["ddp_boundary"]["setup_table_to_ddp_packet_pointer_generation"] == "UNBOUND"
            and evidence["ddp_boundary"]["actual_ddp_row10_semantic_name"] == "UNBOUND"
            and evidence["ddp_boundary"]["normalizer_row_plus_2_classifier_byte"] == "NOT_CAPTURED",
            "DDP identity boundary mismatch")
    require(evidence["access_boundary"]["runtime_capture"] == "NOT_RUN"
            and evidence["access_boundary"]["dee_job_started"] is False
            and evidence["access_boundary"]["pcm_media_payload_key_license_read"] is False,
            "runtime/data access boundary changed")

    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["evidence_class"] == "SYNTHETIC_COPY_BEHAVIOR_ONLY_NOT_NATIVE_DDP_ROW_MAPPING"
            and len(vectors["cases"]) == 3,
            "vector scope/count mismatch")
    clone_case, setup_case, join_case = vectors["cases"]
    record = clone_case["input"]["source_record_hex"]
    count = clone_case["input"]["record_count"]
    stride = clone_case["input"]["record_stride_bytes"]
    require(type(count) is int and count >= 0 and stride == 6, "clone vector dimensions invalid")
    require(clone_case["expected_output"]["allocated_bytes"] == count * stride
            and clone_case["expected_output"]["cloned_record_hex"] == copy_record(record)
            and clone_case["expected_output"]["source_and_clone_pointer_alias"] is False,
            "header clone vector mismatch")
    setup_input = setup_case["input"]["setup_input_record_hex"]
    require(setup_case["expected_output"]["initial_setup_record_hex"] == copy_record(setup_input)
            and setup_case["expected_output"]["actual_ddp_packet_row_identity"] == "NOT_SUPPLIED",
            "setup-copy vector mismatch")
    require(join_case["expected_output"] == "NOT_SUPPLIED" and join_case["status"] == "UNBOUND",
            "DDP join was incorrectly supplied")

    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS",
            "incoming package binding mismatch")
    previous = sources["previous_wash_candidate_package"]
    require(previous["manifest_sha256"] == "57d7e6579e8bb9c5bfd0a05d693e602154ee332c0057fc09a536072e45c4f6fd",
            "previous Wash addendum binding mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST,
            "Wash verification provenance mismatch")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "copy_vectors_recomputed": len(vectors["cases"]),
                      "actual_ddp_row_identity": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
