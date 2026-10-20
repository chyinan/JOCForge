// pattern: Functional Core
//! F05O direct reader tests against manifest-pinned, input-only body vectors.

use jocforge_domain::joc::oamd::body_reader::{
    OamdBodyMetadata, OamdBodyReadError, decode_oamd_body,
};
use jocforge_domain::joc::oamd::rowdata::{OamdRowKind, OamdTiming};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

const PACKAGE_MANIFEST: &[u8] = include_bytes!("fixtures/f01e_provider/MANIFEST.json");
const COORDINATE_GRID: &[u8] =
    include_bytes!("fixtures/reader_target/oamd-coordinate-grid-q15.json");
const SUPPLEMENT_CONTRACT: &[u8] = include_bytes!("fixtures/oamd_wire/body_decoder/CONTRACT.md");
const HEADER_VECTORS: &[u8] =
    include_bytes!("fixtures/oamd_wire/body_decoder/INPUT-ONLY-VECTORS.json");
const READ_VECTORS: &[u8] =
    include_bytes!("fixtures/oamd_wire/body_decoder/unit-read-vectors.json");

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const GRID_PROVIDER_SHA256: &str =
    "b1814fec24ac57a45c5bf2eacaba91c0c581df19362c6ca7da4cc911ed336243";
const CONTRACT_BYTES: usize = 1340;
const CONTRACT_SHA256: &str = "35d2ecf49dfc9d2a546898026e1c1b4f14a8a0fd4396662b9fea8de6c61f1a73";
const HEADER_VECTORS_BYTES: usize = 1461;
const HEADER_VECTORS_SHA256: &str =
    "7e2d094ece73153f577536d7ea0255a112216aae09d9b5ef1f716cc6932b8e90";
const READ_VECTORS_SHA256: &str =
    "696e03400299e8577661e2e6dc1617f09b2547fa232ca9e2a3edab20ffa15ffb";

fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut result, "{byte:02x}").expect("writing to String is infallible");
    }
    result
}

fn manifest_asset<'a>(manifest: &'a Value, name: &str) -> &'a Value {
    manifest["files"]
        .as_array()
        .expect("manifest files are an array")
        .iter()
        .find(|entry| entry["file"] == name)
        .unwrap_or_else(|| panic!("missing manifest asset {name}"))
}

fn grid_tables() -> ([u32; 63], [u32; 16]) {
    let package_manifest: Value =
        serde_json::from_slice(PACKAGE_MANIFEST).expect("package manifest is JSON");
    assert_eq!(sha256_hex(PACKAGE_MANIFEST), PACKAGE_MANIFEST_SHA256);
    let grid_entry = manifest_asset(
        &package_manifest,
        "parameters/oamd-coordinate-grid-q15.json",
    );
    assert_eq!(
        grid_entry["bytes"].as_u64(),
        Some(COORDINATE_GRID.len() as u64)
    );
    assert_eq!(grid_entry["sha256"].as_str(), Some(GRID_PROVIDER_SHA256));
    assert_eq!(sha256_hex(COORDINATE_GRID), GRID_PROVIDER_SHA256);

    let grid: Value = serde_json::from_slice(COORDINATE_GRID).expect("coordinate grid is JSON");
    let xy = grid["xy_q15"].as_array().expect("XY table is an array");
    let z = grid["z_magnitude_q15"]
        .as_array()
        .expect("Z table is an array");
    assert_eq!(xy.len(), 63);
    assert_eq!(z.len(), 16);
    (
        std::array::from_fn(|index| u32::try_from(xy[index].as_u64().unwrap()).unwrap()),
        std::array::from_fn(|index| u32::try_from(z[index].as_u64().unwrap()).unwrap()),
    )
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).unwrap();
            let low = (pair[1] as char).to_digit(16).unwrap();
            u8::try_from((high << 4) | low).unwrap()
        })
        .collect()
}

fn read_vectors() -> Value {
    serde_json::from_slice(READ_VECTORS).expect("read vectors are JSON")
}

fn read_vector<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["cases"]
        .as_array()
        .expect("read vector cases are an array")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("missing read vector {id}"))
}

fn header_vector<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["cases"]
        .as_array()
        .expect("header vector cases are an array")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("missing header vector {id}"))
}

fn assert_metadata_matches_vectors(
    metadata: &OamdBodyMetadata,
    header_assertions: &Value,
    expected: &Value,
) {
    assert_eq!(
        u64::from(metadata.header.version),
        header_assertions["version"].as_u64().unwrap()
    );
    assert_eq!(
        u64::from(metadata.header.count_code),
        header_assertions["count_code"].as_u64().unwrap()
    );
    assert_eq!(
        u64::try_from(metadata.header.row_count).unwrap(),
        header_assertions["row_count"].as_u64().unwrap()
    );
    assert_eq!(
        metadata.header.shortcut,
        header_assertions["shortcut"].as_u64().unwrap() == 1
    );
    assert_eq!(
        metadata.header.leading_lfe,
        header_assertions["leading_lfe"].as_bool().unwrap()
    );
    assert_eq!(
        metadata.header.protected_sections,
        header_assertions["protected_sections"].as_u64().unwrap() == 1
    );
    assert_eq!(
        u64::from(metadata.header.section_count),
        header_assertions["section_count"].as_u64().unwrap()
    );

    let timing = &expected["timing"];
    assert_eq!(
        metadata.timing,
        OamdTiming {
            base: u8::try_from(timing["base"].as_u64().unwrap()).unwrap(),
            position: u8::try_from(timing["position"].as_u64().unwrap()).unwrap(),
            duration: u16::try_from(timing["duration"].as_u64().unwrap()).unwrap(),
            precision: u8::try_from(timing["precision"].as_u64().unwrap()).unwrap(),
        }
    );

    let expected_rows = expected["rows"].as_array().unwrap();
    assert_eq!(metadata.rows.len(), expected_rows.len());
    for (actual, expected) in metadata.rows.iter().zip(expected_rows) {
        let xyz = expected["xyz_q15"].as_array().unwrap();
        assert_eq!(
            actual.kind,
            match expected["kind"].as_str().unwrap() {
                "LFE" => OamdRowKind::Lfe,
                "OBJECT" => OamdRowKind::Object,
                other => panic!("unknown vector row kind {other}"),
            }
        );
        assert_eq!(actual.mute, expected["mute"].as_u64().unwrap() == 1);
        assert_eq!(
            actual.gain,
            u32::try_from(expected["gain"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.spread,
            u32::try_from(expected["spread"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.xyz_q15,
            [
                i32::try_from(xyz[0].as_i64().unwrap()).unwrap(),
                i32::try_from(xyz[1].as_i64().unwrap()).unwrap(),
                i32::try_from(xyz[2].as_i64().unwrap()).unwrap(),
            ]
        );
        assert_eq!(
            actual.component,
            expected["component"].as_u64().unwrap() as u8
        );
        assert_eq!(actual.z_valid, expected["z_valid"].as_u64().unwrap() == 1);
        assert_eq!(actual.diffuse, expected["diffuse"].as_u64().unwrap() == 1);
    }
}

#[test]
fn decoder_supplement_and_selected_read_vectors_are_manifest_pinned() {
    assert_eq!(sha256_hex(PACKAGE_MANIFEST), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256_hex(READ_VECTORS), READ_VECTORS_SHA256);
    for (name, bytes, expected_size, expected_sha256) in [
        (
            "CONTRACT.md",
            SUPPLEMENT_CONTRACT,
            CONTRACT_BYTES,
            CONTRACT_SHA256,
        ),
        (
            "INPUT-ONLY-VECTORS.json",
            HEADER_VECTORS,
            HEADER_VECTORS_BYTES,
            HEADER_VECTORS_SHA256,
        ),
    ] {
        assert_eq!(bytes.len(), expected_size, "{name}");
        assert_eq!(sha256_hex(bytes), expected_sha256, "{name}");
    }

    let package: Value = serde_json::from_slice(PACKAGE_MANIFEST).unwrap();
    let unit_entry = manifest_asset(&package, "test-vectors/unit/vectors.json");
    let local_vectors: Value = serde_json::from_slice(READ_VECTORS).unwrap();
    assert_eq!(
        local_vectors["source_manifest_sha256"].as_str(),
        Some(PACKAGE_MANIFEST_SHA256)
    );
    assert_eq!(
        local_vectors["source_asset_sha256"].as_str(),
        unit_entry["sha256"].as_str()
    );
}

#[test]
fn direct_decoder_reads_header_timing_and_rows_from_all_selected_read_vectors() {
    let (xy_table, z_table) = grid_tables();
    let headers: Value = serde_json::from_slice(HEADER_VECTORS).unwrap();
    let reads = read_vectors();

    for id in [
        "U-OAMD-ROWS-READ",
        "U-OAMD-TIME-TABLE-READ",
        "U-OAMD-TIME-EXPLICIT-READ",
    ] {
        let header_case = header_vector(&headers, id);
        let read_case = read_vector(&reads, id);
        assert_eq!(
            read_case["input"]["payload_hex"],
            header_case["payload_hex"]
        );
        let payload = decode_hex(header_case["payload_hex"].as_str().unwrap());
        let metadata = decode_oamd_body(&payload, &xy_table, &z_table)
            .unwrap_or_else(|error| panic!("{id} should decode, got {error:?}"));
        assert_metadata_matches_vectors(
            &metadata,
            &header_case["header_assertions"],
            &read_case["expected"],
        );
    }
}

#[test]
fn direct_decoder_rejects_protected_layouts_and_truncated_payloads() {
    let (xy_table, z_table) = grid_tables();
    let vectors = serde_json::from_slice(HEADER_VECTORS).unwrap();
    let case = header_vector(&vectors, "U-OAMD-ROWS-READ");
    let payload = decode_hex(case["payload_hex"].as_str().unwrap());

    let mut unsupported_version = payload.clone();
    unsupported_version[0] |= 0x80;
    assert_eq!(
        decode_oamd_body(&unsupported_version, &xy_table, &z_table),
        Err(OamdBodyReadError::UnsupportedProfile)
    );

    let mut protected_sections = payload.clone();
    protected_sections[1] |= 0x40;
    assert_eq!(
        decode_oamd_body(&protected_sections, &xy_table, &z_table),
        Err(OamdBodyReadError::UnsupportedProfile)
    );

    let mut alternate_shortcut = payload.clone();
    alternate_shortcut[0] &= !0x01;
    assert_eq!(
        decode_oamd_body(&alternate_shortcut, &xy_table, &z_table),
        Err(OamdBodyReadError::UnsupportedProfile)
    );

    let mut extra_section = payload.clone();
    extra_section[1] = (extra_section[1] & !0x0c) | 0x08;
    assert_eq!(
        decode_oamd_body(&extra_section, &xy_table, &z_table),
        Err(OamdBodyReadError::UnsupportedProfile)
    );

    let mut truncated = payload;
    truncated.pop();
    assert_eq!(
        decode_oamd_body(&truncated, &xy_table, &z_table),
        Err(OamdBodyReadError::InvalidInput)
    );
}
