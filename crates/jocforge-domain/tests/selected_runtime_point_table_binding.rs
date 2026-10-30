// pattern: Imperative Shell
use jocforge_domain::spatial::{
    selected_p11_geometry::generate_selected_p11_geometry,
    selected_p15_geometry::generate_selected_p15_geometry,
};
use serde_json::Value;
use std::{fs, path::PathBuf};

const SUPPLEMENT: &str = "SPATIAL-P11-P15-RUNTIME-POINT-TABLE-BINDING-V1";

fn observation() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../outputs/IMPLEMENTER_SUPPLEMENTS")
        .join(SUPPLEMENT)
        .join("OBSERVATION.json");
    serde_json::from_slice(&fs::read(path).expect("Wash observation is present"))
        .expect("Wash observation is valid JSON")
}

fn words(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .expect("point words are an array")
        .iter()
        .map(|word| {
            u32::try_from(word.as_u64().expect("point word is an unsigned integer"))
                .expect("point word fits U32")
        })
        .collect()
}

fn indices(value: &Value) -> Vec<u8> {
    value
        .as_array()
        .expect("source indices are an array")
        .iter()
        .map(|index| {
            u8::try_from(index.as_u64().expect("source index is an unsigned integer"))
                .expect("source index fits U8")
        })
        .collect()
}

#[test]
fn runtime_point_table_words_match_selected_math_values_without_closing_layout_origin() {
    let p11 = generate_selected_p11_geometry(0x111b, true)
        .expect("selected P11 mathematical basis is generated");
    let p15 = generate_selected_p15_geometry(0x159b, true)
        .expect("selected P15 mathematical basis is generated");
    let observation = observation();
    let p11_observation = &observation["profiles"]["P11"];
    let p15_observation = &observation["profiles"]["P15"];

    let p11_words = p11
        .points()
        .iter()
        .flat_map(|point| point.xyz_words())
        .collect::<Vec<_>>();
    let p15_words = p15
        .points()
        .iter()
        .flat_map(|point| point.xyz_f32_bits())
        .collect::<Vec<_>>();
    let p11_indices = p11
        .points()
        .iter()
        .map(|point| point.source_layout_index())
        .collect::<Vec<_>>();
    let p15_indices = p15
        .points()
        .iter()
        .map(|point| point.source_layout_index())
        .collect::<Vec<_>>();

    assert_eq!(
        p11_words,
        words(&p11_observation["native_point_table_xyz_u32"])
    );
    assert_eq!(
        p15_words,
        words(&p15_observation["native_point_table_xyz_u32"])
    );
    assert_eq!(
        p11_indices,
        indices(&p11_observation["approved_p11_full_geometry_source_indices"])
    );
    assert_eq!(
        p15_indices,
        indices(&p15_observation["approved_p15_full_geometry_source_indices"])
    );
    assert_eq!(p15.layout(), 0x159b);
    assert!(
        p11_observation["exact_approved_word_match_all_captured_calls"]
            .as_bool()
            .expect("P11 word-match status is boolean")
    );
    assert!(
        p15_observation["exact_approved_word_match_all_captured_calls"]
            .as_bool()
            .expect("P15 word-match status is boolean")
    );
    assert_eq!(
        p11_observation["consumer_pointer_identity_matches_all_captured_calls"],
        true
    );
    assert_eq!(
        p15_observation["consumer_pointer_identity_matches_all_captured_calls"],
        true
    );

    assert_eq!(p11_observation["raw_layout_mask_identity"], "NOT_CAPTURED");
    assert_eq!(p15_observation["raw_layout_mask_identity"], "NOT_CAPTURED");
    assert_eq!(
        observation["domain_separation"]["selected_profile_code_semantic_mapping"],
        "UNBOUND"
    );
    assert_eq!(
        observation["domain_separation"]["host_frame_join"],
        "NOT_CAPTURED"
    );
    assert_eq!(
        observation["domain_separation"]["ADM_sample_position_join"],
        "NOT_CAPTURED"
    );
}
