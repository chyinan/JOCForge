// pattern: Functional Core
use jocforge_domain::spatial::candidate_selector::SelectorStatus;
use jocforge_domain::spatial::u04_projection::{
    U04AdmissionError, U04ProjectionError, project_targets, u04_refinement_admission,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_SPEC_SHA256: &str = "ca235e2f373d95d5c628ca61c651d01805bb9cb14a7c3457d9a77b1d87941879";
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

fn words(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect()
}

fn matrix(value: &Value) -> Vec<Vec<u32>> {
    value.as_array().unwrap().iter().map(words).collect()
}

fn points(value: &Value) -> Vec<[u32; 3]> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|point| {
            let point = words(point);
            [point[0], point[1], point[2]]
        })
        .collect()
}

#[test]
fn source_assets_and_six_u04_cases_are_hash_bound() {
    let manifest_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    let vector_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f04_component/unit-vectors.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256(&vector_bytes), SOURCE_VECTORS_SHA256);
    for (path, bytes, hash) in [
        (
            "specs/03-u04-refinement-and-projection.md",
            5_458,
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
    let ids = source_vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["functional_id"] == "F03U")
        .map(|case| case["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 6);
    assert_eq!(
        ids,
        [
            "U-PROJ-AXES",
            "U-PROJ-PROTECTED",
            "U-PROJ-ZERO",
            "U-PROJ-ADMIT-CAPACITY_REACHED-15",
            "U-PROJ-ADMIT-CANDIDATES_EXHAUSTED-3",
            "U-PROJ-ADMIT-CANDIDATES_EXHAUSTED-15"
        ]
    );
}

#[test]
fn u04_projection_matches_axis_normalization_and_protected_anchor_vectors() {
    let vectors = vectors();
    for id in ["U-PROJ-AXES", "U-PROJ-PROTECTED", "U-PROJ-ZERO"] {
        let case = vectors["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        let input = &case["input"];
        let result = project_targets(
            &points(&input["source_xyz_u32"]),
            &points(&input["target_xyz_u32"]),
            &words(&input["target_attributes_u32"]),
            &words(&input["source_weight_u32"]),
            &matrix(&input["gain_u32"]),
        )
        .unwrap_or_else(|error| panic!("{id}: {error:?}"));
        assert_eq!(
            result.normalized_m_u32(),
            &matrix(&case["expected"]["normalized_m_u32"]),
            "{id}"
        );
        assert_eq!(
            result.projected_xyz_u32(),
            &points(&case["expected"]["projected_xyz_u32"]),
            "{id}"
        );
    }
}

#[test]
fn refinement_admission_uses_status_not_selected_count_alone() {
    let vectors = vectors();
    for id in [
        "U-PROJ-ADMIT-CAPACITY_REACHED-15",
        "U-PROJ-ADMIT-CANDIDATES_EXHAUSTED-3",
        "U-PROJ-ADMIT-CANDIDATES_EXHAUSTED-15",
    ] {
        let case = vectors["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        let selector_status = match case["input"]["selector_status"].as_str().unwrap() {
            "CAPACITY_REACHED" => SelectorStatus::CapacityReached,
            "CANDIDATES_EXHAUSTED" => SelectorStatus::CandidatesExhausted,
            status => panic!("unmapped selector status {status}"),
        };
        let result = u04_refinement_admission(
            selector_status,
            u8::try_from(case["input"]["selected_count"].as_u64().unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            result.refinement_called(),
            case["expected"]["refinement_called"]
        );
        assert_eq!(result.selected_count(), case["expected"]["selected_count"]);
        assert_eq!(
            result.initial_positions_preserved(),
            case["expected"]["initial_positions_preserved"]
        );
    }
}

#[test]
fn malformed_projection_shapes_and_nonfinite_inputs_reject() {
    assert_eq!(
        project_targets(&[], &[[0; 3]], &[0], &[], &[]),
        Err(U04ProjectionError::InvalidInput)
    );
    assert_eq!(
        project_targets(&[[0; 3]], &[[0; 3]], &[], &[0], &[vec![0]]),
        Err(U04ProjectionError::InvalidInput)
    );
    assert_eq!(
        project_targets(&[[0x7fc0_0000, 0, 0]], &[[0; 3]], &[0], &[0], &[vec![0]]),
        Err(U04ProjectionError::InvalidInput)
    );
    assert_eq!(
        u04_refinement_admission(SelectorStatus::CapacityReached, 16),
        Err(U04AdmissionError::SelectedCountOutOfRange)
    );
}
