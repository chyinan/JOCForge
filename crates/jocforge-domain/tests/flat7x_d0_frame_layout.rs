// pattern: Functional Core
//! Synthetic, input-only layout cases for the caller-owned D0 block-two insertion boundary.

use jocforge_domain::{
    codec::{
        bitio::{FrameBitCursor, FrameBitWriter},
        d0_frame_layout::{
            CallerOwnedUnboundD0Block2InsertionPoint, D0AudioBlockBitSpan, D0AudioBlockLayoutInput,
            D0Block2SkipFieldLayoutInput, D0SixBlockLayoutError, D0SixBlockLayoutInput,
            propose_flat7x_d0_six_block_layout,
        },
        emdf_skip_field::{
            D0EmdfSkipFieldWriteError, flat7x_d0_block2_skipfld_bit_length,
            write_flat7x_d0_block2_skipfld,
        },
        frame_builder::Flat7xD0BsiStructure,
    },
    profile::Flat7xI0D0TargetPlan,
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &str =
    include_str!("fixtures/flat7x_d0_frame_layout/INPUT-ONLY-VECTORS.json");

fn vector<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap()
}

fn audio_block_lengths(case: &Value) -> [u32; 6] {
    case["audio_block_bit_lengths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn six_block_input(
    lengths: [u32; 6],
    fields_by_block: [&[D0Block2SkipFieldLayoutInput]; 6],
) -> D0SixBlockLayoutInput<'_> {
    D0SixBlockLayoutInput {
        blocks: std::array::from_fn(|index| D0AudioBlockLayoutInput {
            audio_span: D0AudioBlockBitSpan::new(lengths[index]),
            skip_fields: fields_by_block[index],
        }),
    }
}

fn fields_by_block(case: &Value) -> [Vec<D0Block2SkipFieldLayoutInput>; 6] {
    let mut fields: [Vec<D0Block2SkipFieldLayoutInput>; 6] = std::array::from_fn(|_| Vec::new());
    for item in case["skip_fields"].as_array().unwrap() {
        let block_index = usize::try_from(item["block_index"].as_u64().unwrap()).unwrap();
        fields[block_index].push(D0Block2SkipFieldLayoutInput {
            insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(
                u32::try_from(item["insertion_offset_bits"].as_u64().unwrap()).unwrap(),
            ),
            payload_bytes: u16::try_from(item["payload_bytes"].as_u64().unwrap()).unwrap(),
        });
    }
    fields
}

fn block_field_refs(
    fields: &[Vec<D0Block2SkipFieldLayoutInput>; 6],
) -> [&[D0Block2SkipFieldLayoutInput]; 6] {
    std::array::from_fn(|index| fields[index].as_slice())
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

#[test]
fn synthetic_layout_vectors_define_block_order_and_caller_owned_insertion() {
    let vectors: Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(
        vectors["schema"],
        "jocforge.flat7x_d0_six_block_layout.synthetic_input_only.v1"
    );
    assert!(!vectors["contains_audio_or_pcm_bytes"].as_bool().unwrap());
    assert_eq!(
        vectors["native_insertion_cursor_claim"],
        "UNBOUND_CALLER_OWNED_OFFSET_ONLY"
    );

    for case_id in ["valid-single-block-two-field", "valid-u9-maximum-container"] {
        let case = vector(&vectors, case_id);
        let fields = fields_by_block(case);
        let input = six_block_input(audio_block_lengths(case), block_field_refs(&fields));
        let layout = propose_flat7x_d0_six_block_layout(
            Flat7xI0D0TargetPlan::approved_contract(),
            baseline_bsi_structure(),
            input,
        )
        .unwrap();
        let expected = &case["expected"];
        assert_eq!(
            layout.bsi_span().end_bit(),
            expected["bsi_end_bit"].as_u64().unwrap() as u32
        );
        assert_eq!(
            layout.block_two_skip_field().bit_span().start_bit(),
            expected["skip_field_start_bit"].as_u64().unwrap() as u32
        );
        assert_eq!(
            layout.block_two_skip_field().bit_span().bit_length(),
            expected["skip_field_bit_length"].as_u64().unwrap() as u32
        );
        assert_eq!(
            layout.total_frame_span_bits(),
            expected["total_frame_span_bits"].as_u64().unwrap() as u32
        );
        assert_eq!(layout.target_frame_bytes(), 4096);
        assert!(!layout.production_allowed());
        assert_eq!(
            layout
                .audio_blocks()
                .map(|block| block.block_index().zero_based_value()),
            [0, 1, 2, 3, 4, 5]
        );
    }
}

#[test]
fn synthetic_layout_vectors_reject_missing_duplicate_misplaced_and_oversized_fields() {
    let vectors: Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
    let missing = vector(&vectors, "reject-missing-block-two-field");
    assert_eq!(
        plan_error(missing),
        D0SixBlockLayoutError::MissingBlockTwoSkipField
    );

    let duplicate = vector(&vectors, "reject-duplicate-block-two-fields");
    assert_eq!(
        plan_error(duplicate),
        D0SixBlockLayoutError::DuplicateBlockTwoSkipFields
    );

    let misplaced = vector(&vectors, "reject-field-outside-block-two");
    assert!(matches!(
        plan_error(misplaced),
        D0SixBlockLayoutError::SkipFieldOnNonBlockTwo { block_index }
            if block_index.zero_based_value() == 1
    ));

    let outside = vector(&vectors, "reject-caller-insertion-outside-block-two-span");
    assert!(matches!(
        plan_error(outside),
        D0SixBlockLayoutError::InsertionPointOutsideAudioSpan {
            offset_bits: 5,
            span_bits: 4
        }
    ));

    let too_large = vector(&vectors, "reject-u9-overflow-container");
    assert_eq!(
        plan_error(too_large),
        D0SixBlockLayoutError::SkipField(D0EmdfSkipFieldWriteError::ContainerTooLarge)
    );
}

#[test]
fn synthetic_layout_vectors_reject_frame_limit_and_arithmetic_overflow() {
    let vectors: Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
    let oversized = vector(&vectors, "reject-frame-span-over-target");
    assert!(matches!(
        plan_error(oversized),
        D0SixBlockLayoutError::FrameSpanExceedsTarget {
            maximum_bits: 32_768,
            ..
        }
    ));

    let overflow = vector(&vectors, "reject-frame-span-arithmetic-overflow");
    assert_eq!(plan_error(overflow), D0SixBlockLayoutError::BitSpanOverflow);
}

#[test]
fn skip_field_bit_extent_matches_the_existing_writer_for_synthetic_containers() {
    for payload_bytes in [1_usize, 511] {
        let synthetic_container = vec![0xa5; payload_bytes];
        let payload_len = u16::try_from(payload_bytes).unwrap();
        let expected = flat7x_d0_block2_skipfld_bit_length(payload_len).unwrap();
        let mut scratch = [0_u8; 513];
        let end = {
            let mut writer = FrameBitWriter::new(&mut scratch, FrameBitCursor::new(0)).unwrap();
            write_flat7x_d0_block2_skipfld(&mut writer, &synthetic_container)
                .unwrap()
                .bit_offset()
        };
        assert_eq!(end, expected);
        assert_eq!(end, 10 + (payload_bytes as u32 * 8));
    }
}

fn plan_error(case: &Value) -> D0SixBlockLayoutError {
    let fields = fields_by_block(case);
    let input = six_block_input(audio_block_lengths(case), block_field_refs(&fields));
    propose_flat7x_d0_six_block_layout(
        Flat7xI0D0TargetPlan::approved_contract(),
        baseline_bsi_structure(),
        input,
    )
    .unwrap_err()
}
