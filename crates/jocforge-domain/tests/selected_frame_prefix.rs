use jocforge_domain::codec::bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError};
use jocforge_domain::codec::frame_builder::{
    SelectedEac3PrefixError, write_selected_eac3_fixed_prefix,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const PREFIX_INPUTS: &[u8] =
    include_bytes!("fixtures/selected_frame_prefix/input-only-vectors.json");
const PREFIX_INPUTS_SHA256: &str =
    "865ad85dbef724b6896257baa9454db23d0bd82020297330ada4c8a65b816088";

#[test]
fn selected_fixed_prefix_matches_hash_bound_45_bit_input_vector() {
    assert_eq!(hex(&Sha256::digest(PREFIX_INPUTS)), PREFIX_INPUTS_SHA256);
    let document: Value = serde_json::from_slice(PREFIX_INPUTS).unwrap();
    let vector = document["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|vector| vector["id"] == "F12S7-FIXED-PREFIX-45-BITS")
        .unwrap();
    assert_eq!(vector["grade"], "SPEC_INPUT_ONLY");
    assert_eq!(vector["profile"]["frame_bytes"].as_u64(), Some(4_096));
    assert_eq!(vector["bit_order"], "MSB_FIRST");

    let mut bytes = [0_u8; 6];
    let final_cursor = {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        write_selected_eac3_fixed_prefix(&mut writer).unwrap()
    };
    let expected = &vector["expected"];
    assert_eq!(final_cursor.bit_offset(), 45);
    assert_eq!(expected["bit_count"].as_u64(), Some(45));
    assert_eq!(hex(&bytes), expected["display_byte_hex"].as_str().unwrap());
    assert_eq!(
        bit_string(&bytes, 45),
        expected["bit_string"].as_str().unwrap(),
    );
    assert_eq!(expected["display_pad_bits"], "000");
    assert_eq!(expected["pad_bits_are_not_prefix"], true);

    let mut cursor = 0_u32;
    for field in vector["fields"].as_array().unwrap() {
        let width = u8::try_from(field["width_bits"].as_u64().unwrap()).unwrap();
        let value = u32::try_from(field["value"].as_u64().unwrap()).unwrap();
        assert_eq!(read_bits(&bytes, cursor, width), value, "{}", field["name"]);
        cursor += u32::from(width);
    }
    assert_eq!(cursor, 45);
}

#[test]
fn selected_fixed_prefix_rejects_wrong_cursor_or_capacity_without_mutation() {
    let mut wrong_cursor_bytes = [0xA5_u8; 6];
    let wrong_cursor_before = wrong_cursor_bytes;
    let wrong_cursor_offset = {
        let mut writer =
            FrameBitWriter::new(&mut wrong_cursor_bytes, FrameBitCursor::new(1)).unwrap();
        assert_eq!(
            write_selected_eac3_fixed_prefix(&mut writer),
            Err(SelectedEac3PrefixError::CursorMismatch { actual: 1 }),
        );
        writer.cursor().bit_offset()
    };
    assert_eq!(wrong_cursor_offset, 1);
    assert_eq!(wrong_cursor_bytes, wrong_cursor_before);

    let mut short_bytes = [0x5A_u8; 5];
    let short_before = short_bytes;
    let short_cursor = {
        let mut writer = FrameBitWriter::new(&mut short_bytes, FrameBitCursor::new(0)).unwrap();
        assert_eq!(
            write_selected_eac3_fixed_prefix(&mut writer),
            Err(SelectedEac3PrefixError::BitWriter(
                FrameBitWriterError::BufferTooSmall,
            )),
        );
        writer.cursor().bit_offset()
    };
    assert_eq!(short_cursor, 0);
    assert_eq!(short_bytes, short_before);
}

fn bit_string(bytes: &[u8], bit_count: u32) -> String {
    (0..bit_count)
        .map(|bit| {
            if read_bits(bytes, bit, 1) == 0 {
                '0'
            } else {
                '1'
            }
        })
        .collect()
}

fn read_bits(bytes: &[u8], bit_offset: u32, width: u8) -> u32 {
    (0..u32::from(width)).fold(0, |value, bit| {
        let absolute = bit_offset + bit;
        let byte = bytes[usize::try_from(absolute / 8).unwrap()];
        let shift = 7 - (absolute % 8);
        (value << 1) | u32::from((byte >> shift) & 1)
    })
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}
