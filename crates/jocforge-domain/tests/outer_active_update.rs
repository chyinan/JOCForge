// pattern: Functional Core
use jocforge_domain::spatial::outer_active_update::{OuterActiveUpdateError, active_energy_update};
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

fn vectors() -> Value {
    serde_json::from_slice(
        &std::fs::read(crate_root().join("tests/fixtures/f04_component/unit-vectors.json"))
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn source_assets_and_selected_active_cases_are_hash_bound() {
    let manifest_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    let vector_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f04_component/unit-vectors.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256(&vector_bytes), SOURCE_VECTORS_SHA256);
    for (path, bytes, hash) in [
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
        assert_eq!(entry["sha256"], hash);
    }
    let source_vectors = vectors();
    let cases = source_vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["entry_id"] == "outer-active-update")
        .map(|case| case["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(cases, ["U-OUT-ACTIVE-UPDATE"]);
}

#[test]
fn active_update_matches_the_selected_bitwise_vector() {
    let vectors = vectors();
    let case = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "U-OUT-ACTIVE-UPDATE")
        .unwrap();
    let input = &case["input"];
    let result = active_energy_update(
        &parse_matrix(&input["context_u32"]),
        &parse_words(&input["response_u32"]),
        &parse_words(&input["seed_u32"]),
        &parse_words(&input["correction_u32"]),
    )
    .unwrap();
    let expected = &case["expected"];
    assert_eq!(
        result.discriminant_u32(),
        u32::try_from(expected["discriminant_u32"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        result.step_u32(),
        u32::try_from(expected["step_u32"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        result.scaled_seed_u32(),
        parse_words(&expected["scaled_seed_u32"])
    );
    assert_eq!(result.output_u32(), parse_words(&expected["output_u32"]));
}

#[test]
fn negative_discriminant_and_input_frontiers_reject_without_a_result() {
    let vectors = vectors();
    let case = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "U-OUT-REJECT-NEGATIVE-D")
        .unwrap();
    assert_eq!(case["input"]["operation"], "outer-active-update");
    let input = &case["input"]["arguments"];
    assert_eq!(
        active_energy_update(
            &parse_matrix(&input["context_u32"]),
            &parse_words(&input["response_u32"]),
            &parse_words(&input["seed_u32"]),
            &parse_words(&input["correction_u32"]),
        ),
        Err(OuterActiveUpdateError::NegativeDiscriminant)
    );

    assert_eq!(
        active_energy_update(&[], &[0; 15], &[0], &[0]),
        Err(OuterActiveUpdateError::InvalidInput)
    );
    let mut nonfinite = vec![vec![0]; 15];
    nonfinite[0][0] = 0x7fc0_0000;
    assert_eq!(
        active_energy_update(&nonfinite, &[0; 15], &[0], &[0]),
        Err(OuterActiveUpdateError::InvalidInput)
    );
}

fn parse_matrix(value: &Value) -> Vec<Vec<u32>> {
    value.as_array().unwrap().iter().map(parse_words).collect()
}

fn parse_words(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect()
}
