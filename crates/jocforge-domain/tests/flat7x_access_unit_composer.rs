// pattern: Functional Core
//! Synthetic, non-real-output tests for opaque Flat7x I0→D0 AU byte ordering.

use jocforge_domain::{
    codec::{
        access_unit_composer::{
            Flat7xI0D0AuCompositionError, Flat7xI0D0OpaqueFrameSpans, OpaqueD0FrameBytes,
            OpaqueI0FrameBytes, write_flat7x_i0_d0_access_unit_bytes,
        },
        bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    },
    profile::Flat7xI0D0TargetPlan,
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &str =
    include_str!("fixtures/flat7x_au_composer/INPUT-ONLY-VECTORS.json");

fn vector<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap()
}

fn synthetic_bytes(length: usize, fill: &Value) -> Vec<u8> {
    vec![u8::from_str_radix(fill.as_str().unwrap(), 16).unwrap(); length]
}

#[test]
fn synthetic_opaque_frames_compose_i0_then_d0_at_exact_target_extents() {
    let vectors: Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(vectors["status"], "SYNTHETIC_NOT_REAL_OUTPUT");
    assert!(!vectors["contains_audio_or_frame_bytes"].as_bool().unwrap());
    assert!(
        !vectors["crc_or_source_generation_validation"]
            .as_bool()
            .unwrap()
    );

    let case = vector(&vectors, "synthetic-i0-then-d0-exact-target-extents");
    let i0 = synthetic_bytes(
        case["i0_frame_bytes"].as_u64().unwrap() as usize,
        &case["i0_test_fill_hex"],
    );
    let d0 = synthetic_bytes(
        case["d0_frame_bytes"].as_u64().unwrap() as usize,
        &case["d0_test_fill_hex"],
    );
    let frames = Flat7xI0D0OpaqueFrameSpans {
        i0: OpaqueI0FrameBytes::new(&i0),
        d0: OpaqueD0FrameBytes::new(&d0),
    };
    let mut output = vec![0xa5; 6_656];
    let receipt;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
        receipt = write_flat7x_i0_d0_access_unit_bytes(
            Flat7xI0D0TargetPlan::approved_contract(),
            frames,
            &mut writer,
        )
        .unwrap();
        cursor = writer.cursor();
    }

    assert_eq!(receipt.i0_span().start_byte(), 0);
    assert_eq!(receipt.i0_span().byte_length(), 2_560);
    assert_eq!(receipt.d0_span().start_byte(), 2_560);
    assert_eq!(receipt.d0_span().byte_length(), 4_096);
    assert_eq!(receipt.total_byte_length(), 6_656);
    assert_eq!(receipt.total_bit_length(), 53_248);
    assert!(!receipt.production_allowed());
    assert_eq!(cursor.bit_offset(), 53_248);
    assert!(output[..2_560].iter().all(|byte| *byte == 0x31));
    assert!(output[2_560..].iter().all(|byte| *byte == 0x72));
}

#[test]
fn extent_and_capacity_errors_leave_caller_bytes_and_cursor_unchanged() {
    let vectors: Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
    let i0_wrong = vector(&vectors, "reject-i0-wrong-extent");
    let d0_wrong = vector(&vectors, "reject-d0-wrong-extent");
    let valid = vector(&vectors, "synthetic-i0-then-d0-exact-target-extents");
    let i0 = synthetic_bytes(2_560, &valid["i0_test_fill_hex"]);
    let d0 = synthetic_bytes(4_096, &valid["d0_test_fill_hex"]);

    let short_independent_frame = &i0[..i0_wrong["i0_frame_bytes"].as_u64().unwrap() as usize];
    assert_rejected_unchanged(
        Flat7xI0D0OpaqueFrameSpans {
            i0: OpaqueI0FrameBytes::new(short_independent_frame),
            d0: OpaqueD0FrameBytes::new(&d0),
        },
        6_656,
        0,
        Flat7xI0D0AuCompositionError::I0FrameExtentMismatch {
            expected: 2_560,
            actual: 2_559,
        },
    );

    let short_dependent_frame = &d0[..d0_wrong["d0_frame_bytes"].as_u64().unwrap() as usize];
    assert_rejected_unchanged(
        Flat7xI0D0OpaqueFrameSpans {
            i0: OpaqueI0FrameBytes::new(&i0),
            d0: OpaqueD0FrameBytes::new(short_dependent_frame),
        },
        6_656,
        0,
        Flat7xI0D0AuCompositionError::D0FrameExtentMismatch {
            expected: 4_096,
            actual: 4_095,
        },
    );

    let capacity = vector(&vectors, "reject-caller-writer-capacity");
    let output_bytes = capacity["output_writer_bytes"].as_u64().unwrap() as usize;
    assert_rejected_unchanged(
        Flat7xI0D0OpaqueFrameSpans {
            i0: OpaqueI0FrameBytes::new(&i0),
            d0: OpaqueD0FrameBytes::new(&d0),
        },
        output_bytes,
        0,
        Flat7xI0D0AuCompositionError::BitWriter(FrameBitWriterError::BufferTooSmall),
    );
}

#[test]
fn nonzero_writer_cursor_rejects_without_mutation() {
    let i0 = vec![0x31; 2_560];
    let d0 = vec![0x72; 4_096];
    let frames = Flat7xI0D0OpaqueFrameSpans {
        i0: OpaqueI0FrameBytes::new(&i0),
        d0: OpaqueD0FrameBytes::new(&d0),
    };
    assert_rejected_unchanged(
        frames,
        6_656,
        1,
        Flat7xI0D0AuCompositionError::CursorMismatch { actual: 1 },
    );
}

fn assert_rejected_unchanged(
    frames: Flat7xI0D0OpaqueFrameSpans<'_>,
    output_bytes: usize,
    start_cursor: u32,
    expected_error: Flat7xI0D0AuCompositionError,
) {
    let mut output = vec![0xa5; output_bytes];
    let original = output.clone();
    let result;
    let cursor;
    {
        let mut writer =
            FrameBitWriter::new(&mut output, FrameBitCursor::new(start_cursor)).unwrap();
        result = write_flat7x_i0_d0_access_unit_bytes(
            Flat7xI0D0TargetPlan::approved_contract(),
            frames,
            &mut writer,
        );
        cursor = writer.cursor();
    }
    assert_eq!(result.unwrap_err(), expected_error);
    assert_eq!(cursor.bit_offset(), start_cursor);
    assert_eq!(output, original);
}
