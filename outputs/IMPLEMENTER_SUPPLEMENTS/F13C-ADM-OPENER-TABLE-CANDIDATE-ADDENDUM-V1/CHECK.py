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
PACKAGE_ID = "F13C-ADM-OPENER-TABLE-CANDIDATE-ADDENDUM-V1"
INCOMING_MANIFEST = "a49fbf51ec8f8cf572c5b1eabb1724ee1b348d2d0c1cdd7db48f26acef396d38"


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


def candidate_slot(track_index):
    require(type(track_index) is int and track_index >= 1, "TrackIndex must be a positive integer")
    return track_index - 1


def build_manifest():
    expected = PAYLOADS
    actual = {path.name for path in PACKAGE.iterdir() if path.is_file()} - {"MANIFEST.json"}
    require(actual == expected, f"payload inventory mismatch: {sorted(actual ^ expected)}")
    entries = [{"file": name, "bytes": (PACKAGE / name).stat().st_size, "sha256": sha256(PACKAGE / name)}
               for name in sorted(PAYLOADS)]
    manifest = {
        "schema": "jocforge-washed-implementer-supplement.v1",
        "package_id": PACKAGE_ID,
        "status": "ADM_TYPED_CANDIDATE_STATICALLY_BOUND; SELECTED_READER_TO_DDP_ROW_IDENTITY_UNBOUND",
        "implementation_ready": False,
        "source_reverse_manifest_sha256": INCOMING_MANIFEST,
        "files": entries,
    }
    (PACKAGE / "MANIFEST.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def verify():
    expected_files = PAYLOADS | {"MANIFEST.json"}
    actual_files = {path.name for path in PACKAGE.iterdir() if path.is_file()}
    require(actual_files == expected_files, f"file inventory mismatch: {sorted(actual_files ^ expected_files)}")
    manifest = read_json("MANIFEST.json")
    require(manifest["schema"] == "jocforge-washed-implementer-supplement.v1"
            and manifest["package_id"] == PACKAGE_ID
            and manifest["implementation_ready"] is False
            and manifest["source_reverse_manifest_sha256"] == INCOMING_MANIFEST,
            "manifest identity/readiness mismatch")
    rows = {row["file"]: row for row in manifest["files"]}
    require(len(rows) == len(manifest["files"]) and set(rows) == PAYLOADS, "manifest file list mismatch")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"],
                f"payload hash mismatch: {name}")

    evidence = read_json("EVIDENCE.json")
    require(evidence["status"] == "ADM_TYPED_CANDIDATE_STATICALLY_BOUND; SELECTED_READER_TO_DDP_ROW_IDENTITY_UNBOUND",
            "candidate/join status mismatch")
    require(evidence["selected_input"]["one_based_track_index"] == 11
            and evidence["selected_input"]["track_uid"] == "ATU_0000000B"
            and evidence["selected_input"]["audio_object_id"] == "AO_100B",
            "metadata identity mismatch")
    require(evidence["selected_input"]["metadata_read_scope"]["bytes_read"] == 96
            and evidence["selected_input"]["metadata_read_scope"]["audio_data_bytes_read"] == 0
            and evidence["selected_input"]["metadata_read_scope"]["axml_reread_in_this_addendum"] is False,
            "metadata access scope mismatch")
    require(evidence["native_candidate_path"]["opener_record"]["stride_bytes"] == 6
            and evidence["native_candidate_path"]["opener_record"]["plus_2_u8"] == "0xFF object sentinel"
            and "not assigned" in evidence["native_candidate_path"]["opener_record"]["plus_3"],
            "opener record boundary mismatch")
    require(evidence["unbound"]["actual_ddp_row_semantic_field_assignment"] is True
            and evidence["unbound"]["selected_audio_adm_reader_factory"] is True,
            "semantic identity incorrectly promoted")
    require(evidence["comparison_only_ddp_observation"]["normalizer_classifier_plus_2"] == "NOT_CAPTURED"
            and evidence["comparison_only_ddp_observation"]["join_disposition"]
            == "VALUES_AND_ROW_ORDINAL_ARE_NOT_IDENTITY_EVIDENCE",
            "DDP comparison boundary mismatch")
    require(evidence["access_boundary"]["new_dee_run"] == "NOT_RUN"
            and evidence["access_boundary"]["pcm_payload_bitstream_read"] is False
            and evidence["access_boundary"]["private_key_or_license_contents_read"] is False,
            "access boundary mismatch")

    vectors = read_json("INPUT-ONLY-VECTORS.json")
    require(vectors["evidence_class"] == "SYNTHETIC_STATIC_MODEL_ONLY" and len(vectors["cases"]) == 3,
            "candidate vector scope mismatch")
    for case in vectors["cases"]:
        inp, out = case["input"], case["expected_output"]
        if case["id"].startswith("chna-") or "candidate_slot_zero_based" in out:
            require(out["candidate_slot_zero_based"] == candidate_slot(inp["one_based_track_index"]),
                    f"candidate slot mismatch: {case['id']}")
        require(case["expected_ddp_packet_row_identity"] == "NOT_SUPPLIED", "DDP identity promoted")

    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS",
            "incoming source binding mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST,
            "wash verification source mismatch")

    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "candidate_vectors_recomputed": len(vectors["cases"]),
                      "actual_ddp_row_identity": "UNBOUND"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
