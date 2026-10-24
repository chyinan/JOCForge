// pattern: Functional Core
use jocforge_domain::codec::db_selector::{DB_MEAN_COUNT, DbSelectorError, db_selector_from_means};
use jocforge_domain::codec::policy_parameters::{PolicyProfile, load_policy_parameters};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/policy_authorized";
const ROOT_PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const PARAMETER_SHA256: &str = "a99d077900058eb4a862f5ea29bc87408bf6d7a4f4cc42e3da66ae6c68a93c6a";
const GAIN_DB_SPEC_SHA256: &str =
    "f6e73ca2e361c06413974f4b7725fdc94f8e1d13a6a299062edde9dc3a7e6c12";
const INPUT_VECTORS_SHA256: &str =
    "afed7b0fdd7b80374a63cfb0b5922d6c988ea0c4a4417f09d2f28af5de462bc5";
const RESULT_SHA256: &str = "2cbca405e9af2cf6f95e8bc36546a387ed7f26ff73afd222715e9bf720136849";
const REVIEW_STATUS_SHA256: &str =
    "4ee71cd301c607b953f0dfc1a4f19894a1a97939925bbce1e1fa3fa4d67f151d";

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

fn f32_word(value: &Value) -> u32 {
    let encoded = value.as_str().unwrap();
    u32::from_str_radix(encoded.strip_prefix("0x").unwrap(), 16).unwrap()
}

#[test]
fn selected_profile_mean_to_db_vectors_match_frozen_model_results() {
    let parameter_bytes = fixture_bytes("PARAMETERS.json");
    let input_bytes = fixture_bytes("POLICY-AUTHORIZED-INPUT-VECTORS.json");
    let result_bytes = fixture_bytes("POLICY-AUTHORIZED-RESULT.json");
    let review_bytes = fixture_bytes("POLICY-ACTUAL-LOCAL-REVIEW-STATUS.json");
    let inputs: Value = serde_json::from_slice(&input_bytes).unwrap();
    let result: Value = serde_json::from_slice(&result_bytes).unwrap();
    let review: Value = serde_json::from_slice(&review_bytes).unwrap();
    let provenance = fixture_json("DB-SELECTOR-PROVENANCE.json");

    assert_eq!(sha256(&parameter_bytes), PARAMETER_SHA256);
    assert_eq!(sha256(&input_bytes), INPUT_VECTORS_SHA256);
    assert_eq!(sha256(&result_bytes), RESULT_SHA256);
    assert_eq!(sha256(&review_bytes), REVIEW_STATUS_SHA256);
    assert_eq!(
        provenance["source_manifest_sha256"],
        ROOT_PACKAGE_MANIFEST_SHA256
    );
    assert_eq!(provenance["gain_db_spec_sha256"], GAIN_DB_SPEC_SHA256);
    assert_eq!(provenance["parameter_artifact_sha256"], PARAMETER_SHA256);
    assert_eq!(inputs["parameter_artifact_sha256"], PARAMETER_SHA256);
    assert_eq!(inputs["input_only"], true);
    assert_eq!(review["evidence"]["numeric_post_comparisons"], 17);
    assert_eq!(review["evidence"]["numeric_mismatches"], 0);

    let parameters = load_policy_parameters(PolicyProfile::selected(), Some(&parameter_bytes))
        .expect("the hash-pinned selected provider is available");
    let cases = inputs["cases"]["db_selector"].as_array().unwrap();
    let outputs = result["outputs"]["db_selector"].as_array().unwrap();
    assert_eq!(cases.len(), 4);
    assert_eq!(outputs.len(), cases.len());

    for (case, expected_entry) in cases.iter().zip(outputs) {
        let input_words = case["mean_f32_words_hex"].as_array().unwrap();
        assert_eq!(input_words.len(), DB_MEAN_COUNT);
        let mean_words: [u32; DB_MEAN_COUNT] = input_words
            .iter()
            .map(f32_word)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let selector_before = u16::try_from(case["selector_before"].as_u64().unwrap()).unwrap();
        let actual = db_selector_from_means(&mean_words, selector_before, &parameters).unwrap();
        let expected = &expected_entry["computed"];

        assert_eq!(case["case_id"], expected_entry["case_id"]);
        assert_eq!(expected_entry["frozen_model_equal"], true);
        assert_eq!(
            actual.selector_after(),
            u16::try_from(expected["selector_after"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.maximum_mean_f32_bits(),
            f32_word(&expected["maximum_mean_f32_word"])
        );
        assert_eq!(
            actual.level_f32_bits(),
            f32_word(&expected["level_f32_word"])
        );
        assert_eq!(
            actual.difference_f32_bits(),
            f32_word(&expected["difference_f32_word"])
        );
        assert_eq!(
            actual.strict_hysteresis_triggered(),
            expected["strict_hysteresis_triggered"].as_bool().unwrap()
        );
    }
}

#[test]
fn invalid_selector_and_mean_domains_fail_without_returning_a_db_code() {
    let parameter_bytes = fixture_bytes("PARAMETERS.json");
    let parameters = load_policy_parameters(PolicyProfile::selected(), Some(&parameter_bytes))
        .expect("the hash-pinned selected provider is available");
    let valid_means = [0x3e80_0000, 0, 0, 0, 0, 0];

    assert_eq!(
        db_selector_from_means(&valid_means, 4, &parameters),
        Err(DbSelectorError::ExponentInputError)
    );

    let mut negative_mean = valid_means;
    negative_mean[0] = 0xbf80_0000;
    assert_eq!(
        db_selector_from_means(&negative_mean, 0, &parameters),
        Err(DbSelectorError::ExponentInputError)
    );

    let mut infinite_mean = valid_means;
    infinite_mean[0] = f32::INFINITY.to_bits();
    assert_eq!(
        db_selector_from_means(&infinite_mean, 0, &parameters),
        Err(DbSelectorError::ExponentInputError)
    );
}

#[test]
fn nonfinite_polynomial_intermediate_is_a_numeric_failure() {
    let parameter_bytes = fixture_bytes("PARAMETERS.json");
    let parameters = load_policy_parameters(PolicyProfile::selected(), Some(&parameter_bytes))
        .expect("the hash-pinned selected provider is available");
    let means = [f32::MAX.to_bits(), 0, 0, 0, 0, 0];

    assert_eq!(
        db_selector_from_means(&means, 0, &parameters),
        Err(DbSelectorError::NumericFailure)
    );
}
