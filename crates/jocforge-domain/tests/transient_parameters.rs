// pattern: Functional Core
use jocforge_domain::analysis::transient_parameters::{
    TransientParameterError, load_transient_parameters,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/transient_channel";
const SOURCE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const PARAMETERS_BYTES: usize = 7_548;
const PARAMETERS_SHA256: &str = "130cd1a446ea8413a1d0d4c5eea631e890b7f1cd675975efa92697e258402991";
const INPUTS_SHA256: &str = "ddf9a4d0344f7f03f12768c5bac62cc7622f8691bf9e7c27b02bf4eedc6bda98";
const RESULTS_SHA256: &str = "0c6f3350631c0f54770e75f1a16a8d77cb46042f7f87cbfcf6002e89eb38ff64";
const REVIEW_SHA256: &str = "db32255a7b66f7b96f61fdc3989a7f9c6153557efeddff301c886b0674bf1fe2";
const SERIALIZED_PARAMETERS_SHA256: &str =
    "05bd7906942e0d786a4e31bcd7ca4352b1bb41d63e30e14b723c1c6ab4f86eee";

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

fn words_from_group(document: &Value, name: &str) -> Vec<u32> {
    document["groups"][name]["f32_words_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
        .collect()
}

#[test]
fn actual_d05_provider_bytes_and_parameter_receipt_are_hash_bound() {
    let parameters_bytes = fixture_bytes("PARAMETERS.json");
    let inputs_bytes = fixture_bytes("INPUTS.json");
    let results_bytes = fixture_bytes("EXPECTED.json");
    let review_bytes = fixture_bytes("REVIEW.json");
    let provenance = fixture_json("PROVENANCE.json");

    assert_eq!(parameters_bytes.len(), PARAMETERS_BYTES);
    assert_eq!(sha256(&parameters_bytes), PARAMETERS_SHA256);
    assert_eq!(sha256(&inputs_bytes), INPUTS_SHA256);
    assert_eq!(sha256(&results_bytes), RESULTS_SHA256);
    assert_eq!(sha256(&review_bytes), REVIEW_SHA256);
    assert_eq!(
        provenance["source_package_manifest_sha256"],
        SOURCE_MANIFEST_SHA256
    );
    assert_eq!(provenance["source_parameters_sha256"], PARAMETERS_SHA256);
    assert_eq!(provenance["source_input_vectors_sha256"], INPUTS_SHA256);
    assert_eq!(provenance["source_expected_results_sha256"], RESULTS_SHA256);
    assert_eq!(provenance["source_actual_review_sha256"], REVIEW_SHA256);
    assert_eq!(provenance["whole_f07_ready"], false);
    assert_eq!(provenance["whole_audio_pipeline_ready"], false);
}

#[test]
fn transient_d05_groups_load_in_the_authorized_word_order() {
    let parameter_bytes = fixture_bytes("PARAMETERS.json");
    let source: Value = serde_json::from_slice(&parameter_bytes).unwrap();
    let parameters = load_transient_parameters(Some(&parameter_bytes))
        .expect("only the exact selected D05 artifact is admitted");

    let two: [u32; 10] = words_from_group(&source, "D05-P").try_into().unwrap();
    let three: [u32; 15] = words_from_group(&source, "D05-A").try_into().unwrap();
    let short: [u32; 3] = words_from_group(&source, "D05-S").try_into().unwrap();
    let other: [u32; 3] = words_from_group(&source, "D05-O").try_into().unwrap();
    let minimum = words_from_group(&source, "D05-M")[0];
    let clear = words_from_group(&source, "D05-C")[0];

    assert_eq!(
        parameters
            .two_section_words()
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>(),
        two
    );
    assert_eq!(
        parameters
            .three_section_words()
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>(),
        three
    );
    assert_eq!(parameters.short_threshold_words(), &short);
    assert_eq!(parameters.other_threshold_words(), &other);
    assert_eq!(parameters.minimum_power_word(), minimum);
    assert_eq!(parameters.history_clear_threshold_word(), clear);
    assert_eq!(parameters.serialized_words_le().len(), 132);
    assert_eq!(
        sha256(&parameters.serialized_words_le()),
        SERIALIZED_PARAMETERS_SHA256
    );
}

#[test]
fn missing_or_changed_transient_provider_fails_closed() {
    assert_eq!(
        load_transient_parameters(None),
        Err(TransientParameterError::ParameterUnavailable)
    );
    assert_eq!(
        TransientParameterError::ParameterUnavailable.code(),
        "PARAMETER_UNAVAILABLE"
    );

    let mut changed = fixture_bytes("PARAMETERS.json");
    changed[0] ^= 1;
    assert_eq!(
        load_transient_parameters(Some(&changed)),
        Err(TransientParameterError::InvalidDocument)
    );
    assert_eq!(
        TransientParameterError::InvalidDocument.code(),
        "TRANSIENT_PARAMETER_ERROR"
    );
}
