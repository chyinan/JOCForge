// pattern: Functional Core
//! Wash-pinned input-only vectors for the target-scoped chanmap field writer.

use jocforge_domain::{
    codec::{
        bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
        eac3_channel_map::write_flat7x_d0_chanmap_field,
    },
    profile::{DependentChannelMap, Flat7xI0D0TargetPlan},
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/JOCFORGE-EAC3-CHANMAP-U16-SERIALIZATION-V1/INPUT-ONLY-VECTORS.json"
);

fn written_field_bits(bytes: &[u8]) -> String {
    (0..16)
        .map(|offset| {
            if bytes[offset / 8] & (1_u8 << (7 - offset % 8)) != 0 {
                '1'
            } else {
                '0'
            }
        })
        .collect()
}

fn write_field(mask: u16, capacity_bytes: usize) -> (Vec<u8>, FrameBitCursor) {
    let mut bytes = vec![0; capacity_bytes];
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        cursor = write_flat7x_d0_chanmap_field(
            &mut writer,
            DependentChannelMap::from_numeric_value(mask),
        )
        .unwrap();
    }
    (bytes, cursor)
}

#[test]
fn washed_one_hot_vectors_match_the_target_scoped_field_writer() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(
        vectors["schema"],
        "jocforge.eac3.chanmap-u16-msb-first-vectors.v1"
    );
    assert_eq!(vectors["field_width_bits"], 16);
    assert_eq!(vectors["field_emission_order"], "MSB_FIRST");
    assert_eq!(vectors["one_hot_vectors"].as_array().unwrap().len(), 16);

    for vector in vectors["one_hot_vectors"].as_array().unwrap() {
        let numeric_bit_index = vector["numeric_bit_index"].as_u64().unwrap();
        let mask_hex = vector["mask_hex"].as_str().unwrap();
        let mask = u16::from_str_radix(mask_hex.trim_start_matches("0x"), 16).unwrap();
        let (bytes, cursor) = write_field(mask, 2);

        assert_eq!(cursor.bit_offset(), 16);
        assert_eq!(
            15 - numeric_bit_index,
            vector["expected_emitted_slot"].as_u64().unwrap()
        );
        assert_eq!(written_field_bits(&bytes), vector["expected_field_bits"]);
    }
}

#[test]
fn flat7x_target_plan_chanmap_matches_the_washed_composite_vector() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    let target = Flat7xI0D0TargetPlan::approved_contract();
    let channel_map = target.d0().channel_map().unwrap();
    let composite = &vectors["target_composite"];
    let expected_mask = u16::from_str_radix(
        composite["mask_hex"]
            .as_str()
            .unwrap()
            .trim_start_matches("0x"),
        16,
    )
    .unwrap();

    assert_eq!(channel_map.value(), expected_mask);
    assert_eq!(channel_map.value(), 0x1a00);
    assert_eq!(target.production_allowed(), false);

    let (bytes, cursor) = write_field(channel_map.value(), 2);
    assert_eq!(cursor.bit_offset(), 16);
    assert_eq!(written_field_bits(&bytes), composite["expected_field_bits"]);
}

#[test]
fn insufficient_capacity_preserves_the_caller_cursor_and_bytes() {
    let mut bytes = [0xa5_u8];
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        error = write_flat7x_d0_chanmap_field(
            &mut writer,
            DependentChannelMap::from_numeric_value(0x1a00),
        )
        .unwrap_err();
        cursor = writer.cursor();
    }

    assert_eq!(error, FrameBitWriterError::BufferTooSmall);
    assert_eq!(cursor.bit_offset(), 0);
    assert_eq!(bytes, [0xa5]);
}
