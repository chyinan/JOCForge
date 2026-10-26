// pattern: Functional Core
use jocforge_domain::codec::mantissa::{MantissaQuantizerError, quantize_mantissa};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_DIRECTORY: &str =
    "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-QUANTIZER-MANTISSA-PRODUCTION-CONTRACT-V1";
const PACKAGE_MANIFEST_SHA256: &str =
    "fd819c3ec27e7bade55efda5e648792c9c33892a9be7a36bbfce89231b9e961f";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn package_root() -> PathBuf {
    repository_root().join(PACKAGE_DIRECTORY)
}

fn read_json(name: &str) -> (Vec<u8>, Value) {
    let bytes = std::fs::read(package_root().join(name)).unwrap();
    let value = serde_json::from_slice(&bytes).unwrap();
    (bytes, value)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn approved_vectors() -> Value {
    let (manifest_bytes, manifest) = read_json("MANIFEST.json");
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(
        manifest["package_id"],
        "CODEC-QUANTIZER-MANTISSA-PRODUCTION-CONTRACT-V1"
    );

    let (vector_bytes, vectors) = read_json("INPUT-ONLY-VECTORS.json");
    let vector_entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["file"] == "INPUT-ONLY-VECTORS.json")
        .unwrap();
    assert_eq!(
        vector_entry["bytes"].as_u64().unwrap(),
        vector_bytes.len() as u64
    );
    assert_eq!(
        vector_entry["sha256"].as_str().unwrap(),
        sha256(&vector_bytes)
    );
    vectors
}

fn input(case: &Value) -> (u32, i16, u8) {
    (
        u32::from_str_radix(case["coefficient_f32_bits"].as_str().unwrap(), 16).unwrap(),
        i16::try_from(case["decoded_exponent"].as_i64().unwrap()).unwrap(),
        u8::try_from(case["bap"].as_u64().unwrap()).unwrap(),
    )
}

#[test]
fn approved_package_is_hash_bound_and_contains_inputs_only() {
    let vectors = approved_vectors();
    assert_eq!(vectors["schema"], "eac3-quantizer-mantissa-input-only.v1");
    assert_eq!(vectors["input_only"], true);
    assert_eq!(vectors["expected_quantizer_codes_included"], false);
    assert_eq!(vectors["expected_au_body_included"], false);
    assert_eq!(vectors["pcm_included"], false);
    assert_eq!(vectors["private_key_included"], false);
}

#[test]
fn approved_quantizer_inputs_are_accepted_except_reserved_sentinels() {
    let vectors = approved_vectors();
    let cases = vectors["domain_sweep"]
        .as_array()
        .unwrap()
        .iter()
        .chain(vectors["boundary_cases"].as_array().unwrap())
        .chain(vectors["precision_cases"].as_array().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 45);

    for case in cases {
        let (coefficient_word, decoded_exponent, bap) = input(case);
        let actual = quantize_mantissa(coefficient_word, decoded_exponent, bap);
        let is_sentinel = matches!(
            case["id"].as_str().unwrap(),
            "bap3_sentinel" | "bap5_sentinel"
        );
        if is_sentinel {
            assert_eq!(
                actual,
                Err(MantissaQuantizerError::ReservedCodeSentinel),
                "{}",
                case["id"]
            );
        } else {
            assert!(actual.is_ok(), "{}", case["id"]);
        }
    }
}

#[test]
fn selected_bap_formulas_use_the_scaled_coefficient() {
    let quarter = 0x3e80_0000_u32;
    assert_eq!(quantize_mantissa(quarter, 0, 0).unwrap().value(), 0);
    assert_eq!(quantize_mantissa(quarter, 0, 1).unwrap().value(), 1);
    assert_eq!(quantize_mantissa(quarter, 0, 2).unwrap().value(), 3);
    assert_eq!(quantize_mantissa(quarter, 0, 4).unwrap().value(), 6);
    assert_eq!(quantize_mantissa(0x3f00_0000, 0, 3).unwrap().value(), 5);
    assert_eq!(quantize_mantissa(0x3f00_0000, 0, 5).unwrap().value(), 11);

    // F64 scaling happens before the single conversion to R32.
    assert_eq!(quantize_mantissa(quarter, 2, 1).unwrap().value(), 2);
    assert_eq!(quantize_mantissa(0x3f80_0000, 24, 0).unwrap().width(), 0);
}

#[test]
fn fixed_point_ties_round_to_even_before_the_arithmetic_shift() {
    let widths = [5_u8, 6, 7, 8, 9, 10, 11, 12, 14, 16];
    for (offset, width) in widths.into_iter().enumerate() {
        let bap = u8::try_from(offset + 6).unwrap();
        let tie = 2.0_f32.powi(-i32::from(width));
        assert_eq!(
            quantize_mantissa(tie.to_bits(), 0, bap).unwrap().value(),
            1,
            "BAP {bap}"
        );
    }
}

#[test]
fn invalid_inputs_and_unrepresentable_intermediates_fail() {
    assert_eq!(
        quantize_mantissa(f32::INFINITY.to_bits(), 0, 0),
        Err(MantissaQuantizerError::NonFiniteInput)
    );
    assert_eq!(
        quantize_mantissa(f32::NAN.to_bits(), 0, 15),
        Err(MantissaQuantizerError::NonFiniteInput)
    );
    assert_eq!(
        quantize_mantissa(0.0_f32.to_bits(), 0, 16),
        Err(MantissaQuantizerError::UnsupportedBap)
    );
    assert_eq!(
        quantize_mantissa(f32::MAX.to_bits(), 0, 1),
        Err(MantissaQuantizerError::NonFiniteIntermediate)
    );
    assert_eq!(
        quantize_mantissa(f32::MAX.to_bits(), 24, 0),
        Err(MantissaQuantizerError::NonFiniteIntermediate)
    );
}
