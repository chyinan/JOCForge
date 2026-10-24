// pattern: Functional Core
use jocforge_domain::codec::exponent_packing::{
    ExponentPackingError, pack_exponent_difference_group,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const INPUT_FIXTURE: &str = "tests/fixtures/exponent_packing/input-vectors.json";
const INPUT_FIXTURE_SHA256: &str =
    "c021bb4e076841a35de2a39d8ac0866a4a5ec48bcbc640d0c6f51961cee8c5f5";
const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const EXPONENT_NORMALIZATION_SPEC_SHA256: &str =
    "a538f0af5c70c8e8f4e4b5212438a1e5abd304c3fb2efa7165617014a55fc5fc";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn input_fixture() -> (Vec<u8>, Value) {
    let bytes = std::fs::read(crate_root().join(INPUT_FIXTURE)).unwrap();
    let value = serde_json::from_slice(&bytes).unwrap();
    (bytes, value)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn differences(values: &[Value]) -> Vec<i8> {
    values
        .iter()
        .map(|value| i8::try_from(value.as_i64().unwrap()).unwrap())
        .collect()
}

fn packed_value(differences: &[i8]) -> u8 {
    pack_exponent_difference_group(differences)
        .expect("the test vector is in the F10.5 input domain")
        .value()
}

#[test]
fn washed_spec_identity_and_input_only_fixture_are_bound() {
    let (bytes, fixture) = input_fixture();
    assert_eq!(sha256(&bytes), INPUT_FIXTURE_SHA256);
    assert_eq!(fixture["schema"], "f10-exponent-digit-packing-input.v1");
    assert_eq!(
        fixture["source_package_manifest_sha256"],
        PACKAGE_MANIFEST_SHA256
    );
    assert_eq!(
        fixture["source_spec_sha256"],
        EXPONENT_NORMALIZATION_SPEC_SHA256
    );
    assert_eq!(fixture["input_only"], true);
    assert_eq!(
        fixture["forbidden_seed_fields"],
        serde_json::json!(["expected", "actual", "result", "output"])
    );
}

#[test]
fn all_125_three_digit_groups_match_f10_base_five_packing() {
    let mut seen = [false; 125];
    for first in -2_i8..=2 {
        for second in -2_i8..=2 {
            for third in -2_i8..=2 {
                let digits = [first + 2, second + 2, third + 2];
                let expected = 25_u16 * u16::try_from(digits[0]).unwrap()
                    + 5_u16 * u16::try_from(digits[1]).unwrap()
                    + u16::try_from(digits[2]).unwrap();
                let encoded = packed_value(&[first, second, third]);
                assert_eq!(u16::from(encoded), expected);
                assert!(!seen[usize::from(encoded)]);
                seen[usize::from(encoded)] = true;
            }
        }
    }
    assert!(seen.into_iter().all(|encoded| encoded));
}

#[test]
fn partial_tail_groups_fill_missing_digits_with_two() {
    let (_, fixture) = input_fixture();
    let partial = fixture["partial_groups"].as_array().unwrap();
    let expected = [12_u8, 62, 112, 22, 62, 122];
    assert_eq!(partial.len(), expected.len());

    for (case, expected) in partial.iter().zip(expected) {
        assert_eq!(
            packed_value(&differences(case.as_array().unwrap())),
            expected
        );
    }
}

#[test]
fn empty_long_and_out_of_range_groups_return_exponent_input_error() {
    let (_, fixture) = input_fixture();
    for case in fixture["invalid_groups"].as_array().unwrap() {
        let differences = differences(case.as_array().unwrap());
        let actual = pack_exponent_difference_group(&differences);
        assert_eq!(actual, Err(ExponentPackingError::ExponentInputError));
        assert_eq!(actual.unwrap_err().code(), "EXPONENT_INPUT_ERROR");
    }
}
