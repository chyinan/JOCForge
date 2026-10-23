// pattern: Functional Core
use jocforge_domain::joc::emdf::id2_wire::{
    Id2WireError, build_id2_body, map_drc_profile, serialize_id2_body,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_SPEC_SHA256: &str = "35aa89d0dd67fb0133c4898c1007c84d48b39c5df6683dd88a173600975b48cc";
const SOURCE_ADAPTER_SHA256: &str =
    "9bce2e35ce47147c76c13e58ee976d44c5109938f0b19e1f3a81f1d8a61fa1ae";
const SOURCE_VECTORS_SHA256: &str =
    "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8";
const LOCAL_FIXTURE_SHA256: &str =
    "b3b5cf65d35a77a0df31ff0211949e6805882c0d79ad1c7e975daded0d91f8cf";
const APPROVED_UNIT_VECTORS: &[u8] = include_bytes!("fixtures/f04_component/unit-vectors.json");

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixtures() -> PathBuf {
    crate_root().join("tests/fixtures/id2_wire")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn vectors() -> Value {
    serde_json::from_slice(&std::fs::read(fixtures().join("input-vectors.json")).unwrap()).unwrap()
}

fn case_by_id<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("missing F13D2 vector {id}"))
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16).unwrap();
            let low = char::from(pair[1]).to_digit(16).unwrap();
            u8::try_from((high << 4) | low).unwrap()
        })
        .collect()
}

#[test]
fn selected_f13d2_entries_are_a_hash_bound_subset_of_the_approved_package() {
    let manifest_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let vector_bytes = std::fs::read(fixtures().join("input-vectors.json")).unwrap();
    let selected = vectors();
    let source: Value = serde_json::from_slice(APPROVED_UNIT_VECTORS).unwrap();

    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256(&vector_bytes), LOCAL_FIXTURE_SHA256);
    assert_eq!(selected["source_manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(selected["source_asset_bytes"], 1_503_142);
    assert_eq!(selected["source_asset_sha256"], SOURCE_VECTORS_SHA256);
    assert_eq!(selected["source_spec_sha256"], SOURCE_SPEC_SHA256);
    assert_eq!(
        selected["source_adapter_contract_sha256"],
        SOURCE_ADAPTER_SHA256
    );
    assert_eq!(selected["cases"].as_array().unwrap().len(), 7);
    assert_eq!(selected["input_only_synthetic"], true);
    assert_eq!(sha256(APPROVED_UNIT_VECTORS), SOURCE_VECTORS_SHA256);
    for selected_case in selected["cases"].as_array().unwrap() {
        let source_case = source["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source_case| source_case["id"] == selected_case["id"])
            .unwrap();
        for key in ["id", "entry_id", "input", "expected"] {
            assert_eq!(selected_case[key], source_case[key]);
        }
    }

    for (path, bytes, hash) in [
        (
            "specs/13-id2-profile-metadata.md",
            3_236,
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
}

#[test]
fn profile_names_map_exactly_and_unknown_names_reject() {
    let vectors = vectors();
    let case = case_by_id(&vectors, "U-ID2-ENUMS");
    let profiles = case["input"]["profiles"].as_array().unwrap();
    let expected = case["expected"]["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), expected.len());

    for (profile, expected) in profiles.iter().zip(expected) {
        let result = map_drc_profile(profile.as_str().unwrap());
        if expected["status"] == "ACCEPTED" {
            assert_eq!(
                result.unwrap().value(),
                u8::try_from(expected["internal_code"].as_u64().unwrap()).unwrap()
            );
        } else {
            assert_eq!(result, Err(Id2WireError::UnsupportedProfile));
        }
    }
}

#[test]
fn type11_header_lfe_binding_and_id2_wire_vectors_match() {
    let vectors = vectors();
    for id in [
        "U-ID2-LFE",
        "U-ID2-NO-LFE",
        "U-ID2-EXTENDED",
        "U-ID2-VERSION",
        "U-ID2-SHORTCUT",
    ] {
        let case = case_by_id(&vectors, id);
        let input = &case["input"];
        let result = build_id2_body(
            input["line_profile"].as_str().unwrap(),
            input["rf_profile"].as_str().unwrap(),
            &decode_hex(input["type11_header_hex"].as_str().unwrap()),
        );
        if case["expected"]["status"] == "ACCEPTED" {
            let body = result.unwrap();
            assert_eq!(to_hex(&body.bytes), case["expected"]["payload_hex"]);
            assert_eq!(body.valid_bits, case["expected"]["valid_bits"]);
            assert_eq!(body.bytes.len(), 3);
        } else {
            assert_eq!(result, Err(Id2WireError::InvalidType11Header));
        }
    }
}

#[test]
fn id2_integer_fields_use_bitwise_and_and_exact_21_bit_padding() {
    let vectors = vectors();
    let case = case_by_id(&vectors, "U-ID2-BITWISE-AND");
    let input = &case["input"]["parameters"];
    let body = serialize_id2_body(
        u32::try_from(input["phase_indicator"].as_u64().unwrap()).unwrap(),
        u32::try_from(input["format_mask"].as_u64().unwrap()).unwrap(),
        u32::try_from(input["lfe_mask"].as_u64().unwrap()).unwrap(),
        u32::try_from(input["line_code"].as_u64().unwrap()).unwrap(),
        u32::try_from(input["rf_code"].as_u64().unwrap()).unwrap(),
    )
    .unwrap();

    assert_eq!(to_hex(&body.bytes), case["expected"]["payload_hex"]);
    assert_eq!(body.valid_bits, 21);
    assert_eq!(body.bytes.len(), 3);
    assert_eq!(body.bytes[2] & 0b0000_0111, 0);
}

#[test]
fn malformed_profiles_headers_and_wire_codes_reject() {
    assert_eq!(
        map_drc_profile("NONE"),
        Err(Id2WireError::UnsupportedProfile)
    );
    assert_eq!(
        build_id2_body("none", "none", &[0x01]),
        Err(Id2WireError::InvalidType11Header)
    );
    assert_eq!(
        build_id2_body("unknown", "none", &[0x01, 0x00]),
        Err(Id2WireError::UnsupportedProfile)
    );
    assert_eq!(
        serialize_id2_body(1, 1, 0, 8, 0),
        Err(Id2WireError::FieldOutOfRange)
    );
    assert_eq!(
        serialize_id2_body(1, 1, 0, 0, 8),
        Err(Id2WireError::FieldOutOfRange)
    );
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}
