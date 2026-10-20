// Functional Core: validate a frozen boundary fixture without running spatial DSP.
// Imperative Shell: read only the bundled washed contract and JSON evidence.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

const FIXTURE_MANIFEST_SHA256: &str =
    "051bca43be7f1cf3191ceecc374dd64f7f597e62ca2c159b3b357fbb61988bd3";
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
const SOURCE_IDENTITIES: &[(&str, u64, &str)] = &[
    (
        "provider",
        11_054,
        "32cdd7d80f62db4f2eb5e53f9bcd28b83bfc2b3b9fac23f8bef6143b9668ee0a",
    ),
    (
        "state",
        7_237,
        "5138318de304b83d26e47271c1661bcc903cebcd2cea457ae98e0048b161ed46",
    ),
    (
        "input",
        297_763,
        "1b405277987597f6d176d0cabd236bc437645c0e770fbfb9c7b9319950c26bdf",
    ),
    (
        "verification",
        573,
        "c4d255a5b916f08cfdfec6b50ea60a00eb440365369ec90423e0e94b1886a69e",
    ),
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

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn decode_lower_hex(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }
    bytes
        .chunks_exact(2)
        .map(|pair| Some((hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?))
        .collect()
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

fn assert_fixture_manifest() {
    let root = fixture_root();
    let bytes = fs::read(root.join("SPATIAL-CONTROL-PARENT-SEAM-MANIFEST.json"))
        .expect("spatial parent seam manifest exists");
    assert_eq!(digest_hex(&bytes), FIXTURE_MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(&bytes).expect("manifest JSON");
    assert_eq!(
        manifest["schema"],
        "washed-implementer-supplement-manifest.v1"
    );
    assert_eq!(
        manifest["identity"],
        "spatial-control-parent.test-only-seam.v1"
    );
    assert_eq!(manifest["delivery_scope"], "internal test seam only");
    assert_eq!(manifest["files"].as_array().unwrap().len(), 5);

    for entry in manifest["files"].as_array().unwrap() {
        let name = entry["file"].as_str().expect("manifest file name");
        let file = fs::read(root.join(name)).expect("manifested fixture exists");
        assert_eq!(
            file.len() as u64,
            entry["bytes"].as_u64().unwrap(),
            "{name}"
        );
        assert_eq!(
            digest_hex(&file),
            entry["sha256"].as_str().unwrap(),
            "{name}"
        );
    }

    assert_eq!(manifest["hard_boundaries"]["production_allowed"], false);
    assert_eq!(manifest["hard_boundaries"]["commit_count"], 0);
    assert_eq!(manifest["hard_boundaries"]["state_delta"], Value::Null);
    assert_eq!(manifest["hard_boundaries"]["receiver_qmf_mutated"], false);
    assert_eq!(manifest["hard_boundaries"]["f04c_parent"], "excluded");
    assert_eq!(
        manifest["hard_boundaries"]["global_callback_transaction"],
        "blocked"
    );
}

#[test]
fn synthetic_spatial_parent_boundary_is_bound_without_commit() {
    assert_fixture_manifest();
    let provider = read_fixture("spatial-control-production-provider.json");
    let vectors = read_fixture("SPATIAL-CONTROL-BOUNDARY-VECTORS.json");
    let schemas = read_fixture("SPATIAL-CONTROL-RECEIPTS.schema.json");
    let independent = read_fixture("INDEPENDENT-CHECK.json");

    assert_eq!(provider["schema"], "spatial-control-production-provider.v1");
    assert_eq!(
        provider["profile_id"],
        "selected-adm-installed48-spatial-control.v1"
    );
    assert_eq!(provider["production_allowed"], false);
    assert_eq!(provider["source_count"], 48);
    assert_eq!(provider["feature_bands"], 24);
    assert_eq!(provider["target_capacity"], 15);
    assert_eq!(provider["installed_samples"], 3072);
    assert_eq!(provider["callback_samples"], 1536);
    assert_eq!(provider["cached_drains"], 4);
    assert_eq!(provider["stage_order"], json!(EXPECTED_STAGES));
    assert_eq!(provider["shapes"]["installed48_f32"], json!([48, 3072]));
    assert_eq!(provider["shapes"]["matrix_f32"], json!([15, 48]));
    assert_eq!(provider["shapes"]["destinations"], json!([15]));
    assert_eq!(provider["shapes"]["markers_i32"], json!([15]));
    assert_eq!(provider["shapes"]["target_rows_f32"], json!([16, 6]));
    assert_eq!(provider["shapes"]["control_u32"], json!([38, 13]));

    assert_eq!(
        vectors["schema"],
        "spatial-control-parent-boundary-vectors.v1"
    );
    assert_eq!(vectors["production_allowed"], false);
    assert_eq!(vectors["real_adm_cold_execution"], "NOT_RUN");
    assert_eq!(vectors["expected_used_as_seed"], false);
    for (name, byte_count, sha256) in SOURCE_IDENTITIES {
        assert_eq!(vectors["source_identities"][name]["bytes"], *byte_count);
        assert_eq!(vectors["source_identities"][name]["sha256"], *sha256);
    }
    assert_eq!(
        provider["source_identities"]["provider_file_sha256"],
        vectors["source_identities"]["provider"]["sha256"]
    );
    assert_eq!(
        provider["source_identities"]["state_file_sha256"],
        vectors["source_identities"]["state"]["sha256"]
    );
    assert_eq!(
        provider["source_identities"]["input_vectors_sha256"],
        vectors["source_identities"]["input"]["sha256"]
    );
    assert_eq!(
        provider["source_identities"]["verification_sha256"],
        vectors["source_identities"]["verification"]["sha256"]
    );

    let receipt_schema = &schemas["$defs"]["spatial_control_receipt"];
    let required_receipt_fields = receipt_schema["required"].as_array().unwrap();
    for name in [
        "programme_sha256",
        "input_sha256",
        "config_sha256",
        "before_state_sha256",
        "installed48",
        "selector",
        "u04",
        "outer",
        "matrix",
        "destinations",
        "markers",
        "writer_set",
        "targets",
        "proposed_after_state_sha256",
        "commit_authority",
    ] {
        assert!(required_receipt_fields.iter().any(|field| field == name));
    }
    assert_eq!(
        receipt_schema["properties"]["schema"]["const"],
        "spatial-control-receipt.v1"
    );
    assert_eq!(
        receipt_schema["properties"]["commit_authority"]["const"],
        "ENCLOSING_CALLBACK_TRANSACTION_ONCE"
    );
    assert_eq!(receipt_schema["properties"]["destinations"]["minItems"], 15);
    assert_eq!(receipt_schema["properties"]["destinations"]["maxItems"], 15);
    assert_eq!(receipt_schema["properties"]["markers"]["minItems"], 15);
    assert_eq!(receipt_schema["properties"]["markers"]["maxItems"], 15);
    let proposal_schema = &schemas["$defs"]["spatial_audio_proposal"];
    assert_eq!(
        proposal_schema["properties"]["schema"]["const"],
        "spatial-audio-proposal.v1"
    );
    assert_eq!(
        proposal_schema["properties"]["receiver_qmf_mutated"]["const"],
        false
    );
    assert_eq!(
        schemas["production_schema_status"],
        "BLOCKED_UNTIL_REAL_ADM_COLD_STATE_AND_CALLBACK"
    );

    let callback = &vectors["callback_fixture"];
    assert_eq!(callback["generation"], 1);
    assert_eq!(callback["phase"], "NORMAL");
    assert_eq!(callback["actual_samples"], 1536);
    let stage_receipts = callback["stage_receipts"].as_array().unwrap();
    assert_eq!(stage_receipts.len(), EXPECTED_STAGES.len());
    for (expected_stage, receipt) in EXPECTED_STAGES.iter().zip(stage_receipts) {
        assert_eq!(receipt["stage"], *expected_stage);
        assert_eq!(receipt["generation"], callback["generation"]);
        assert_eq!(receipt["status"], "PASS");
        assert!(is_sha256(&receipt["input_digest"]));
        assert!(is_sha256(&receipt["output_digest"]));
        assert!(receipt["input_bytes"].as_u64().is_some());
        assert!(receipt["output_bytes"].as_u64().is_some());
    }
    assert_eq!(stage_receipts[2]["shape"], json!([48, 3072]));
    assert_eq!(stage_receipts[5]["selected_count"], 4);
    assert_eq!(stage_receipts[6]["executed"], true);

    let installed48_sha256 = callback["installed48_sha256"].as_str().unwrap();
    assert!(is_sha256(&callback["installed48_sha256"]));
    assert_eq!(callback["installed48_words"], 147_456);
    assert_eq!(stage_receipts[2]["output_digest"], installed48_sha256);
    let validation = &vectors["synthetic_validation"];
    assert_eq!(
        validation["schema"],
        "spatial-control-synthetic-validation-receipt.v1"
    );
    assert_eq!(validation["production_allowed"], false);
    assert_eq!(validation["callback"], 0);
    assert_eq!(validation["generation"], callback["generation"]);
    assert_eq!(validation["stage_order"], json!(EXPECTED_STAGES));
    assert_eq!(validation["installed48_sha256"], installed48_sha256);
    assert_eq!(
        validation["before_state_sha256"],
        vectors["cold_fixture"]["sha256"]
    );
    assert_eq!(
        validation["recorded_after_state_sha256"],
        callback["full_state_sha256"]
    );
    assert_eq!(validation["receiver_qmf_mutated"], false);
    assert_eq!(validation["state_delta_committed"], false);
    assert_eq!(validation["commit_count"], 0);

    let matrix_bytes = decode_lower_hex(callback["matrix_f32le_hex"].as_str().unwrap())
        .expect("matrix fixture uses lowercase hex");
    assert_eq!(matrix_bytes.len(), 15 * 48 * 4);
    assert_eq!(digest_hex(&matrix_bytes), validation["matrix_sha256"]);
    let target_bytes = decode_lower_hex(callback["targets_f32le_hex"].as_str().unwrap())
        .expect("target fixture uses lowercase hex");
    assert_eq!(target_bytes.len(), 16 * 6 * 4);
    assert_eq!(digest_hex(&target_bytes), validation["targets_sha256"]);
    let markers = callback["markers_i32"].as_array().unwrap();
    assert_eq!(markers.len(), 15);
    assert!(markers.iter().all(|marker| marker.as_i64().is_some()));

    let writer_entries = callback["writer_entries"].as_array().unwrap();
    assert_eq!(writer_entries.len(), 4);
    for (index, entry) in writer_entries.iter().enumerate() {
        assert_eq!(entry["generation"], callback["generation"]);
        assert_eq!(entry["global_source_id"], (index * 5) as u64);
        assert_eq!(entry["destination"], index as u64);
        assert_eq!(entry["owner_token"], format!("F03W:S{}:G1", index * 5));
    }

    let after_state = decode_lower_hex(callback["full_state_hex"].as_str().unwrap())
        .expect("diagnostic state fixture uses lowercase hex");
    assert_eq!(after_state.len(), 8469);
    assert_eq!(&after_state[..8], b"AI48ST01");
    assert_eq!(digest_hex(&after_state), callback["full_state_sha256"]);
    assert_eq!(
        vectors["cold_fixture"]["encoding"],
        "AI48ST01_TEST_ONLY_NOT_PRODUCTION"
    );
    assert_eq!(vectors["cold_fixture"]["bytes"], 8469);
    assert!(is_sha256(&vectors["cold_fixture"]["sha256"]));

    let failures = vectors["failure_vectors"].as_array().unwrap();
    assert_eq!(failures.len(), FAILURE_IDS.len());
    for (expected_id, failure) in FAILURE_IDS.iter().zip(failures) {
        assert_eq!(failure["id"], *expected_id);
        assert_eq!(failure["fault_plan"]["kind"], "TEST_HARNESS_ONLY");
        assert_eq!(failure["fault_plan"]["production_parameter"], false);
        let expected = &failure["expected"];
        assert_eq!(expected["status"], "ERROR");
        assert_eq!(
            expected["failure_stage"],
            expected_id.trim_start_matches("after_")
        );
        assert_eq!(expected["spatial_control_receipt"], Value::Null);
        assert_eq!(expected["spatial_audio_proposal"], Value::Null);
        assert_eq!(
            expected["parent_before_sha256"],
            vectors["cold_fixture"]["sha256"]
        );
        assert_eq!(
            expected["parent_after_sha256"],
            expected["parent_before_sha256"]
        );
        assert_eq!(expected["f04_before_unchanged"], true);
        assert_eq!(expected["commit_count"], 0);
        assert_eq!(expected["programme_result"], "RETIRED");
    }

    assert_eq!(vectors["clock"]["callback"], "g");
    assert_eq!(vectors["clock"]["generation_after"], "g+1");
    assert_eq!(vectors["clock"]["source"], "g-2 when g>=2 else null");
    assert_eq!(vectors["clock"]["dispatch"], "g-1 when g>=2 else null");
    assert_eq!(vectors["clock"]["cached_drains"], 4);
    assert_eq!(vectors["reset"]["fixture_reset_equals_cold"], true);
    assert_eq!(vectors["eof"]["extra_callbacks"], 0);
    assert_eq!(vectors["production_success_recipe"]["status"], "NOT_RUN");
    assert_eq!(
        vectors["production_success_recipe"]["expected_output_oracle_included"],
        false
    );

    let checks = independent["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 18);
    assert!(checks.iter().all(|check| check["pass"] == true));
    assert_eq!(independent["all_pass"], true);
    assert_eq!(independent["imports_spatial_model"], false);
    assert_eq!(independent["independent_real_adm_replay"], false);
    assert_eq!(independent["expected_used_as_seed"], false);
}
