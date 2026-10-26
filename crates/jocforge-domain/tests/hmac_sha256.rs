use jocforge_domain::integrity::hmac_sha256::hmac_sha256;
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUT_SHA256: &str = "61d30dffc0fdc08e0cf6b68d2dd559a6533ac8e88398fa0bb0f3bcf17ae27277";
const RESULT_SHA256: &str = "64d5941578e117b6edd63b335556261e75ef4f9e238b7e6014ac8bf6747dee4e";
const PROVENANCE_SHA256: &str = "1f77150683e7a0205ef252e3a0516602207c64bb68f879f3a26769b9e624873a";
const INPUTS: &str = include_str!("fixtures/hmac_sha256/INPUT-ONLY-VECTORS.json");
const RESULT: &str = include_str!("fixtures/hmac_sha256/RESULT.json");
const PROVENANCE: &str = include_str!("fixtures/hmac_sha256/PROVENANCE.json");

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16).unwrap();
            let low = char::from(pair[1]).to_digit(16).unwrap();
            u8::try_from(high * 16 + low).unwrap()
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[test]
fn hmac_sha256_matches_public_rfc4231_primitive_vector() {
    assert_eq!(sha256(INPUTS.as_bytes()), INPUT_SHA256);
    assert_eq!(sha256(RESULT.as_bytes()), RESULT_SHA256);
    assert_eq!(sha256(PROVENANCE.as_bytes()), PROVENANCE_SHA256);

    let inputs: Value = serde_json::from_str(INPUTS).unwrap();
    let expected: Value = serde_json::from_str(RESULT).unwrap();
    let provenance: Value = serde_json::from_str(PROVENANCE).unwrap();
    assert_eq!(inputs["input_only"], true);
    assert_eq!(
        inputs["source_package_manifest_sha256"],
        "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
    );
    assert!(
        provenance["key_class"]
            .as_str()
            .unwrap()
            .contains("not production or vendor key")
    );

    let input = &inputs["cases"][0];
    let expected = &expected["cases"][0];
    assert_eq!(input["id"], "U-HMAC-RFC4231-1");
    let digest = hmac_sha256(
        &decode_hex(input["key_hex"].as_str().unwrap()),
        &decode_hex(input["message_hex"].as_str().unwrap()),
    );
    assert_eq!(
        encode_hex(&digest),
        expected["digest_hex"].as_str().unwrap()
    );
}
