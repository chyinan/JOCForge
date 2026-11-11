// pattern: Functional Core
//! Wash-pinned input-only vectors for the target-scoped chanmap field writer.

use jocforge_domain::{
    codec::{
        bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
        eac3_channel_map::{write_flat7x_d0_chanmap_field, write_flat7x_d0_chanmap_fields},
    },
    profile::{DependentChannelMap, Flat7xI0D0TargetPlan},
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/JOCFORGE-EAC3-CHANMAP-U16-SERIALIZATION-V1/INPUT-ONLY-VECTORS.json"
);
const DEPENDENT_FIELD_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/EAC3-DEPENDENT-CHANMAP-FIELD-WRITER-V1/INPUT-ONLY-VECTORS.json"
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

fn written_bits_at(bytes: &[u8], start_bit: u32, width: u32) -> String {
    (0..width)
        .map(|offset| {
            let bit = start_bit + offset;
            if bytes[(bit / 8) as usize] & (1_u8 << (7 - bit % 8)) != 0 {
                '1'
            } else {
                '0'
            }
        })
        .collect()
}

#[test]
fn dependent_chanmape_and_chanmap_writer_matches_wash_vectors() {
    let vectors: Value = serde_json::from_slice(DEPENDENT_FIELD_VECTORS).unwrap();
    assert_eq!(
        vectors["schema"],
        "eac3-dependent-chanmap-field-writer-input-only.v1"
    );

    let one_hot = vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|vector| vector["id"] == "one-hot-map-bits")
        .unwrap();
    for case in one_hot["cases"].as_array().unwrap() {
        let bit_index = case["bit_index"].as_u64().unwrap();
        let mask_hex = case["mask"].as_str().unwrap();
        let mask = u16::from_str_radix(mask_hex.trim_start_matches("0x"), 16).unwrap();
        assert_eq!(mask, 1_u16 << (15 - bit_index));

        let mut bytes = [0_u8; 3];
        let end;
        {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
            end = write_flat7x_d0_chanmap_fields(
                &mut writer,
                DependentChannelMap::from_numeric_value(mask),
            )
            .unwrap();
        }
        assert_eq!(end.bit_offset(), 17);
        assert_eq!(written_bits_at(&bytes, 0, 17), format!("1{mask:016b}"));
    }

    let target_vector = vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|vector| vector["id"] == "flat7x-target-1a00")
        .unwrap();
    assert_eq!(target_vector["chanmape"], 1);
    let target_map = Flat7xI0D0TargetPlan::approved_contract()
        .d0()
        .channel_map()
        .unwrap();
    assert_eq!(target_map.value(), 0x1a00);
    let mut target_bytes = [0_u8; 3];
    let target_end;
    {
        let mut writer = FrameBitWriter::new(&mut target_bytes, FrameBitCursor::new(0)).unwrap();
        target_end = write_flat7x_d0_chanmap_fields(&mut writer, target_map).unwrap();
    }
    assert_eq!(target_end.bit_offset(), target_vector["expected_end_bit"]);
    assert_eq!(
        written_bits_at(&target_bytes, 0, 17),
        target_vector["field_bits"]
    );
}

#[test]
fn dependent_chanmap_writer_handles_misalignment_and_preflights_all_17_bits() {
    let vectors: Value = serde_json::from_slice(DEPENDENT_FIELD_VECTORS).unwrap();
    let vector_list = vectors["vectors"].as_array().unwrap();
    let misaligned = vector_list
        .iter()
        .find(|vector| vector["id"] == "target-map-misaligned-start")
        .unwrap();
    let start_bit = misaligned["start_bit"].as_u64().unwrap() as u32;
    let target_bits = vector_list
        .iter()
        .find(|vector| vector["id"] == "flat7x-target-1a00")
        .unwrap()["field_bits"]
        .as_str()
        .unwrap();
    let initial = [0xa5_u8, 0x5a, 0xc3];
    let mut bytes = initial;
    let end;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start_bit)).unwrap();
        end = write_flat7x_d0_chanmap_fields(
            &mut writer,
            Flat7xI0D0TargetPlan::approved_contract()
                .d0()
                .channel_map()
                .unwrap(),
        )
        .unwrap();
    }
    assert_eq!(end.bit_offset(), misaligned["expected_end_bit"]);
    assert_eq!(written_bits_at(&bytes, start_bit, 17), target_bits);
    for bit in 0..24_u32 {
        if !(start_bit..end.bit_offset()).contains(&bit) {
            let mask = 1_u8 << (7 - bit % 8);
            assert_eq!(
                bytes[(bit / 8) as usize] & mask,
                initial[(bit / 8) as usize] & mask
            );
        }
    }

    let capacity = vector_list
        .iter()
        .find(|vector| vector["id"] == "capacity-insufficient-by-one-bit")
        .unwrap();
    let mut short_bytes = [0x5a_u8; 2];
    let original_bytes = short_bytes;
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(
            &mut short_bytes,
            FrameBitCursor::new(capacity["start_bit"].as_u64().unwrap() as u32),
        )
        .unwrap();
        error = write_flat7x_d0_chanmap_fields(
            &mut writer,
            Flat7xI0D0TargetPlan::approved_contract()
                .d0()
                .channel_map()
                .unwrap(),
        )
        .unwrap_err();
        cursor = writer.cursor();
    }
    assert_eq!(error, FrameBitWriterError::BufferTooSmall);
    assert_eq!(cursor.bit_offset(), 0);
    assert_eq!(short_bytes, original_bytes);
}
