// pattern: Functional Core
//! Synthetic input-only checks for the bounded Flat7x I0 AC-3 BSI-tail writer.

use jocforge_domain::codec::{
    ac3_bsi_tail::{
        Flat7xI0Ac3AudioProductionInfo, Flat7xI0Ac3BsiTailError, Flat7xI0Ac3BsiTailInput,
        Flat7xI0OfficialDemoBsiTailError, I0Ac3TimeCodeField, write_flat7x_i0_ac3_bsi_tail,
        write_flat7x_i0_official_demo_bsi_tail,
    },
    bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    i0_frame_assembler::Flat7xI0Ac3FrameHeaderFields,
};
use serde_json::Value;

const VECTORS: &str = include_str!("fixtures/i0_bsi_tail/INPUT-ONLY-VECTORS.json");

fn vectors() -> Value {
    let value: Value = serde_json::from_str(VECTORS).unwrap();
    assert_eq!(
        value["schema"],
        "jocforge.flat7x.i0_ac3_bsi_tail.input_only.v1"
    );
    assert_eq!(
        value["classification"],
        "SYNTHETIC_NOT_REAL_OUTPUT_BSI_SYNTAX"
    );
    assert!(!value["contains_pcm_or_real_audio_bytes"].as_bool().unwrap());
    assert!(!value["production_allowed"].as_bool().unwrap());
    value
}

fn case_by_id(document: &Value, id: &str) -> Value {
    document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()
        .clone()
}

fn decode_hex(value: &str) -> Vec<u8> {
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

fn payload_for_case(case: &Value) -> Option<Vec<u8>> {
    let payload = &case["input"]["additional_bsi"];
    if payload.is_null() {
        return None;
    }
    if let Some(hex) = payload["hex"].as_str() {
        return Some(decode_hex(hex));
    }
    let fill = u8::from_str_radix(payload["fill_hex"].as_str().unwrap(), 16).unwrap();
    let length = usize::try_from(payload["length_bytes"].as_u64().unwrap()).unwrap();
    Some(vec![fill; length])
}

fn input_for_case<'a>(case: &Value, payload: Option<&'a [u8]>) -> Flat7xI0Ac3BsiTailInput<'a> {
    let input = &case["input"];
    let audio_production =
        input["audio_production"]
            .as_object()
            .map(|fields| Flat7xI0Ac3AudioProductionInfo {
                mix_level: u8::try_from(fields["mix_level"].as_u64().unwrap()).unwrap(),
                room_type: u8::try_from(fields["room_type"].as_u64().unwrap()).unwrap(),
            });
    Flat7xI0Ac3BsiTailInput {
        langcode: input["langcode"]
            .as_u64()
            .map(|value| u8::try_from(value).unwrap()),
        audio_production,
        copyright: input["copyright"].as_bool().unwrap(),
        original_bitstream: input["original_bitstream"].as_bool().unwrap(),
        time_code_1: input["time_code_1"]
            .as_u64()
            .map(|value| u16::try_from(value).unwrap()),
        time_code_2: input["time_code_2"]
            .as_u64()
            .map(|value| u16::try_from(value).unwrap()),
        additional_bsi: payload,
    }
}

fn read_bits(bytes: &[u8], start_bit: u32, width: u8) -> u32 {
    (0..u32::from(width)).fold(0_u32, |value, offset| {
        let bit = start_bit + offset;
        let byte = usize::try_from(bit / 8).unwrap();
        let shift = 7 - bit % 8;
        (value << 1) | u32::from(bytes[byte] & (1_u8 << shift) != 0)
    })
}

fn assert_outside_extent_unchanged(before: &[u8], after: &[u8], start: u32, end: u32) {
    for bit in 0..u32::try_from(before.len() * 8).unwrap() {
        if bit < start || bit >= end {
            assert_eq!(read_bits(before, bit, 1), read_bits(after, bit, 1));
        }
    }
}

#[test]
fn writes_the_mandatory_seven_tail_bits_at_the_header_boundary() {
    let case = case_by_id(&vectors(), "minimum-tail-without-optionals");
    let payload = payload_for_case(&case);
    let input = input_for_case(&case, payload.as_deref());
    let start = u32::try_from(case["header_end_bit"].as_u64().unwrap()).unwrap();
    let mut bytes = vec![0xa5; 32];
    let before = bytes.clone();
    let receipt;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        receipt = write_flat7x_i0_ac3_bsi_tail(&mut writer, input).unwrap();
        cursor = writer.cursor().bit_offset();
    }

    let expected = &case["expected"];
    assert_eq!(receipt.start_bit(), start);
    assert_eq!(receipt.bit_length(), 7);
    assert_eq!(receipt.end_bit(), expected["end_bit"]);
    assert_eq!(cursor, expected["end_bit"]);
    assert_eq!(read_bits(&bytes, start, 7), 0b001_0000);
    assert_eq!(receipt.additional_bsi_byte_length(), None);
    assert!(!receipt.production_allowed());
    assert_outside_extent_unchanged(&before, &bytes, start, receipt.end_bit());
}

#[test]
fn writes_optional_fields_and_reports_the_exact_unaligned_addbsi_extent() {
    let case = case_by_id(&vectors(), "all-optionals-with-two-byte-addbsi");
    let payload = payload_for_case(&case);
    let input = input_for_case(&case, payload.as_deref());
    let start = u32::try_from(case["header_end_bit"].as_u64().unwrap()).unwrap();
    let mut bytes = vec![0x5a; 32];
    let before = bytes.clone();
    let receipt;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        receipt = write_flat7x_i0_ac3_bsi_tail(&mut writer, input).unwrap();
        cursor = writer.cursor().bit_offset();
    }

    let expected = &case["expected"];
    let offsets = &expected["field_offsets"];
    assert_eq!(receipt.start_bit(), start);
    assert_eq!(receipt.bit_length(), expected["bit_length"]);
    assert_eq!(receipt.end_bit(), expected["end_bit"]);
    assert_eq!(cursor, expected["end_bit"]);
    assert_eq!(receipt.additional_bsi_start_bit(), Some(126));
    assert_eq!(receipt.additional_bsi_bit_length(), Some(16));
    assert_eq!(receipt.additional_bsi_byte_length(), Some(2));
    assert_eq!(
        read_bits(
            &bytes,
            offsets["langcode_present"].as_u64().unwrap() as u32,
            1
        ),
        1
    );
    assert_eq!(
        read_bits(&bytes, offsets["langcode"].as_u64().unwrap() as u32, 8),
        0xa5
    );
    assert_eq!(
        read_bits(&bytes, offsets["audprodie"].as_u64().unwrap() as u32, 1),
        1
    );
    assert_eq!(
        read_bits(&bytes, offsets["mix_level"].as_u64().unwrap() as u32, 5),
        17
    );
    assert_eq!(
        read_bits(&bytes, offsets["room_type"].as_u64().unwrap() as u32, 2),
        2
    );
    assert_eq!(
        read_bits(&bytes, offsets["copyright"].as_u64().unwrap() as u32, 1),
        1
    );
    assert_eq!(
        read_bits(
            &bytes,
            offsets["original_bitstream"].as_u64().unwrap() as u32,
            1
        ),
        1
    );
    assert_eq!(
        read_bits(
            &bytes,
            offsets["time_code_1_present"].as_u64().unwrap() as u32,
            1
        ),
        1
    );
    assert_eq!(
        read_bits(&bytes, offsets["time_code_1"].as_u64().unwrap() as u32, 14),
        0x1234
    );
    assert_eq!(
        read_bits(
            &bytes,
            offsets["time_code_2_present"].as_u64().unwrap() as u32,
            1
        ),
        1
    );
    assert_eq!(
        read_bits(&bytes, offsets["time_code_2"].as_u64().unwrap() as u32, 14),
        0x3fff
    );
    assert_eq!(
        read_bits(&bytes, offsets["addbsie"].as_u64().unwrap() as u32, 1),
        1
    );
    assert_eq!(
        read_bits(&bytes, offsets["addbsil"].as_u64().unwrap() as u32, 6),
        1
    );
    assert_eq!(
        read_bits(
            &bytes,
            offsets["additional_bsi"].as_u64().unwrap() as u32,
            16
        ),
        0xdead
    );
    assert_outside_extent_unchanged(&before, &bytes, start, receipt.end_bit());
}

#[test]
fn accepts_the_maximum_addbsi_length_represented_by_u6() {
    let case = case_by_id(&vectors(), "maximum-addbsi-u6-length");
    let payload = payload_for_case(&case);
    let input = input_for_case(&case, payload.as_deref());
    let start = u32::try_from(case["header_end_bit"].as_u64().unwrap()).unwrap();
    let mut bytes = vec![0_u8; 80];
    let receipt;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        receipt = write_flat7x_i0_ac3_bsi_tail(&mut writer, input).unwrap();
    }
    let expected = &case["expected"];
    assert_eq!(receipt.bit_length(), expected["bit_length"]);
    assert_eq!(receipt.end_bit(), expected["end_bit"]);
    assert_eq!(
        read_bits(
            &bytes,
            expected["addbsil_offset"].as_u64().unwrap() as u32,
            6
        ),
        expected["addbsil_value"].as_u64().unwrap() as u32
    );
    assert_eq!(receipt.additional_bsi_start_bit(), Some(75));
    assert_eq!(receipt.additional_bsi_bit_length(), Some(512));
    assert_eq!(receipt.additional_bsi_byte_length(), Some(64));
    assert_eq!(read_bits(&bytes, 75, 8), 0xa5);
    assert_eq!(read_bits(&bytes, 75 + 63 * 8, 8), 0xa5);
}

fn assert_rejected(
    input: Flat7xI0Ac3BsiTailInput<'_>,
    start: u32,
    buffer_bytes: usize,
    expected_error: Flat7xI0Ac3BsiTailError,
) {
    let mut bytes = vec![0x96; buffer_bytes];
    let before = bytes.clone();
    let result;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        result = write_flat7x_i0_ac3_bsi_tail(&mut writer, input);
        cursor = writer.cursor().bit_offset();
    }
    assert_eq!(result, Err(expected_error));
    assert_eq!(cursor, start);
    assert_eq!(bytes, before);
}

#[test]
fn rejects_invalid_fields_lengths_cursor_and_capacity_before_mutation() {
    let no_payload = Flat7xI0Ac3BsiTailInput {
        langcode: None,
        audio_production: None,
        copyright: false,
        original_bitstream: false,
        time_code_1: None,
        time_code_2: None,
        additional_bsi: None,
    };
    assert_rejected(
        Flat7xI0Ac3BsiTailInput {
            audio_production: Some(Flat7xI0Ac3AudioProductionInfo {
                mix_level: 32,
                room_type: 0,
            }),
            ..no_payload
        },
        62,
        80,
        Flat7xI0Ac3BsiTailError::MixLevelOutOfRange { value: 32 },
    );
    assert_rejected(
        Flat7xI0Ac3BsiTailInput {
            audio_production: Some(Flat7xI0Ac3AudioProductionInfo {
                mix_level: 0,
                room_type: 4,
            }),
            ..no_payload
        },
        62,
        80,
        Flat7xI0Ac3BsiTailError::RoomTypeOutOfRange { value: 4 },
    );
    assert_rejected(
        Flat7xI0Ac3BsiTailInput {
            time_code_1: Some(16_384),
            ..no_payload
        },
        62,
        80,
        Flat7xI0Ac3BsiTailError::TimeCodeOutOfRange {
            field: I0Ac3TimeCodeField::First,
            value: 16_384,
        },
    );
    assert_rejected(
        Flat7xI0Ac3BsiTailInput {
            additional_bsi: Some(&[]),
            ..no_payload
        },
        62,
        80,
        Flat7xI0Ac3BsiTailError::AdditionalBsiLengthOutOfRange { actual_bytes: 0 },
    );
    let oversized_payload = [0_u8; 65];
    assert_rejected(
        Flat7xI0Ac3BsiTailInput {
            additional_bsi: Some(&oversized_payload),
            ..no_payload
        },
        62,
        80,
        Flat7xI0Ac3BsiTailError::AdditionalBsiLengthOutOfRange { actual_bytes: 65 },
    );
    assert_rejected(
        no_payload,
        61,
        80,
        Flat7xI0Ac3BsiTailError::HeaderEndCursorMismatch { actual: 61 },
    );
    assert_rejected(
        no_payload,
        62,
        8,
        Flat7xI0Ac3BsiTailError::BitWriter(FrameBitWriterError::BufferTooSmall),
    );
}

#[test]
fn official_demo_header_fields_match_the_v2_bsi_receipt_without_sample_crc1() {
    let header = Flat7xI0Ac3FrameHeaderFields::official_demo_sample_non_crc_fields();
    assert_eq!(header.frmsizecod, 36);
    assert_eq!(header.bsmod, 0);
    assert_eq!(header.cmixlev, 0);
    assert_eq!(header.surmixlev, 0);
    assert_eq!(header.dialnorm, 31);
    assert_eq!(header.compr, Some(1));
}

#[test]
fn writes_official_demo_bsi_tail_through_absent_addbsie() {
    let mut bytes = [0_u8; 14];
    let before = bytes;
    let receipt;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(70)).unwrap();
        receipt = write_flat7x_i0_official_demo_bsi_tail(&mut writer).unwrap();
        cursor = writer.cursor().bit_offset();
    }

    assert_eq!(receipt.start_bit(), 70);
    assert_eq!(receipt.end_bit(), 105);
    assert_eq!(receipt.bit_length(), 35);
    assert_eq!(cursor, 105);
    assert!(!receipt.values().langcode_present());
    assert!(!receipt.values().audprodie_present());
    assert!(receipt.values().copyrightb());
    assert!(receipt.values().original_bitstream());
    assert!(receipt.values().xbsi1e());
    assert_eq!(receipt.values().dmixmod(), 1);
    assert_eq!(receipt.values().ltrtcmixlev(), 4);
    assert_eq!(receipt.values().ltrtsurmixlev(), 4);
    assert_eq!(receipt.values().lorocmixlev(), 4);
    assert_eq!(receipt.values().lorosurmixlev(), 4);
    assert!(receipt.values().xbsi2e());
    assert_eq!(receipt.values().dsurexmod(), 1);
    assert_eq!(receipt.values().dheadphonmod(), 0);
    assert!(!receipt.values().adconvtyp());
    assert_eq!(receipt.values().xbsi2(), 0);
    assert!(!receipt.values().encinfo());
    assert!(!receipt.values().addbsie_present());
    assert!(!receipt.production_allowed());
    for (start, width, value) in [
        (70, 1, 0),
        (71, 1, 0),
        (72, 1, 1),
        (73, 1, 1),
        (74, 1, 1),
        (75, 2, 1),
        (77, 3, 4),
        (80, 3, 4),
        (83, 3, 4),
        (86, 3, 4),
        (89, 1, 1),
        (90, 2, 1),
        (92, 2, 0),
        (94, 1, 0),
        (95, 8, 0),
        (103, 1, 0),
        (104, 1, 0),
    ] {
        assert_eq!(
            read_bits(&bytes, start, width),
            value,
            "frame bits {start}..{}",
            start + u32::from(width)
        );
    }
    assert_outside_extent_unchanged(&before, &bytes, 70, 105);
}

#[test]
fn official_demo_bsi_tail_rejects_wrong_cursor_and_short_capacity_before_mutation() {
    let mut wrong_cursor_bytes = [0x5a_u8; 12];
    let wrong_cursor_before = wrong_cursor_bytes;
    let wrong_cursor_result;
    let wrong_cursor_after;
    {
        let mut writer =
            FrameBitWriter::new(&mut wrong_cursor_bytes, FrameBitCursor::new(69)).unwrap();
        wrong_cursor_result = write_flat7x_i0_official_demo_bsi_tail(&mut writer);
        wrong_cursor_after = writer.cursor().bit_offset();
    }
    assert_eq!(
        wrong_cursor_result,
        Err(Flat7xI0OfficialDemoBsiTailError::StartCursorMismatch { actual: 69 })
    );
    assert_eq!(wrong_cursor_after, 69);
    assert_eq!(wrong_cursor_bytes, wrong_cursor_before);

    let mut short_bytes = [0x96_u8; 13];
    let short_before = short_bytes;
    let short_result;
    let short_cursor;
    {
        let mut writer = FrameBitWriter::new(&mut short_bytes, FrameBitCursor::new(70)).unwrap();
        short_result = write_flat7x_i0_official_demo_bsi_tail(&mut writer);
        short_cursor = writer.cursor().bit_offset();
    }
    assert!(matches!(
        short_result,
        Err(Flat7xI0OfficialDemoBsiTailError::BitWriter(
            FrameBitWriterError::BufferTooSmall
        ))
    ));
    assert_eq!(short_cursor, 70);
    assert_eq!(short_bytes, short_before);
}
