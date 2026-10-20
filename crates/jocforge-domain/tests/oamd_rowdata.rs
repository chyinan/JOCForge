// pattern: Functional Core
//! Direct conformance checks for the washed OAMD rowdata writer vectors.

use jocforge_domain::joc::oamd::rowdata::{
    OamdRow, OamdRowKind, OamdRowdataError, OamdTiming, decode_escaped_integer,
    encode_escaped_integer, encode_oamd_body, encode_xyz_codes,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

const UNIT_VECTORS: &[u8] = include_bytes!("fixtures/oamd_wire/unit-vectors.json");
const PACKAGE_MANIFEST: &[u8] = include_bytes!("fixtures/f01e_provider/MANIFEST.json");
const SOURCE_UNIT_VECTORS_BYTES: u64 = 1_503_142;
const SOURCE_UNIT_VECTORS_SHA256: &str =
    "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8";
const LOCAL_VECTOR_FIXTURE_SHA256: &str =
    "8428fd8a52cb018c00fff87c7324d4dbcc47e5e1720c76b79476d45e6bd21129";
const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";

fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn vector_case(id: &str) -> Value {
    let vectors: Value = serde_json::from_slice(UNIT_VECTORS).expect("unit vectors are JSON");
    vectors["cases"]
        .as_array()
        .expect("unit vector cases are an array")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("missing vector {id}"))
        .clone()
}

fn timing_from_json(value: &Value) -> OamdTiming {
    OamdTiming {
        base: u8::try_from(value["base"].as_u64().expect("base is unsigned"))
            .expect("base fits u8"),
        position: u8::try_from(value["position"].as_u64().expect("position is unsigned"))
            .expect("position fits u8"),
        duration: u16::try_from(value["duration"].as_u64().expect("duration is unsigned"))
            .expect("duration fits u16"),
        precision: u8::try_from(value["precision"].as_u64().expect("precision is unsigned"))
            .expect("precision fits u8"),
    }
}

fn rows_from_json(value: &Value) -> Vec<OamdRow> {
    value
        .as_array()
        .expect("rows are an array")
        .iter()
        .map(|row| {
            let xyz = row["xyz_q15"].as_array().expect("xyz_q15 is an array");
            OamdRow {
                kind: match row["kind"].as_str().expect("kind is a string") {
                    "LFE" => OamdRowKind::Lfe,
                    "OBJECT" => OamdRowKind::Object,
                    other => panic!("unsupported fixture row kind {other}"),
                },
                mute: row["mute"].as_u64().expect("mute is unsigned") == 1,
                gain: u32::try_from(row["gain"].as_u64().expect("gain is unsigned"))
                    .expect("gain fits u32"),
                spread: u32::try_from(row["spread"].as_u64().expect("spread is unsigned"))
                    .expect("spread fits u32"),
                xyz_q15: [
                    i32::try_from(xyz[0].as_i64().expect("x is signed")).expect("x fits i32"),
                    i32::try_from(xyz[1].as_i64().expect("y is signed")).expect("y fits i32"),
                    i32::try_from(xyz[2].as_i64().expect("z is signed")).expect("z fits i32"),
                ],
                component: u8::try_from(row["component"].as_u64().expect("component is unsigned"))
                    .expect("component fits u8"),
                z_valid: row["z_valid"].as_u64().expect("z_valid is unsigned") == 1,
                diffuse: row["diffuse"].as_u64().expect("diffuse is unsigned") == 1,
            }
        })
        .collect()
}

#[test]
fn vector_fixture_is_pinned_by_the_approved_package_manifest() {
    assert_eq!(sha256_hex(PACKAGE_MANIFEST), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256_hex(UNIT_VECTORS), LOCAL_VECTOR_FIXTURE_SHA256);

    let manifest: Value = serde_json::from_slice(PACKAGE_MANIFEST).expect("manifest is JSON");
    let fixture: Value = serde_json::from_slice(UNIT_VECTORS).expect("fixture is JSON");
    assert_eq!(fixture["schema"], "oamd-rowdata-writer-vectors.v1");
    assert_eq!(
        fixture["source_manifest_sha256"].as_str(),
        Some(PACKAGE_MANIFEST_SHA256)
    );
    assert_eq!(
        fixture["source_asset"].as_str(),
        Some("test-vectors/unit/vectors.json")
    );
    let entry = manifest["files"]
        .as_array()
        .expect("manifest files are an array")
        .iter()
        .find(|entry| entry["file"] == "test-vectors/unit/vectors.json")
        .expect("approved package contains the selected unit vectors");
    assert_eq!(
        fixture["source_asset_bytes"].as_u64(),
        Some(SOURCE_UNIT_VECTORS_BYTES)
    );
    assert_eq!(entry["bytes"].as_u64(), Some(SOURCE_UNIT_VECTORS_BYTES));
    assert_eq!(
        fixture["source_asset_sha256"].as_str(),
        Some(SOURCE_UNIT_VECTORS_SHA256)
    );
    assert_eq!(entry["sha256"].as_str(), Some(SOURCE_UNIT_VECTORS_SHA256));
}

#[test]
fn xyz_codes_match_the_independent_wire_vectors() {
    let case = vector_case("U-OAMD-XYZ");
    let inputs = case["input"]["xyz_q15"]
        .as_array()
        .expect("XYZ inputs are an array");
    let expected = case["expected"]["points"]
        .as_array()
        .expect("XYZ expectations are an array");
    assert_eq!(inputs.len(), expected.len());

    for (input, expected) in inputs.iter().zip(expected) {
        let values = input.as_array().expect("XYZ point is an array");
        let point = [
            i32::try_from(values[0].as_i64().expect("x is signed")).expect("x fits i32"),
            i32::try_from(values[1].as_i64().expect("y is signed")).expect("y fits i32"),
            i32::try_from(values[2].as_i64().expect("z is signed")).expect("z fits i32"),
        ];
        if expected["status"] == "ACCEPTED" {
            let actual = encode_xyz_codes(point).expect("accepted XYZ is encodable");
            let expected_codes = expected["codes"]
                .as_array()
                .expect("accepted XYZ has expected codes");
            assert_eq!(
                actual.map(i64::from),
                [
                    expected_codes[0].as_i64().unwrap(),
                    expected_codes[1].as_i64().unwrap(),
                    expected_codes[2].as_i64().unwrap(),
                ]
            );
        } else {
            assert!(encode_xyz_codes(point).is_err(), "invalid point {point:?}");
        }
    }
}

#[test]
fn escaped_integers_match_the_independent_wire_vectors() {
    let case = vector_case("U-OAMD-ESCAPED");
    let values = case["input"]["values"]
        .as_array()
        .expect("escaped inputs are an array");
    let expected = case["expected"]["values"]
        .as_array()
        .expect("escaped expectations are an array");
    assert_eq!(values.len(), expected.len());

    for (input, expected) in values.iter().zip(expected) {
        let value = input.as_i64().expect("escaped input is signed");
        if expected["status"] == "ACCEPTED" {
            let (bytes, valid_bits) = encode_escaped_integer(value).expect("accepted integer");
            assert_eq!(hex(&bytes), expected["hex"].as_str().unwrap());
            assert_eq!(
                u64::from(valid_bits),
                expected["valid_bits"].as_u64().unwrap()
            );
            assert_eq!(
                decode_escaped_integer(&bytes, valid_bits).unwrap(),
                expected["decoded"].as_u64().unwrap()
            );
        } else {
            assert!(encode_escaped_integer(value).is_err());
        }
    }
}

#[test]
fn rowdata_writer_matches_all_independent_oamd_encoding_vectors() {
    for id in [
        "U-OAMD-ROWS",
        "U-OAMD-COUNT-31",
        "U-OAMD-COUNT-32",
        "U-OAMD-COUNT-159",
        "U-OAMD-TIME-TABLE",
        "U-OAMD-TIME-EXPLICIT",
    ] {
        let case = vector_case(id);
        let timing = timing_from_json(&case["input"]["timing"]);
        let rows = rows_from_json(&case["input"]["rows"]);
        let actual = encode_oamd_body(&timing, &rows)
            .unwrap_or_else(|error| panic!("{id} should encode successfully, got {error:?}"));
        assert_eq!(
            hex(&actual),
            case["expected"]["payload_hex"].as_str().unwrap(),
            "{id}"
        );
        assert_eq!(
            actual.len(),
            usize::try_from(case["expected"]["bytes"].as_u64().unwrap()).unwrap(),
            "{id}"
        );
    }
}

#[test]
fn rowdata_writer_rejects_unsupported_shapes_and_preserves_mute_field_skips() {
    let timing = OamdTiming {
        base: 0,
        position: 0,
        duration: 1536,
        precision: 32,
    };
    let object = OamdRow {
        kind: OamdRowKind::Object,
        mute: false,
        gain: 4096,
        spread: 32767,
        xyz_q15: [16384, 16384, 0],
        component: 0,
        z_valid: true,
        diffuse: false,
    };

    assert_eq!(
        encode_oamd_body(&timing, &[]),
        Err(OamdRowdataError::UnsupportedProfile)
    );
    assert_eq!(
        encode_oamd_body(&timing, &vec![object; 160]),
        Err(OamdRowdataError::UnsupportedProfile)
    );
    assert_eq!(
        encode_xyz_codes([-1, 0, 0]),
        Err(OamdRowdataError::InvalidInput)
    );

    let mut misplaced_lfe = object;
    misplaced_lfe.kind = OamdRowKind::Lfe;
    assert_eq!(
        encode_oamd_body(&timing, &[object, misplaced_lfe]),
        Err(OamdRowdataError::UnsupportedProfile)
    );

    let mut unsupported_gain = object;
    unsupported_gain.gain = 4350;
    assert_eq!(
        encode_oamd_body(&timing, &[unsupported_gain]),
        Err(OamdRowdataError::UnsupportedProfile)
    );

    let mut unsupported_spread = object;
    unsupported_spread.spread = 32766;
    assert_eq!(
        encode_oamd_body(&timing, &[unsupported_spread]),
        Err(OamdRowdataError::UnsupportedProfile)
    );

    let mut unsupported_timing = timing;
    unsupported_timing.duration = 2049;
    assert_eq!(
        encode_oamd_body(&unsupported_timing, &[object]),
        Err(OamdRowdataError::UnsupportedProfile)
    );

    let muted = OamdRow {
        mute: true,
        gain: u32::MAX,
        spread: 0,
        xyz_q15: [-1, 32768, -32768],
        component: u8::MAX,
        z_valid: false,
        diffuse: false,
        ..object
    };
    assert!(encode_oamd_body(&timing, &[muted]).is_ok());
}

#[test]
fn escaped_integer_reader_rejects_extra_bits_nonzero_padding_and_overlong_values() {
    assert_eq!(
        decode_escaped_integer(&[0b0000_0001], 5),
        Err(OamdRowdataError::InvalidEscapedInteger)
    );
    assert_eq!(
        decode_escaped_integer(&[0b0000_0000, 0b0000_0000], 5),
        Err(OamdRowdataError::InvalidEscapedInteger)
    );
    assert_eq!(
        decode_escaped_integer(&[0xff, 0xff, 0xff], 20),
        Err(OamdRowdataError::InvalidEscapedInteger)
    );
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}
