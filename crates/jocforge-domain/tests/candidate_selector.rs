// pattern: Functional Core
use jocforge_domain::spatial::candidate_selector::{
    CandidateSelectorInput, SelectorError, candidate_selector, pairwise_affinity,
    selector_score_choice,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_SPEC_SHA256: &str = "ce95853e007f9282f3d1dce0e43cffb132ed079b5f1cd2fe3e6ce10901d51697";
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

fn ids(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect()
}

fn matrix(value: &Value) -> Vec<Vec<u32>> {
    value.as_array().unwrap().iter().map(ids).collect()
}

fn selector_input(value: &Value) -> CandidateSelectorInput {
    CandidateSelectorInput {
        candidate_ids: ids(&value["candidate_ids"]),
        feature_u32: matrix(&value["feature_u32"]),
        positions_u32: value["positions_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                let words = ids(row);
                [words[0], words[1], words[2]]
            })
            .collect(),
        width_u32: ids(&value["width_u32"]),
        attributes_u32: ids(&value["attributes_u32"]),
        response_u32: matrix(&value["response_u32"]),
        previous_ids: ids(&value["previous_ids"]),
    }
}

#[test]
fn selected_f03c_assets_and_eight_entry_ids_are_bound() {
    let manifest_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    let vector_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f04_component/unit-vectors.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256(&vector_bytes), SOURCE_VECTORS_SHA256);
    for (path, bytes, hash) in [
        (
            "specs/03-selector-affinity-and-selection.md",
            6_405,
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
    let selected = source_vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["functional_id"] == "F03C")
        .map(|case| case["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 8);
    assert_eq!(
        selected,
        [
            "U-SEL-SINGLE",
            "U-SEL-COMPLEMENT",
            "U-SEL-PREVIOUS-ID",
            "U-SEL-CAPACITY",
            "U-SEL-CHOICE-BOUNDARY",
            "U-SEL-CHOICE-BELOW",
            "U-SEL-CHOICE-SECONDARY-TIE",
            "U-SEL-AFFINITY-WIDTH"
        ]
    );
}

#[test]
fn candidate_selector_matches_all_four_full_selection_vectors() {
    let vectors = vectors();
    let cases = vectors["cases"].as_array().unwrap();
    for id in [
        "U-SEL-SINGLE",
        "U-SEL-COMPLEMENT",
        "U-SEL-PREVIOUS-ID",
        "U-SEL-CAPACITY",
    ] {
        let case = cases.iter().find(|case| case["id"] == id).unwrap();
        let result = candidate_selector(selector_input(&case["input"]))
            .unwrap_or_else(|error| panic!("{id}: {error:?}"));
        assert_eq!(result.status().as_str(), case["expected"]["status"]);
        assert_eq!(
            result.selected_ids(),
            ids(&case["expected"]["selected_ids"])
        );
        assert_eq!(
            result.selected_initial_ordinals(),
            ids(&case["expected"]["selected_initial_ordinals"])
        );
        assert_eq!(
            result.selected_attributes(),
            ids(&case["expected"]["selected_attributes"])
        );
        assert_eq!(
            u32::from(result.selected_count()),
            u32::try_from(case["expected"]["selected_count"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            result.initial_affinity_u32(),
            matrix(&case["expected"]["initial_affinity_u32"])
        );
        assert_eq!(
            result.initial_complement_u32(),
            matrix(&case["expected"]["initial_complement_u32"])
        );
        assert_eq!(
            result.initial_support_u32(),
            ids(&case["expected"]["initial_support_u32"])
        );
        assert_eq!(
            result.iterations().len(),
            case["expected"]["iterations"].as_array().unwrap().len()
        );
        for (actual, expected) in result
            .iterations()
            .iter()
            .zip(case["expected"]["iterations"].as_array().unwrap())
        {
            assert_eq!(
                actual.candidate_ids_before(),
                &ids(&expected["candidate_ids_before"])
            );
            assert_eq!(
                actual.gate_slot().map(u32::from),
                expected["gate_slot"]
                    .as_u64()
                    .map(|value| u32::try_from(value).unwrap())
            );
            assert_eq!(
                u32::from(actual.chosen_current_ordinal()),
                u32::try_from(expected["chosen_current_ordinal"].as_u64().unwrap()).unwrap()
            );
            assert_eq!(actual.ties(), &ids(&expected["ties"]));
            assert_eq!(
                actual.primary_max_u32(),
                u32::try_from(expected["primary_max_u32"].as_u64().unwrap()).unwrap()
            );
            assert_eq!(
                actual.secondary_max_u32(),
                u32::try_from(expected["secondary_max_u32"].as_u64().unwrap()).unwrap()
            );
            assert_eq!(actual.mask_after(), &ids(&expected["mask_after"]));
            assert_eq!(actual.coverage_u32(), &ids(&expected["coverage_u32"]));
            assert_eq!(actual.support_u32(), &ids(&expected["support_u32"]));
            assert_eq!(actual.surviving_ids(), &ids(&expected["surviving_ids"]));
        }
    }
}

#[test]
fn score_choice_and_pairwise_affinity_match_their_direct_vectors() {
    let vectors = vectors();
    let cases = vectors["cases"].as_array().unwrap();
    for id in [
        "U-SEL-CHOICE-BOUNDARY",
        "U-SEL-CHOICE-BELOW",
        "U-SEL-CHOICE-SECONDARY-TIE",
    ] {
        let case = cases.iter().find(|case| case["id"] == id).unwrap();
        let result = selector_score_choice(
            &ids(&case["input"]["primary_u32"]),
            &ids(&case["input"]["secondary_u32"]),
        )
        .unwrap();
        assert_eq!(
            result.chosen(),
            u32::try_from(case["expected"]["chosen"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(result.ties(), &ids(&case["expected"]["ties"]));
        assert_eq!(
            result.primary_max_u32(),
            u32::try_from(case["expected"]["primary_max_u32"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            result.secondary_max_u32(),
            u32::try_from(case["expected"]["secondary_max_u32"].as_u64().unwrap()).unwrap()
        );
    }

    let case = cases
        .iter()
        .find(|case| case["id"] == "U-SEL-AFFINITY-WIDTH")
        .unwrap();
    let positions = case["input"]["positions_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let words = ids(row);
            [words[0], words[1], words[2]]
        })
        .collect::<Vec<_>>();
    assert_eq!(
        pairwise_affinity(
            &positions,
            &ids(&case["input"]["attributes_u32"]),
            &ids(&case["input"]["width_u32"]),
        )
        .unwrap(),
        matrix(&case["expected"]["affinity_u32"])
    );
}

#[test]
fn malformed_selector_shapes_and_nonfinite_inputs_reject() {
    let vectors = vectors();
    let source = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "U-SEL-SINGLE")
        .unwrap();
    let mut input = selector_input(&source["input"]);
    input.width_u32.push(0);
    assert_eq!(candidate_selector(input), Err(SelectorError::InvalidInput));

    let mut input = selector_input(&source["input"]);
    input.feature_u32[0][0] = 0x7fc0_0000;
    assert_eq!(candidate_selector(input), Err(SelectorError::InvalidInput));
}
