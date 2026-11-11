// pattern: Functional Core
//! Wash-pinned input-only vectors for the D0 mixing-metadata and infomdate tail.

use jocforge_domain::{
    codec::{
        bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
        frame_builder::{
            Flat7xD0BsiSegmentError, Flat7xD0MixingMetadataError, Flat7xD0MixingMetadataValues,
            append_flat7x_d0_mixing_metadata_and_infomdate, write_flat7x_d0_bsi_through_chanmap,
        },
    },
    profile::Flat7xI0D0TargetPlan,
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/EAC3-DEPENDENT-D0-MIXING-METADATA-V1/INPUT-ONLY-VECTORS.json"
);

fn vector<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap()
}

fn mixing_values(value: &Value) -> Option<Flat7xD0MixingMetadataValues> {
    if value.is_null() {
        return None;
    }
    Some(Flat7xD0MixingMetadataValues {
        mixmdo: value["mixmdo"].as_u64().unwrap() as u8,
        ltrtcmixlev: value["ltrtcmixlev"].as_u64().unwrap() as u8,
        lorocmixlev: value["lorocmixlev"].as_u64().unwrap() as u8,
        ltrtsurmixlev: value["ltrtsurmixlev"].as_u64().unwrap() as u8,
        lorosurmixlev: value["lorosurmixlev"].as_u64().unwrap() as u8,
    })
}

fn block_mix_values(value: &Value) -> [Option<u8>; 6] {
    let values = value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_u64().map(|number| number as u8))
        .collect::<Vec<_>>();
    values.try_into().unwrap()
}

fn written_bits(bytes: &[u8], start_bit: u32, bit_count: u32) -> String {
    (0..bit_count)
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

fn append_parent_segment(bytes: &mut [u8], compr: Option<u8>) -> u32 {
    let mut writer = FrameBitWriter::new(bytes, FrameBitCursor::new(0)).unwrap();
    write_flat7x_d0_bsi_through_chanmap(&mut writer, 16, compr).unwrap();
    writer.cursor().bit_offset()
}

fn append_vector_segment(vector: &Value, parent_compr: Option<u8>) -> (Vec<u8>, u32, u32) {
    let inputs = &vector["inputs"];
    let mut bytes = vec![0_u8; 16];
    let start = append_parent_segment(&mut bytes, parent_compr);
    assert_eq!(start, vector["start_bit_offset"]);
    let end;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        end = append_flat7x_d0_mixing_metadata_and_infomdate(
            &mut writer,
            Flat7xI0D0TargetPlan::approved_contract(),
            mixing_values(&inputs["mixing"]),
            block_mix_values(&inputs["block_mix"]),
            inputs["infomdate"].as_bool().unwrap(),
        )
        .unwrap();
    }
    (bytes, start, end.bit_offset())
}

#[test]
fn mixing_and_infomdate_vectors_match_after_parent_bsi_segment() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(
        vectors["schema"],
        "jocforge.eac3.flat7x_d0_mixing_info_segment.input_only.v1"
    );

    let cases = [
        ("no-mixing-no-block-info", None),
        ("mixing-present-infomdate-present", None),
        ("block-mix-values-without-mixing", Some(165)),
    ];
    for (case_id, parent_compr) in cases {
        let case = vector(&vectors, case_id);
        let (bytes, start, end) = append_vector_segment(case, parent_compr);
        assert_eq!(
            end,
            case["expected_end_bit_offset"].as_u64().unwrap() as u32
        );
        assert_eq!(
            written_bits(
                &bytes,
                start,
                case["expected_segment_bit_count"].as_u64().unwrap() as u32,
            ),
            case["expected_segment_bits"]
        );
    }
}

#[test]
fn all_input_ranges_fail_before_mutation() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    let rejected = vector(&vectors, "preflight-and-range-rejections");

    for invalid_start in rejected["invalid_start_offsets"].as_array().unwrap() {
        let start = invalid_start.as_u64().unwrap() as u32;
        let mut bytes = vec![0xa5_u8; 16];
        let original = bytes.clone();
        let error;
        let cursor;
        {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
            error = append_flat7x_d0_mixing_metadata_and_infomdate(
                &mut writer,
                Flat7xI0D0TargetPlan::approved_contract(),
                None,
                [None; 6],
                false,
            )
            .unwrap_err();
            cursor = writer.cursor();
        }
        assert_eq!(
            error,
            Flat7xD0MixingMetadataError::ParentSegmentCursorMismatch { actual: start }
        );
        assert_eq!(cursor.bit_offset(), start);
        assert_eq!(bytes, original);
    }

    let mut invalid_mixmdo = Flat7xD0MixingMetadataValues {
        mixmdo: 4,
        ltrtcmixlev: 0,
        lorocmixlev: 0,
        ltrtsurmixlev: 0,
        lorosurmixlev: 0,
    };
    assert_segment_rejected_before_mutation(
        Some(invalid_mixmdo),
        [None; 6],
        Flat7xD0MixingMetadataError::MixmdoOutOfRange { value: 4 },
    );
    invalid_mixmdo.mixmdo = 0;

    for invalid_level in rejected["invalid_mix_levels"].as_array().unwrap() {
        let value = invalid_level.as_u64().unwrap() as u8;
        let mut mixing = invalid_mixmdo;
        mixing.ltrtcmixlev = value;
        assert_segment_rejected_before_mutation(
            Some(mixing),
            [None; 6],
            Flat7xD0MixingMetadataError::MixLevelOutOfRange { value },
        );
    }
    for invalid_value in rejected["invalid_block_mix_values"].as_array().unwrap() {
        let value = invalid_value.as_u64().unwrap() as u8;
        let mut block_mix = [None; 6];
        block_mix[0] = Some(value);
        assert_segment_rejected_before_mutation(
            None,
            block_mix,
            Flat7xD0MixingMetadataError::BlockMixValueOutOfRange { block: 0, value },
        );
    }
}

fn assert_segment_rejected_before_mutation(
    mixing: Option<Flat7xD0MixingMetadataValues>,
    block_mix: [Option<u8>; 6],
    expected_error: Flat7xD0MixingMetadataError,
) {
    let mut bytes = vec![0x5a_u8; 16];
    let start = append_parent_segment(&mut bytes, None);
    let original = bytes.clone();
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        error = append_flat7x_d0_mixing_metadata_and_infomdate(
            &mut writer,
            Flat7xI0D0TargetPlan::approved_contract(),
            mixing,
            block_mix,
            false,
        )
        .unwrap_err();
        cursor = writer.cursor();
    }
    assert_eq!(error, expected_error);
    assert_eq!(cursor.bit_offset(), start);
    assert_eq!(bytes, original);
}

#[test]
fn insufficient_segment_capacity_is_rejected_without_partial_write() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    let capacity = vector(&vectors, "preflight-and-range-rejections");
    let available_bits = capacity["available_bits"].as_u64().unwrap() as u32;
    let required_bits = capacity["required_bits"].as_u64().unwrap() as u32;
    assert_eq!(available_bits, 12);
    assert_eq!(required_bits, 17);

    let buffer_bytes = capacity["buffer_bytes"].as_u64().unwrap() as usize;
    let mut bytes = vec![0xa5_u8; buffer_bytes];
    let start = append_parent_segment(&mut bytes, None);
    let actual_available_bits = u32::try_from(bytes.len()).unwrap() * 8 - start;
    assert_eq!(actual_available_bits, available_bits);
    assert!(actual_available_bits < required_bits);
    let original = bytes.clone();
    let mixing = Some(Flat7xD0MixingMetadataValues {
        mixmdo: 2,
        ltrtcmixlev: 1,
        lorocmixlev: 2,
        ltrtsurmixlev: 3,
        lorosurmixlev: 4,
    });
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        error = append_flat7x_d0_mixing_metadata_and_infomdate(
            &mut writer,
            Flat7xI0D0TargetPlan::approved_contract(),
            mixing,
            [None; 6],
            true,
        )
        .unwrap_err();
        cursor = writer.cursor();
    }

    assert_eq!(
        error,
        Flat7xD0MixingMetadataError::BitWriter(FrameBitWriterError::BufferTooSmall)
    );
    assert_eq!(cursor.bit_offset(), start);
    assert_eq!(bytes, original);
}
