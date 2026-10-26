// pattern: Imperative Shell
use jocforge_domain::spatial::selected_p15_geometry::{
    SelectedP15GeometryError, generate_selected_p15_geometry,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const PACKAGE_MANIFEST: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-P15-BASIS-RECIPE-V1/MANIFEST.json"
);
const BASIS_RECIPE: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-P15-BASIS-RECIPE-V1/CONTRACT-AND-INPUT-RECIPE.json"
);
const PACKAGE_MANIFEST_SHA256: &str =
    "211eb0b4bf2e672d0567e61b781986ce0cf9b7eb5e90514f5ad4f6038a4af8b7";

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn selected_p15_generator_matches_the_hash_bound_stable_basis_recipe() {
    assert_eq!(sha256(PACKAGE_MANIFEST), PACKAGE_MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(PACKAGE_MANIFEST).unwrap();
    let recipe_hash = sha256(BASIS_RECIPE);
    let entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["file"] == "CONTRACT-AND-INPUT-RECIPE.json")
        .unwrap();
    assert_eq!(
        entry["bytes"].as_u64().unwrap(),
        u64::try_from(BASIS_RECIPE.len()).unwrap()
    );
    assert_eq!(entry["sha256"].as_str().unwrap(), recipe_hash);

    let recipe: Value = serde_json::from_slice(BASIS_RECIPE).unwrap();
    let geometry = generate_selected_p15_geometry(
        u32::try_from(recipe["layout"].as_u64().unwrap()).unwrap(),
        recipe["profile_enabled"].as_bool().unwrap(),
    )
    .unwrap();
    let expected_q15 = recipe["q15_xyz_rows"].as_array().unwrap();
    let expected_indices = recipe["source_geometry_indices_in_stable_row_order"]
        .as_array()
        .unwrap();
    let expected_words = recipe["diagnostic_f32_words"].as_array().unwrap();
    assert_eq!(geometry.layout(), 0x159b);
    assert_eq!(geometry.front_side_y_q15(), 5_285);
    assert_eq!(
        geometry.identity().generation_spec_sha256(),
        recipe["authority"]["spec_sha256"].as_str().unwrap()
    );
    assert_eq!(
        geometry.identity().front_side_y_parameter_sha256(),
        recipe["authorized_parameter"]["sha256"].as_str().unwrap()
    );
    assert_eq!(
        geometry.identity().approved_manifest_sha256(),
        recipe["authority"]["parent_manifest_sha256"]
            .as_str()
            .unwrap()
    );
    assert_eq!(geometry.points().len(), 15);
    for ordinal in 0..15 {
        let point = geometry.points()[ordinal];
        let expected_q15 = expected_q15[ordinal]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| i32::try_from(value.as_i64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let expected_words = expected_words[ordinal]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(point.ordinal().value(), u8::try_from(ordinal).unwrap());
        assert_eq!(
            point.source_layout_index(),
            u8::try_from(expected_indices[ordinal].as_u64().unwrap()).unwrap()
        );
        assert_eq!(point.q15_xyz().as_slice(), expected_q15.as_slice());
        assert_eq!(point.xyz_f32_bits().as_slice(), expected_words.as_slice());
    }
    let basis_bytes = geometry
        .points()
        .iter()
        .flat_map(|point| point.xyz_f32_bits().into_iter().flat_map(u32::to_le_bytes))
        .collect::<Vec<_>>();
    assert_eq!(
        sha256(&basis_bytes),
        recipe["diagnostic_f32le_sha256"].as_str().unwrap()
    );
    assert_eq!(
        geometry.f32le_sha256(),
        Sha256::digest(&basis_bytes).as_slice()
    );
}

#[test]
fn p15_generator_rejects_other_layouts_and_disabled_profile() {
    assert_eq!(
        generate_selected_p15_geometry(0x111b, true),
        Err(SelectedP15GeometryError::UnsupportedProfile)
    );
    assert_eq!(
        generate_selected_p15_geometry(0x159b, false),
        Err(SelectedP15GeometryError::UnsupportedProfile)
    );
}
