// pattern: Functional Core
//! Wash-pinned input-only vectors for the limited D0 BSI segment serializer.

use jocforge_domain::{
    codec::{
        bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
        frame_builder::{Flat7xD0BsiSegmentError, write_flat7x_d0_bsi_through_chanmap},
    },
    profile::Flat7xI0D0TargetPlan,
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/EAC3-DEPENDENT-D0-BSI-SEGMENT-V1/INPUT-ONLY-VECTORS.json"
);

fn bytes_from_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(pair, 16).unwrap()
        })
        .collect()
}

fn written_bits(bytes: &[u8], bit_count: u32) -> String {
    (0..bit_count)
        .map(|bit| {
            if bytes[(bit / 8) as usize] & (1_u8 << (7 - bit % 8)) != 0 {
                '1'
            } else {
                '0'
            }
        })
        .collect()
}

fn vector<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap()
}

fn assert_valid_segment(vector: &Value) {
    let inputs = &vector["inputs"];
    let dialnorm = inputs["dialnorm"].as_u64().unwrap() as u8;
    let compr = inputs["compr"].as_u64().map(|value| value as u8);
    let target = Flat7xI0D0TargetPlan::approved_contract();
    assert_eq!(
        target.d0().channel_map().unwrap().value(),
        u16::from_str_radix(
            inputs["chanmap"].as_str().unwrap().trim_start_matches("0x"),
            16
        )
        .unwrap()
    );

    let expected_bits = vector["expected_bit_string"].as_str().unwrap();
    let expected_end = vector["expected_end_bit_offset"].as_u64().unwrap() as u32;
    let expected_bytes = bytes_from_hex(
        vector["expected_bytes_hex_zero_initialized"]
            .as_str()
            .unwrap(),
    );
    let mut bytes = vec![0_u8; expected_bytes.len()];
    let end;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        end = write_flat7x_d0_bsi_through_chanmap(&mut writer, dialnorm, compr).unwrap();
    }

    assert_eq!(end.bit_offset(), expected_end);
    assert_eq!(written_bits(&bytes, end.bit_offset()), expected_bits);
    assert_eq!(bytes, expected_bytes);
}

#[test]
fn no_compr_and_with_compr_segments_match_wash_vectors() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(
        vectors["schema"],
        "eac3-dependent-d0-bsi-segment-input-only.v1"
    );
    assert_valid_segment(vector(&vectors, "d0-bsi-through-chanmap-no-compr"));
    assert_valid_segment(vector(&vectors, "d0-bsi-through-chanmap-with-compr"));
}

#[test]
fn full_segment_capacity_preflight_leaves_bytes_and_cursor_unchanged() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    let capacity = vector(&vectors, "full-segment-capacity-preflight");
    let buffer_bytes = capacity["buffer_bytes"].as_u64().unwrap() as usize;
    let start = 0;
    let mut bytes = vec![0xa5_u8; buffer_bytes];
    let initial = bytes.clone();
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        error = write_flat7x_d0_bsi_through_chanmap(&mut writer, 16, None).unwrap_err();
        cursor = writer.cursor();
    }

    assert_eq!(
        error,
        Flat7xD0BsiSegmentError::BitWriter(FrameBitWriterError::BufferTooSmall)
    );
    assert_eq!(cursor.bit_offset(), start);
    assert_eq!(bytes, initial);
}

#[test]
fn reserved_and_out_of_range_dialnorm_values_reject_before_mutation() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    let invalid = vector(&vectors, "invalid-dialnorm");
    let buffer_bytes = 10;

    for value in invalid["dialnorm_values"].as_array().unwrap() {
        let dialnorm = value.as_u64().unwrap() as u8;
        let mut bytes = vec![0x5a_u8; buffer_bytes];
        let initial = bytes.clone();
        let error;
        let cursor;
        {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
            error = write_flat7x_d0_bsi_through_chanmap(&mut writer, dialnorm, None).unwrap_err();
            cursor = writer.cursor();
        }

        assert_eq!(
            error,
            Flat7xD0BsiSegmentError::DialnormOutOfRange { value: dialnorm }
        );
        assert_eq!(cursor.bit_offset(), 0);
        assert_eq!(bytes, initial);
    }
}
