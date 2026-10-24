// pattern: Functional Core
use jocforge_domain::analysis::transient_channel::{
    TRANSIENT_BLOCK_SAMPLES, TransientChannelState, process_transient_channel_block,
};
use jocforge_domain::analysis::transient_parameters::load_transient_parameters;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/transient_channel";
const SOURCE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const F07_SPEC_SHA256: &str = "61a957410a05e1e44c15d7a3f6622d2ff4c2a1135020f35261c0dee1979f1eac";
const PARAMETER_SHA256: &str = "130cd1a446ea8413a1d0d4c5eea631e890b7f1cd675975efa92697e258402991";
const INPUTS_SHA256: &str = "ddf9a4d0344f7f03f12768c5bac62cc7622f8691bf9e7c27b02bf4eedc6bda98";
const RESULTS_SHA256: &str = "0c6f3350631c0f54770e75f1a16a8d77cb46042f7f87cbfcf6002e89eb38ff64";
const REVIEW_SHA256: &str = "db32255a7b66f7b96f61fdc3989a7f9c6153557efeddff301c886b0674bf1fe2";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(crate_root().join(FIXTURE_DIRECTORY).join(name)).unwrap()
}

fn fixture_json(name: &str) -> Value {
    serde_json::from_slice(&fixture_bytes(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn input_words(case: &Value) -> [u32; TRANSIENT_BLOCK_SAMPLES] {
    case["input_f32_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn filtered_sha256(words: &[u32; TRANSIENT_BLOCK_SAMPLES]) -> String {
    let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    sha256(&bytes)
}

#[derive(Debug)]
struct ActualBlockProjection {
    case_id: String,
    filtered_sha256: String,
    state_sha256: String,
    state_bytes: usize,
    mean_word: u32,
    short_source: u8,
    short_consumed: u8,
    other_source: u8,
    other_consumed: u8,
    short_tiers: [u8; 3],
    other_tiers: [u8; 3],
    positions: [u8; 4],
}

#[test]
fn actual_d05_single_channel_sequence_matches_all_36_postcomputed_blocks() {
    let parameters_bytes = fixture_bytes("PARAMETERS.json");
    let input_bytes = fixture_bytes("INPUTS.json");
    let provenance = fixture_json("PROVENANCE.json");
    let input: Value = serde_json::from_slice(&input_bytes).unwrap();

    assert_eq!(sha256(&parameters_bytes), PARAMETER_SHA256);
    assert_eq!(sha256(&input_bytes), INPUTS_SHA256);
    assert_eq!(
        provenance["source_package_manifest_sha256"],
        SOURCE_MANIFEST_SHA256
    );
    assert_eq!(provenance["source_f07_spec_sha256"], F07_SPEC_SHA256);
    assert_eq!(provenance["source_parameters_sha256"], PARAMETER_SHA256);
    assert_eq!(provenance["source_input_vectors_sha256"], INPUTS_SHA256);
    assert_eq!(provenance["source_expected_results_sha256"], RESULTS_SHA256);
    assert_eq!(provenance["source_actual_review_sha256"], REVIEW_SHA256);
    assert_eq!(
        provenance["input_scope"],
        "36 input-only blocks in one continuous single-channel sequence from cold state"
    );
    assert_eq!(provenance["whole_f07_ready"], false);

    let parameters = load_transient_parameters(Some(&parameters_bytes))
        .expect("the exact selected D05 provider is hash-pinned");
    let input_cases = input["cases"].as_array().unwrap();
    assert_eq!(input_cases.len(), 36);
    assert_eq!(
        input["prediction_rule"],
        "Inputs contain no expected output or next state. Never use the result artifact to seed state, flags, filtered samples, tiers or positions."
    );
    assert_eq!(
        input["sequence_semantics"],
        "All 36 blocks form one ordered sequence from cold state. Carry the independently calculated next state into the next block."
    );

    let mut state = TransientChannelState::cold();
    let mut actual = Vec::with_capacity(input_cases.len());
    for (sequence_step, input_case) in input_cases.iter().enumerate() {
        assert_eq!(input_case["sequence_id"], "ACTUAL-D05-CONTINUOUS-0");
        assert_eq!(
            input_case["sequence_step"].as_u64(),
            Some(sequence_step as u64)
        );
        let words = input_words(input_case);
        let output = process_transient_channel_block(state, &words, &parameters)
            .unwrap_or_else(|error| panic!("{}: {}", input_case["case_id"], error.code()));
        let next_state_bytes = output.next_state().canonical_bytes();
        actual.push(ActualBlockProjection {
            case_id: input_case["case_id"].as_str().unwrap().to_owned(),
            filtered_sha256: filtered_sha256(output.filtered_words()),
            state_sha256: sha256(&next_state_bytes),
            state_bytes: next_state_bytes.len(),
            mean_word: output.mean_word(),
            short_source: output.short_source(),
            short_consumed: output.short_consumed(),
            other_source: output.other_source(),
            other_consumed: output.other_consumed(),
            short_tiers: *output.short_tiers(),
            other_tiers: *output.other_tiers(),
            positions: *output.peak_positions(),
        });
        state = output.into_next_state();
    }

    // The expected file is opened only after the full input-only sequence was computed.
    let expected_bytes = fixture_bytes("EXPECTED.json");
    let review_bytes = fixture_bytes("REVIEW.json");
    let expected: Value = serde_json::from_slice(&expected_bytes).unwrap();
    let review: Value = serde_json::from_slice(&review_bytes).unwrap();
    assert_eq!(sha256(&expected_bytes), RESULTS_SHA256);
    assert_eq!(sha256(&review_bytes), REVIEW_SHA256);
    assert_eq!(expected["functional_id"], "F07");
    assert_eq!(expected["parameter_artifact_sha256"], PARAMETER_SHA256);
    assert_eq!(expected["input_vectors_sha256"], INPUTS_SHA256);
    assert_eq!(expected["mismatches"], 0);
    assert_eq!(review["verdict"], "PASS_FOR_DECLARED_LOCAL_SCOPE");
    assert_eq!(review["numeric_verdict"], "PASS");
    assert_eq!(review["evidence_policy"]["expected_used_as_seed"], false);
    assert_eq!(review["implementation_ready"], "NO");
    assert_eq!(review["whole_package_ready"], "NO");
    assert_eq!(review["numeric_evidence"]["continuous_blocks"], 36);
    assert_eq!(
        review["numeric_evidence"]["projected_field_comparisons"],
        11_088
    );
    let f07_binding = review["current_bindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|binding| binding["file"] == "specs/07-history-and-transient.md")
        .unwrap();
    assert_eq!(f07_binding["current_sha256"], F07_SPEC_SHA256);
    assert_eq!(f07_binding["same_bytes"], false);

    let expected_cases = expected["cases"].as_array().unwrap();
    assert_eq!(expected_cases.len(), actual.len());
    for (actual, expected_case) in actual.iter().zip(expected_cases) {
        assert_eq!(actual.case_id, expected_case["case_id"].as_str().unwrap());
        assert_eq!(
            actual.filtered_sha256,
            expected_case["filtered_f32le_sha256"]
        );
        assert_eq!(actual.state_sha256, expected_case["next_state_sha256"]);
        assert_eq!(
            actual.state_bytes,
            expected_case["next_state_bytes"].as_u64().unwrap() as usize
        );
        assert_eq!(
            actual.mean_word,
            u32::try_from(expected_case["mean_u32"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.short_source,
            u8::try_from(expected_case["short_source"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.short_consumed,
            u8::try_from(expected_case["short_consumed"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.other_source,
            u8::try_from(expected_case["other_source"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.other_consumed,
            u8::try_from(expected_case["other_consumed"].as_u64().unwrap()).unwrap()
        );
        let expected_short_tiers: [u8; 3] = expected_case["short_tiers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let expected_other_tiers: [u8; 3] = expected_case["other_tiers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let expected_positions: [u8; 4] = expected_case["positions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        assert_eq!(actual.short_tiers, expected_short_tiers);
        assert_eq!(actual.other_tiers, expected_other_tiers);
        assert_eq!(actual.positions, expected_positions);
    }
}

#[test]
fn transient_block_rejects_nonfinite_input_and_reports_numeric_overflow() {
    let parameter_bytes = fixture_bytes("PARAMETERS.json");
    let parameters = load_transient_parameters(Some(&parameter_bytes))
        .expect("the selected D05 provider is hash-pinned");

    let mut nonfinite = [0_u32; TRANSIENT_BLOCK_SAMPLES];
    nonfinite[0] = f32::NAN.to_bits();
    let nonfinite_error =
        process_transient_channel_block(TransientChannelState::cold(), &nonfinite, &parameters)
            .unwrap_err();
    assert_eq!(nonfinite_error.code(), "F07_INPUT_ERROR");

    let mut overflowing = [0_u32; TRANSIENT_BLOCK_SAMPLES];
    overflowing[0] = f32::MAX.to_bits();
    let numeric_error =
        process_transient_channel_block(TransientChannelState::cold(), &overflowing, &parameters)
            .unwrap_err();
    assert_eq!(numeric_error.code(), "F07_NUMERIC_FAILURE");
}
