// pattern: Functional Core
use jocforge_domain::spatial::outer_geometric_correction::{
    OuterGeometricCorrectionError, geometric_correction,
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
fn source_assets_and_two_geometry_vectors_are_hash_bound() {
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
        .filter(|case| case["entry_id"] == "outer-geometric-correction")
        .map(|case| case["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(cases, ["U-OUT-CORRECTION-ONE", "U-OUT-CORRECTION-TWO"]);
}

#[test]
fn one_and_two_target_geometric_corrections_match_exact_f32_words() {
    let vectors = unit_vectors();
    for id in ["U-OUT-CORRECTION-ONE", "U-OUT-CORRECTION-TWO"] {
        let case = vectors["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        let input = &case["input"];
        let target = parse_matrix(&input["target_xyz_u32"]);
        let source = parse_words(&input["source_xyz_u32"]);
        let index_state = parse_words(&input["index_state"]);
        let result = geometric_correction(
            &target,
            &source,
            u32::try_from(input["scalar_flag"].as_u64().unwrap()).unwrap(),
            &index_state,
        )
        .unwrap();
        assert_eq!(
            result.left_u32(),
            parse_matrix(&case["expected"]["left_u32"]),
            "{id}"
        );
        assert_eq!(
            result.system_u32(),
            parse_matrix(&case["expected"]["system_u32"]),
            "{id}"
        );
        assert_eq!(
            result.correction_u32(),
            parse_words(&case["expected"]["correction_u32"]),
            "{id}"
        );
    }
}

#[test]
fn malformed_geometry_shapes_and_nonfinite_words_reject() {
    assert_eq!(
        geometric_correction(&[], &[0; 3], 0, &[]),
        Err(OuterGeometricCorrectionError::InvalidInput)
    );
    assert_eq!(
        geometric_correction(&[vec![0; 2]], &[0; 3], 0, &[0]),
        Err(OuterGeometricCorrectionError::InvalidInput)
    );
    assert_eq!(
        geometric_correction(&[vec![0; 3]], &[0; 2], 0, &[0]),
        Err(OuterGeometricCorrectionError::InvalidInput)
    );
    assert_eq!(
        geometric_correction(&[vec![0; 3]], &[0; 3], 0, &[]),
        Err(OuterGeometricCorrectionError::InvalidInput)
    );
    assert_eq!(
        geometric_correction(&[vec![0x7fc0_0000, 0, 0]], &[0; 3], 0, &[0]),
        Err(OuterGeometricCorrectionError::InvalidInput)
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
