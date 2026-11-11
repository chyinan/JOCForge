// pattern: Functional Core
//! Wash-pinned input-only vectors for the bounded Flat7x I0 AC-3 header segment.

use jocforge_domain::codec::{
    bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    frame_builder::{
        Flat7xI0Ac3HeaderSegmentError, Flat7xI0Ac3HeaderValues,
        write_flat7x_i0_ac3_header_through_compression,
    },
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-I0-AC3-HEADER-SEGMENT-V1/INPUT-ONLY-VECTORS.json"
);

fn vector<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap()
}

fn values_from_vector(vector: &Value) -> Flat7xI0Ac3HeaderValues {
    let inputs = &vector["inputs"];
    Flat7xI0Ac3HeaderValues {
        crc1: inputs["crc1"].as_u64().unwrap() as u16,
        frmsizecod: inputs["frmsizecod"].as_u64().unwrap() as u8,
        bsmod: inputs["bsmod"].as_u64().unwrap() as u8,
        cmixlev: inputs["cmixlev"].as_u64().unwrap() as u8,
        surmixlev: inputs["surmixlev"].as_u64().unwrap() as u8,
        dialnorm: inputs["dialnorm"].as_u64().unwrap() as u8,
        compr: inputs["compr"].as_u64().map(|value| value as u8),
    }
}

fn bit_string(bytes: &[u8], bit_count: u32) -> String {
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

fn assert_valid_vector(vector: &Value) {
    let expected_end = vector["expected_end_bit_offset"].as_u64().unwrap() as u32;
    let expected_bytes = bytes_from_hex(vector["expected_bytes_hex_zero_padded"].as_str().unwrap());
    let expected_bits = vector["expected_bits"].as_str().unwrap();
    let mut bytes = vec![0_u8; expected_bytes.len()];
    let end;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        end =
            write_flat7x_i0_ac3_header_through_compression(&mut writer, values_from_vector(vector))
                .unwrap();
    }

    assert_eq!(end.bit_offset(), expected_end);
    assert_eq!(bit_string(&bytes, expected_end), expected_bits);
    assert_eq!(bytes, expected_bytes);
}

fn assert_invalid_before_write(
    values: Flat7xI0Ac3HeaderValues,
    expected_error: Flat7xI0Ac3HeaderSegmentError,
) {
    let mut bytes = vec![0xa5_u8; 10];
    let initial = bytes.clone();
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        error = write_flat7x_i0_ac3_header_through_compression(&mut writer, values).unwrap_err();
        cursor = writer.cursor();
    }

    assert_eq!(error, expected_error);
    assert_eq!(cursor.bit_offset(), 0);
    assert_eq!(bytes, initial);
}

#[test]
fn no_compr_and_with_compr_segments_match_wash_vectors() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(
        vectors["schema"],
        "jocforge.flat7x.i0_ac3_header_segment.input_only.v1"
    );
    assert_valid_vector(vector(&vectors, "i0-640-frmsizecod36-no-compr"));
    assert_valid_vector(vector(&vectors, "i0-640-frmsizecod37-with-compr"));
}

#[test]
fn capacity_and_field_ranges_reject_before_any_write() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    let rejected = vector(&vectors, "preflight-and-range-rejections");
    let buffer_bytes = 8;
    let initial = vec![0x5a_u8; buffer_bytes];
    let mut bytes = initial.clone();
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        error = write_flat7x_i0_ac3_header_through_compression(
            &mut writer,
            values_from_vector(vector(&vectors, "i0-640-frmsizecod37-with-compr")),
        )
        .unwrap_err();
        cursor = writer.cursor();
    }
    assert_eq!(
        rejected["available_bits"].as_u64().unwrap(),
        u64::try_from(buffer_bytes).unwrap() * 8
    );
    assert_eq!(
        error,
        Flat7xI0Ac3HeaderSegmentError::BitWriter(FrameBitWriterError::BufferTooSmall)
    );
    assert_eq!(cursor.bit_offset(), 0);
    assert_eq!(bytes, initial);

    let valid = values_from_vector(vector(&vectors, "i0-640-frmsizecod36-no-compr"));
    for value in rejected["invalid_frmsizecod"].as_array().unwrap() {
        let mut input = valid;
        input.frmsizecod = value.as_u64().unwrap() as u8;
        assert_invalid_before_write(
            input,
            Flat7xI0Ac3HeaderSegmentError::InvalidFrameSizeCode {
                value: input.frmsizecod,
            },
        );
    }
    for value in rejected["invalid_bsmod"].as_array().unwrap() {
        let mut input = valid;
        input.bsmod = value.as_u64().unwrap() as u8;
        assert_invalid_before_write(
            input,
            Flat7xI0Ac3HeaderSegmentError::InvalidBsmod { value: input.bsmod },
        );
    }
    for value in rejected["invalid_cmixlev"].as_array().unwrap() {
        let mut input = valid;
        input.cmixlev = value.as_u64().unwrap() as u8;
        assert_invalid_before_write(
            input,
            Flat7xI0Ac3HeaderSegmentError::InvalidCmixlev {
                value: input.cmixlev,
            },
        );
    }
    for value in rejected["invalid_surmixlev"].as_array().unwrap() {
        let mut input = valid;
        input.surmixlev = value.as_u64().unwrap() as u8;
        assert_invalid_before_write(
            input,
            Flat7xI0Ac3HeaderSegmentError::InvalidSurmixlev {
                value: input.surmixlev,
            },
        );
    }
    for value in rejected["invalid_dialnorm"].as_array().unwrap() {
        let mut input = valid;
        input.dialnorm = value.as_u64().unwrap() as u8;
        assert_invalid_before_write(
            input,
            Flat7xI0Ac3HeaderSegmentError::InvalidDialnorm {
                value: input.dialnorm,
            },
        );
    }
}
