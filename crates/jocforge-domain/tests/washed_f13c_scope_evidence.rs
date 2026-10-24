// pattern: Imperative Shell
use std::collections::BTreeMap;
use std::fmt::Write;

use serde_json::Value;
use sha2::{Digest, Sha256};

const ID14_MANIFEST: &[u8] = include_bytes!("fixtures/id14_model_native_bsi/WASH-MANIFEST.json");
const ID14_CHECK: &[u8] = include_bytes!("fixtures/id14_model_native_bsi/INDEPENDENT-CHECK.json");
const ID14_ENVELOPE: &[u8] =
    include_bytes!("fixtures/id14_model_native_bsi/DELIVERY-ENVELOPE.json");
const ID14_FILES: &[(&str, &[u8])] = &[
    (
        "CONTRACT.md",
        include_bytes!("fixtures/id14_model_native_bsi/CONTRACT.md"),
    ),
    (
        "MODEL-PARSER-VECTORS.json",
        include_bytes!("fixtures/id14_model_native_bsi/MODEL-PARSER-VECTORS.json"),
    ),
    (
        "NATIVE-BSI-SOURCE.json",
        include_bytes!("fixtures/id14_model_native_bsi/NATIVE-BSI-SOURCE.json"),
    ),
    (
        "PROVENANCE.json",
        include_bytes!("fixtures/id14_model_native_bsi/PROVENANCE.json"),
    ),
    (
        "WASH-VALIDATION.json",
        include_bytes!("fixtures/id14_model_native_bsi/WASH-VALIDATION.json"),
    ),
];

const SCRATCH_MANIFEST: &[u8] =
    include_bytes!("fixtures/container_fragmentation_scratch/WASH-MANIFEST.json");
const SCRATCH_CHECK: &[u8] =
    include_bytes!("fixtures/container_fragmentation_scratch/INDEPENDENT-CHECK.json");
const SCRATCH_ENVELOPE: &[u8] =
    include_bytes!("fixtures/container_fragmentation_scratch/DELIVERY-ENVELOPE.json");
const SCRATCH_FILES: &[(&str, &[u8])] = &[
    (
        "CHECK-NORMAL.json",
        include_bytes!("fixtures/container_fragmentation_scratch/CHECK-NORMAL.json"),
    ),
    (
        "CHECK-OPTIMIZED.json",
        include_bytes!("fixtures/container_fragmentation_scratch/CHECK-OPTIMIZED.json"),
    ),
    (
        "CONTRACT.md",
        include_bytes!("fixtures/container_fragmentation_scratch/CONTRACT.md"),
    ),
    (
        "INPUT-ONLY.json",
        include_bytes!("fixtures/container_fragmentation_scratch/INPUT-ONLY.json"),
    ),
    (
        "LENGTH-SCOPE-CONFLICT.json",
        include_bytes!("fixtures/container_fragmentation_scratch/LENGTH-SCOPE-CONFLICT.json"),
    ),
    (
        "NATIVE-OBSERVATIONS.json",
        include_bytes!("fixtures/container_fragmentation_scratch/NATIVE-OBSERVATIONS.json"),
    ),
    (
        "PROVENANCE.json",
        include_bytes!("fixtures/container_fragmentation_scratch/PROVENANCE.json"),
    ),
    (
        "replay_container_parent_admission.py",
        include_bytes!(
            "fixtures/container_fragmentation_scratch/replay_container_parent_admission.py"
        ),
    ),
    (
        "RUNTIME-RECEIPT.json",
        include_bytes!("fixtures/container_fragmentation_scratch/RUNTIME-RECEIPT.json"),
    ),
    (
        "WASH-VALIDATION.json",
        include_bytes!("fixtures/container_fragmentation_scratch/WASH-VALIDATION.json"),
    ),
];

const COLD_ROOTS_MANIFEST: &[u8] =
    include_bytes!("fixtures/production_cold_roots/WASH-MANIFEST.json");
const COLD_ROOTS_CHECK: &[u8] =
    include_bytes!("fixtures/production_cold_roots/INDEPENDENT-CHECK.json");
const COLD_ROOTS_ENVELOPE: &[u8] =
    include_bytes!("fixtures/production_cold_roots/DELIVERY-ENVELOPE.json");
const COLD_ROOTS_FILES: &[(&str, &[u8])] = &[
    (
        "CHECK-NORMAL.json",
        include_bytes!("fixtures/production_cold_roots/CHECK-NORMAL.json"),
    ),
    (
        "CHECK-OPTIMIZED.json",
        include_bytes!("fixtures/production_cold_roots/CHECK-OPTIMIZED.json"),
    ),
    (
        "CONSTRUCTION-GRAPH.json",
        include_bytes!("fixtures/production_cold_roots/CONSTRUCTION-GRAPH.json"),
    ),
    (
        "CONTRACT.md",
        include_bytes!("fixtures/production_cold_roots/CONTRACT.md"),
    ),
    (
        "FAILURE-AND-PUBLICATION.json",
        include_bytes!("fixtures/production_cold_roots/FAILURE-AND-PUBLICATION.json"),
    ),
    (
        "INDEPENDENT-GRAPH-READINESS.json",
        include_bytes!("fixtures/production_cold_roots/INDEPENDENT-GRAPH-READINESS.json"),
    ),
    (
        "INPUT-ONLY.json",
        include_bytes!("fixtures/production_cold_roots/INPUT-ONLY.json"),
    ),
    (
        "NATIVE-OBSERVATIONS.json",
        include_bytes!("fixtures/production_cold_roots/NATIVE-OBSERVATIONS.json"),
    ),
    (
        "replay_production_cold_roots.py",
        include_bytes!("fixtures/production_cold_roots/replay_production_cold_roots.py"),
    ),
    (
        "RUNTIME-RECEIPT.json",
        include_bytes!("fixtures/production_cold_roots/RUNTIME-RECEIPT.json"),
    ),
    (
        "SOURCE-IDENTITIES.json",
        include_bytes!("fixtures/production_cold_roots/SOURCE-IDENTITIES.json"),
    ),
    (
        "WASH-VALIDATION.json",
        include_bytes!("fixtures/production_cold_roots/WASH-VALIDATION.json"),
    ),
];

const ID11_QUEUE_MANIFEST: &[u8] =
    include_bytes!("fixtures/id11_native_queue_capacity/WASH-MANIFEST.json");
const ID11_QUEUE_CHECK: &[u8] =
    include_bytes!("fixtures/id11_native_queue_capacity/INDEPENDENT-CHECK.json");
const ID11_QUEUE_ENVELOPE: &[u8] =
    include_bytes!("fixtures/id11_native_queue_capacity/DELIVERY-ENVELOPE.json");
const ID11_QUEUE_FILES: &[(&str, &[u8])] = &[
    (
        "APPROVED-CONTRACT-COMPARISON.json",
        include_bytes!("fixtures/id11_native_queue_capacity/APPROVED-CONTRACT-COMPARISON.json"),
    ),
    (
        "CHECK-NORMAL.json",
        include_bytes!("fixtures/id11_native_queue_capacity/CHECK-NORMAL.json"),
    ),
    (
        "CHECK-OPTIMIZED.json",
        include_bytes!("fixtures/id11_native_queue_capacity/CHECK-OPTIMIZED.json"),
    ),
    (
        "CONTRACT.md",
        include_bytes!("fixtures/id11_native_queue_capacity/CONTRACT.md"),
    ),
    (
        "FIELD-AND-CONSUMER.json",
        include_bytes!("fixtures/id11_native_queue_capacity/FIELD-AND-CONSUMER.json"),
    ),
    (
        "INPUT-ONLY.json",
        include_bytes!("fixtures/id11_native_queue_capacity/INPUT-ONLY.json"),
    ),
    (
        "NATIVE-OBSERVATIONS.json",
        include_bytes!("fixtures/id11_native_queue_capacity/NATIVE-OBSERVATIONS.json"),
    ),
    (
        "PROVENANCE.json",
        include_bytes!("fixtures/id11_native_queue_capacity/PROVENANCE.json"),
    ),
    (
        "replay_metadata_queue_capacity.py",
        include_bytes!("fixtures/id11_native_queue_capacity/replay_metadata_queue_capacity.py"),
    ),
    (
        "RUNTIME-RECEIPT.json",
        include_bytes!("fixtures/id11_native_queue_capacity/RUNTIME-RECEIPT.json"),
    ),
    (
        "WASH-VALIDATION.json",
        include_bytes!("fixtures/id11_native_queue_capacity/WASH-VALIDATION.json"),
    ),
];

const ID14_READER_MANIFEST: &[u8] =
    include_bytes!("fixtures/selected_id14_reader_path/WASH-MANIFEST.json");
const ID14_READER_CHECK: &[u8] =
    include_bytes!("fixtures/selected_id14_reader_path/INDEPENDENT-CHECK.json");
const ID14_READER_ENVELOPE: &[u8] =
    include_bytes!("fixtures/selected_id14_reader_path/DELIVERY-ENVELOPE.json");
const ID14_READER_FILES: &[(&str, &[u8])] = &[
    (
        "CANDIDATE-AUDIT-SUMMARY.json",
        include_bytes!("fixtures/selected_id14_reader_path/CANDIDATE-AUDIT-SUMMARY.json"),
    ),
    (
        "CONTRACT.md",
        include_bytes!("fixtures/selected_id14_reader_path/CONTRACT.md"),
    ),
    (
        "SELECTED-PATH-READER-STATUS.json",
        include_bytes!("fixtures/selected_id14_reader_path/SELECTED-PATH-READER-STATUS.json"),
    ),
    (
        "WASH-VALIDATION.json",
        include_bytes!("fixtures/selected_id14_reader_path/WASH-VALIDATION.json"),
    ),
];

const TYPE11_LENGTH_MANIFEST: &[u8] =
    include_bytes!("fixtures/type11_natural_length/WASH-MANIFEST.json");
const TYPE11_LENGTH_CHECK: &[u8] =
    include_bytes!("fixtures/type11_natural_length/INDEPENDENT-CHECK.json");
const TYPE11_LENGTH_ENVELOPE: &[u8] =
    include_bytes!("fixtures/type11_natural_length/DELIVERY-ENVELOPE.json");
const TYPE11_LENGTH_FILES: &[(&str, &[u8])] = &[
    (
        "CHECK-NORMAL.json",
        include_bytes!("fixtures/type11_natural_length/CHECK-NORMAL.json"),
    ),
    (
        "CHECK-OPTIMIZED.json",
        include_bytes!("fixtures/type11_natural_length/CHECK-OPTIMIZED.json"),
    ),
    (
        "CONTRACT.md",
        include_bytes!("fixtures/type11_natural_length/CONTRACT.md"),
    ),
    (
        "COVERAGE-AND-LIMITS.json",
        include_bytes!("fixtures/type11_natural_length/COVERAGE-AND-LIMITS.json"),
    ),
    (
        "EXECUTION-RECEIPT.json",
        include_bytes!("fixtures/type11_natural_length/EXECUTION-RECEIPT.json"),
    ),
    (
        "MODEL-OBSERVATIONS.json.gz",
        include_bytes!("fixtures/type11_natural_length/MODEL-OBSERVATIONS.json.gz"),
    ),
    (
        "NATIVE-OBSERVATIONS.json",
        include_bytes!("fixtures/type11_natural_length/NATIVE-OBSERVATIONS.json"),
    ),
    (
        "PROVENANCE.json",
        include_bytes!("fixtures/type11_natural_length/PROVENANCE.json"),
    ),
    (
        "SOURCE-IDENTITIES.json",
        include_bytes!("fixtures/type11_natural_length/SOURCE-IDENTITIES.json"),
    ),
    (
        "SUMMARY.json",
        include_bytes!("fixtures/type11_natural_length/SUMMARY.json"),
    ),
    (
        "WASH-VALIDATION.json",
        include_bytes!("fixtures/type11_natural_length/WASH-VALIDATION.json"),
    ),
];

#[test]
fn id14_helper_and_native_bsi_source_evidence_remain_separately_scoped() {
    verify_package(
        ID14_MANIFEST,
        ID14_CHECK,
        ID14_ENVELOPE,
        ID14_FILES,
        "d248175ea03a6cb6838b8c14995ea197a3529db2639afb51cb23185ab490bea8",
        "5453130aa79eac9e2dd405bfd22cd3661ac8f5e3838be64d1ee6f5e2dfa6c90e",
        "59f0dc40cfab277b2c8dd6d88f030d0189cc14d2970ae1e2839a2e13f01f3b99",
    );

    let model: Value = serde_json::from_slice(ID14_FILES[1].1).unwrap();
    let native: Value = serde_json::from_slice(ID14_FILES[2].1).unwrap();
    let wash: Value = serde_json::from_slice(ID14_FILES[4].1).unwrap();
    assert_eq!(model["classification"], "MODEL_EXECUTION_NOT_NATIVE");
    assert_eq!(model["vectors"].as_array().unwrap().len(), 13);
    assert_eq!(model["debug_release_observations_equal"], true);
    assert_eq!(native["observed_values"].as_array().unwrap().len(), 2);
    for observed in native["observed_values"].as_array().unwrap() {
        assert_eq!(observed["ID14_body_queue_serializer_present"], false);
    }
    assert!(
        native["selected_native_source"]
            .as_str()
            .unwrap()
            .contains("validated configuration count payload")
    );
    assert_eq!(
        wash["status"],
        "SCOPED_CLARIFICATION_READY_WHOLE_F13C_NOT_READY"
    );
    assert!(
        wash["native_limits"]
            .as_array()
            .unwrap()
            .iter()
            .any(|limit| { limit.as_str().unwrap() == "No global native parser absence claim." })
    );
}

#[test]
fn scratch_fragment_and_failure_observations_do_not_override_f13_1() {
    verify_package(
        SCRATCH_MANIFEST,
        SCRATCH_CHECK,
        SCRATCH_ENVELOPE,
        SCRATCH_FILES,
        "982ce51a60097d43a39abe6c126016ed94abf61df979ab73ff54e533dd3664f9",
        "62a1ac419441a6366d4e36df1c6dcd68debb8d9e9e07c8cabce74a2b2c13c523",
        "d32a26a7c49c87193955ee76811abb46936ae25f0b930c39bfe9d0146860409a",
    );

    let normal: Value = serde_json::from_slice(SCRATCH_FILES[0].1).unwrap();
    let optimized: Value = serde_json::from_slice(SCRATCH_FILES[1].1).unwrap();
    let wash: Value = serde_json::from_slice(SCRATCH_FILES[9].1).unwrap();
    let conflict: Value = serde_json::from_slice(SCRATCH_FILES[4].1).unwrap();
    assert_eq!(normal["parent_cases"], 16);
    assert_eq!(normal["raw_reservation_cases"], 2);
    assert_eq!(normal["mismatches"].as_array().unwrap().len(), 0);
    assert_eq!(optimized["mismatches"].as_array().unwrap().len(), 0);
    assert_eq!(wash["observed"]["512_byte_case_wire_fragments"][0], 511);
    assert_eq!(wash["observed"]["512_byte_case_wire_fragments"][1], 5);
    assert_eq!(wash["observed"]["empty_provider_child_status"], 9);
    assert_eq!(wash["observed"]["empty_provider_parent_status"], 7);
    assert_eq!(wash["observed"]["empty_provider_published_bytes"], 0);
    assert_eq!(
        wash["observed"]["some_scratch_state_changed_on_failure"],
        true
    );
    assert_eq!(
        conflict["classification"],
        "SCOPED_SCRATCH_OBSERVATION_NOT_YET_APPLICABLE_TO_APPROVED_SELECTED_PRODUCTION_PATH"
    );
    assert!(
        conflict["interim_implementation_rule"]
            .as_str()
            .unwrap()
            .contains("Do not remove the approved F13.1 limit")
    );
    assert!(wash["limits"].as_array().unwrap().iter().any(|limit| {
        limit
            .as_str()
            .unwrap()
            .contains("F13.1 selected production length rule remains unchanged")
    }));
}

#[test]
fn production_cold_roots_remain_scoped_with_r20_capacity_caveat_recorded() {
    verify_package(
        COLD_ROOTS_MANIFEST,
        COLD_ROOTS_CHECK,
        COLD_ROOTS_ENVELOPE,
        COLD_ROOTS_FILES,
        "986ea528cda6687c14cc87f790b7eec90b4e10567bef1e547a07ecc226de5928",
        "c386fcdc97656a45534950c74f824744ef1a743f0efed86a3ae349c72c2d146c",
        "6aa708785a359a5523719775246d5477ea67a6ee680ceda84c461ac73fac2a19",
    );

    let construction: Value = serde_json::from_slice(COLD_ROOTS_FILES[2].1).unwrap();
    let normal: Value = serde_json::from_slice(COLD_ROOTS_FILES[0].1).unwrap();
    let optimized: Value = serde_json::from_slice(COLD_ROOTS_FILES[1].1).unwrap();
    let failure: Value = serde_json::from_slice(COLD_ROOTS_FILES[4].1).unwrap();
    let readiness: Value = serde_json::from_slice(COLD_ROOTS_FILES[5].1).unwrap();
    let wash: Value = serde_json::from_slice(COLD_ROOTS_FILES[11].1).unwrap();

    assert_eq!(construction["production_constructor_observed"], true);
    let queues = construction["queue_roots"].as_array().unwrap();
    assert_eq!(queues.len(), 5);
    assert_eq!(
        queues.iter().find(|queue| queue["role"] == "ID11").unwrap()["capacity_bytes"],
        2_103
    );
    assert_eq!(
        queues
            .iter()
            .find(|queue| queue["role"] == "generic ID6")
            .unwrap()["capacity_bytes"],
        3
    );
    assert_eq!(
        queues.iter().find(|queue| queue["role"] == "ID2").unwrap()["capacity_bytes"],
        8
    );
    assert!(normal["mismatches"].as_array().unwrap().is_empty());
    assert!(optimized["mismatches"].as_array().unwrap().is_empty());
    assert_eq!(normal["queue_constructs_per_mode"], 5);
    assert_eq!(normal["frontend_calls"], 6);
    assert_eq!(normal["parent_boundary_replays"], 6);
    assert_eq!(
        failure["durable_file_commit"],
        "NOT_OBSERVED as a same-invocation transaction. Final reference-file hash is interference control, not a durable rollback receipt."
    );
    assert_eq!(
        failure["production_failure_injection"],
        "NOT_RUN; no live root, branch or business parameter changed."
    );
    assert_eq!(
        readiness["independent_production_cold_codec_parent_equivalence"],
        "NO"
    );
    assert_eq!(
        wash["open_capacity_semantics"],
        "ID11 constructor capacity field 2103 bytes has not been reconciled with approved per-payload capacity 160 bytes; implementation must not treat them as the same field."
    );
    assert_eq!(wash["whole_F13C_ready"], false);
}

#[test]
fn native_id11_queue_capacity_and_approved_api_policy_remain_distinct() {
    verify_package(
        ID11_QUEUE_MANIFEST,
        ID11_QUEUE_CHECK,
        ID11_QUEUE_ENVELOPE,
        ID11_QUEUE_FILES,
        "6697b548bd66c108180b8d1d084b287c275e8bdb1816192bf812d59bdb86d098",
        "5cc3f05ed3dc1c4f9ae631750dfcc36046030dd7eea0e8dc324bc824de13f4d4",
        "3b91582d1ad2b34d7516a37122a80733155599aee933b1f8acc9285b29da4bb2",
    );

    let field: Value = serde_json::from_slice(ID11_QUEUE_FILES[4].1).unwrap();
    let comparison: Value = serde_json::from_slice(ID11_QUEUE_FILES[0].1).unwrap();
    let normal: Value = serde_json::from_slice(ID11_QUEUE_FILES[1].1).unwrap();
    let optimized: Value = serde_json::from_slice(ID11_QUEUE_FILES[2].1).unwrap();
    let wash: Value = serde_json::from_slice(ID11_QUEUE_FILES[10].1).unwrap();
    let id11 = &field["ID11"];

    assert_eq!(id11["native_selected_parameter"], 2_103);
    assert_eq!(
        id11["unit"],
        "bytes per descriptor payload buffer and enqueue admission limit"
    );
    assert_eq!(id11["selected_inputs"]["object_capacity"], 16);
    assert_eq!(id11["selected_inputs"]["frame_capacity"], 8);
    assert_eq!(
        id11["separate_parser_state_allocation"]["selected_bytes"],
        4_763
    );
    assert!(
        id11["not_aggregate"]
            .as_str()
            .unwrap()
            .contains("not the total arena extent")
    );
    assert_eq!(
        id11["wrapper_failure_mapping"],
        "ID11 ingress recognizes queue status4 as occupied and maps it to2, but does not reject queue status5 before continuing parse/validation. This selected valid seed with2104 supplied bytes yields queue5, ingress0, empty queue and owner-valid1."
    );
    assert_eq!(
        comparison["approved_clause"],
        "F13.4 declares ID11 FIFO payload capacity160 bytes as interface capacity, not fixed payload length."
    );
    assert_eq!(
        comparison["authority"],
        "Proposal to Wash only; approved files unchanged."
    );
    assert_eq!(normal["cases"], 18);
    assert!(normal["mismatches"].as_array().unwrap().is_empty());
    assert!(optimized["mismatches"].as_array().unwrap().is_empty());
    assert_eq!(normal["approved160_policy_implemented_or_changed"], false);
    assert_eq!(
        wash["conclusions"]["approved_public_ID11_limit"],
        "160 bytes, separate narrower interface policy"
    );
    assert_eq!(
        wash["conclusions"]["natural_selected_producer_over160"],
        "NOT_PROVEN"
    );
    assert_eq!(wash["conclusions"]["ID6_semantic_name"], "NOT_ESTABLISHED");
    assert_eq!(
        wash["conclusions"]["ID2_distinction"],
        "SEPARATE_OWNER_CAPACITY_8"
    );
    assert_eq!(wash["conclusions"]["whole_F13C_ready"], false);
}

#[test]
fn id14_semantic_reader_is_not_required_on_the_recovered_selected_outgoing_path() {
    verify_package(
        ID14_READER_MANIFEST,
        ID14_READER_CHECK,
        ID14_READER_ENVELOPE,
        ID14_READER_FILES,
        "9a0b164c09d467e42b838bf6290ecfb7b660f21564b67315961241793113c41f",
        "de90ae59a29aba2ba2afbfb96d8a0da00ca6a395c00c894d4fae0f35bcfb179f",
        "ae802ba5bd895f76ac9dcfd1c2f5725d61bf1c261d783f2faed96ba274296280",
    );

    let audit: Value = serde_json::from_slice(ID14_READER_FILES[0].1).unwrap();
    let path: Value = serde_json::from_slice(ID14_READER_FILES[2].1).unwrap();
    let wash: Value = serde_json::from_slice(ID14_READER_FILES[3].1).unwrap();
    assert_eq!(audit["checked_candidate_count"], 70);
    assert_eq!(audit["semantic_reader_family_count"], 18);
    assert_eq!(
        audit["semantic_reader_family_reverse_root"],
        "ID11 semantic parser root"
    );
    assert_eq!(
        path["selected_outgoing_path"],
        "NOT_APPLICABLE_NO_SEMANTIC_READER_IN_RECOVERED_PATH"
    );
    assert_eq!(path["native_id14_reader_invocations"], 0);
    assert!(
        path["native_id14_malformed_vectors"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        path["ingress_dispatch"]["ID14"],
        "generic route/queue-copy branch; no semantic body dispatch observed"
    );
    assert_eq!(
        path["four_prior_lineage_cases"].as_array().unwrap().len(),
        4
    );
    assert!(
        path["four_prior_lineage_cases"]
            .as_array()
            .unwrap()
            .iter()
            .all(|case| {
                case["classification"] == "REHASHED_PRIOR_NATIVE_SERIALIZER_LINEAGE_NOT_PARSER_TEST"
            })
    );
    assert_eq!(path["global_DLL_parser_absence"], "NOT_CLAIMED");
    assert_eq!(
        wash["status"],
        "SELECTED_PATH_NATIVE_SEMANTIC_READER_NOT_APPLICABLE_WHOLE_F13C_NOT_READY"
    );
    assert_eq!(wash["limitations"].as_array().unwrap().len(), 5);
}

#[test]
fn sampled_natural_type11_lengths_support_but_do_not_universalize_the_160_byte_policy() {
    verify_package(
        TYPE11_LENGTH_MANIFEST,
        TYPE11_LENGTH_CHECK,
        TYPE11_LENGTH_ENVELOPE,
        TYPE11_LENGTH_FILES,
        "b9f31a89a26b567e9f004580b4c5a90cfadad8e38bea3d0def92fda4d0d09328",
        "81a7665442610af89365b03edea6f3ec8c42479e77ccfd59046fe91f5ce7d466",
        "835556bf7741665ef1be404c39f458b283230de5aea81fcc90dd7af0d7c6f205",
    );

    let summary: Value = serde_json::from_slice(TYPE11_LENGTH_FILES[9].1).unwrap();
    let normal: Value = serde_json::from_slice(TYPE11_LENGTH_FILES[0].1).unwrap();
    let optimized: Value = serde_json::from_slice(TYPE11_LENGTH_FILES[1].1).unwrap();
    let coverage: Value = serde_json::from_slice(TYPE11_LENGTH_FILES[3].1).unwrap();
    let wash: Value = serde_json::from_slice(TYPE11_LENGTH_FILES[10].1).unwrap();
    let model = &summary["model"];
    let native = &summary["native"];

    assert_eq!(model["frontend_callbacks"], 4_251);
    assert_eq!(model["natural_Type11_outputs"], 4_249);
    assert_eq!(model["unique_metadata_inputs"], 3_735);
    assert_eq!(model["minimum_payload_bytes"], 69);
    assert_eq!(model["maximum_payload_bytes"], 69);
    assert_eq!(model["archive_byte_mismatches"], 0);
    assert_eq!(model["upstream_producer_reexecuted"], false);
    assert_eq!(native["source_invocations"], 71);
    assert_eq!(native["natural_producer_outputs"], 69);
    assert_eq!(native["minimum_payload_bytes"], 69);
    assert_eq!(native["maximum_payload_bytes"], 69);
    assert_eq!(native["producer_output_buffer_capacity_bytes"][0], 375);
    assert_eq!(native["ID11_queue_capacity_bytes"][0], 2_103);
    assert_eq!(normal["model_outputs"], 4_249);
    assert_eq!(normal["native_natural_outputs"], 69);
    assert!(normal["mismatches"].as_array().unwrap().is_empty());
    assert!(optimized["mismatches"].as_array().unwrap().is_empty());
    assert_eq!(
        normal["standalone_checker_regenerates_payload_bytes"],
        false
    );
    assert!(
        coverage["not_proven"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| {
                item.as_str()
                    .unwrap()
                    .contains("All possible selected metadata/profile inputs")
            })
    );
    assert_eq!(
        wash["status"],
        "SELECTED_SAMPLE_SUPPORTS_160_API_POLICY_NOT_UNIVERSAL"
    );
    assert_eq!(
        wash["independent_checks"]["model_execution_receipt"],
        "4249 fresh writes byte-matched archived expected outputs before expected opened"
    );
    assert!(wash["limits"].as_array().unwrap().iter().any(|limit| {
        limit
            .as_str()
            .unwrap()
            .contains("Whole F13C and product acceptance remain NOT_READY")
    }));
}

fn verify_package(
    manifest_bytes: &[u8],
    check_bytes: &[u8],
    envelope_bytes: &[u8],
    files: &[(&str, &[u8])],
    manifest_sha256: &str,
    check_sha256: &str,
    envelope_sha256: &str,
) {
    assert_eq!(sha256(manifest_bytes), manifest_sha256);
    assert_eq!(sha256(check_bytes), check_sha256);
    assert_eq!(sha256(envelope_bytes), envelope_sha256);

    let manifest: Value = serde_json::from_slice(manifest_bytes).unwrap();
    let check: Value = serde_json::from_slice(check_bytes).unwrap();
    let envelope: Value = serde_json::from_slice(envelope_bytes).unwrap();
    assert_eq!(check["manifest_sha256"], manifest_sha256);
    assert_eq!(check["status"], "PASS");

    let entries = manifest["payloads"].as_array().unwrap();
    assert_eq!(entries.len(), files.len());
    let files_by_name: BTreeMap<&str, &[u8]> =
        files.iter().map(|(name, bytes)| (*name, *bytes)).collect();
    assert_eq!(files_by_name.len(), files.len());
    for entry in entries {
        let name = entry["file"].as_str().unwrap();
        let bytes = files_by_name[name];
        assert_eq!(
            entry["bytes"].as_u64().unwrap(),
            bytes.len() as u64,
            "{name}"
        );
        assert_eq!(entry["sha256"], sha256(bytes), "{name}");
    }

    let bindings: BTreeMap<&str, &Value> = envelope["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|binding| (binding["file"].as_str().unwrap(), binding))
        .collect();
    for (name, bytes, expected_hash) in [
        ("MANIFEST.json", manifest_bytes, manifest_sha256),
        ("INDEPENDENT-CHECK.json", check_bytes, check_sha256),
    ] {
        let binding = bindings[name];
        assert_eq!(binding["bytes"].as_u64().unwrap(), bytes.len() as u64);
        assert_eq!(binding["sha256"], expected_hash);
    }
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        write!(encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}
