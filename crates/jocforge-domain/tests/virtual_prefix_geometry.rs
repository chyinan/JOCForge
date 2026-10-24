// pattern: Imperative Shell
use std::fmt::Write as _;

use jocforge_domain::spatial::virtual_prefix_geometry::build_candidate_virtual_prefix;
use serde_json::Value;
use sha2::{Digest, Sha256};

const PROFILE: &str = include_str!("fixtures/selected_adm_graph/INPUT-ONLY-PROFILE.json");
const CONSTRUCTOR: &str =
    include_str!("fixtures/selected_adm_graph/NATIVE-CONSTRUCTOR-OBSERVATIONS.json");
const CONSUMERS: &str = include_str!("fixtures/selected_adm_graph/NATIVE-BOUND-CONSUMERS.json");

const PROFILE_SHA256: &str = "1c9d6ae0e4437eab2a1ee1587f37d8dfe620bf4876d822eaa47eda04d41fa822";
const CONSTRUCTOR_SHA256: &str = "c9be7ccf99936f9fa75482604bcab699d1906461b2c92c1c7d7cfbf4ead0bf0d";
const CONSUMERS_SHA256: &str = "c434ed9276b6674c941748e398b149f77a9331172813cff91261cfdaa846db04";

#[test]
fn candidate_q15_prefix_matches_all_constructor_xyz_and_defined_words() {
    assert_bound_fixture_hashes();
    let profile: Value = serde_json::from_str(PROFILE).unwrap();
    let constructor: Value = serde_json::from_str(CONSTRUCTOR).unwrap();
    let prefix = build_candidate_virtual_prefix();
    let points = constructor["points"].as_array().unwrap();
    let rows = constructor["rows"].as_array().unwrap();
    assert_eq!(profile["source_count"].as_u64(), Some(11));
    assert_eq!(profile["endpoint_q15"].as_i64(), Some(32_767));
    assert_eq!(profile["midpoint_q15"].as_i64(), Some(16_384));
    assert_eq!(profile["scale_denominator"].as_u64(), Some(32_768));
    assert_eq!(points.len(), 11);
    assert_eq!(rows.len(), 11);
    assert_eq!(prefix.source_count(), 11);
    assert_eq!(prefix.total_source_count(), 48);

    for (ordinal, row) in prefix.rows().iter().copied().enumerate() {
        let expected = &points[ordinal];
        assert_eq!(u64::from(row.index().value()), expected["ordinal_arg"]);
        assert_eq!(
            json_u32x3(row.q15_xyz().map(|value| u32::try_from(value).unwrap())),
            expected["effective_xyz_q15"]
        );
        assert_eq!(json_u32x3(row.xyz_words()), expected["xyz_f32_words"]);
        assert_eq!(row.attribute_components().as_slice(), &[0, 0, 0, 3, 2]);
        assert_eq!(row.packed_attribute(), 0x0000_0b00);
        assert_eq!(
            row.defined_metadata_words().as_slice(),
            rows[ordinal]["defined_words_4_to_55"]
                .as_array()
                .unwrap()
                .iter()
                .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
                .collect::<Vec<_>>()
                .as_slice()
        );
    }
}

#[test]
fn candidate_prefix_values_match_eight_washed_consumer_word_sets() {
    let consumers: Value = serde_json::from_str(CONSUMERS).unwrap();
    let calls = consumers["calls"].as_array().unwrap();
    let prefix = build_candidate_virtual_prefix();
    assert_eq!(calls.len(), 8);

    for call in calls {
        assert_eq!(call["rows"].as_array().unwrap().len(), 11);
        for (ordinal, row) in call["rows"].as_array().unwrap().iter().enumerate() {
            assert_eq!(row["source_index"].as_u64(), Some(ordinal as u64));
            assert_eq!(
                json_u32x3(prefix.rows()[ordinal].xyz_words()),
                row["xyz_words"]
            );
            assert_eq!(
                row["attribute_component_words"],
                serde_json::json!([0, 0, 0, 3, 2])
            );
        }
    }
}

fn json_u32x3(words: [u32; 3]) -> Value {
    serde_json::json!(words)
}

fn assert_bound_fixture_hashes() {
    assert_eq!(sha256_hex(PROFILE.as_bytes()), PROFILE_SHA256);
    assert_eq!(sha256_hex(CONSTRUCTOR.as_bytes()), CONSTRUCTOR_SHA256);
    assert_eq!(sha256_hex(CONSUMERS.as_bytes()), CONSUMERS_SHA256);
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").unwrap();
    }
    encoded
}
