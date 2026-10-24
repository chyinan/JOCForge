// pattern: Functional Core
use jocforge_domain::codec::policy_parameters::{
    PolicyParameterError, PolicyProfile, load_policy_parameters,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/policy_authorized";
const ROOT_PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const PARAMETER_BYTES: usize = 4559;
const PARAMETER_SHA256: &str = "a99d077900058eb4a862f5ea29bc87408bf6d7a4f4cc42e3da66ae6c68a93c6a";
const INPUT_VECTORS_SHA256: &str =
    "afed7b0fdd7b80374a63cfb0b5922d6c988ea0c4a4417f09d2f28af5de462bc5";
const RESULT_SHA256: &str = "2cbca405e9af2cf6f95e8bc36546a387ed7f26ff73afd222715e9bf720136849";
const REVIEW_STATUS_SHA256: &str =
    "4ee71cd301c607b953f0dfc1a4f19894a1a97939925bbce1e1fa3fa4d67f151d";
const SERIALIZED_PARAMETER_SHA256: &str =
    "3dd2300224d0e459d1c4353fa2fc809f8c188775031f6a74cc64bbb1694fdc29";

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

#[test]
fn parameter_provider_is_bound_to_the_manifest_asset_and_reviewed_result() {
    let parameter_bytes = fixture_bytes("PARAMETERS.json");
    let input_bytes = fixture_bytes("POLICY-AUTHORIZED-INPUT-VECTORS.json");
    let result_bytes = fixture_bytes("POLICY-AUTHORIZED-RESULT.json");
    let review_bytes = fixture_bytes("POLICY-ACTUAL-LOCAL-REVIEW-STATUS.json");
    let inputs: Value = serde_json::from_slice(&input_bytes).unwrap();
    let result: Value = serde_json::from_slice(&result_bytes).unwrap();
    let review: Value = serde_json::from_slice(&review_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");

    assert_eq!(parameter_bytes.len(), PARAMETER_BYTES);
    assert_eq!(sha256(&parameter_bytes), PARAMETER_SHA256);
    assert_eq!(sha256(&input_bytes), INPUT_VECTORS_SHA256);
    assert_eq!(sha256(&result_bytes), RESULT_SHA256);
    assert_eq!(sha256(&review_bytes), REVIEW_STATUS_SHA256);
    assert_eq!(
        inputs["parameter_artifact"],
        "parameters/policy-authorized-v1.json"
    );
    assert_eq!(inputs["parameter_artifact_sha256"], PARAMETER_SHA256);
    assert_eq!(inputs["input_only"], true);
    assert_eq!(
        inputs["forbidden_seed_fields"],
        serde_json::json!(["expected", "actual", "result", "output"])
    );
    assert_eq!(result["parameter_artifact_sha256"], PARAMETER_SHA256);
    assert_eq!(result["input_vector_artifact_sha256"], INPUT_VECTORS_SHA256);
    assert_eq!(result["input_seed_field_count"], 0);
    assert_eq!(result["serialized_parameter_bytes"], 76);
    assert_eq!(
        result["serialized_parameter_f32_i32le_sha256"],
        SERIALIZED_PARAMETER_SHA256
    );
    assert_eq!(result["mismatch_count"], 0);
    assert_eq!(
        result["scope_limits"]["whole_pipeline_or_bit_exact_claim"],
        false
    );
    assert_eq!(review["decision"]["localized_policy_actual_ready"], "YES");
    assert_eq!(
        review["decision"]["whole_package_implementation_ready"],
        "NO"
    );
    assert_eq!(review["evidence"]["parameter_scalar_count"], 19);
    assert_eq!(review["evidence"]["parameter_byte_count"], 76);
    assert_eq!(
        provenance["source_manifest_sha256"],
        ROOT_PACKAGE_MANIFEST_SHA256
    );
    assert_eq!(provenance["parameter_artifact"]["sha256"], PARAMETER_SHA256);
    assert_eq!(provenance["input_vectors_sha256"], INPUT_VECTORS_SHA256);
    assert_eq!(provenance["result_sha256"], RESULT_SHA256);

    let parameters = load_policy_parameters(PolicyProfile::selected(), Some(&parameter_bytes))
        .expect("selected profile and exact provider are admitted");
    assert_eq!(parameters.d06_normal_threshold_bits(), 0x3ebd_a123);
    assert_eq!(parameters.d06_weights(), &[3, 6, 10, 10]);
    assert_eq!(parameters.d06_normalization_factor_bits(), 0x35cc_cccd);
    assert_eq!(
        parameters.d07_polynomial_bits(),
        &[
            0x3c62_2bca,
            0x3caa_5bd7,
            0x3be6_35de,
            0x3caa_aaab,
            0x4080_0000
        ]
    );
    assert_eq!(
        parameters.d07_anchors_bits(),
        &[0x3f24_d014, 0x3ef4_9518, 0x3e9f_9724, 0x3e15_182b]
    );
    assert_eq!(
        parameters.d07_thresholds_bits(),
        &[0x3f0f_9097, 0x3eca_161e, 0x3e6a_161e]
    );
    assert_eq!(parameters.d07_hysteresis_bits(), 0x3dff_2e49);
    let serialized = parameters.serialized_words_le();
    assert_eq!(serialized.len(), 76);
    assert_eq!(sha256(&serialized), SERIALIZED_PARAMETER_SHA256);
}

#[test]
fn profile_rejections_and_missing_or_corrupt_provider_map_to_washed_errors() {
    let parameter_bytes = fixture_bytes("PARAMETERS.json");
    let inputs = fixture_json("POLICY-AUTHORIZED-INPUT-VECTORS.json");
    let rejects = inputs["profile_rejections"].as_array().unwrap();
    assert_eq!(rejects.len(), 2);

    let selected = PolicyProfile::selected();
    let wrong_id = PolicyProfile {
        id: rejects[0]["profile_id"].as_str().unwrap(),
        ..selected
    };
    let wrong_id_error = load_policy_parameters(wrong_id, Some(&parameter_bytes)).unwrap_err();
    assert_eq!(wrong_id_error, PolicyParameterError::UnsupportedProfile);
    assert_eq!(
        wrong_id_error.code(),
        rejects[0]["expected_error"].as_str().unwrap()
    );
    assert_eq!(rejects[0]["state_advance"], false);

    let coupled = PolicyProfile {
        coupling: rejects[1]["override"]["coupling"].as_bool().unwrap(),
        ..selected
    };
    let coupling_error = load_policy_parameters(coupled, Some(&parameter_bytes)).unwrap_err();
    assert_eq!(coupling_error, PolicyParameterError::UnsupportedProfile);
    assert_eq!(
        coupling_error.code(),
        rejects[1]["expected_error"].as_str().unwrap()
    );
    assert_eq!(rejects[1]["state_advance"], false);

    let missing = load_policy_parameters(selected, None).unwrap_err();
    assert_eq!(missing, PolicyParameterError::ParameterUnavailable);
    assert_eq!(missing.code(), "PARAMETER_UNAVAILABLE");

    let mut corrupted = parameter_bytes;
    corrupted[0] ^= 1;
    let corrupt = load_policy_parameters(selected, Some(&corrupted)).unwrap_err();
    assert_eq!(corrupt, PolicyParameterError::InvalidParameterDocument);
    assert_eq!(corrupt.code(), "POLICY_PARAMETER_ERROR");
}
