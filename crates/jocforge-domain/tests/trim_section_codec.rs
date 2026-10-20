// pattern: Functional Core
//! Direct checks for the washed selected Type11 trim-section codec contract.

use jocforge_domain::joc::oamd::trim_section::{
    Trim, TrimSectionError, decode_trim_body, encode_trim_body, parse_dbmd_sections,
    selected_trim_from_dbmd_sections,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;

const PACKAGE_MANIFEST: &[u8] = include_bytes!("fixtures/f01e_provider/MANIFEST.json");
const TRIM_CONTRACT: &[u8] = include_bytes!("fixtures/oamd_wire/trim_section/CONTRACT.md");
const TRIM_MACHINE_MAP: &[u8] = include_bytes!("fixtures/oamd_wire/trim_section/MACHINE-MAP.json");
const TRIM_VECTORS: &[u8] =
    include_bytes!("fixtures/oamd_wire/trim_section/TRIM-SECTION-VECTORS.json");
const TRIM_INDEPENDENT_CHECK: &[u8] =
    include_bytes!("fixtures/oamd_wire/trim_section/INDEPENDENT-CHECK.json");

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const TRIM_CONTRACT_SHA256: &str =
    "d2c57bb0df05f0a50ca1950114d87a42802e677dcb5a5364e336e3b3ae8b7ace";
const TRIM_MACHINE_MAP_SHA256: &str =
    "0887a4d9358b6e9f85ebb269860956da5ff54b94663ca9cea2d20d1cd84d92a8";
const TRIM_VECTORS_SHA256: &str =
    "d836a336dec4bfad9f624562f761e7b6717e2cad6d6040dfeb105dce639a83fe";
const TRIM_INDEPENDENT_CHECK_SHA256: &str =
    "bf175664350f678f9a948df5a5a17a4141151dd4fbcf897230ac7acbbfa8472a";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0, "hex input must have an even length");
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("hex digits are UTF-8");
            u8::from_str_radix(text, 16).expect("fixture contains valid hex")
        })
        .collect()
}

fn vectors() -> Value {
    serde_json::from_slice(TRIM_VECTORS).expect("trim vectors are JSON")
}

fn trim_case<'a>(cases: &'a Value, id: &str) -> &'a Value {
    cases
        .as_array()
        .expect("cases are an array")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("missing trim vector {id}"))
}

fn trim_from_json(value: &Value) -> Trim {
    let flags = value["group_flags"]
        .as_array()
        .expect("group flags are an array");
    assert_eq!(flags.len(), 9);
    let mut group_flags = [0_u8; 9];
    for (target, source) in group_flags.iter_mut().zip(flags) {
        *target = u8::try_from(source.as_u64().expect("group flag is unsigned"))
            .expect("group flag fits u8");
    }

    let groups = value["groups"].as_array().expect("groups are an array");
    assert_eq!(groups.len(), 9);
    let mut group_values = [[0_i32; 5]; 9];
    for (target_group, source_group) in group_values.iter_mut().zip(groups) {
        let fields = source_group.as_array().expect("group is an array");
        assert_eq!(fields.len(), 5);
        for (target, source) in target_group.iter_mut().zip(fields) {
            *target = i32::try_from(source.as_i64().expect("group value is signed"))
                .expect("group value fits i32");
        }
    }

    let row_flags = value["row_flags"]
        .as_array()
        .expect("row flags are an array")
        .iter()
        .map(|flag| {
            u8::try_from(flag.as_u64().expect("row flag is unsigned")).expect("row flag fits u8")
        })
        .collect();

    Trim {
        trim_mode: u8::try_from(value["trim_mode"].as_u64().expect("trim mode is unsigned"))
            .expect("trim mode fits u8"),
        group_flags,
        groups: group_values,
        row_flags,
    }
}

fn expected_error_name(error: TrimSectionError) -> &'static str {
    match error {
        TrimSectionError::TrimShape => "TRIM_SHAPE",
        TrimSectionError::TrimDomain => "TRIM_DOMAIN",
        TrimSectionError::TrimU4 => "TRIM_U4",
        TrimSectionError::TrimSignedBalance => "TRIM_SIGNED_BALANCE",
        TrimSectionError::TrimTruncated => "TRIM_TRUNCATED",
        TrimSectionError::TrimReserved => "TRIM_RESERVED",
        TrimSectionError::TrimGroupMode => "TRIM_GROUP_MODE",
        TrimSectionError::TrimNonzeroPadding => "TRIM_NONZERO_PADDING",
        TrimSectionError::DbmdHeader => "DBMD_HEADER",
        TrimSectionError::DbmdTruncatedHeader => "DBMD_TRUNCATED_HEADER",
        TrimSectionError::DbmdTruncatedPayload => "DBMD_TRUNCATED_PAYLOAD",
        TrimSectionError::DbmdChecksum => "DBMD_CHECKSUM",
        TrimSectionError::DbmdDuplicateSection => "DBMD_DUPLICATE_SECTION",
        TrimSectionError::DbmdTerminatorTail => "DBMD_TERMINATOR_TAIL",
        TrimSectionError::DbmdRequiredSections => "DBMD_REQUIRED_SECTIONS",
        TrimSectionError::DbmdSupplemental => "DBMD_SUPPLEMENTAL",
        TrimSectionError::DbmdWarmMode => "DBMD_WARM_MODE",
        TrimSectionError::DbmdNondefaultTrim => "DBMD_NONDEFAULT_TRIM",
    }
}

fn assert_error<T>(expected: &str, result: Result<T, TrimSectionError>) {
    let error = result
        .err()
        .unwrap_or_else(|| panic!("expected {expected}, got success"));
    assert_eq!(expected_error_name(error), expected);
}

fn selected_dbmd_chunk(vectors: &Value) -> Vec<u8> {
    decode_hex(
        vectors["dbmd_selected_default"]["input"]["chunk_hex"]
            .as_str()
            .expect("selected DBMD input is hex"),
    )
}

fn first_dbmd_section(chunk: &[u8]) -> (usize, usize, usize) {
    assert!(chunk.len() >= 7);
    let body_length = usize::from(u16::from_le_bytes([chunk[5], chunk[6]]));
    let body_start = 7;
    let checksum_index = body_start + body_length;
    let record_end = checksum_index + 1;
    assert!(record_end <= chunk.len());
    (body_start, checksum_index, record_end)
}

#[test]
fn trim_fixture_assets_match_the_approved_package_and_supplement_identities() {
    assert_eq!(sha256_hex(PACKAGE_MANIFEST), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256_hex(TRIM_CONTRACT), TRIM_CONTRACT_SHA256);
    assert_eq!(sha256_hex(TRIM_MACHINE_MAP), TRIM_MACHINE_MAP_SHA256);
    assert_eq!(sha256_hex(TRIM_VECTORS), TRIM_VECTORS_SHA256);
    assert_eq!(
        sha256_hex(TRIM_INDEPENDENT_CHECK),
        TRIM_INDEPENDENT_CHECK_SHA256
    );

    let machine_map: Value = serde_json::from_slice(TRIM_MACHINE_MAP).expect("machine map is JSON");
    assert_eq!(machine_map["identity"], "selected-trim-section-codec.v1");
    assert_eq!(machine_map["stateful"], false);

    let vector_file = vectors();
    assert_eq!(vector_file["schema"], "trim-section-direct-vectors.v1");
    assert_eq!(vector_file["input_only"], true);

    let independent: Value =
        serde_json::from_slice(TRIM_INDEPENDENT_CHECK).expect("independent check is JSON");
    assert_eq!(independent["all_pass"], true);
    assert_eq!(independent["expected_used_as_seed"], false);
}

#[test]
fn trim_body_success_vectors_match_bytes_valid_bits_and_decoded_values() {
    let vector_file = vectors();
    let cases = vector_file["success_cases"]
        .as_array()
        .expect("success cases are an array");
    assert_eq!(cases.len(), 5);

    for case in cases {
        let input = &case["input"];
        let expected = &case["expected"];
        let row_count = u32::try_from(input["row_count"].as_u64().unwrap()).unwrap();
        let trim = trim_from_json(input);
        let encoded = encode_trim_body(&trim, row_count)
            .unwrap_or_else(|error| panic!("{} should encode, got {error:?}", case["id"]));
        let expected_bytes = decode_hex(expected["body_hex"].as_str().unwrap());

        assert_eq!(encoded.body_bytes, expected_bytes, "{}", case["id"]);
        assert_eq!(
            u64::from(encoded.valid_bits),
            expected["valid_bits"].as_u64().unwrap(),
            "{}",
            case["id"]
        );
        assert_eq!(
            u64::from(encoded.padding_bits),
            expected["padding_bits"].as_u64().unwrap(),
            "{}",
            case["id"]
        );
        assert_eq!(
            u64::from(encoded.group_mode),
            expected["group_mode"].as_u64().unwrap(),
            "{}",
            case["id"]
        );
        assert_eq!(
            encoded.body_bytes.len() * 8 - encoded.valid_bits as usize,
            encoded.padding_bits as usize,
            "{}",
            case["id"]
        );

        let decoded = decode_trim_body(&encoded.body_bytes, row_count)
            .unwrap_or_else(|error| panic!("{} should decode, got {error:?}", case["id"]));
        assert_eq!(
            decoded,
            trim_from_json(&expected["decoded"]),
            "{}",
            case["id"]
        );
    }
}

#[test]
fn selected_dbmd_default_projection_matches_its_input_only_vector() {
    let vector_file = vectors();
    let selected = &vector_file["dbmd_selected_default"];
    let row_count = u32::try_from(selected["input"]["row_count"].as_u64().unwrap()).unwrap();
    let sections = parse_dbmd_sections(&selected_dbmd_chunk(&vector_file))
        .expect("selected DBMD sections are well framed");
    let actual = selected_trim_from_dbmd_sections(&sections, row_count)
        .expect("selected DBMD default trim is supported");
    assert_eq!(actual, trim_from_json(&selected["expected"]));
}

#[test]
fn trim_rejection_cases_match_their_declared_error_categories() {
    let vector_file = vectors();
    let rejection_cases = vector_file["rejection_cases"]
        .as_array()
        .expect("rejection cases are an array");
    assert_eq!(rejection_cases.len(), 8);

    for case in rejection_cases {
        let expected = case["expected_error"].as_str().unwrap();
        let actual = match case["id"].as_str().unwrap() {
            "duplicate_section" => {
                let chunk = selected_dbmd_chunk(&vector_file);
                let (_, _, first_record_end) = first_dbmd_section(&chunk);
                let mut duplicate = chunk.clone();
                let first_record = chunk[4..first_record_end].to_vec();
                duplicate.splice(first_record_end..first_record_end, first_record);
                parse_dbmd_sections(&duplicate).map(|_| ())
            }
            "truncated_header" => parse_dbmd_sections(&[0, 0, 0, 0, 9]).map(|_| ()),
            "truncated_payload" => parse_dbmd_sections(&[0, 0, 0, 0, 9, 2, 0, 0xaa]).map(|_| ()),
            "nonzero_terminator_tail" => parse_dbmd_sections(&[0, 0, 0, 0, 0, 1]).map(|_| ()),
            "checksum_mismatch" => {
                let mut chunk = selected_dbmd_chunk(&vector_file);
                let (_, checksum_index, _) = first_dbmd_section(&chunk);
                chunk[checksum_index] ^= 1;
                parse_dbmd_sections(&chunk).map(|_| ())
            }
            "nondefault_trim" => {
                let sections = parse_dbmd_sections(&selected_dbmd_chunk(&vector_file))
                    .expect("base DBMD sections are valid");
                let mut changed_sections = sections;
                let section10 = changed_sections.get_mut(&10).unwrap();
                let record_start = (0..9)
                    .map(|ordinal| 7 + ordinal * 15)
                    .find(|start| section10[*start] & 1 == 0)
                    .expect("selected DBMD contains a default-trim record");
                section10[record_start + 1..record_start + 5]
                    .copy_from_slice(&1.0_f32.to_le_bytes());
                selected_trim_from_dbmd_sections(&changed_sections, 16).map(|_| ())
            }
            "nonzero_padding" => {
                let success = trim_case(&vector_file["success_cases"], "group_mode0_all_skip");
                let mut body = decode_hex(success["expected"]["body_hex"].as_str().unwrap());
                *body.last_mut().unwrap() |= 1;
                decode_trim_body(&body, 3).map(|_| ())
            }
            "truncated_trim_body" => decode_trim_body(&[], 3).map(|_| ()),
            unknown => panic!("unhandled rejection vector {unknown}"),
        };
        assert_error(expected, actual);
    }
}

#[test]
fn trim_encoder_rejects_shape_domain_u4_and_signed_balance_errors() {
    let vector_file = vectors();
    let case = trim_case(
        &vector_file["success_cases"],
        "group_mode2_mask_signed_rows",
    );
    let mut trim = trim_from_json(&case["input"]);

    trim.row_flags.pop();
    assert_error("TRIM_SHAPE", encode_trim_body(&trim, 3));

    let oversized_trim = Trim {
        trim_mode: 0,
        group_flags: [1; 9],
        groups: [[16, 16, 16, 0, 0]; 9],
        row_flags: vec![0; 160],
    };
    assert_error("TRIM_SHAPE", encode_trim_body(&oversized_trim, 160));
    assert_error("TRIM_SHAPE", decode_trim_body(&[0], 160));

    let mut trim = trim_from_json(&case["input"]);
    trim.trim_mode = 4;
    assert_error("TRIM_DOMAIN", encode_trim_body(&trim, 3));

    let mut trim = trim_from_json(&case["input"]);
    trim.group_flags[0] = 2;
    assert_error("TRIM_DOMAIN", encode_trim_body(&trim, 3));

    let mut trim = trim_from_json(&case["input"]);
    trim.groups[0][0] = 17;
    assert_error("TRIM_U4", encode_trim_body(&trim, 3));

    let mut trim = trim_from_json(&case["input"]);
    trim.groups[0][3] = 17;
    assert_error("TRIM_SIGNED_BALANCE", encode_trim_body(&trim, 3));
}

#[test]
fn trim_decoder_rejects_reserved_bits_and_group_mode_three() {
    assert_error("TRIM_RESERVED", decode_trim_body(&[0x20], 0));
    assert_error("TRIM_GROUP_MODE", decode_trim_body(&[0x0c], 0));
    assert_error("TRIM_SHAPE", decode_trim_body(&[0, 0], 0));
}

#[test]
fn selected_dbmd_projection_rejects_header_required_section_and_supplement_errors() {
    assert_error(
        "DBMD_HEADER",
        parse_dbmd_sections(&[2, 0, 0, 0, 0, 0]).map(|_| ()),
    );
    assert_error(
        "DBMD_REQUIRED_SECTIONS",
        selected_trim_from_dbmd_sections(&BTreeMap::new(), 0),
    );

    let vector_file = vectors();
    let sections = parse_dbmd_sections(&selected_dbmd_chunk(&vector_file)).unwrap();

    let mut invalid_magic = sections.clone();
    invalid_magic.get_mut(&10).unwrap()[0] = 0;
    assert_error(
        "DBMD_SUPPLEMENTAL",
        selected_trim_from_dbmd_sections(&invalid_magic, 16),
    );

    let mut invalid_warm_mode = sections;
    invalid_warm_mode.get_mut(&9).unwrap()[152] = 4;
    assert_error(
        "DBMD_WARM_MODE",
        selected_trim_from_dbmd_sections(&invalid_warm_mode, 16),
    );
}

#[test]
fn selected_dbmd_projection_accepts_ignored_values_and_signed_zero() {
    let vector_file = vectors();
    let selected = &vector_file["dbmd_selected_default"];
    let row_count = u32::try_from(selected["input"]["row_count"].as_u64().unwrap()).unwrap();
    let base_sections = parse_dbmd_sections(&selected_dbmd_chunk(&vector_file)).unwrap();
    let expected = trim_from_json(&selected["expected"]);

    let mut signed_zero_sections = base_sections.clone();
    let section10 = signed_zero_sections.get_mut(&10).unwrap();
    let default_record = (0..9)
        .map(|ordinal| 7 + ordinal * 15)
        .find(|start| section10[*start] & 1 == 0)
        .expect("selected DBMD contains a default-trim record");
    section10[default_record + 1..default_record + 5].copy_from_slice(&(-0.0_f32).to_le_bytes());
    section10[default_record + 13] |= 0x80;
    section10[default_record + 14] |= 0x80;
    assert_eq!(
        selected_trim_from_dbmd_sections(&signed_zero_sections, row_count).unwrap(),
        expected
    );

    let mut ignored_record_sections = base_sections;
    let section10 = ignored_record_sections.get_mut(&10).unwrap();
    let ignored_record = (0..9)
        .map(|ordinal| 7 + ordinal * 15)
        .find(|start| section10[*start] & 1 == 1)
        .expect("selected DBMD contains a skipped record");
    section10[ignored_record + 1..ignored_record + 5].copy_from_slice(&f32::INFINITY.to_le_bytes());
    section10[ignored_record + 13] |= 0x7f;
    section10[ignored_record + 14] |= 0x7f;
    assert_eq!(
        selected_trim_from_dbmd_sections(&ignored_record_sections, row_count).unwrap(),
        expected
    );
}
