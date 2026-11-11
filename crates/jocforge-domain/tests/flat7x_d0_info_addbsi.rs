// pattern: Functional Core
//! Wash-pinned input-only vectors for the D0 informational BSI/addbsi suffix.

use jocforge_domain::{
    codec::{
        bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
        frame_builder::{
            Flat7xD0AudioProductionMetadata, Flat7xD0InfoAddbsiError, Flat7xD0InformationMetadata,
            append_flat7x_d0_info_and_addbsi, append_flat7x_d0_mixing_metadata_and_infomdate,
            write_flat7x_d0_bsi_through_chanmap,
        },
    },
    profile::Flat7xI0D0TargetPlan,
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/EAC3-DEPENDENT-D0-INFO-ADDBSI-SEGMENT-V1/INPUT-ONLY-VECTORS.json"
);

fn vector<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap()
}

fn information(value: &Value) -> Option<Flat7xD0InformationMetadata> {
    if value.is_null() {
        return None;
    }
    let audio_production = &value["audio_production"];
    Some(Flat7xD0InformationMetadata {
        bsmod: value["bsmod"].as_u64().unwrap() as u8,
        copyrightb: value["copyrightb"].as_bool().unwrap(),
        origbs: value["origbs"].as_bool().unwrap(),
        audio_production: if audio_production.is_null() {
            None
        } else {
            Some(Flat7xD0AudioProductionMetadata {
                mixlevel: audio_production["mixlevel"].as_u64().unwrap() as u8,
                roomtyp: audio_production["roomtyp"].as_u64().unwrap() as u8,
                convtyp: audio_production["convtyp"].as_u64().unwrap() as u8,
            })
        },
        sourcefscod: value["sourcefscod"].as_u64().unwrap() as u8,
    })
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => panic!("invalid hexadecimal test byte"),
            };
            (digit(pair[0]) << 4) | digit(pair[1])
        })
        .collect()
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

fn byte_bits(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 8);
    for byte in bytes {
        for shift in (0..8).rev() {
            output.push(if (byte >> shift) & 1 == 0 { '0' } else { '1' });
        }
    }
    output
}

fn parent_cursor(bytes: &mut [u8], infomdate: bool) -> u32 {
    let mut writer = FrameBitWriter::new(bytes, FrameBitCursor::new(0)).unwrap();
    write_flat7x_d0_bsi_through_chanmap(&mut writer, 16, None).unwrap();
    append_flat7x_d0_mixing_metadata_and_infomdate(
        &mut writer,
        Flat7xI0D0TargetPlan::approved_contract(),
        None,
        [None; 6],
        infomdate,
    )
    .unwrap()
    .bit_offset()
}

#[test]
fn suffix_vectors_match_caller_owned_information_and_opaque_bytes() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(
        vectors["schema"],
        "jocforge.eac3.flat7x_d0_info_addbsi_suffix.input_only.v1"
    );
    assert_eq!(vectors["contains_media_or_audio_payload"], false);

    for case_id in [
        "infomdate-absent-no-addbsi",
        "informational-fields-and-two-byte-addbsi",
        "audio-production-fields-without-addbsi",
    ] {
        let case = vector(&vectors, case_id);
        let inputs = &case["inputs"];
        let info = information(&inputs["information"]);
        let addbsi = inputs["addbsi_hex"].as_str().map(decode_hex);
        let start = case["start_bit_offset"].as_u64().unwrap() as u32;
        let mut bytes = vec![0_u8; 16];
        if start == 71 {
            assert_eq!(
                parent_cursor(&mut bytes, inputs["infomdate_present"].as_bool().unwrap()),
                start
            );
        }
        let end;
        {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
            end = append_flat7x_d0_info_and_addbsi(
                &mut writer,
                Flat7xI0D0TargetPlan::approved_contract(),
                inputs["infomdate_present"].as_bool().unwrap(),
                info,
                addbsi.as_deref(),
            )
            .unwrap()
            .bit_offset();
        }
        let suffix_bits = case["expected_suffix_bit_count"].as_u64().unwrap() as u32;
        assert_eq!(
            end,
            case["expected_end_bit_offset"].as_u64().unwrap() as u32
        );
        assert_eq!(
            written_bits(&bytes, start, suffix_bits),
            case["expected_suffix_bits"].as_str().unwrap()
        );
    }
}

#[test]
fn information_and_addbsi_rejections_leave_bytes_and_cursor_unchanged() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    let rejected = vector(&vectors, "capacity-and-range-rejections");
    let base = Flat7xD0InformationMetadata {
        bsmod: 3,
        copyrightb: true,
        origbs: false,
        audio_production: Some(Flat7xD0AudioProductionMetadata {
            mixlevel: 18,
            roomtyp: 1,
            convtyp: 1,
        }),
        sourcefscod: 1,
    };

    let mut invalid = base;
    invalid.bsmod = rejected["invalid_information_values"]["bsmod"]
        .as_u64()
        .unwrap() as u8;
    assert_rejected_unchanged(
        true,
        Some(invalid),
        None,
        Flat7xD0InfoAddbsiError::FieldOutOfRange {
            field: "bsmod",
            value: invalid.bsmod,
        },
    );

    for (field, value) in [
        (
            "mixlevel",
            rejected["invalid_information_values"]["mixlevel"]
                .as_u64()
                .unwrap() as u8,
        ),
        (
            "roomtyp",
            rejected["invalid_information_values"]["roomtyp"]
                .as_u64()
                .unwrap() as u8,
        ),
        (
            "convtyp",
            rejected["invalid_information_values"]["convtyp"]
                .as_u64()
                .unwrap() as u8,
        ),
    ] {
        let mut invalid = base;
        let audio_production = invalid.audio_production.as_mut().unwrap();
        match field {
            "mixlevel" => audio_production.mixlevel = value,
            "roomtyp" => audio_production.roomtyp = value,
            "convtyp" => audio_production.convtyp = value,
            _ => unreachable!(),
        }
        assert_rejected_unchanged(
            true,
            Some(invalid),
            None,
            Flat7xD0InfoAddbsiError::FieldOutOfRange { field, value },
        );
    }

    let mut invalid = base;
    invalid.sourcefscod = rejected["invalid_information_values"]["sourcefscod"]
        .as_u64()
        .unwrap() as u8;
    assert_rejected_unchanged(
        true,
        Some(invalid),
        None,
        Flat7xD0InfoAddbsiError::FieldOutOfRange {
            field: "sourcefscod",
            value: invalid.sourcefscod,
        },
    );

    for length in rejected["invalid_addbsi_lengths"].as_array().unwrap() {
        let length = length.as_u64().unwrap() as usize;
        assert_rejected_unchanged(
            false,
            None,
            Some(vec![0x5a; length]),
            Flat7xD0InfoAddbsiError::AddbsiLengthOutOfRange { bytes: length },
        );
    }
}

#[test]
fn infomdate_presence_must_match_information_option_before_mutation() {
    let information = Flat7xD0InformationMetadata {
        bsmod: 0,
        copyrightb: false,
        origbs: false,
        audio_production: None,
        sourcefscod: 0,
    };
    for (infomdate, input, expected) in [
        (
            true,
            None,
            Flat7xD0InfoAddbsiError::InfomdateInputMismatch {
                infomdate: true,
                information_present: false,
            },
        ),
        (
            false,
            Some(information),
            Flat7xD0InfoAddbsiError::InfomdateInputMismatch {
                infomdate: false,
                information_present: true,
            },
        ),
    ] {
        assert_rejected_unchanged(infomdate, input, None, expected);
    }
}

#[test]
fn addbsi_length_endpoints_fit_the_six_bit_length_field() {
    for length in [1, 64] {
        let addbsi = vec![0xa5; length];
        let mut bytes = vec![0_u8; 66];
        let end;
        {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
            end = append_flat7x_d0_info_and_addbsi(
                &mut writer,
                Flat7xI0D0TargetPlan::approved_contract(),
                false,
                None,
                Some(&addbsi),
            )
            .unwrap()
            .bit_offset();
        }
        assert_eq!(end, 7 + (length as u32 * 8));
        let header_bits = written_bits(&bytes, 0, 7);
        assert_eq!(&header_bits[..1], "1");
        assert_eq!(
            u32::from_str_radix(&header_bits[1..], 2).unwrap(),
            u32::try_from(length - 1).unwrap()
        );
        assert_eq!(
            written_bits(&bytes, 7, length as u32 * 8),
            byte_bits(&addbsi)
        );
    }
}

#[test]
fn insufficient_suffix_capacity_is_rejected_atomically() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    let capacity = vector(&vectors, "capacity-and-range-rejections");
    let start = capacity["start_bit_offset"].as_u64().unwrap() as u32;
    let mut bytes = vec![0xa5_u8; capacity["buffer_bytes"].as_u64().unwrap() as usize];
    let original = bytes.clone();
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        error = append_flat7x_d0_info_and_addbsi(
            &mut writer,
            Flat7xI0D0TargetPlan::approved_contract(),
            true,
            Some(Flat7xD0InformationMetadata {
                bsmod: 3,
                copyrightb: true,
                origbs: false,
                audio_production: None,
                sourcefscod: 1,
            }),
            Some(&[0x12, 0x34]),
        )
        .unwrap_err();
        cursor = writer.cursor();
    }
    assert_eq!(
        error,
        Flat7xD0InfoAddbsiError::BitWriter(FrameBitWriterError::BufferTooSmall)
    );
    assert_eq!(cursor.bit_offset(), start);
    assert_eq!(bytes, original);
}

fn assert_rejected_unchanged(
    infomdate: bool,
    information: Option<Flat7xD0InformationMetadata>,
    addbsi: Option<Vec<u8>>,
    expected_error: Flat7xD0InfoAddbsiError,
) {
    let mut bytes = vec![0x5a_u8; 80];
    let original = bytes.clone();
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(71)).unwrap();
        error = append_flat7x_d0_info_and_addbsi(
            &mut writer,
            Flat7xI0D0TargetPlan::approved_contract(),
            infomdate,
            information,
            addbsi.as_deref(),
        )
        .unwrap_err();
        cursor = writer.cursor();
    }
    assert_eq!(error, expected_error);
    assert_eq!(cursor.bit_offset(), 71);
    assert_eq!(bytes, original);
}
