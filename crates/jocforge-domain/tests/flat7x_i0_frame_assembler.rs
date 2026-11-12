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
            Ac3CplinuValueError, CallerOwnedAc3Cplinu, CallerOwnedI0CrcrsvSeed,
            Flat7xI0Ac3FrameAssemblyReceipt, Flat7xI0Ac3FrameHeaderFields, Flat7xI0Ac3FrameInput,
            Flat7xI0FrameAssemblyError, Flat7xI0FrameBitInput, I0Ac3AudioBlockFieldExtent,
            I0Ac3AudioBlockPrefixError, I0Ac3AudioBlockPrefixInput, I0Ac3CouplingFieldsError,
            I0Ac3CouplingFieldsInput, I0Ac3CouplingFrequencyField, I0Ac3CplinuWriteError,
            I0AuxDataInput, I0CrcValidationStatus, I0FramePart, OpaqueI0Ac3BitSpan,
            assemble_flat7x_i0_ac3_frame, assemble_flat7x_i0_ac3_frame_from_fields,
            write_i0_ac3_first_audio_block_prefix, write_i0_ac3_first_block_coupling_fields,
            write_i0_ac3_first_block_cplinu,
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

fn first_audio_block_prefix_case(vectors: &Value, id: &str) -> Value {
    vectors["first_audio_block_prefix_vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()
        .clone()
}

fn cplinu_case(vectors: &Value, id: &str) -> Value {
    vectors["cplinu_vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()
        .clone()
}

fn coupling_fields_case(vectors: &Value, id: &str) -> Value {
    vectors["coupling_fields_vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()
        .clone()
}

fn bool_array<const N: usize>(value: &Value) -> [bool; N] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_bool().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn bool_values(value: &Value) -> Vec<bool> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_bool().unwrap())
        .collect()
}

fn coupling_fields_input<'a>(
    case: &Value,
    band_structure_bits: &'a [bool],
) -> I0Ac3CouplingFieldsInput<'a> {
    I0Ac3CouplingFieldsInput {
        channel_in_coupling: bool_array(&case["channel_in_coupling"]),
        cplbegf: u8::try_from(case["cplbegf"].as_u64().unwrap()).unwrap(),
        cplendf: u8::try_from(case["cplendf"].as_u64().unwrap()).unwrap(),
        band_structure_bits,
    }
}

fn first_audio_block_prefix_input(case: &Value) -> I0Ac3AudioBlockPrefixInput {
    let bool_array = |name: &str| {
        case[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_bool().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    };
    I0Ac3AudioBlockPrefixInput {
        block_switch_flags: bool_array("block_switch_flags"),
        dither_flags: bool_array("dither_flags"),
        dynamic_range_code: case["dynamic_range_code"]
            .as_u64()
            .map(|value| u8::try_from(value).unwrap()),
    }
}

fn decode_hex_bytes(value: &str) -> Vec<u8> {
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

fn assert_prefix_extent(actual: I0Ac3AudioBlockFieldExtent, expected: &Value) {
    assert_eq!(
        actual.start().bit_offset(),
        u32::try_from(expected[0].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        u64::from(actual.bit_length()),
        expected[1].as_u64().unwrap()
    );
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

#[test]
fn synthetic_first_audio_block_prefix_vector_writes_standard_fields_and_extents() {
    let vectors = vectors();
    for case_id in [
        "synthetic-first-block-prefix-with-dynrng",
        "synthetic-first-block-prefix-without-dynrng",
    ] {
        let case = first_audio_block_prefix_case(&vectors, case_id);
        let expected = &case["expected"];
        let expected_bytes = decode_hex_bytes(expected["bytes_hex"].as_str().unwrap());
        let mut output = vec![0; expected_bytes.len()];
        let receipt;
        let cursor;
        {
            let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
            receipt = write_i0_ac3_first_audio_block_prefix(
                &mut writer,
                first_audio_block_prefix_input(&case),
            )
            .unwrap();
            cursor = writer.cursor().bit_offset();
        }

        assert_eq!(output, expected_bytes);
        assert_eq!(
            cursor,
            u32::try_from(expected["next_field_cursor"].as_u64().unwrap()).unwrap()
        );
        assert_prefix_extent(receipt.block_switch(), &expected["block_switch_span"]);
        assert_prefix_extent(receipt.dither(), &expected["dither_span"]);
        assert_prefix_extent(
            receipt.dynamic_range_exists(),
            &expected["dynamic_range_exists_span"],
        );
        assert_eq!(
            receipt
                .dynamic_range()
                .map(|span| [span.start().bit_offset(), u32::from(span.bit_length())]),
            expected["dynamic_range_span"].as_array().map(|span| [
                u32::try_from(span[0].as_u64().unwrap()).unwrap(),
                u32::try_from(span[1].as_u64().unwrap()).unwrap(),
            ])
        );
        assert_prefix_extent(
            receipt.coupling_strategy_exists(),
            &expected["coupling_strategy_exists_span"],
        );
        assert_eq!(receipt.next_field_cursor().bit_offset(), cursor);
        assert!(read_bit(
            &output,
            receipt.coupling_strategy_exists().start().bit_offset()
        ));
        assert!(!receipt.production_allowed());
    }
}

#[test]
fn first_audio_block_prefix_capacity_failure_leaves_bytes_and_cursor_unchanged() {
    let vectors = vectors();
    let case = first_audio_block_prefix_case(&vectors, "synthetic-first-block-prefix-with-dynrng");
    let mut output = [0xa5, 0x5a];
    let before = output;
    let result;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
        result = write_i0_ac3_first_audio_block_prefix(
            &mut writer,
            first_audio_block_prefix_input(&case),
        );
        cursor = writer.cursor().bit_offset();
    }

    assert_eq!(
        result,
        Err(I0Ac3AudioBlockPrefixError::BitWriter(
            FrameBitWriterError::BufferTooSmall
        ))
    );
    assert_eq!(output, before);
    assert_eq!(cursor, 0);
}

#[test]
fn synthetic_cplinu_vectors_write_the_caller_owned_bit_after_cplstre() {
    let vectors = vectors();
    for case_id in [
        "synthetic-first-block-cplinu-independent",
        "synthetic-first-block-cplinu-coupled",
    ] {
        let case = cplinu_case(&vectors, case_id);
        let prefix_case =
            first_audio_block_prefix_case(&vectors, case["prefix_vector_id"].as_str().unwrap());
        let expected = &case["expected"];
        let mut output = [0; 3];
        let prefix;
        let receipt;
        let cursor;
        {
            let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
            prefix = write_i0_ac3_first_audio_block_prefix(
                &mut writer,
                first_audio_block_prefix_input(&prefix_case),
            )
            .unwrap();
            let caller_value = CallerOwnedAc3Cplinu::try_from(
                u8::try_from(case["field_value"].as_u64().unwrap()).unwrap(),
            )
            .unwrap();
            receipt = write_i0_ac3_first_block_cplinu(&mut writer, prefix, caller_value).unwrap();
            cursor = writer.cursor().bit_offset();
        }

        assert_eq!(
            output.as_slice(),
            decode_hex_bytes(expected["bytes_hex"].as_str().unwrap())
        );
        assert_eq!(
            receipt.field().start().bit_offset(),
            u32::try_from(expected["field_span"][0].as_u64().unwrap()).unwrap()
        );
        assert_eq!(receipt.field().bit_length(), 1);
        assert_eq!(
            cursor,
            u32::try_from(expected["next_field_cursor"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(receipt.next_field_cursor().bit_offset(), cursor);
        assert_eq!(
            read_bit(&output, receipt.field().start().bit_offset()),
            case["field_value"].as_u64().unwrap() == 1
        );
        assert!(!prefix.production_allowed());
        assert!(!receipt.production_allowed());
    }
}

#[test]
fn cplinu_range_and_capacity_rejections_leave_bytes_and_cursor_unchanged() {
    let vectors = vectors();
    let prefix_case =
        first_audio_block_prefix_case(&vectors, "synthetic-first-block-prefix-with-dynrng");
    for invalid in vectors["invalid_cplinu_field_values"].as_array().unwrap() {
        let value = u8::try_from(invalid.as_u64().unwrap()).unwrap();
        let mut output = [0xa5; 3];
        let prefix = {
            let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
            write_i0_ac3_first_audio_block_prefix(
                &mut writer,
                first_audio_block_prefix_input(&prefix_case),
            )
            .unwrap()
        };
        let before = output;
        let before_cursor = prefix.next_field_cursor();
        let rejected = CallerOwnedAc3Cplinu::try_from(value);
        assert_eq!(rejected, Err(Ac3CplinuValueError::OutOfRange { value }));
        assert_eq!(output, before);
        assert_eq!(prefix.next_field_cursor(), before_cursor);

        let mut exact_edge_output = [0xa5; 3];
        let prefix;
        {
            let mut writer =
                FrameBitWriter::new(&mut exact_edge_output, FrameBitCursor::new(4)).unwrap();
            prefix = write_i0_ac3_first_audio_block_prefix(
                &mut writer,
                first_audio_block_prefix_input(&prefix_case),
            )
            .unwrap();
            assert_eq!(writer.cursor(), FrameBitCursor::new(24));
        }
        let before = exact_edge_output;
        let result;
        let after_cursor;
        {
            let mut writer =
                FrameBitWriter::new(&mut exact_edge_output, prefix.next_field_cursor()).unwrap();
            result = write_i0_ac3_first_block_cplinu(
                &mut writer,
                prefix,
                CallerOwnedAc3Cplinu::Independent,
            );
            after_cursor = writer.cursor();
        }
        assert_eq!(
            result,
            Err(I0Ac3CplinuWriteError::BitWriter(
                FrameBitWriterError::BufferTooSmall
            ))
        );
        assert_eq!(exact_edge_output, before);
        assert_eq!(after_cursor, prefix.next_field_cursor());
    }
}

#[test]
fn cplinu_rejects_a_cursor_that_does_not_follow_cplstre() {
    let vectors = vectors();
    let prefix_case =
        first_audio_block_prefix_case(&vectors, "synthetic-first-block-prefix-with-dynrng");
    let mut output = [0xa5; 4];
    let prefix = {
        let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
        write_i0_ac3_first_audio_block_prefix(
            &mut writer,
            first_audio_block_prefix_input(&prefix_case),
        )
        .unwrap()
    };
    let before = output;
    let wrong_cursor = FrameBitCursor::new(prefix.next_field_cursor().bit_offset() + 1);
    let result;
    let after_cursor;
    {
        let mut writer = FrameBitWriter::new(&mut output, wrong_cursor).unwrap();
        result =
            write_i0_ac3_first_block_cplinu(&mut writer, prefix, CallerOwnedAc3Cplinu::Independent);
        after_cursor = writer.cursor();
    }

    assert_eq!(
        result,
        Err(I0Ac3CplinuWriteError::PrefixCursorMismatch {
            expected: prefix.next_field_cursor(),
            actual: wrong_cursor,
        })
    );
    assert_eq!(output, before);
    assert_eq!(after_cursor, wrong_cursor);
}

#[test]
fn synthetic_coupling_fields_write_caller_owned_channel_and_subband_choices() {
    let vectors = vectors();
    let prefix_case =
        first_audio_block_prefix_case(&vectors, "synthetic-first-block-prefix-with-dynrng");
    for case_id in [
        "synthetic-first-block-coupling-single-subband",
        "synthetic-first-block-coupling-three-subband",
    ] {
        let case = coupling_fields_case(&vectors, case_id);
        let expected = &case["expected"];
        let band_structure = bool_values(&case["cplbndstrc_bits"]);
        let mut output = vec![0; decode_hex_bytes(expected["bytes_hex"].as_str().unwrap()).len()];
        let prefix;
        let cplinu;
        let receipt;
        let cursor;
        {
            let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
            prefix = write_i0_ac3_first_audio_block_prefix(
                &mut writer,
                first_audio_block_prefix_input(&prefix_case),
            )
            .unwrap();
            cplinu =
                write_i0_ac3_first_block_cplinu(&mut writer, prefix, CallerOwnedAc3Cplinu::Coupled)
                    .unwrap();
            receipt = write_i0_ac3_first_block_coupling_fields(
                &mut writer,
                cplinu,
                coupling_fields_input(&case, &band_structure),
            )
            .unwrap();
            cursor = writer.cursor().bit_offset();
        }

        assert_eq!(
            output,
            decode_hex_bytes(expected["bytes_hex"].as_str().unwrap())
        );
        assert_eq!(cplinu.value(), CallerOwnedAc3Cplinu::Coupled);
        assert_prefix_extent(
            receipt.channel_in_coupling(),
            &expected["channel_in_coupling_span"],
        );
        assert_prefix_extent(receipt.cplbegf(), &expected["cplbegf_span"]);
        assert_prefix_extent(receipt.cplendf(), &expected["cplendf_span"]);
        assert_eq!(
            receipt
                .band_structure()
                .map(|span| [span.start().bit_offset(), u32::from(span.bit_length())]),
            expected["cplbndstrc_span"].as_array().map(|span| [
                u32::try_from(span[0].as_u64().unwrap()).unwrap(),
                u32::try_from(span[1].as_u64().unwrap()).unwrap(),
            ])
        );
        assert_eq!(
            receipt.active_subband_count(),
            u8::try_from(expected["active_subband_count"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            cursor,
            u32::try_from(expected["next_field_cursor"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(receipt.next_field_cursor().bit_offset(), cursor);
        assert!(!prefix.production_allowed());
        assert!(!cplinu.production_allowed());
        assert!(!receipt.production_allowed());
    }
}

#[test]
fn invalid_coupling_field_inputs_fail_before_writer_mutation() {
    let vectors = vectors();
    let prefix_case =
        first_audio_block_prefix_case(&vectors, "synthetic-first-block-prefix-with-dynrng");
    for case in vectors["invalid_coupling_field_cases"].as_array().unwrap() {
        let band_structure = bool_values(&case["cplbndstrc_bits"]);
        let mut output = [0xa5; 8];
        let cplinu;
        {
            let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
            let prefix = write_i0_ac3_first_audio_block_prefix(
                &mut writer,
                first_audio_block_prefix_input(&prefix_case),
            )
            .unwrap();
            cplinu =
                write_i0_ac3_first_block_cplinu(&mut writer, prefix, CallerOwnedAc3Cplinu::Coupled)
                    .unwrap();
        }
        let before = output;
        let cursor = cplinu.next_field_cursor();
        let result;
        {
            let mut writer = FrameBitWriter::new(&mut output, cursor).unwrap();
            result = write_i0_ac3_first_block_coupling_fields(
                &mut writer,
                cplinu,
                coupling_fields_input(case, &band_structure),
            );
            assert_eq!(writer.cursor(), cursor);
        }

        let expected_error = match case["expected_error"].as_str().unwrap() {
            "FewerThanTwoChannelsSelected" => {
                I0Ac3CouplingFieldsError::FewerThanTwoChannelsSelected
            }
            "FrequencyCodeOutOfRangeBegin" => I0Ac3CouplingFieldsError::FrequencyCodeOutOfRange {
                field: I0Ac3CouplingFrequencyField::Begin,
                value: 16,
            },
            "FrequencyCodeOutOfRangeEnd" => I0Ac3CouplingFieldsError::FrequencyCodeOutOfRange {
                field: I0Ac3CouplingFrequencyField::End,
                value: 16,
            },
            "ActiveSubbandCountOutOfRange" => {
                I0Ac3CouplingFieldsError::ActiveSubbandCountOutOfRange { value: 0 }
            }
            "BandStructureLengthMismatch" => {
                I0Ac3CouplingFieldsError::BandStructureLengthMismatch {
                    expected: 2,
                    actual: 1,
                }
            }
            other => panic!("unknown expected coupling-field error {other}"),
        };
        assert_eq!(result, Err(expected_error), "case {}", case["id"]);
        assert_eq!(output, before, "case {}", case["id"]);
    }
}

#[test]
fn coupling_fields_reject_a_cursor_that_does_not_follow_cplinu() {
    let vectors = vectors();
    let prefix_case =
        first_audio_block_prefix_case(&vectors, "synthetic-first-block-prefix-with-dynrng");
    let case = coupling_fields_case(&vectors, "synthetic-first-block-coupling-single-subband");
    let band_structure = bool_values(&case["cplbndstrc_bits"]);
    let mut output = [0xa5; 8];
    let prefix;
    let cplinu;
    {
        let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
        prefix = write_i0_ac3_first_audio_block_prefix(
            &mut writer,
            first_audio_block_prefix_input(&prefix_case),
        )
        .unwrap();
        cplinu =
            write_i0_ac3_first_block_cplinu(&mut writer, prefix, CallerOwnedAc3Cplinu::Coupled)
                .unwrap();
    }
    let before = output;
    let wrong_cursor = FrameBitCursor::new(cplinu.next_field_cursor().bit_offset() + 1);
    let result;
    let after_cursor;
    {
        let mut writer = FrameBitWriter::new(&mut output, wrong_cursor).unwrap();
        result = write_i0_ac3_first_block_coupling_fields(
            &mut writer,
            cplinu,
            coupling_fields_input(&case, &band_structure),
        );
        after_cursor = writer.cursor();
    }

    assert_eq!(
        result,
        Err(I0Ac3CouplingFieldsError::PrefixCursorMismatch {
            expected: cplinu.next_field_cursor(),
            actual: wrong_cursor,
        })
    );
    assert_eq!(output, before);
    assert_eq!(after_cursor, wrong_cursor);
}

#[test]
fn coupling_fields_require_cplinu_and_preflight_total_capacity() {
    let vectors = vectors();
    let prefix_case =
        first_audio_block_prefix_case(&vectors, "synthetic-first-block-prefix-with-dynrng");
    let case = coupling_fields_case(&vectors, "synthetic-first-block-coupling-single-subband");
    let band_structure = bool_values(&case["cplbndstrc_bits"]);

    let mut output = [0xa5; 8];
    let prefix;
    let independent;
    {
        let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
        prefix = write_i0_ac3_first_audio_block_prefix(
            &mut writer,
            first_audio_block_prefix_input(&prefix_case),
        )
        .unwrap();
        independent =
            write_i0_ac3_first_block_cplinu(&mut writer, prefix, CallerOwnedAc3Cplinu::Independent)
                .unwrap();
    }
    let before = output;
    let cursor = independent.next_field_cursor();
    let result;
    {
        let mut writer = FrameBitWriter::new(&mut output, cursor).unwrap();
        result = write_i0_ac3_first_block_coupling_fields(
            &mut writer,
            independent,
            coupling_fields_input(&case, &band_structure),
        );
        assert_eq!(writer.cursor(), cursor);
    }
    assert_eq!(result, Err(I0Ac3CouplingFieldsError::CouplingNotInUse));
    assert_eq!(output, before);

    let mut short_output = [0xa5; 4];
    let prefix;
    let cplinu;
    {
        let mut writer = FrameBitWriter::new(&mut short_output, FrameBitCursor::new(0)).unwrap();
        prefix = write_i0_ac3_first_audio_block_prefix(
            &mut writer,
            first_audio_block_prefix_input(&prefix_case),
        )
        .unwrap();
        cplinu =
            write_i0_ac3_first_block_cplinu(&mut writer, prefix, CallerOwnedAc3Cplinu::Coupled)
                .unwrap();
        assert_eq!(writer.cursor(), FrameBitCursor::new(21));
    }
    let before = short_output;
    let result;
    let cursor;
    {
        let mut writer =
            FrameBitWriter::new(&mut short_output, cplinu.next_field_cursor()).unwrap();
        result = write_i0_ac3_first_block_coupling_fields(
            &mut writer,
            cplinu,
            coupling_fields_input(&case, &band_structure),
        );
        cursor = writer.cursor();
    }
    assert_eq!(
        result,
        Err(I0Ac3CouplingFieldsError::BitWriter(
            FrameBitWriterError::BufferTooSmall
        ))
    );
    assert_eq!(short_output, before);
    assert_eq!(cursor, cplinu.next_field_cursor());
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
