// pattern: Functional Core
use jocforge_domain::codec::refresh_difference::{RefreshDifferenceError, map_refresh_difference};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/refresh_difference";
const BASE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const PACKAGE_MANIFEST_SHA256: &str =
    "ad4708faaf4f1f9d71a93bc0fdebe4eca4f6a4aff1080579ef5f1cce77808d57";
const INDEPENDENT_CHECK_SHA256: &str =
    "59accb7a74f523b44043a07b28162c75a5c2c9dbdef26791adbe35e0fb0b076c";
const GENERATED_TABLE_SHA256: &str =
    "a0e75dafa9f3c1627ec36351ed857711c5a79f60fd7fda0b9f988f0505a4a070";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 4] = [
    (
        "CONTRACT.md",
        2101,
        "1861534c4a2c44e8755c40bccbfe9aa63ffc90462d589c703bfc2c51124a2023",
    ),
    (
        "PARAMETERS.json",
        1661,
        "6cf3ae8d899b334d5aa87e0744e4c58b7dba1d92177ca07738726f8ea0ee2c6d",
    ),
    (
        "PROVENANCE.json",
        1643,
        "871b03569dd29305227861a9ad6e7c536ff9af0dacf92b51ff38ac76dfc67873",
    ),
    (
        "VECTORS.json",
        3664,
        "bc8841bce23069d392d67f59bf06ae5ab7e6e6b7fb6ae0f17396e03c2a8767b8",
    ),
];

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(crate_root().join(FIXTURE_DIRECTORY).join(name)).unwrap()
}

fn fixture_json(name: &str) -> Value {
    serde_json::from_slice(&fixture_bytes(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn package_and_payload_identities_are_hash_bound_without_a_second_manifest_fixture() {
    let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 4);
    assert_eq!(check["all_payload_hashes_match"], true);
    assert_eq!(check["formula_matches_all_valid_vectors"], 32);
    assert_eq!(check["valid_vector_count"], 32);
    assert_eq!(check["rejected_input_count"], 7);
    assert_eq!(check["generated_i16le_sha256"], GENERATED_TABLE_SHA256);
    assert_eq!(check["whole_implementation_ready"], "NO");

    let base_manifest =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    assert_eq!(sha256(&base_manifest), BASE_MANIFEST_SHA256);
    assert_eq!(check["source_manifest_sha256_match"], true);

    for (name, expected_bytes, expected_sha256) in PACKAGE_PAYLOADS {
        let bytes = fixture_bytes(name);
        assert_eq!(bytes.len(), expected_bytes, "{name} size");
        assert_eq!(sha256(&bytes), expected_sha256, "{name} hash");
    }
}

#[test]
fn all_32_refresh_difference_indices_match_the_integer_map_and_serialized_digest() {
    let vectors = fixture_json("VECTORS.json");
    let cases = vectors["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 32);

    let mut serialized = Vec::with_capacity(64);
    for (expected_index, case) in cases.iter().enumerate() {
        let index = i16::try_from(case["input"].as_i64().unwrap()).unwrap();
        let expected = i16::try_from(case["expected_i16"].as_i64().unwrap()).unwrap();
        assert_eq!(index, i16::try_from(expected_index).unwrap());
        let actual = map_refresh_difference(index).unwrap();
        assert_eq!(actual, expected, "D06E_VALID_{expected_index:02}");
        serialized.extend_from_slice(&actual.to_le_bytes());
    }

    let parameters = fixture_json("PARAMETERS.json");
    assert_eq!(
        serialized.len(),
        parameters["generated_bytes"].as_u64().unwrap() as usize
    );
    assert_eq!(sha256(&serialized), GENERATED_TABLE_SHA256);
    assert_eq!(
        sha256(&serialized),
        parameters["generated_i16le_sha256"].as_str().unwrap()
    );
}

#[test]
fn out_of_range_typed_indices_return_the_exponent_input_error() {
    let vectors = fixture_json("VECTORS.json");
    let rejections = vectors["rejections"].as_array().unwrap();
    assert_eq!(rejections.len(), 7);

    let mut numeric_rejections = 0;
    for case in rejections {
        let Some(index) = case["input"].as_i64() else {
            continue;
        };
        numeric_rejections += 1;
        let index = i16::try_from(index).unwrap();
        let error = map_refresh_difference(index).unwrap_err();
        assert_eq!(error, RefreshDifferenceError::IndexOutOfRange);
        assert_eq!(error.code(), case["expected_error"].as_str().unwrap());
    }

    // Boolean, float, null, and string rejections are bound in the washed vectors,
    // but are not passed through a synthetic JSON API: the production function is typed.
    assert_eq!(numeric_rejections, 2);
}
