use jocforge_domain::codec::bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError};
use jocforge_domain::codec::frame_builder::{
    SelectedComplexityFieldWriteError, write_selected_complexity_index_type_a,
};
use jocforge_domain::codec::frame_syntax::SelectedComplexityIndexTypeA;
use jocforge_domain::joc::model_complexity::derive_model_joc_complexity;
use serde_json::Value;

const INPUT_ONLY_VECTORS: &[u8] =
    include_bytes!("fixtures/selected_complexity_field/input-only-vectors.json");

#[test]
fn model_complexity_u8_writes_only_the_selected_f12_field() {
    let vectors: Value = serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(vectors["input_only"], true);
    assert_eq!(vectors["contains_emdf_bytes_or_payload"], false);

    for vector in vectors["vectors"].as_array().unwrap() {
        let object_count =
            u32::try_from(vector["input"]["joc_object_count"].as_u64().unwrap()).unwrap();
        let expected = &vector["derived_model_value"];
        let model_complexity = derive_model_joc_complexity(object_count).unwrap();
        let value = u8::try_from(expected["complexity_u8"].as_u64().unwrap()).unwrap();
        assert_eq!(model_complexity.value(), value);
        let field = SelectedComplexityIndexTypeA::from_model_value(model_complexity.value());
        let mut bytes = vec![0xA5; 15];
        let before = bytes.clone();
        let cursor = {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(105)).unwrap();
            write_selected_complexity_index_type_a(&mut writer, field)
                .unwrap()
                .bit_offset()
        };

        assert_eq!(expected["wire_start"], 105);
        assert_eq!(expected["wire_width"], 8);
        assert_eq!(cursor, 113);
        let actual_bits: String = (105..113)
            .map(|offset| if read_bit(&bytes, offset) { '1' } else { '0' })
            .collect();
        let expected_bits = expected["wire_bits"]
            .as_str()
            .map_or_else(|| format!("{value:08b}"), str::to_owned);
        assert_eq!(actual_bits, expected_bits, "{}", vector["id"]);
        assert_eq!(read_bit(&bytes, 104), read_bit(&before, 104));
        assert_eq!(
            (113..120).map(|i| read_bit(&bytes, i)).collect::<Vec<_>>(),
            (113..120).map(|i| read_bit(&before, i)).collect::<Vec<_>>()
        );
    }
}

#[test]
fn complexity_field_rejects_wrong_cursor_and_short_capacity_atomically() {
    let field = SelectedComplexityIndexTypeA::from_model_value(16);
    let mut bytes = vec![0xA5; 15];
    let before = bytes.clone();
    let cursor_error = {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(104)).unwrap();
        write_selected_complexity_index_type_a(&mut writer, field).unwrap_err()
    };
    assert_eq!(
        cursor_error,
        SelectedComplexityFieldWriteError::CursorMismatch { actual: 104 }
    );
    assert_eq!(bytes, before);

    let mut short = vec![0x5A; 14];
    let short_before = short.clone();
    let capacity_error = {
        let mut writer = FrameBitWriter::new(&mut short, FrameBitCursor::new(105)).unwrap();
        write_selected_complexity_index_type_a(&mut writer, field).unwrap_err()
    };
    assert_eq!(
        capacity_error,
        SelectedComplexityFieldWriteError::BitWriter(FrameBitWriterError::BufferTooSmall)
    );
    assert_eq!(short, short_before);
}

fn read_bit(bytes: &[u8], offset: u32) -> bool {
    let byte = bytes[usize::try_from(offset / 8).unwrap()];
    let shift = 7 - (offset % 8);
    byte & (1 << shift) != 0
}
