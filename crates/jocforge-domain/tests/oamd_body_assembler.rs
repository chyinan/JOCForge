// pattern: Functional Core

use jocforge_domain::joc::oamd::body_assembler::{
    OamdBodyAssemblyError, OamdMetadata, OamdSectionDescriptor, assemble_body, decode_body,
};
use jocforge_domain::joc::oamd::rowdata::{OamdRow, OamdRowKind, OamdTiming};
use jocforge_domain::joc::oamd::trim_section::Trim;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CONTRACT: &[u8] = include_bytes!("fixtures/oamd_wire/body_assembler/CONTRACT.md");
const MACHINE_MAP: &[u8] = include_bytes!("fixtures/oamd_wire/body_assembler/MACHINE-MAP.json");
const VECTORS: &[u8] =
    include_bytes!("fixtures/oamd_wire/body_assembler/BODY-ASSEMBLER-VECTORS.json");
const INDEPENDENT_CHECK: &[u8] =
    include_bytes!("fixtures/oamd_wire/body_assembler/INDEPENDENT-CHECK.json");
const COORDINATE_GRID: &[u8] =
    include_bytes!("fixtures/reader_target/oamd-coordinate-grid-q15.json");

const CONTRACT_BYTES: usize = 3522;
const CONTRACT_SHA256: &str = "bb983fa71ac9468cf4d790047ba9ee6115dbbaac4afaf7eed4d9c30f04dd5b3e";
const MACHINE_MAP_BYTES: usize = 943;
const MACHINE_MAP_SHA256: &str = "3208c6b3eccfce3c5d74b74f2697b561bdf7753271b69a99be0ad338512c8c19";
const VECTORS_BYTES: usize = 30419;
const VECTORS_SHA256: &str = "6ae2b54fcb3efa7ca716517c0c78e0dfac3eddefd92dc72d1f606c53db25fd2c";
const INDEPENDENT_CHECK_BYTES: usize = 1035;
const INDEPENDENT_CHECK_SHA256: &str =
    "5bb3c64ce6d7a56feeb968776749c4789d4fc348a8c81dc046db5fcc6de9ebeb";
const COORDINATE_GRID_SHA256: &str =
    "b1814fec24ac57a45c5bf2eacaba91c0c581df19362c6ca7da4cc911ed336243";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String is infallible");
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
    serde_json::from_slice(VECTORS).expect("assembler vectors are JSON")
}

fn vector_case<'a>(cases: &'a Value, id: &str) -> &'a Value {
    cases
        .as_array()
        .expect("cases are an array")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("missing body-assembler vector {id}"))
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
                kind: match row["kind"].as_u64().expect("kind is unsigned") {
                    0 => OamdRowKind::Lfe,
                    2 => OamdRowKind::Object,
                    other => panic!("unsupported selected row kind {other}"),
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

fn metadata_from_json(value: &Value) -> OamdMetadata {
    OamdMetadata {
        rows: rows_from_json(&value["rows"]),
        timing: timing_from_json(&value["timing"]),
        trim: trim_from_json(&value["trim"]),
    }
}

fn section_descriptors_from_json(value: &Value) -> [OamdSectionDescriptor; 2] {
    let sections = value.as_array().expect("section descriptors are an array");
    assert_eq!(sections.len(), 2);
    std::array::from_fn(|index| {
        let section = &sections[index];
        OamdSectionDescriptor {
            kind: u8::try_from(section["kind"].as_u64().unwrap()).unwrap(),
            payload_bytes: usize::try_from(section["payload_bytes"].as_u64().unwrap()).unwrap(),
            payload_valid_bits: u32::try_from(section["payload_valid_bits"].as_u64().unwrap())
                .unwrap(),
            escaped_value: u32::try_from(section["escaped_value"].as_u64().unwrap()).unwrap(),
        }
    })
}

fn grid_tables() -> ([u32; 63], [u32; 16]) {
    assert_eq!(sha256_hex(COORDINATE_GRID), COORDINATE_GRID_SHA256);
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

fn append_bits(bits: &mut Vec<bool>, value: u32, width: u8) {
    for shift in (0..width).rev() {
        bits.push((value >> shift) & 1 != 0);
    }
}

fn pack_bits(bits: &[bool]) -> Vec<u8> {
    let mut bytes = vec![0_u8; bits.len().div_ceil(8)];
    for (position, bit) in bits.iter().copied().enumerate() {
        if bit {
            bytes[position / 8] |= 1 << (7 - position % 8);
        }
    }
    bytes
}

fn bit_at(bytes: &[u8], position: usize) -> bool {
    (bytes[position / 8] >> (7 - position % 8)) & 1 != 0
}

fn set_bit(bytes: &mut [u8], position: usize, value: bool) {
    let mask = 1_u8 << (7 - position % 8);
    if value {
        bytes[position / 8] |= mask;
    } else {
        bytes[position / 8] &= !mask;
    }
}

fn set_bits(bytes: &mut [u8], position: usize, width: u8, value: u32) {
    for offset in 0..width {
        let shift = width - offset - 1;
        set_bit(
            bytes,
            position + usize::from(offset),
            (value >> shift) & 1 != 0,
        );
    }
}

fn escaped_group_count(value: u32) -> usize {
    let mut current = value;
    let mut groups = 1;
    while current >= 16 {
        current = current / 16 - 1;
        groups += 1;
    }
    groups
}

fn error_name(error: OamdBodyAssemblyError) -> &'static str {
    use OamdBodyAssemblyError as E;
    match error {
        E::RowCount => "ROW_COUNT",
        E::LeadingLfe => "LEADING_LFE",
        E::RowDomain => "ROW_DOMAIN",
        E::RowXyz => "ROW_XYZ",
        E::RowGain => "ROW_GAIN",
        E::RowSpread => "ROW_SPREAD",
        E::RowExtension => "ROW_EXTENSION",
        E::TimingBase => "TIMING_BASE",
        E::TimingDomain => "TIMING_DOMAIN",
        E::TimingDuration => "TIMING_DURATION",
        E::TimingReserved => "TIMING_RESERVED",
        E::TimingMultiframe => "TIMING_MULTIFRAME",
        E::TrimShape => "TRIM_SHAPE",
        E::TrimDomain => "TRIM_DOMAIN",
        E::TrimU4 => "TRIM_U4",
        E::TrimSignedBalance => "TRIM_SIGNED_BALANCE",
        E::SectionCount => "SECTION_COUNT",
        E::SectionKind => "SECTION_KIND",
        E::DuplicateSection => "DUPLICATE_SECTION",
        E::SectionMissing => "SECTION_MISSING",
        E::SectionControl => "SECTION_CONTROL",
        E::Truncated => "TRUNCATED",
        E::SectionOverflow => "SECTION_OVERFLOW",
        E::EscapedOverflow => "ESCAPED_OVERFLOW",
        E::EscapedUnterminated => "ESCAPED_UNTERMINATED",
        E::NonzeroPadding => "NONZERO_PADDING",
        E::UnconsumedFields => "UNCONSUMED_FIELDS",
        E::BodyCapacity => "BODY_CAPACITY",
    }
}

#[test]
fn washed_assembler_assets_match_the_declared_identity() {
    for (bytes, expected_bytes, expected_sha256) in [
        (CONTRACT, CONTRACT_BYTES, CONTRACT_SHA256),
        (MACHINE_MAP, MACHINE_MAP_BYTES, MACHINE_MAP_SHA256),
        (VECTORS, VECTORS_BYTES, VECTORS_SHA256),
        (
            INDEPENDENT_CHECK,
            INDEPENDENT_CHECK_BYTES,
            INDEPENDENT_CHECK_SHA256,
        ),
    ] {
        assert_eq!(bytes.len(), expected_bytes);
        assert_eq!(sha256_hex(bytes), expected_sha256);
    }

    let machine: Value = serde_json::from_slice(MACHINE_MAP).unwrap();
    assert_eq!(machine["identity"], "single-frame-oamd-body-assembler.v1");
    assert_eq!(machine["stateful"], false);
    assert_eq!(machine["section_count"], 2);
    assert_eq!(machine["capacity_bytes"], 160);
    assert_eq!(
        machine["section_order"],
        json!(["kind1 rowdata", "kind2 trim"])
    );

    let vector_file = vectors();
    assert_eq!(vector_file["schema"], "oamd-body-assembler-vectors.v1");
    assert_eq!(vector_file["input_only"], true);
    assert_eq!(vector_file["expected_used_as_seed"], false);
    assert_eq!(vector_file["success_cases"].as_array().unwrap().len(), 4);
    assert_eq!(vector_file["rejection_cases"].as_array().unwrap().len(), 9);

    let independent: Value = serde_json::from_slice(INDEPENDENT_CHECK).unwrap();
    assert_eq!(independent["all_pass"], true);
    assert_eq!(independent["expected_used_as_seed"], false);
    assert_eq!(independent["checks"].as_array().unwrap().len(), 4);
    assert!(
        independent["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|check| {
                check["bytes_match"] == true
                    && check["row_count_match"] == true
                    && check["leading_lfe_match"] == true
                    && check["timing_match"] == true
                    && check["trim_match"] == true
            })
    );
}

#[test]
fn assembled_bodies_match_all_four_success_vectors_and_decode_projections() {
    let vector_file = vectors();
    let xy_table = grid_tables().0;
    let z_table = grid_tables().1;
    let cases = vector_file["success_cases"].as_array().unwrap();

    for case in cases {
        let input = metadata_from_json(&case["input"]);
        let expected = &case["expected"];
        let assembled = assemble_body(&input)
            .unwrap_or_else(|error| panic!("{} should assemble, got {error:?}", case["id"]));
        assert_eq!(
            assembled.bytes,
            decode_hex(expected["body_hex"].as_str().unwrap()),
            "{}",
            case["id"]
        );
        assert_eq!(
            assembled.bytes.len(),
            expected["bytes"].as_u64().unwrap() as usize
        );
        assert_eq!(
            assembled.valid_bits,
            expected["valid_bits"].as_u64().unwrap() as u32
        );
        assert_eq!(
            assembled.padding_bits,
            expected["padding_bits"].as_u64().unwrap() as u8
        );
        assert_eq!(
            assembled.row_count,
            expected["row_count"].as_u64().unwrap() as usize
        );
        assert_eq!(
            assembled.leading_lfe,
            expected["leading_lfe"].as_bool().unwrap()
        );
        assert_eq!(
            assembled.section_count,
            expected["section_count"].as_u64().unwrap() as u8
        );
        assert_eq!(
            assembled.sections,
            section_descriptors_from_json(&expected["sections"]),
            "{}",
            case["id"]
        );

        let decoded = decode_body(&assembled.bytes, &xy_table, &z_table)
            .unwrap_or_else(|error| panic!("{} should decode, got {error:?}", case["id"]));
        let projection = &expected["decode_projection"];
        assert_eq!(
            decoded.row_count,
            projection["row_count"].as_u64().unwrap() as usize
        );
        assert_eq!(
            decoded.leading_lfe,
            projection["leading_lfe"].as_bool().unwrap()
        );
        assert_eq!(
            decoded.section_count,
            projection["section_count"].as_u64().unwrap() as u8
        );
        assert_eq!(
            decoded.metadata.timing,
            timing_from_json(&projection["timing"])
        );
        assert_eq!(decoded.metadata.rows, rows_from_json(&projection["rows"]));
        assert_eq!(decoded.metadata.trim, trim_from_json(&projection["trim"]));
        assert_eq!(
            decoded.sections,
            section_descriptors_from_json(&expected["sections"]),
            "{}",
            case["id"]
        );
    }
}

#[test]
fn assembler_rejection_cases_match_the_declared_error_categories() {
    let vector_file = vectors();
    let rejection_cases = vector_file["rejection_cases"].as_array().unwrap();
    assert_eq!(rejection_cases.len(), 9);

    for case in rejection_cases {
        let id = case["id"].as_str().unwrap();
        let result = match id {
            "row_count_zero" => {
                let mut input = metadata_from_json(
                    &vector_case(&vector_file["success_cases"], "normal_multiline_trim0")["input"],
                );
                input.rows.clear();
                assemble_body(&input).map(|_| ())
            }
            "leading_lfe_not_first" => {
                let mut input = metadata_from_json(
                    &vector_case(&vector_file["success_cases"], "escaped_one_group_edge")["input"],
                );
                input.rows[1].kind = OamdRowKind::Lfe;
                assemble_body(&input).map(|_| ())
            }
            "timing_position_64" => {
                let mut input = metadata_from_json(
                    &vector_case(&vector_file["success_cases"], "normal_multiline_trim0")["input"],
                );
                input.timing.position = 64;
                assemble_body(&input).map(|_| ())
            }
            "timing_duration_2049" => {
                let mut input = metadata_from_json(
                    &vector_case(&vector_file["success_cases"], "normal_multiline_trim0")["input"],
                );
                input.timing.duration = 2049;
                assemble_body(&input).map(|_| ())
            }
            "invalid_trim_signed_balance_17" => {
                let mut input = metadata_from_json(
                    &vector_case(&vector_file["success_cases"], "trim2_sentinel_mask")["input"],
                );
                input.trim.groups[1][3] = 17;
                assemble_body(&input).map(|_| ())
            }
            "body_capacity" => {
                let mut input = metadata_from_json(
                    &vector_case(&vector_file["success_cases"], "escaped_one_group_edge")["input"],
                );
                let object = input.rows[0];
                input.rows.resize(159, object);
                input.trim.row_flags.resize(159, 0);
                assemble_body(&input).map(|_| ())
            }
            "truncated_body" => {
                let input = metadata_from_json(
                    &vector_case(&vector_file["success_cases"], "normal_multiline_trim0")["input"],
                );
                let mut bytes = assemble_body(&input).unwrap().bytes;
                bytes.pop();
                let (xy_table, z_table) = grid_tables();
                decode_body(&bytes, &xy_table, &z_table).map(|_| ())
            }
            "truncated_escaped_length" => {
                let input = metadata_from_json(
                    &vector_case(&vector_file["success_cases"], "normal_multiline_trim0")["input"],
                );
                let bytes = assemble_body(&input).unwrap().bytes;
                let (xy_table, z_table) = grid_tables();
                decode_body(&bytes[..3], &xy_table, &z_table).map(|_| ())
            }
            "overflow_escaped_length" => {
                let input = metadata_from_json(
                    &vector_case(&vector_file["success_cases"], "normal_multiline_trim0")["input"],
                );
                let bytes = assemble_body(&input).unwrap().bytes;
                let mut bits = (0..18)
                    .map(|position| bit_at(&bytes, position))
                    .collect::<Vec<_>>();
                for _ in 0..4 {
                    append_bits(&mut bits, 0, 4);
                    append_bits(&mut bits, 1, 1);
                }
                let escaped = pack_bits(&bits);
                let (xy_table, z_table) = grid_tables();
                decode_body(&escaped, &xy_table, &z_table).map(|_| ())
            }
            other => panic!("unhandled assembler rejection vector {other}"),
        };

        let error = result
            .err()
            .unwrap_or_else(|| panic!("{id} unexpectedly succeeded"));
        assert_eq!(
            error_name(error),
            case["expected_error"].as_str().unwrap(),
            "{id}"
        );
    }
}

#[test]
fn decoder_rejects_section_order_count_duplicates_controls_and_nonzero_padding() {
    let vector_file = vectors();
    let (xy_table, z_table) = grid_tables();
    let trim_case = vector_case(&vector_file["success_cases"], "trim2_sentinel_mask");
    let trim_input = metadata_from_json(&trim_case["input"]);
    let trim_body = assemble_body(&trim_input).unwrap().bytes;
    let trim_sections = &trim_case["expected"]["sections"];
    let rowdata_payload_bytes =
        usize::try_from(trim_sections[0]["payload_bytes"].as_u64().unwrap()).unwrap();
    let rowdata_escaped =
        u32::try_from(trim_sections[0]["escaped_value"].as_u64().unwrap()).unwrap();
    let trim_payload_bytes =
        usize::try_from(trim_sections[1]["payload_bytes"].as_u64().unwrap()).unwrap();
    let trim_escaped = u32::try_from(trim_sections[1]["escaped_value"].as_u64().unwrap()).unwrap();
    let first_payload_start = 14 + 4 + escaped_group_count(rowdata_escaped) * 5;
    let second_section_start = first_payload_start + rowdata_payload_bytes * 8;
    let second_payload_start = second_section_start + 4 + escaped_group_count(trim_escaped) * 5;

    let mut wrong_count = trim_body.clone();
    set_bits(&mut wrong_count, 10, 4, 1);
    assert_eq!(
        error_name(decode_body(&wrong_count, &xy_table, &z_table).unwrap_err()),
        "SECTION_COUNT"
    );

    let mut reordered = trim_body.clone();
    set_bits(&mut reordered, 14, 4, 2);
    assert_eq!(
        error_name(decode_body(&reordered, &xy_table, &z_table).unwrap_err()),
        "SECTION_KIND"
    );

    let mut duplicate = trim_body.clone();
    set_bits(&mut duplicate, second_section_start, 4, 1);
    assert_eq!(
        error_name(decode_body(&duplicate, &xy_table, &z_table).unwrap_err()),
        "DUPLICATE_SECTION"
    );

    let mut missing_trim = trim_body.clone();
    missing_trim.truncate(second_section_start.div_ceil(8));
    assert_eq!(
        error_name(decode_body(&missing_trim, &xy_table, &z_table).unwrap_err()),
        "SECTION_MISSING"
    );

    let mut nonzero_control = trim_body.clone();
    set_bit(&mut nonzero_control, first_payload_start, true);
    assert_eq!(
        error_name(decode_body(&nonzero_control, &xy_table, &z_table).unwrap_err()),
        "SECTION_CONTROL"
    );

    let mut nonzero_trim_padding = trim_body;
    set_bit(
        &mut nonzero_trim_padding,
        second_payload_start + trim_payload_bytes * 8 - 1,
        true,
    );
    assert_eq!(
        error_name(decode_body(&nonzero_trim_padding, &xy_table, &z_table).unwrap_err()),
        "NONZERO_PADDING"
    );

    let two_group_case = vector_case(&vector_file["success_cases"], "escaped_two_group_edge");
    let two_group_body = assemble_body(&metadata_from_json(&two_group_case["input"]))
        .unwrap()
        .bytes;
    let mut nonzero_outer_padding = two_group_body;
    let last_bit = nonzero_outer_padding.len() * 8 - 1;
    set_bit(&mut nonzero_outer_padding, last_bit, true);
    assert_eq!(
        error_name(decode_body(&nonzero_outer_padding, &xy_table, &z_table).unwrap_err()),
        "NONZERO_PADDING"
    );
}
