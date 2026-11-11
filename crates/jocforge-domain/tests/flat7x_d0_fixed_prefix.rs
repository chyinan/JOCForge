// pattern: Functional Core
//! Wash-pinned input-only vectors for the limited Flat7x D0 prefix writer.

use jocforge_domain::codec::{
    bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    frame_builder::{Flat7xD0FixedPrefixError, write_flat7x_d0_fixed_prefix},
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/EAC3-DEPENDENT-D0-FIXED-PREFIX-V1/INPUT-ONLY-VECTORS.json"
);

fn written_bits(bytes: &[u8], width: u32) -> String {
    (0..width)
        .map(|offset| {
            if bytes[(offset / 8) as usize] & (1_u8 << (7 - offset % 8)) != 0 {
                '1'
            } else {
                '0'
            }
        })
        .collect()
}

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

fn prefix_vector(vectors: &Value) -> &Value {
    vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|vector| vector["id"] == "flat7x-d0-fixed-prefix")
        .unwrap()
}

#[test]
fn flat7x_d0_fixed_prefix_matches_the_wash_bit_and_byte_vector() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(
        vectors["schema"],
        "eac3-dependent-d0-fixed-prefix-input-only.v1"
    );
    let prefix = prefix_vector(&vectors);
    let mut bytes = vec![0_u8; 6];
    let end;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        end = write_flat7x_d0_fixed_prefix(&mut writer).unwrap();
    }

    assert_eq!(end.bit_offset(), prefix["expected_end_bit_offset"]);
    assert_eq!(
        written_bits(&bytes, end.bit_offset()),
        prefix["expected_bit_string"]
    );
    assert_eq!(
        bytes,
        bytes_from_hex(
            prefix["expected_bytes_hex_zero_initialized"]
                .as_str()
                .unwrap()
        )
    );
    assert_eq!(end.bit_offset(), 45);
}

#[test]
fn flat7x_d0_prefix_preflights_capacity_before_mutation() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    let capacity = vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|vector| {
            vector["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("capacity-preflight-"))
        })
        .unwrap();
    let start_bit = capacity["start_bit_offset"].as_u64().unwrap() as u32;
    let buffer_bytes = capacity["buffer_bytes"].as_u64().unwrap() as usize;
    assert_eq!(start_bit, 0);
    assert!(u64::try_from(buffer_bytes).unwrap() * 8 < 45);

    let original = vec![0xa5_u8; buffer_bytes];
    let mut bytes = original.clone();
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start_bit)).unwrap();
        error = write_flat7x_d0_fixed_prefix(&mut writer).unwrap_err();
        cursor = writer.cursor();
    }

    assert_eq!(
        error,
        Flat7xD0FixedPrefixError::BitWriter(FrameBitWriterError::BufferTooSmall)
    );
    assert_eq!(cursor.bit_offset(), start_bit);
    assert_eq!(bytes, original);
}
