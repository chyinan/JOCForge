// pattern: Functional Core
//! Synthetic, non-real-output tests for Flat7x elementary-stream sequence composition.

use jocforge_domain::{
    codec::{
        bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
        elementary_stream_composer::{
            Flat7xElementaryStreamCompositionError, Flat7xOpaqueAccessUnitBytes,
            compose_flat7x_elementary_stream, measure_flat7x_elementary_stream,
        },
    },
    profile::Flat7xI0D0TargetPlan,
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &str =
    include_str!("fixtures/flat7x_elementary_stream/INPUT-ONLY-VECTORS.json");

fn vector<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap()
}

fn synthetic_access_unit(length: usize, fill: &Value) -> Vec<u8> {
    vec![u8::from_str_radix(fill.as_str().unwrap(), 16).unwrap(); length]
}

#[test]
fn composes_multiple_ordered_access_units_at_exact_extents() {
    let vectors: Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(vectors["status"], "SYNTHETIC_NOT_REAL_OUTPUT");
    assert!(!vectors["contains_audio_or_frame_bytes"].as_bool().unwrap());
    let case = vector(&vectors, "compose-two-ordered-access-units");
    let au_bytes = case["bytes_per_access_unit"].as_u64().unwrap() as usize;
    let fills = case["test_fill_hex_by_access_unit"].as_array().unwrap();
    let first = synthetic_access_unit(au_bytes, &fills[0]);
    let second = synthetic_access_unit(au_bytes, &fills[1]);
    let access_units = [
        Flat7xOpaqueAccessUnitBytes::new(&first),
        Flat7xOpaqueAccessUnitBytes::new(&second),
    ];
    let mut output = vec![
        0xa5;
        case["expected"]["elementary_stream_bytes"]
            .as_u64()
            .unwrap() as usize
    ];

    let receipt;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
        receipt = compose_flat7x_elementary_stream(
            Flat7xI0D0TargetPlan::approved_contract(),
            &access_units,
            &mut writer,
        )
        .unwrap();
        cursor = writer.cursor();
    }

    assert_eq!(receipt.start_byte(), 0);
    assert_eq!(receipt.extent().access_unit_count(), 2);
    assert_eq!(
        receipt.extent().byte_length(),
        case["expected"]["elementary_stream_bytes"]
            .as_u64()
            .unwrap() as u32
    );
    assert_eq!(
        receipt.extent().bit_length(),
        case["expected"]["elementary_stream_bits"].as_u64().unwrap() as u32
    );
    assert!(!receipt.production_allowed());
    assert_eq!(cursor.bit_offset(), receipt.extent().bit_length());
    assert!(output[..au_bytes].iter().all(|byte| *byte == 0x31));
    assert!(output[au_bytes..].iter().all(|byte| *byte == 0x72));
}

#[test]
fn rejects_empty_sequence_and_bad_au_extent_without_mutation() {
    let vectors: Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
    let empty_case = vector(&vectors, "reject-empty-sequence");
    assert_eq!(empty_case["access_unit_count"].as_u64().unwrap(), 0);
    let valid = vector(&vectors, "compose-two-ordered-access-units");
    let au_bytes = valid["bytes_per_access_unit"].as_u64().unwrap() as usize;
    let fills = valid["test_fill_hex_by_access_unit"].as_array().unwrap();
    let first = synthetic_access_unit(au_bytes, &fills[0]);
    let second = synthetic_access_unit(au_bytes, &fills[1]);
    let mut output = vec![0xa5; au_bytes * 2];
    let original = output.clone();
    let result;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
        result = compose_flat7x_elementary_stream(
            Flat7xI0D0TargetPlan::approved_contract(),
            &[],
            &mut writer,
        );
        cursor = writer.cursor();
    }
    assert_eq!(
        result.unwrap_err(),
        Flat7xElementaryStreamCompositionError::EmptyAccessUnitSequence
    );
    assert_eq!(cursor.bit_offset(), 0);
    assert_eq!(output, original);

    let wrong_case = vector(&vectors, "reject-wrong-access-unit-extent");
    let short_length = wrong_case["actual_access_unit_bytes"].as_u64().unwrap() as usize;
    let short_second = &second[..short_length];
    let access_units = [
        Flat7xOpaqueAccessUnitBytes::new(&first),
        Flat7xOpaqueAccessUnitBytes::new(short_second),
    ];
    let mut output = vec![0xa5; au_bytes * 2];
    let original = output.clone();
    let result;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
        result = compose_flat7x_elementary_stream(
            Flat7xI0D0TargetPlan::approved_contract(),
            &access_units,
            &mut writer,
        );
        cursor = writer.cursor();
    }
    assert_eq!(
        result.unwrap_err(),
        Flat7xElementaryStreamCompositionError::AccessUnitExtentMismatch {
            au_index: wrong_case["wrong_access_unit_index"].as_u64().unwrap() as u32,
            expected: au_bytes as u32,
            actual: short_length,
        }
    );
    assert_eq!(cursor.bit_offset(), 0);
    assert_eq!(output, original);
}

#[test]
fn rejects_size_overflow_and_short_capacity_without_mutation() {
    let vectors: Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
    let overflow = vector(&vectors, "reject-checked-size-overflow");
    assert_eq!(overflow["access_unit_count"], "usize::MAX");
    assert_eq!(
        measure_flat7x_elementary_stream(Flat7xI0D0TargetPlan::approved_contract(), usize::MAX,),
        Err(Flat7xElementaryStreamCompositionError::SequenceExtentOverflow)
    );

    let capacity = vector(&vectors, "reject-one-byte-short-writer-capacity");
    let au_bytes = 6_656;
    let first = vec![0x31; au_bytes];
    let second = vec![0x72; au_bytes];
    let access_units = [
        Flat7xOpaqueAccessUnitBytes::new(&first),
        Flat7xOpaqueAccessUnitBytes::new(&second),
    ];
    let output_bytes = capacity["writer_capacity_bytes"].as_u64().unwrap() as usize;
    let mut output = vec![0xa5; output_bytes];
    let original = output.clone();
    let result;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
        result = compose_flat7x_elementary_stream(
            Flat7xI0D0TargetPlan::approved_contract(),
            &access_units,
            &mut writer,
        );
        cursor = writer.cursor();
    }
    assert_eq!(
        result.unwrap_err(),
        Flat7xElementaryStreamCompositionError::BitWriter(FrameBitWriterError::BufferTooSmall)
    );
    assert_eq!(cursor.bit_offset(), 0);
    assert_eq!(output, original);
}

#[test]
fn rejects_non_byte_aligned_cursor_without_mutation() {
    let vectors: Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
    let case = vector(&vectors, "reject-non-byte-aligned-writer-cursor");
    let first = vec![0x31; 6_656];
    let access_units = [Flat7xOpaqueAccessUnitBytes::new(&first)];
    let mut output = vec![0xa5; 6_657];
    let original = output.clone();
    let result;
    let cursor;
    {
        let start_bit_offset = case["start_bit_offset"].as_u64().unwrap() as u32;
        let mut writer =
            FrameBitWriter::new(&mut output, FrameBitCursor::new(start_bit_offset)).unwrap();
        result = compose_flat7x_elementary_stream(
            Flat7xI0D0TargetPlan::approved_contract(),
            &access_units,
            &mut writer,
        );
        cursor = writer.cursor();
    }
    assert_eq!(
        result.unwrap_err(),
        Flat7xElementaryStreamCompositionError::WriterCursorNotByteAligned {
            actual_bit_offset: 1,
        }
    );
    assert_eq!(cursor.bit_offset(), 1);
    assert_eq!(output, original);
}
