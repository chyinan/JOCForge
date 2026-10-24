// pattern: Imperative Shell
use jocforge_domain::spatial::selected_p11_geometry::generate_selected_p11_geometry;
use serde_json::Value;
use sha2::{Digest, Sha256};

const GEOMETRY_VECTORS: &str =
    include_str!("fixtures/primary_installed48/SELECTED-GEOMETRY-VECTORS.json");
const SOURCE_BINDING: &str = include_str!("fixtures/primary_installed48/P11-SOURCE-BINDING.json");

#[test]
fn selected_p11_is_generated_from_the_approved_layout_recipe() {
    let generated = generate_selected_p11_geometry(0x111b, true).unwrap();
    let vectors: Value = serde_json::from_str(GEOMETRY_VECTORS).unwrap();
    let p11 = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["case_id"] == "selected_P11")
        .unwrap();

    assert_eq!(generated.points().len(), 11);
    assert_eq!(p11["expected_point_count"].as_u64(), Some(11));
    assert_eq!(p11["expected_f32_word_count"].as_u64(), Some(33));
    let expected_points = p11["expected_basis_words_hex"].as_array().unwrap();
    let source_layout_indices = [0_u8, 1, 2, 4, 5, 6, 7, 13, 14, 21, 22];
    let mut serialized = Vec::with_capacity(132);
    for ((point, expected), layout_index) in generated
        .points()
        .iter()
        .zip(expected_points)
        .zip(source_layout_indices)
    {
        assert_eq!(point.source_layout_index(), layout_index);
        let expected_words = expected
            .as_array()
            .unwrap()
            .iter()
            .map(|word| u32::from_str_radix(word.as_str().unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(point.xyz_words(), expected_words.as_slice());
        for word in point.xyz_words() {
            serialized.extend_from_slice(&word.to_le_bytes());
        }
    }
    assert_eq!(
        hex(&Sha256::digest(&serialized)),
        p11["expected_basis_f32le_sha256"].as_str().unwrap()
    );

    let binding: Value = serde_json::from_str(SOURCE_BINDING).unwrap();
    assert_eq!(binding["case_id"], "selected_P11");
    assert_eq!(binding["matched_points"].as_u64(), Some(11));
    assert_eq!(binding["matched_f32_words"].as_u64(), Some(33));
    assert_eq!(
        binding["expected_f32le_sha256"],
        hex(&Sha256::digest(&serialized))
    );
}

#[test]
fn other_geometry_profiles_are_rejected() {
    assert_eq!(
        generate_selected_p11_geometry(0x159b, true)
            .unwrap_err()
            .code(),
        "UNSUPPORTED_PROFILE"
    );
    assert_eq!(
        generate_selected_p11_geometry(0x111b, false)
            .unwrap_err()
            .code(),
        "UNSUPPORTED_PROFILE"
    );
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut result, "{byte:02x}").expect("writing to a String cannot fail");
    }
    result
}
