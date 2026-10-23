// pattern: Functional Core
use jocforge_domain::spatial::outer_empty_matrix::{EmptyOuterMatrixError, empty_target_matrix};
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
fn source_assets_and_empty_matrix_case_are_bound() {
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
    assert!(
        unit_vectors()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "U-OUT-EMPTY")
    );
}

#[test]
fn empty_target_matrix_skips_all_solver_calls_without_reading_other_inputs() {
    let vectors = unit_vectors();
    let case = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "U-OUT-EMPTY")
        .unwrap();
    let source_order = case["input"]["source_call_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();

    let result = empty_target_matrix(0, &source_order).unwrap();
    assert!(result.matrix_u32().is_empty());
    assert_eq!(result.shape(), [0, 48]);
    assert_eq!(result.solver_calls(), 0);
    assert!(result.empty_targets_skipped());
    let expected_shape = case["expected"]["shape"].as_array().unwrap();
    assert_eq!(
        result.shape(),
        [
            u8::try_from(expected_shape[0].as_u64().unwrap()).unwrap(),
            u8::try_from(expected_shape[1].as_u64().unwrap()).unwrap()
        ]
    );
    assert_eq!(
        result.solver_calls(),
        u8::try_from(case["expected"]["solver_calls"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        result.empty_targets_skipped(),
        case["expected"]["empty_targets_skipped"]
    );
}

#[test]
fn nonempty_target_or_invalid_source_order_rejects_without_solver_calls() {
    assert_eq!(
        empty_target_matrix(1, &[0]),
        Err(EmptyOuterMatrixError::UnsupportedNonemptyTargetSet)
    );
    assert_eq!(
        empty_target_matrix(0, &[0, 0]),
        Err(EmptyOuterMatrixError::InvalidSourceCallOrder)
    );
    assert_eq!(
        empty_target_matrix(0, &[48]),
        Err(EmptyOuterMatrixError::InvalidSourceCallOrder)
    );
}
