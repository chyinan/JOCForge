// pattern: Functional Core
//! Synthetic, non-playable tests for caller-supplied D0 frame bit assembly.

use jocforge_domain::{
    codec::{
        access_unit_composer::{
            Flat7xI0D0OpaqueFrameSpans, OpaqueD0FrameBytes, OpaqueI0FrameBytes,
            write_flat7x_i0_d0_access_unit_bytes,
        },
        bitio::{FrameBitCursor, FrameBitWriter},
        d0_frame_assembler::{
            D0Block2SkipFieldInput, D0BlockTwoAudioSpans, D0FrameAudioBlockInput,
            Flat7xD0FrameAssemblyError, Flat7xD0FrameBitInput, OpaqueD0AudioBitSpan,
            assemble_flat7x_d0_frame_bits,
        },
        d0_frame_layout::{CallerOwnedUnboundD0Block2InsertionPoint, D0SixBlockLayoutError},
        elementary_stream_composer::{
            Flat7xOpaqueAccessUnitBytes, compose_flat7x_elementary_stream,
        },
        frame_builder::Flat7xD0BsiStructure,
    },
    profile::Flat7xI0D0TargetPlan,
};
use serde_json::Value;

const FRAME_VECTORS: &str =
    include_str!("fixtures/flat7x_d0_frame_layout/FRAME-ASSEMBLY-INPUT-ONLY-VECTORS.json");

#[derive(Clone, Debug)]
struct SyntheticFrameData {
    audio_blocks: [Vec<u8>; 6],
    block_two_pre: Vec<u8>,
    block_two_post: Vec<u8>,
    skipfield_container: Vec<u8>,
}

fn frame_case() -> Value {
    let vectors: Value = serde_json::from_str(FRAME_VECTORS).unwrap();
    assert_eq!(
        vectors["schema"],
        "jocforge.flat7x_d0_frame_assembler.synthetic_input_only.v1"
    );
    assert_eq!(vectors["status"], "SYNTHETIC_NOT_REAL_OUTPUT");
    assert!(
        !vectors["contains_pcm_or_real_audio_bytes"]
            .as_bool()
            .unwrap()
    );
    vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "synthetic-exact-4096-byte-frame")
        .unwrap()
        .clone()
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

fn parse_hex_byte(value: &Value) -> u8 {
    u8::from_str_radix(value.as_str().unwrap(), 16).unwrap()
}

fn bytes_for_bits(bit_length: u32, pattern: u8) -> Vec<u8> {
    let byte_length = usize::try_from(bit_length / 8 + u32::from(bit_length % 8 != 0)).unwrap();
    vec![pattern; byte_length]
}

fn synthetic_frame_data(case: &Value) -> SyntheticFrameData {
    let lengths = bit_lengths(case);
    let patterns = case["audio_block_test_byte_patterns_hex"]
        .as_array()
        .unwrap();
    let audio_blocks = std::array::from_fn(|index| {
        bytes_for_bits(lengths[index], parse_hex_byte(&patterns[index]))
    });
    SyntheticFrameData {
        audio_blocks,
        block_two_pre: bytes_for_bits(
            case["block_two_pre_skip_bits"].as_u64().unwrap() as u32,
            parse_hex_byte(&case["block_two_pre_test_pattern_hex"]),
        ),
        block_two_post: bytes_for_bits(
            case["block_two_post_skip_bits"].as_u64().unwrap() as u32,
            parse_hex_byte(&case["block_two_post_test_pattern_hex"]),
        ),
        skipfield_container: vec![parse_hex_byte(
            &case["block_two_skipfield_test_container_hex"],
        )],
    }
}

fn baseline_bsi_structure() -> Flat7xD0BsiStructure<'static> {
    Flat7xD0BsiStructure {
        dialnorm: 16,
        compr: None,
        mixing: None,
        block_mix: [None; 6],
        information: None,
        addbsi: None,
    }
}

fn frame_input<'a>(
    data: &'a SyntheticFrameData,
    lengths: [u32; 6],
    source_start_bits: [u32; 6],
    insertion_offset_bits: u32,
    skipfield_fields: &'a [D0Block2SkipFieldInput<'a>],
    crc2: u16,
    block_two_at_index: usize,
) -> Flat7xD0FrameBitInput<'a> {
    assert_eq!(
        lengths[2],
        u32::try_from(data.block_two_pre.len() * 8 + data.block_two_post.len() * 8,).unwrap()
    );
    if let [field] = skipfield_fields {
        assert_eq!(
            field.insertion_point.offset_within_audio_span_bits(),
            insertion_offset_bits
        );
    }
    let audio_blocks = std::array::from_fn(|index| {
        if index == block_two_at_index {
            D0FrameAudioBlockInput::BlockTwo(D0BlockTwoAudioSpans {
                before_skipfield: OpaqueD0AudioBitSpan::new(
                    &data.block_two_pre,
                    0,
                    u32::try_from(data.block_two_pre.len() * 8).unwrap(),
                ),
                skip_fields: skipfield_fields,
                after_skipfield: OpaqueD0AudioBitSpan::new(
                    &data.block_two_post,
                    0,
                    u32::try_from(data.block_two_post.len() * 8).unwrap(),
                ),
            })
        } else {
            D0FrameAudioBlockInput::Opaque(OpaqueD0AudioBitSpan::new(
                &data.audio_blocks[index],
                source_start_bits[index],
                lengths[index],
            ))
        }
    });
    Flat7xD0FrameBitInput { audio_blocks, crc2 }
}

fn read_bit(bytes: &[u8], bit_offset: u32) -> bool {
    bytes[(bit_offset / 8) as usize] & (1_u8 << (7 - bit_offset % 8)) != 0
}

fn assert_span_copied(
    frame: &[u8],
    frame_start: u32,
    source: &[u8],
    source_start: u32,
    bit_length: u32,
) {
    for offset in 0..bit_length {
        assert_eq!(
            read_bit(frame, frame_start + offset),
            read_bit(source, source_start + offset)
        );
    }
}

#[test]
fn synthetic_assembler_copies_opaque_spans_and_caller_crc2_into_exact_target_extent() {
    let case = frame_case();
    let data = synthetic_frame_data(&case);
    let lengths = bit_lengths(&case);
    let insertion_offset = case["block_two_insertion_offset_bits"].as_u64().unwrap() as u32;
    let crc2 = u16::from_str_radix(
        case["crc2_u16"].as_str().unwrap().trim_start_matches("0x"),
        16,
    )
    .unwrap();
    let skipfield_fields = [D0Block2SkipFieldInput {
        insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(insertion_offset),
        complete_emdf_container: &data.skipfield_container,
    }];
    let input = frame_input(
        &data,
        lengths,
        [0; 6],
        insertion_offset,
        &skipfield_fields,
        crc2,
        2,
    );
    let mut output = vec![0xa5; 4096];
    let result;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut output, FrameBitCursor::new(0)).unwrap();
        result = assemble_flat7x_d0_frame_bits(
            Flat7xI0D0TargetPlan::approved_contract(),
            baseline_bsi_structure(),
            input,
            &mut writer,
        );
        cursor = writer.cursor().bit_offset();
    }
    let receipt = result.unwrap();
    let expected = &case["expected"];
    assert_eq!(receipt.total_frame_span_bits(), 32_768);
    assert_eq!(cursor, 32_768);
    assert_eq!(receipt.target_frame_bytes(), 4096);
    assert_eq!(receipt.layout().bsi_span().end_bit(), 72);
    assert_eq!(
        receipt.layout().total_frame_span_bits(),
        expected["pre_crc2_span_bits"].as_u64().unwrap() as u32
    );
    assert_eq!(
        receipt
            .layout()
            .block_two_skip_field()
            .bit_span()
            .start_bit(),
        expected["block_two_skipfield_start_bit"].as_u64().unwrap() as u32
    );
    assert_eq!(
        receipt
            .layout()
            .block_two_skip_field()
            .bit_span()
            .bit_length(),
        expected["block_two_skipfield_bit_length"].as_u64().unwrap() as u32
    );
    assert_eq!(receipt.crc2_span().start_bit(), 32_752);
    assert_eq!(receipt.crc2_span().end_bit(), 32_768);
    assert_eq!(&output[4094..], &[0xbe, 0xef]);
    assert!(!receipt.production_allowed());

    let block_layouts = receipt.layout().audio_blocks();
    assert_span_copied(
        &output,
        block_layouts[0].leading_audio_span().start_bit(),
        &data.audio_blocks[0],
        0,
        lengths[0],
    );
    assert_span_copied(
        &output,
        block_layouts[1].leading_audio_span().start_bit(),
        &data.audio_blocks[1],
        0,
        lengths[1],
    );
    assert_span_copied(
        &output,
        block_layouts[2].leading_audio_span().start_bit(),
        &data.block_two_pre,
        0,
        insertion_offset,
    );
    let block_two_after = block_layouts[2].trailing_audio_span().unwrap();
    assert_span_copied(
        &output,
        block_two_after.start_bit(),
        &data.block_two_post,
        0,
        lengths[2] - insertion_offset,
    );
    assert_span_copied(
        &output,
        block_layouts[3].leading_audio_span().start_bit(),
        &data.audio_blocks[3],
        0,
        lengths[3],
    );
    assert_span_copied(
        &output,
        block_layouts[4].leading_audio_span().start_bit(),
        &data.audio_blocks[4],
        0,
        lengths[4],
    );
    assert_span_copied(
        &output,
        block_layouts[5].leading_audio_span().start_bit(),
        &data.audio_blocks[5],
        0,
        lengths[5],
    );
}

#[test]
fn synthetic_d0_frame_composes_into_au_and_two_au_stream_with_crc2_preserved() {
    let case = frame_case();
    let data = synthetic_frame_data(&case);
    let lengths = bit_lengths(&case);
    let insertion_offset = case["block_two_insertion_offset_bits"].as_u64().unwrap() as u32;
    let crc2 = u16::from_str_radix(
        case["crc2_u16"].as_str().unwrap().trim_start_matches("0x"),
        16,
    )
    .unwrap();
    let crc2_bytes = crc2.to_be_bytes();
    let skipfield_fields = [D0Block2SkipFieldInput {
        insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(insertion_offset),
        complete_emdf_container: &data.skipfield_container,
    }];
    let d0_input = frame_input(
        &data,
        lengths,
        [0; 6],
        insertion_offset,
        &skipfield_fields,
        crc2,
        2,
    );
    let mut d0_frame = vec![0xa5; 4_096];
    let d0_assembly_receipt;
    {
        let mut writer = FrameBitWriter::new(&mut d0_frame, FrameBitCursor::new(0)).unwrap();
        d0_assembly_receipt = assemble_flat7x_d0_frame_bits(
            Flat7xI0D0TargetPlan::approved_contract(),
            baseline_bsi_structure(),
            d0_input,
            &mut writer,
        )
        .unwrap();
    }
    assert_eq!(d0_assembly_receipt.target_frame_bytes(), 4_096);
    assert_eq!(d0_assembly_receipt.crc2_span().start_bit(), 32_752);
    assert_eq!(d0_frame[4_094..], crc2_bytes);
    assert!(!d0_assembly_receipt.production_allowed());

    let first_i0 = vec![0x31; 2_560];
    let second_i0 = vec![0x52; 2_560];
    let mut first_au = vec![0xa5; 6_656];
    let mut second_au = vec![0xa5; 6_656];
    let first_au_receipt;
    let second_au_receipt;
    {
        let mut writer = FrameBitWriter::new(&mut first_au, FrameBitCursor::new(0)).unwrap();
        first_au_receipt = write_flat7x_i0_d0_access_unit_bytes(
            Flat7xI0D0TargetPlan::approved_contract(),
            Flat7xI0D0OpaqueFrameSpans {
                i0: OpaqueI0FrameBytes::new(&first_i0),
                d0: OpaqueD0FrameBytes::new(&d0_frame),
            },
            &mut writer,
        )
        .unwrap();
    }
    {
        let mut writer = FrameBitWriter::new(&mut second_au, FrameBitCursor::new(0)).unwrap();
        second_au_receipt = write_flat7x_i0_d0_access_unit_bytes(
            Flat7xI0D0TargetPlan::approved_contract(),
            Flat7xI0D0OpaqueFrameSpans {
                i0: OpaqueI0FrameBytes::new(&second_i0),
                d0: OpaqueD0FrameBytes::new(&d0_frame),
            },
            &mut writer,
        )
        .unwrap();
    }
    assert_eq!(first_au_receipt.d0_span().start_byte(), 2_560);
    assert_eq!(second_au_receipt.d0_span().byte_length(), 4_096);
    assert_eq!(&first_au[2_560..], &d0_frame);
    assert_eq!(&second_au[2_560..], &d0_frame);

    let access_units = [
        Flat7xOpaqueAccessUnitBytes::new(&first_au),
        Flat7xOpaqueAccessUnitBytes::new(&second_au),
    ];
    let mut stream = vec![0xa5; 2 * 6_656];
    let stream_receipt;
    {
        let mut writer = FrameBitWriter::new(&mut stream, FrameBitCursor::new(0)).unwrap();
        stream_receipt = compose_flat7x_elementary_stream(
            Flat7xI0D0TargetPlan::approved_contract(),
            &access_units,
            &mut writer,
        )
        .unwrap();
    }
    assert_eq!(stream_receipt.start_byte(), 0);
    assert_eq!(stream_receipt.extent().access_unit_count(), 2);
    assert_eq!(stream_receipt.extent().byte_length(), 13_312);
    assert!(!stream_receipt.production_allowed());

    for (index, (access_unit, composition_receipt)) in [
        (&first_au, first_au_receipt),
        (&second_au, second_au_receipt),
    ]
    .into_iter()
    .enumerate()
    {
        let stream_au_start = stream_receipt.start_byte()
            + u32::try_from(index).unwrap() * composition_receipt.total_byte_length();
        let stream_au_end = stream_au_start + composition_receipt.total_byte_length();
        let d0_start = stream_au_start + u32::from(composition_receipt.d0_span().start_byte());
        let d0_end = d0_start + u32::from(composition_receipt.d0_span().byte_length());
        assert_eq!(
            &stream[stream_au_start as usize..stream_au_end as usize],
            access_unit
        );
        assert_eq!(&stream[d0_start as usize..d0_end as usize], &d0_frame);
        assert_eq!(&stream[d0_end as usize - 2..d0_end as usize], &crc2_bytes);
    }
}

#[test]
fn assembler_errors_leave_caller_bytes_and_cursor_unchanged() {
    let case = frame_case();
    let data = synthetic_frame_data(&case);
    let lengths = bit_lengths(&case);
    let crc2 = 0xbeef;

    let field = D0Block2SkipFieldInput {
        insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(4096),
        complete_emdf_container: &data.skipfield_container,
    };
    let one_field = [field];

    let mut short_lengths = lengths;
    short_lengths[5] -= 1;
    assert_rejected_unchanged(
        &data,
        short_lengths,
        [0; 6],
        4096,
        &one_field,
        crc2,
        2,
        4096,
        0,
        Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: 32_767,
            required_bits: 32_768,
        },
    );

    assert_rejected_unchanged(
        &data,
        lengths,
        [0; 6],
        4096,
        &[],
        crc2,
        2,
        4096,
        0,
        Flat7xD0FrameAssemblyError::BlockTwoMissing,
    );

    let duplicate_fields = [field, field];
    assert_rejected_unchanged(
        &data,
        lengths,
        [0; 6],
        4096,
        &duplicate_fields,
        crc2,
        2,
        4096,
        0,
        Flat7xD0FrameAssemblyError::DuplicateBlockTwoSkipFields,
    );

    assert_rejected_unchanged(
        &data,
        lengths,
        [0; 6],
        4096,
        &one_field,
        crc2,
        1,
        4096,
        0,
        Flat7xD0FrameAssemblyError::BlockTwoAtWrongIndex { actual: 1 },
    );

    let mut too_large_container_data = data.clone();
    too_large_container_data.skipfield_container = vec![0xa5; 512];
    let oversized_field = [D0Block2SkipFieldInput {
        insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(4096),
        complete_emdf_container: &too_large_container_data.skipfield_container,
    }];
    assert_rejected_unchanged(
        &too_large_container_data,
        lengths,
        [0; 6],
        4096,
        &oversized_field,
        crc2,
        2,
        4096,
        0,
        Flat7xD0FrameAssemblyError::Layout(D0SixBlockLayoutError::SkipField(
            jocforge_domain::codec::emdf_skip_field::D0EmdfSkipFieldWriteError::ContainerTooLarge,
        )),
    );

    assert_rejected_unchanged(
        &data,
        lengths,
        [0, 0, 0, 0, 0, 4],
        4096,
        &one_field,
        crc2,
        2,
        4096,
        0,
        Flat7xD0FrameAssemblyError::SourceSpanOutOfBounds {
            block_index: 5,
            required_end_bit: 2_666,
            available_source_bits: 2_664,
        },
    );

    let wrong_insertion = [D0Block2SkipFieldInput {
        insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(4095),
        complete_emdf_container: &data.skipfield_container,
    }];
    assert_rejected_unchanged(
        &data,
        lengths,
        [0; 6],
        4095,
        &wrong_insertion,
        crc2,
        2,
        4096,
        0,
        Flat7xD0FrameAssemblyError::BlockTwoInsertionPointMismatch {
            declared_bits: 4095,
            pre_span_bits: 4096,
        },
    );

    assert_rejected_unchanged(
        &data,
        lengths,
        [0; 6],
        4096,
        &one_field,
        crc2,
        2,
        4095,
        0,
        Flat7xD0FrameAssemblyError::BitWriter(
            jocforge_domain::codec::bitio::FrameBitWriterError::BufferTooSmall,
        ),
    );

    assert_rejected_unchanged(
        &data,
        lengths,
        [0; 6],
        4096,
        &one_field,
        crc2,
        2,
        4096,
        1,
        Flat7xD0FrameAssemblyError::CursorMismatch { actual: 1 },
    );
}

fn assert_rejected_unchanged(
    data: &SyntheticFrameData,
    lengths: [u32; 6],
    source_starts: [u32; 6],
    insertion_offset: u32,
    skip_fields: &[D0Block2SkipFieldInput<'_>],
    crc2: u16,
    block_two_at_index: usize,
    output_bytes: usize,
    start_cursor: u32,
    expected_error: Flat7xD0FrameAssemblyError,
) {
    let input = frame_input(
        data,
        lengths,
        source_starts,
        insertion_offset,
        skip_fields,
        crc2,
        block_two_at_index,
    );
    let mut output = vec![0x5a; output_bytes];
    let original = output.clone();
    let result;
    let cursor;
    {
        let mut writer =
            FrameBitWriter::new(&mut output, FrameBitCursor::new(start_cursor)).unwrap();
        result = assemble_flat7x_d0_frame_bits(
            Flat7xI0D0TargetPlan::approved_contract(),
            baseline_bsi_structure(),
            input,
            &mut writer,
        );
        cursor = writer.cursor().bit_offset();
    }
    assert_eq!(result.unwrap_err(), expected_error);
    assert_eq!(cursor, start_cursor);
    assert_eq!(output, original);
}
