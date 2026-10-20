// Functional Core: validate only the amended callback-zero identity fixture.
// Imperative Shell: read the exact local supplement assets; no spatial DSP runs.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

const AMENDMENT_MANIFEST_SHA256: &str =
    "f8d06880fc988a37cc7a8796f223bcaa97958079c05093722a84ad9e7fcec9ad";
const EXPECTED_STAGES: &[&str] = &[
    "source",
    "controls",
    "render_installed48",
    "flags",
    "feature",
    "selector",
    "u04",
    "outer_assignment",
    "matrix_writer_set",
    "component",
    "lfe_ring",
    "reader56_target",
    "receiver",
];
const FAILURE_IDS: &[&str] = &[
    "after_selector",
    "after_u04",
    "after_outer_assignment",
    "after_reader56_target",
];

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/spatial_control_parent_seam")
}

fn digest_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}

fn read_fixture(name: &str) -> Value {
    let bytes = fs::read(fixture_root().join(name)).expect("fixture file exists");
    serde_json::from_slice(&bytes).expect("fixture JSON parses")
}

fn is_sha256(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        text.len() == 64
            && text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn assert_amendment_manifest() {
    let root = fixture_root();
    let manifest_bytes = fs::read(root.join("SPATIAL-CONTROL-SEAM-AMENDMENT-MANIFEST.json"))
        .expect("amendment manifest exists");
    assert_eq!(digest_hex(&manifest_bytes), AMENDMENT_MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(&manifest_bytes).expect("manifest JSON");
    assert_eq!(
        manifest["schema"],
        "washed-implementer-supplement-manifest.v1"
    );
    assert_eq!(
        manifest["identity"],
        "spatial-control-parent.callback0-amendment.v1"
    );
    assert_eq!(manifest["delivery_scope"], "internal test seam only");
    let files = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(files.len(), 4);
    for entry in files {
        let name = entry["file"].as_str().expect("manifest file name");
        let bytes = fs::read(root.join(name)).expect("manifested amendment file exists");
        assert_eq!(
            bytes.len() as u64,
            entry["bytes"].as_u64().unwrap(),
            "{name}"
        );
        assert_eq!(
            digest_hex(&bytes),
            entry["sha256"].as_str().unwrap(),
            "{name}"
        );
    }

    let boundaries = &manifest["hard_boundaries"];
    assert_eq!(
        boundaries["scope"],
        "callback0 generation1 NORMAL 13 stage summaries only"
    );
    assert_eq!(boundaries["seven_callbacks"], "UNVERIFIED_NOT_DELIVERED");
    assert_eq!(
        boundaries["ninety_one_stage_receipts"],
        "UNVERIFIED_NOT_DELIVERED"
    );
    assert_eq!(boundaries["cold_preimage"], "NOT_DELIVERED");
    assert_eq!(boundaries["source_preimage"], "NOT_DELIVERED");
    assert_eq!(boundaries["production_allowed"], false);
    assert_eq!(boundaries["commit_count"], 0);
    assert_eq!(boundaries["state_delta"], Value::Null);
    assert_eq!(boundaries["production_spatial_control_receipt"], "blocked");
    assert_eq!(boundaries["global_callback_transaction"], "blocked");
}

#[test]
fn callback_zero_stage_summaries_are_identity_checked_without_commit() {
    assert_amendment_manifest();
    let boundary = read_fixture("CALLBACK0-BOUNDARY.json");
    let independent = read_fixture("INDEPENDENT-CHECK.json");

    assert_eq!(
        boundary["schema"],
        "spatial-control-callback0-sanitized-boundary.v1"
    );
    let scope = &boundary["scope"];
    assert_eq!(scope["callback"], 0);
    assert_eq!(scope["generation"], 1);
    assert_eq!(scope["phase"], "NORMAL");
    assert_eq!(scope["actual_samples"], 1536);
    assert_eq!(scope["stage_receipt_count"], 13);
    assert_eq!(scope["stage_order"], json!(EXPECTED_STAGES));
    assert_eq!(scope["production_allowed"], false);
    assert_eq!(scope["commit_count"], 0);
    assert_eq!(scope["state_delta"], Value::Null);

    let stages = boundary["stage_receipts"].as_array().unwrap();
    assert_eq!(stages.len(), EXPECTED_STAGES.len());
    for (expected_stage, receipt) in EXPECTED_STAGES.iter().zip(stages) {
        assert_eq!(receipt["stage"], *expected_stage);
        assert_eq!(receipt["generation"], 1);
        assert_eq!(receipt["status"], "PASS");
        assert!(is_sha256(&receipt["input_digest"]));
        assert!(is_sha256(&receipt["output_digest"]));
        assert!(receipt["input_bytes"].as_u64().is_some());
        assert!(receipt["output_bytes"].as_u64().is_some());
    }

    let materials = &boundary["materials"];
    for (name, shape, words) in [
        ("installed48", json!([48, 3072]), 147_456),
        ("matrix", json!([15, 48]), 720),
        ("targets", json!([16, 6]), 96),
    ] {
        assert_eq!(materials[name]["shape"], shape);
        assert_eq!(materials[name]["words"], words);
        assert!(is_sha256(&materials[name]["sha256"]));
        assert_eq!(materials[name]["material"], "NOT_DELIVERED");
    }
    for (name, count) in [("markers", 15), ("writer_set", 4)] {
        assert_eq!(materials[name]["count"], count);
        assert!(is_sha256(&materials[name]["sha256"]));
        assert_eq!(materials[name]["material"], "NOT_DELIVERED");
    }

    let state_identity = &boundary["fixture_state_identity"];
    assert!(is_sha256(&state_identity["before_sha256"]));
    assert!(is_sha256(&state_identity["after_sha256"]));
    assert_eq!(state_identity["before_preimage"], "NOT_DELIVERED");
    assert_eq!(state_identity["after_preimage"], "NOT_DELIVERED");
    assert_eq!(
        state_identity["canonical_state_independently_replayable"],
        false
    );
    assert_eq!(boundary["ownership"]["f04_receiver_qmf"], "UNIQUE_OWNER");
    assert_eq!(boundary["ownership"]["parent_receiver_qmf_mutated"], false);

    let failures = boundary["failure_vectors"].as_array().unwrap();
    assert_eq!(failures.len(), FAILURE_IDS.len());
    for (expected_id, failure) in FAILURE_IDS.iter().zip(failures) {
        assert_eq!(failure["id"], *expected_id);
        assert_eq!(failure["production_parameter"], false);
        assert_eq!(failure["expected"]["receipt"], Value::Null);
        assert_eq!(failure["expected"]["proposal"], Value::Null);
        assert_eq!(failure["expected"]["parent_state_unchanged"], true);
        assert_eq!(failure["expected"]["f04_before_unchanged"], true);
        assert_eq!(failure["expected"]["commit_count"], 0);
        assert_eq!(failure["expected"]["programme_result"], "RETIRED");
    }

    let excluded = &boundary["excluded_claims"];
    assert_eq!(excluded["seven_callbacks"], "UNVERIFIED_NOT_DELIVERED");
    assert_eq!(
        excluded["ninety_one_stage_receipts"],
        "UNVERIFIED_NOT_DELIVERED"
    );
    assert_eq!(excluded["cold_preimage"], "NOT_DELIVERED");
    assert_eq!(excluded["source_preimage"], "NOT_DELIVERED");
    assert_eq!(excluded["numeric_replay"], "NOT_RUN");
    assert_eq!(excluded["real_adm"], "NOT_RUN");
    assert_eq!(excluded["production_receipt"], "BLOCKED");
    assert_eq!(excluded["global_transaction"], "BLOCKED");
    assert_eq!(
        boundary["verification_boundary"]["kind"],
        "INPUT_IDENTITY_ONLY_NOT_NUMERIC_REPLAY"
    );
    assert_eq!(
        boundary["verification_boundary"]["expected_used_as_seed"],
        false
    );
    assert_eq!(
        boundary["verification_boundary"]["raw_research_material_included"],
        false
    );

    let checks = independent["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 10);
    assert!(checks.iter().all(|check| check["pass"] == true));
    assert_eq!(independent["all_pass"], true);
    assert_eq!(
        independent["input_files"],
        json!(["CALLBACK0-BOUNDARY.json"])
    );
    assert_eq!(independent["raw_research_accessed"], false);
    assert_eq!(independent["independent_numeric_replay"], false);
    assert_eq!(independent["expected_used_as_seed"], false);
}
