// pattern: Functional Core
use jocforge_domain::spatial::outer_energy_fallback::{
    OuterEnergyFallbackError, evaluate_outer_energy_fallback,
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
fn package_identity_and_selected_energy_cases_are_bound() {
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
        .filter(|case| case["entry_id"] == "outer-energy-fallback")
        .map(|case| case["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        cases,
        [
            "U-OUT-ENERGY-ZERO",
            "U-OUT-ENERGY-MATCH",
            "U-OUT-ENERGY-EQUAL"
        ]
    );
}

#[test]
fn energy_branch_diagnostics_and_fallback_results_match_vectors() {
    let vectors = unit_vectors();
    for id in [
        "U-OUT-ENERGY-ZERO",
        "U-OUT-ENERGY-MATCH",
        "U-OUT-ENERGY-EQUAL",
    ] {
        let case = vectors["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        let input = &case["input"];
        let matrix = input["context_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                row.as_array()
                    .unwrap()
                    .iter()
                    .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let response = input["response_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let seed = input["seed_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let actual = evaluate_outer_energy_fallback(&matrix, &response, &seed)
            .unwrap_or_else(|error| panic!("{id}: {error:?}"));
        let expected = &case["expected"];
        assert_eq!(
            actual.active(),
            expected["active"].as_bool().unwrap(),
            "{id}"
        );
        assert_eq!(
            actual.response_energy_u32(),
            u32::try_from(expected["response_energy_u32"].as_u64().unwrap()).unwrap(),
            "{id}"
        );
        assert_eq!(
            actual.context_energy_u32(),
            u32::try_from(expected["context_energy_u32"].as_u64().unwrap()).unwrap(),
            "{id}"
        );
        assert_eq!(
            actual.threshold_u32(),
            u32::try_from(expected["threshold_u32"].as_u64().unwrap()).unwrap(),
            "{id}"
        );
        assert_eq!(
            actual.scale_u32(),
            u32::try_from(expected["scale_u32"].as_u64().unwrap()).unwrap(),
            "{id}"
        );
        assert_eq!(
            actual.final_u32(),
            parse_words(&expected["final_u32"]),
            "{id}"
        );
    }
}

#[test]
fn malformed_shapes_nonfinite_and_numeric_frontier_reject() {
    assert_eq!(
        evaluate_outer_energy_fallback(&[], &[0; 15], &[0]),
        Err(OuterEnergyFallbackError::InvalidInput)
    );
    assert_eq!(
        evaluate_outer_energy_fallback(&vec![vec![0]; 15], &[0; 15], &[]),
        Err(OuterEnergyFallbackError::InvalidInput)
    );
    let mut nonfinite = vec![vec![0]; 15];
    nonfinite[0][0] = 0x7fc0_0000;
    assert_eq!(
        evaluate_outer_energy_fallback(&nonfinite, &[0; 15], &[0]),
        Err(OuterEnergyFallbackError::InvalidInput)
    );
    let overflow = vec![vec![0x7f7f_ffff]; 15];
    let seed = vec![0x7f7f_ffff];
    assert_eq!(
        evaluate_outer_energy_fallback(&overflow, &[0x7f7f_ffff; 15], &seed),
        Err(OuterEnergyFallbackError::NumericFailure)
    );
}

fn parse_words(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect()
}
