// pattern: Functional Core
use jocforge_domain::spatial::outer_postprocess::{
    OuterPostprocessError, postprocess_coefficients,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_SPEC_SHA256: &str = "7ca47864c9fe7573b6cc6edfafdbd2049c5e74aa73f9b240c4c15bf4a427cdb2";
const SOURCE_ADAPTER_SHA256: &str =
    "9bce2e35ce47147c76c13e58ee976d44c5109938f0b19e1f3a81f1d8a61fa1ae";
const SOURCE_VECTORS_SHA256: &str =
    "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn unit_vectors() -> Value {
    serde_json::from_slice(
        &std::fs::read(crate_root().join("tests/fixtures/f04_component/unit-vectors.json"))
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn package_identity_and_selected_postprocess_cases_are_bound() {
    let manifest_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    let vectors_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f04_component/unit-vectors.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256(&vectors_bytes), SOURCE_VECTORS_SHA256);
    for (path, bytes, expected_hash) in [
        (
            "specs/03-outer-energy-correction.md",
            5_150,
            SOURCE_SPEC_SHA256,
        ),
        (
            "test-vectors/unit/ADAPTER-CONTRACT.md",
            22_205,
            SOURCE_ADAPTER_SHA256,
        ),
        (
            "test-vectors/unit/vectors.json",
            1_503_142,
            SOURCE_VECTORS_SHA256,
        ),
    ] {
        let entry = manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["file"] == path)
            .unwrap();
        assert_eq!(entry["bytes"], bytes);
        assert_eq!(entry["sha256"], expected_hash);
    }

    let vectors = unit_vectors();
    let cases = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["entry_id"] == "outer-postprocess")
        .map(|case| case["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        cases,
        [
            "U-OUT-POST-FLOOR",
            "U-OUT-POST-DOMINANCE-EQUAL",
            "U-OUT-POST-TIE"
        ]
    );
}

#[test]
fn floor_dominance_equality_and_first_maximum_ties_match_source_vectors() {
    let vectors = unit_vectors();
    let cases = vectors["cases"].as_array().unwrap();
    for id in [
        "U-OUT-POST-FLOOR",
        "U-OUT-POST-DOMINANCE-EQUAL",
        "U-OUT-POST-TIE",
    ] {
        let case = cases.iter().find(|case| case["id"] == id).unwrap();
        let input = &case["input"];
        let coefficients = input["coefficient_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let result = postprocess_coefficients(
            &coefficients,
            u32::try_from(input["floor_u32"].as_u64().unwrap()).unwrap(),
            u32::try_from(input["dominance_ratio_u32"].as_u64().unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            result.after_floor_u32(),
            parse_u32_array(&case["expected"]["after_floor_u32"])
        );
        assert_eq!(
            result.final_u32(),
            parse_u32_array(&case["expected"]["final_u32"])
        );
    }
}

#[test]
fn empty_and_nonfinite_inputs_reject_before_a_result_is_returned() {
    assert_eq!(
        postprocess_coefficients(&[], 0, 0),
        Err(OuterPostprocessError::InvalidInput)
    );
    assert_eq!(
        postprocess_coefficients(&[0x7fc0_0000], 0, 0),
        Err(OuterPostprocessError::InvalidInput)
    );
    assert_eq!(
        postprocess_coefficients(&[0], 0x7f80_0000, 0),
        Err(OuterPostprocessError::InvalidInput)
    );
    assert_eq!(
        postprocess_coefficients(&[0], 0, 0x7fc0_0000),
        Err(OuterPostprocessError::InvalidInput)
    );
    assert_eq!(
        postprocess_coefficients(&[f32::MAX.to_bits(), f32::MAX.to_bits()], 0, 0x3f80_0000),
        Err(OuterPostprocessError::NumericFailure)
    );
}

fn parse_u32_array(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect()
}
