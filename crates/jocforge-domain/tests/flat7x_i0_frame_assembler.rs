// pattern: Functional Core
//! Synthetic, non-playable tests for caller-owned Flat7x I0 AC-3 frame assembly.

use jocforge_domain::{
    codec::{
        ac3_bsi_tail::{
            Flat7xI0Ac3AudioProductionInfo, Flat7xI0Ac3BsiTailError, Flat7xI0Ac3BsiTailInput,
        },
        bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
        frame_builder::Flat7xI0Ac3HeaderSegmentError,
        i0_frame_assembler::{
            CallerOwnedI0CrcrsvSeed, Flat7xI0Ac3FrameAssemblyReceipt, Flat7xI0Ac3FrameHeaderFields,
            Flat7xI0Ac3FrameInput, Flat7xI0FrameAssemblyError, Flat7xI0FrameBitInput,
            I0AuxDataInput, I0CrcValidationStatus, I0FramePart, OpaqueI0Ac3BitSpan,
            assemble_flat7x_i0_ac3_frame, assemble_flat7x_i0_ac3_frame_from_fields,
        },
    },
    integrity::crc16::eac3_crc16,
    profile::Flat7xI0D0TargetPlan,
};
use serde_json::Value;

const FRAME_VECTORS: &str = include_str!("fixtures/i0_frame_assembler/INPUT-ONLY-VECTORS.json");
const HEADER_VECTORS: &str = include_str!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-I0-AC3-HEADER-SEGMENT-V1/INPUT-ONLY-VECTORS.json"
);
const BSI_TAIL_VECTORS: &str = include_str!("fixtures/i0_bsi_tail/INPUT-ONLY-VECTORS.json");

struct SyntheticI0FrameData {
    bsi_tail: Vec<u8>,
    audio_blocks: [Vec<u8>; 6],
    auxdata_payload: Option<Vec<u8>>,
}

fn vectors() -> Value {
    let value: Value = serde_json::from_str(FRAME_VECTORS).unwrap();
    assert_eq!(
        value["schema"],
        "jocforge.flat7x.i0_ac3_frame_assembler.input_only.v1"
    );
    assert_eq!(value["classification"], "SYNTHETIC_NOT_REAL_OUTPUT");
    assert!(!value["contains_pcm_or_real_audio_bytes"].as_bool().unwrap());
    assert!(!value["production_allowed"].as_bool().unwrap());
    value
}

fn case_by_id(vectors: &Value, id: &str) -> Value {
    vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()
        .clone()
}

fn header_vector_by_id(id: &Value) -> Value {
    let vectors: Value = serde_json::from_str(HEADER_VECTORS).unwrap();
    vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == *id)
        .unwrap()
        .clone()
}

fn bsi_tail_case_by_id(id: &str) -> Value {
    let vectors: Value = serde_json::from_str(BSI_TAIL_VECTORS).unwrap();
    assert_eq!(
        vectors["schema"],
        "jocforge.flat7x.i0_ac3_bsi_tail.input_only.v1"
    );
    assert!(!vectors["production_allowed"].as_bool().unwrap());
    vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()
        .clone()
}

fn bsi_tail_payload(case: &Value) -> Option<Vec<u8>> {
    let input = &case["input"]["additional_bsi"];
    if input.is_null() {
        return None;
    }
    if let Some(hex) = input["hex"].as_str() {
        return Some(
            hex.as_bytes()
                .chunks_exact(2)
                .map(|pair| {
                    let high = (pair[0] as char).to_digit(16).unwrap();
                    let low = (pair[1] as char).to_digit(16).unwrap();
                    u8::try_from((high << 4) | low).unwrap()
                })
                .collect(),
        );
    }
    let fill = u8::from_str_radix(input["fill_hex"].as_str().unwrap(), 16).unwrap();
    let byte_length = usize::try_from(input["length_bytes"].as_u64().unwrap()).unwrap();
    Some(vec![fill; byte_length])
}

fn bsi_tail_fields<'a>(case: &Value, payload: Option<&'a [u8]>) -> Flat7xI0Ac3BsiTailInput<'a> {
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

fn synthetic_audio_blocks(bit_lengths: [u32; 6]) -> [Vec<u8>; 6] {
    std::array::from_fn(|index| {
        bytes_for_bits(bit_lengths[index], u8::try_from(index + 1).unwrap())
    })
}

fn opaque_audio_blocks<'a>(
    bit_lengths: [u32; 6],
    bytes: &'a [Vec<u8>; 6],
) -> [OpaqueI0Ac3BitSpan<'a>; 6] {
    std::array::from_fn(|index| OpaqueI0Ac3BitSpan::new(&bytes[index], 0, bit_lengths[index]))
}

fn assert_caller_writer_unchanged(
    result: Result<Flat7xI0Ac3FrameAssemblyReceipt, Flat7xI0FrameAssemblyError>,
    expected_error: Flat7xI0FrameAssemblyError,
    cursor: u32,
    before: &[u8],
    after: &[u8],
    final_cursor: u32,
) {
    assert_eq!(result, Err(expected_error));
    assert_eq!(final_cursor, cursor);
    assert_eq!(after, before);
}

fn parse_hex_byte(value: &Value) -> u8 {
    u8::from_str_radix(value.as_str().unwrap(), 16).unwrap()
}

fn expected_span(value: &Value) -> [u32; 2] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| u32::try_from(item.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn bytes_for_bits(bit_length: u32, pattern: u8) -> Vec<u8> {
    let byte_length = usize::try_from(bit_length / 8 + u32::from(bit_length % 8 != 0)).unwrap();
    vec![pattern; byte_length]
}

fn bit_lengths(case: &Value) -> [u32; 6] {
    case["audio_block_bit_lengths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn synthetic_data(case: &Value) -> SyntheticI0FrameData {
    let patterns = case["audio_block_test_byte_patterns_hex"]
        .as_array()
        .unwrap();
    let lengths = bit_lengths(case);
    let audio_blocks = std::array::from_fn(|index| {
        bytes_for_bits(lengths[index], parse_hex_byte(&patterns[index]))
    });
    let bsi_tail = bytes_for_bits(
        u32::try_from(case["bsi_tail_bits"].as_u64().unwrap()).unwrap(),
        parse_hex_byte(&case["bsi_test_pattern_hex"]),
    );
    let auxdata_payload = case["auxdata"].as_object().map(|auxdata| {
        bytes_for_bits(
            u32::try_from(auxdata["payload_bits"].as_u64().unwrap()).unwrap(),
            parse_hex_byte(&auxdata["test_pattern_hex"]),
        )
    });
    SyntheticI0FrameData {
        bsi_tail,
        audio_blocks,
        auxdata_payload,
    }
}

fn header_values(case: &Value) -> Flat7xI0Ac3FrameHeaderFields {
    let header = header_vector_by_id(&case["header_vector_id"]);
    let inputs = &header["inputs"];
    Flat7xI0Ac3FrameHeaderFields {
        frmsizecod: u8::try_from(inputs["frmsizecod"].as_u64().unwrap()).unwrap(),
        bsmod: u8::try_from(inputs["bsmod"].as_u64().unwrap()).unwrap(),
        cmixlev: u8::try_from(inputs["cmixlev"].as_u64().unwrap()).unwrap(),
        surmixlev: u8::try_from(inputs["surmixlev"].as_u64().unwrap()).unwrap(),
        dialnorm: u8::try_from(inputs["dialnorm"].as_u64().unwrap()).unwrap(),
        compr: inputs["compr"]
            .as_u64()
            .map(|value| u8::try_from(value).unwrap()),
    }
}

fn frame_input<'a>(case: &Value, data: &'a SyntheticI0FrameData) -> Flat7xI0FrameBitInput<'a> {
    let lengths = bit_lengths(case);
    let audio_blocks = std::array::from_fn(|index| {
        OpaqueI0Ac3BitSpan::new(&data.audio_blocks[index], 0, lengths[index])
    });
    let bsi_tail = OpaqueI0Ac3BitSpan::new(
        &data.bsi_tail,
        0,
        u32::try_from(case["bsi_tail_bits"].as_u64().unwrap()).unwrap(),
    );
    let auxdata = match &data.auxdata_payload {
        Some(payload) => I0AuxDataInput::Present(OpaqueI0Ac3BitSpan::new(
            payload,
            0,
            u32::try_from(case["auxdata"]["payload_bits"].as_u64().unwrap()).unwrap(),
        )),
        None => I0AuxDataInput::Absent,
    };
    Flat7xI0FrameBitInput {
        header_fields: header_values(case),
        bsi_tail,
        audio_blocks,
        auxdata,
        crcrsv_seed: CallerOwnedI0CrcrsvSeed {
            value: case["integrity_fields"]["crcrsv_initial"]
                .as_bool()
                .unwrap(),
        },
    }
}

fn read_bit(bytes: &[u8], bit_offset: u32) -> bool {
    let byte = usize::try_from(bit_offset / 8).unwrap();
    let shift = 7 - bit_offset % 8;
    bytes[byte] & (1_u8 << shift) != 0
}

fn assert_span_copied(
    frame: &[u8],
    destination_start_bit: u32,
    source: &[u8],
    source_start_bit: u32,
    bit_length: u32,
) {
    for offset in 0..bit_length {
        assert_eq!(
            read_bit(frame, destination_start_bit + offset),
            read_bit(source, source_start_bit + offset)
        );
    }
}

#[test]
fn synthetic_i0_assembler_generates_and_residual_checks_crc_fields_after_full_staging() {
    let vectors = vectors();
    for case_id in [
        "synthetic-no-compr-absent-auxdata",
        "synthetic-with-compr-present-auxdata",
    ] {
        let case = case_by_id(&vectors, case_id);
        let data = synthetic_data(&case);
        let input = frame_input(&case, &data);
        let mut output = vec![0xa5; 2_560];
        let receipt;
        let cursor;
        {
            let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
            receipt = assemble_flat7x_i0_ac3_frame(
                Flat7xI0D0TargetPlan::approved_contract(),
                input,
                &mut writer,
            )
            .unwrap();
            cursor = writer.cursor().bit_offset();
        }

        let expected = &case["expected"];
        assert_eq!(receipt.target_frame_bytes(), 2_560);
        assert_eq!(receipt.total_frame_span_bits(), 20_480);
        assert_eq!(cursor, 20_480);
        assert_eq!(
            receipt.header_span().end_bit(),
            u32::try_from(expected["header_end_bit"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            [
                receipt.bsi_tail_span().start_bit(),
                receipt.bsi_tail_span().end_bit()
            ],
            expected_span(&expected["bsi_tail_span"])
        );
        assert_eq!(
            receipt.audio_block_spans()[0].start_bit(),
            u32::try_from(expected["first_audio_block_start_bit"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            receipt.audio_block_spans()[5].end_bit(),
            u32::try_from(expected["last_audio_block_end_bit"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            [
                receipt.auxdata_span().start_bit(),
                receipt.auxdata_span().end_bit()
            ],
            expected_span(&expected["auxdata_span"])
        );
        assert_eq!(
            [
                receipt.errorcheck_span().start_bit(),
                receipt.errorcheck_span().end_bit()
            ],
            expected_span(&expected["errorcheck_span"])
        );

        let integrity = receipt.integrity_receipts();
        assert_eq!(integrity.crc1().residual(), 0);
        assert_eq!(integrity.crc1().span().start_bit(), 16);
        assert_eq!(integrity.crc1().span().bit_length(), 16);
        assert_eq!(
            integrity.crc1().validation_status(),
            I0CrcValidationStatus::GeneratedAndResidualVerified
        );
        assert_eq!(
            integrity.crc1().value(),
            u16::from_be_bytes(output[2..4].try_into().unwrap())
        );
        assert_eq!(eac3_crc16(&output[2..1_600]), 0);
        assert_eq!(integrity.crc2().residual(), 0);
        assert_eq!(integrity.crc2().span().start_bit(), 20_464);
        assert_eq!(integrity.crc2().span().bit_length(), 16);
        assert_eq!(
            integrity.crc2().validation_status(),
            I0CrcValidationStatus::GeneratedAndResidualVerified
        );
        assert_eq!(
            integrity.crc2().value(),
            u16::from_be_bytes(output[2_558..].try_into().unwrap())
        );
        assert_eq!(eac3_crc16(&output[2..]), 0);
        assert_eq!(integrity.crcrsv_initial_value(), input.crcrsv_seed.value);
        assert_eq!(
            integrity.crcrsv_toggled(),
            expected["crcrsv_toggled"].as_bool().unwrap()
        );
        assert_eq!(integrity.crcrsv_span().start_bit(), 20_463);
        assert_eq!(integrity.crcrsv_span().bit_length(), 1);
        assert_eq!(read_bit(&output, 20_463), integrity.crcrsv_value());
        assert!(!receipt.production_allowed());

        assert_span_copied(
            &output,
            receipt.bsi_tail_span().start_bit(),
            &data.bsi_tail,
            0,
            receipt.bsi_tail_span().bit_length(),
        );
        for (index, span) in receipt.audio_block_spans().into_iter().enumerate() {
            assert_span_copied(
                &output,
                span.start_bit(),
                &data.audio_blocks[index],
                0,
                span.bit_length(),
            );
        }
        if let Some(payload) = &data.auxdata_payload {
            let payload_bits =
                u32::try_from(case["auxdata"]["payload_bits"].as_u64().unwrap()).unwrap();
            assert_span_copied(
                &output,
                receipt.auxdata_span().start_bit(),
                payload,
                0,
                payload_bits,
            );
            assert!(read_bit(&output, receipt.auxdata_span().end_bit() - 1));
        } else {
            assert!(!read_bit(&output, receipt.auxdata_span().start_bit()));
        }
    }
}

#[test]
fn typed_i0_entry_serializes_washed_bsi_then_six_caller_spans_and_crc() {
    let case = bsi_tail_case_by_id("all-optionals-with-two-byte-addbsi");
    let payload = bsi_tail_payload(&case).unwrap();
    let bsi_tail = bsi_tail_fields(&case, Some(&payload));
    let bit_lengths = [3_387, 3_387, 3_387, 3_387, 3_386, 3_386];
    assert_eq!(bit_lengths.iter().sum::<u32>(), 20_320);
    let audio_bytes = synthetic_audio_blocks(bit_lengths);
    let input = Flat7xI0Ac3FrameInput {
        header_fields: Flat7xI0Ac3FrameHeaderFields {
            frmsizecod: 37,
            bsmod: 0,
            cmixlev: 1,
            surmixlev: 1,
            dialnorm: 31,
            compr: Some(0x55),
        },
        bsi_tail,
        audio_blocks: opaque_audio_blocks(bit_lengths, &audio_bytes),
        auxdata: I0AuxDataInput::Absent,
        crcrsv_seed: CallerOwnedI0CrcrsvSeed { value: false },
    };
    let mut frame_bytes = vec![0xa5; 2_560];
    let receipt;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut frame_bytes, FrameBitCursor::new(0)).unwrap();
        receipt = assemble_flat7x_i0_ac3_frame_from_fields(
            Flat7xI0D0TargetPlan::approved_contract(),
            input,
            &mut writer,
        )
        .unwrap();
        cursor = writer.cursor().bit_offset();
    }

    let frame = receipt.frame();
    let bsi_expected = &case["expected"];
    assert_eq!(cursor, 20_480);
    assert_eq!(frame.target_frame_bytes(), 2_560);
    assert_eq!(
        [
            frame.bsi_tail_span().start_bit(),
            frame.bsi_tail_span().end_bit()
        ],
        [70, 142]
    );
    assert_eq!(receipt.bsi_tail().start_bit(), 70);
    assert_eq!(
        receipt.bsi_tail().end_bit(),
        u32::try_from(bsi_expected["end_bit"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        receipt.bsi_tail().bit_length(),
        u32::try_from(bsi_expected["bit_length"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(receipt.bsi_tail().additional_bsi_start_bit(), Some(126));
    assert_eq!(receipt.bsi_tail().additional_bsi_bit_length(), Some(16));
    assert_eq!(frame.audio_block_spans()[0].start_bit(), 142);
    assert_eq!(frame.audio_block_spans()[5].end_bit(), 20_462);
    assert_eq!(frame.auxdata_span().start_bit(), 20_462);
    assert_eq!(frame.auxdata_span().bit_length(), 1);
    assert_eq!(
        [
            frame.errorcheck_span().start_bit(),
            frame.errorcheck_span().end_bit()
        ],
        [20_463, 20_480]
    );
    assert_eq!(eac3_crc16(&frame_bytes[2..1_600]), 0);
    assert_eq!(eac3_crc16(&frame_bytes[2..]), 0);
    assert!(!receipt.production_allowed());
}

#[test]
fn typed_i0_entry_fails_before_caller_write_for_invalid_bsi_or_incomplete_frame_extent() {
    let empty_audio: [Vec<u8>; 6] = std::array::from_fn(|_| Vec::new());
    let empty_lengths = [0; 6];
    let audio_blocks = opaque_audio_blocks(empty_lengths, &empty_audio);
    let minimum_bsi = Flat7xI0Ac3BsiTailInput {
        langcode: None,
        audio_production: None,
        copyright: false,
        original_bitstream: false,
        time_code_1: None,
        time_code_2: None,
        additional_bsi: None,
    };

    let invalid_fields = Flat7xI0Ac3FrameInput {
        header_fields: Flat7xI0Ac3FrameHeaderFields {
            frmsizecod: 36,
            bsmod: 0,
            cmixlev: 0,
            surmixlev: 0,
            dialnorm: 31,
            compr: None,
        },
        bsi_tail: Flat7xI0Ac3BsiTailInput {
            audio_production: Some(Flat7xI0Ac3AudioProductionInfo {
                mix_level: 32,
                room_type: 0,
            }),
            ..minimum_bsi
        },
        audio_blocks,
        auxdata: I0AuxDataInput::Absent,
        crcrsv_seed: CallerOwnedI0CrcrsvSeed { value: false },
    };
    let mut invalid_output = vec![0x96; 2_560];
    let invalid_before = invalid_output.clone();
    let invalid_result;
    let invalid_cursor;
    {
        let mut writer = FrameBitWriter::new(&mut invalid_output, FrameBitCursor::new(0)).unwrap();
        invalid_result = assemble_flat7x_i0_ac3_frame_from_fields(
            Flat7xI0D0TargetPlan::approved_contract(),
            invalid_fields,
            &mut writer,
        );
        invalid_cursor = writer.cursor().bit_offset();
    }
    assert_caller_writer_unchanged(
        invalid_result,
        Flat7xI0FrameAssemblyError::BsiTail(Flat7xI0Ac3BsiTailError::MixLevelOutOfRange {
            value: 32,
        }),
        0,
        &invalid_before,
        &invalid_output,
        invalid_cursor,
    );

    let wrong_extent = Flat7xI0Ac3FrameInput {
        header_fields: Flat7xI0Ac3FrameHeaderFields {
            frmsizecod: 36,
            bsmod: 0,
            cmixlev: 0,
            surmixlev: 0,
            dialnorm: 31,
            compr: None,
        },
        bsi_tail: minimum_bsi,
        audio_blocks,
        auxdata: I0AuxDataInput::Absent,
        crcrsv_seed: CallerOwnedI0CrcrsvSeed { value: false },
    };
    let mut short_candidate_output = vec![0x69; 2_560];
    let short_candidate_before = short_candidate_output.clone();
    let result;
    let cursor;
    {
        let mut writer =
            FrameBitWriter::new(&mut short_candidate_output, FrameBitCursor::new(0)).unwrap();
        result = assemble_flat7x_i0_ac3_frame_from_fields(
            Flat7xI0D0TargetPlan::approved_contract(),
            wrong_extent,
            &mut writer,
        );
        cursor = writer.cursor().bit_offset();
    }
    assert_caller_writer_unchanged(
        result,
        Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: 87,
            required_bits: 20_480,
        },
        0,
        &short_candidate_before,
        &short_candidate_output,
        cursor,
    );
}

#[test]
fn i0_assembler_rejections_leave_caller_bytes_and_cursor_unchanged() {
    let vectors = vectors();
    let case = case_by_id(&vectors, "synthetic-no-compr-absent-auxdata");
    let data = synthetic_data(&case);
    let input = frame_input(&case, &data);
    let target = Flat7xI0D0TargetPlan::approved_contract();

    let short_bsi = Flat7xI0FrameBitInput {
        bsi_tail: OpaqueI0Ac3BitSpan::new(&data.bsi_tail, 0, 6),
        ..input
    };
    assert_rejected_unchanged(
        target,
        short_bsi,
        2_560,
        0,
        Flat7xI0FrameAssemblyError::BsiTailTooShort {
            actual_bits: 6,
            minimum_bits: 7,
        },
    );

    let mut short_blocks = input.audio_blocks;
    short_blocks[5] = OpaqueI0Ac3BitSpan::new(
        &data.audio_blocks[5],
        0,
        input.audio_blocks[5].bit_length() - 1,
    );
    let short_frame = Flat7xI0FrameBitInput {
        audio_blocks: short_blocks,
        ..input
    };
    assert_rejected_unchanged(
        target,
        short_frame,
        2_560,
        0,
        Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: 20_479,
            required_bits: 20_480,
        },
    );

    let mut out_of_bounds_blocks = input.audio_blocks;
    out_of_bounds_blocks[5] =
        OpaqueI0Ac3BitSpan::new(&data.audio_blocks[5], 4, input.audio_blocks[5].bit_length());
    let out_of_bounds = Flat7xI0FrameBitInput {
        audio_blocks: out_of_bounds_blocks,
        ..input
    };
    assert_rejected_unchanged(
        target,
        out_of_bounds,
        2_560,
        0,
        Flat7xI0FrameAssemblyError::SourceSpanOutOfBounds {
            part: I0FramePart::AudioBlock { index: 5 },
            required_end_bit: 3_402,
            available_source_bits: 3_400,
        },
    );

    let invalid_header = Flat7xI0FrameBitInput {
        header_fields: Flat7xI0Ac3FrameHeaderFields {
            frmsizecod: 35,
            ..input.header_fields
        },
        ..input
    };
    assert_rejected_unchanged(
        target,
        invalid_header,
        2_560,
        0,
        Flat7xI0FrameAssemblyError::HeaderSegment(
            Flat7xI0Ac3HeaderSegmentError::InvalidFrameSizeCode { value: 35 },
        ),
    );

    assert_rejected_unchanged(
        target,
        input,
        2_559,
        0,
        Flat7xI0FrameAssemblyError::BitWriter(FrameBitWriterError::BufferTooSmall),
    );
    assert_rejected_unchanged(
        target,
        input,
        2_560,
        1,
        Flat7xI0FrameAssemblyError::CursorMismatch { actual: 1 },
    );
}

fn assert_rejected_unchanged(
    target: Flat7xI0D0TargetPlan,
    input: Flat7xI0FrameBitInput<'_>,
    output_bytes: usize,
    start_cursor: u32,
    expected_error: Flat7xI0FrameAssemblyError,
) {
    let mut output = vec![0xa5; output_bytes];
    let before = output.clone();
    let result;
    let cursor;
    {
        let mut writer =
            FrameBitWriter::new(&mut output, FrameBitCursor::new(start_cursor)).unwrap();
        result = assemble_flat7x_i0_ac3_frame(target, input, &mut writer);
        cursor = writer.cursor().bit_offset();
    }
    assert_eq!(result, Err(expected_error));
    assert_eq!(output, before);
    assert_eq!(cursor, start_cursor);
}
