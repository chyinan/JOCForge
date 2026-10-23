// pattern: Functional Core
use jocforge_domain::meter::k_matrix::{
    KMatrixError, KMatrixMode, KMatrixProfile, generate_k_matrix,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_SPEC_SHA256: &str = "d6287e17dec2a4b93b5756eb25345fd41cc4329eebeacefabd8e71ad84d33e31";
const SOURCE_MATRIX_PARAMETERS_SHA256: &str =
    "c8b4ffd52fffd62f5d0fdeba36a993f02188933a43593abe6969243f44f7f536";
const SOURCE_STATE_BOUNDS_SHA256: &str =
    "2faaa9d0aeefb105a913589590fcbafb97362a54192b0b21c5fdda7da1974e68";
const SOURCE_GENERATION_VERIFICATION_SHA256: &str =
    "88bfb0d0b9041d41ddf5ae30a0de18ac9b7be22ff06f45713b6601489eeda8b5";
const SOURCE_LOCAL_REVIEW_SHA256: &str =
    "1816539047fcac1c62f604e3975ffc3fcae49617dd763a60f0842f0bc8153c44";
const MATRIX_SHA256: &str = "08dc825e016f703df13b023bf29486996cba7c1eac2ebc1c7741da54ef98fb1a";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixtures() -> PathBuf {
    crate_root().join("tests/fixtures/k_matrix")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_json(path: PathBuf) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn selected_profile() -> KMatrixProfile {
    KMatrixProfile {
        sample_rate_hz: 48_000,
        mode: KMatrixMode::Modern,
        stage_count: 2,
    }
}

#[test]
fn source_assets_and_input_only_generation_parameters_are_hash_bound() {
    let manifest_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);

    for (path, expected_hash, local_name) in [
        (
            "specs/06-meter-k-matrix-generation.md",
            SOURCE_SPEC_SHA256,
            "source-spec.md",
        ),
        (
            "parameters/meter-k-matrix-math-v1.json",
            SOURCE_MATRIX_PARAMETERS_SHA256,
            "source-matrix-parameters.json",
        ),
        (
            "parameters/meter-k-state-bounds-v1.json",
            SOURCE_STATE_BOUNDS_SHA256,
            "source-state-bounds.json",
        ),
        (
            "test-vectors/METER-K-MATRIX-GENERATION-VERIFICATION.json",
            SOURCE_GENERATION_VERIFICATION_SHA256,
            "generation-verification.json",
        ),
        (
            "test-vectors/METER-K-MATRIX-LOCAL-REVIEW-STATUS.json",
            SOURCE_LOCAL_REVIEW_SHA256,
            "local-review-status.json",
        ),
    ] {
        let entry = manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["file"] == path)
            .unwrap();
        let bytes = std::fs::read(fixtures().join(local_name)).unwrap();
        assert_eq!(entry["sha256"], expected_hash);
        assert_eq!(sha256(&bytes), expected_hash);
        assert_eq!(
            entry["bytes"].as_u64().unwrap(),
            u64::try_from(bytes.len()).unwrap()
        );
    }

    let source_parameters = read_json(fixtures().join("source-matrix-parameters.json"));
    let input_parameters: Value =
        serde_json::from_str(include_str!("../src/meter/k_matrix_parameters.json")).unwrap();
    assert_eq!(input_parameters["schema"], source_parameters["schema"]);
    assert_eq!(
        input_parameters["parameter_route"],
        source_parameters["parameter_route"]
    );
    assert_eq!(input_parameters["stages"], source_parameters["stages"]);
    assert_eq!(
        input_parameters["matrix_generation"],
        source_parameters["matrix_generation"]
    );
    assert_eq!(
        input_parameters["shelf_gain_rounding_proof"],
        source_parameters["shelf_gain_rounding_proof"]
    );
    assert!(input_parameters.get("generated_f32_words").is_none());
}

#[test]
fn selected_profile_generates_all_64_matrix_words_and_eight_cold_words() {
    let provider = generate_k_matrix(selected_profile()).unwrap();
    let parameters = read_json(fixtures().join("source-matrix-parameters.json"));
    let expected = parameters["generated_f32_words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(expected.len(), 72);
    assert_eq!(provider.complete_words().as_slice(), expected);
    assert_eq!(provider.matrix_words().len(), 64);
    assert_eq!(provider.cold_state_words().len(), 8);

    let bytes = provider.complete_f32le_bytes();
    assert_eq!(bytes.len(), 288);
    assert_eq!(sha256(&bytes), MATRIX_SHA256);
}

#[test]
fn selected_state_bounds_are_exact_and_independent_of_matrix_generation() {
    let provider = generate_k_matrix(selected_profile()).unwrap();
    let bounds = read_json(fixtures().join("source-state-bounds.json"));
    assert_eq!(
        provider.cold_state_words().as_slice(),
        parse_words(&bounds["cold_state_words"])
    );
    assert_eq!(
        provider.lower_words().as_slice(),
        parse_hex_words(&bounds["lower_words"]).as_slice()
    );
    assert_eq!(
        provider.upper_words().as_slice(),
        parse_hex_words(&bounds["upper_words"]).as_slice()
    );
    assert_eq!(
        provider.noise_word(),
        word(bounds["noise_word"].as_str().unwrap())
    );
}

#[test]
fn other_sampling_rates_modes_and_stage_counts_are_unsupported() {
    let mut profile = selected_profile();
    profile.sample_rate_hz = 44_100;
    assert_eq!(
        generate_k_matrix(profile),
        Err(KMatrixError::UnsupportedProfile)
    );

    let mut profile = selected_profile();
    profile.mode = KMatrixMode::Other;
    assert_eq!(
        generate_k_matrix(profile),
        Err(KMatrixError::UnsupportedProfile)
    );

    let mut profile = selected_profile();
    profile.stage_count = 1;
    assert_eq!(
        generate_k_matrix(profile),
        Err(KMatrixError::UnsupportedProfile)
    );
}

fn parse_words(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| row.as_array().unwrap())
        .map(|value| word(value.as_str().unwrap()))
        .collect()
}

fn parse_hex_words(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| word(value.as_str().unwrap()))
        .collect()
}

fn word(hex: &str) -> u32 {
    u32::from_str_radix(hex, 16).unwrap()
}
