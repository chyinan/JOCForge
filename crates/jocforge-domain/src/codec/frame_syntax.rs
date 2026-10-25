// pattern: Functional Core
//! Boundary for the approved frame schema. It consumes validated field proposals only.

use crate::drc::none_drc_bsi_projection::project_u16_high_byte;
use crate::drc::none_drc_callback::NoneDrcCallbackTransition;
use crate::timeline::CodecSourceIndex;

const LINE_BLOCK_COUNT: u8 = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecBlockIndex(u8);

impl CodecBlockIndex {
    pub fn new(value: u8) -> Result<Self, SelectedBsiDrcSyntaxError> {
        if value >= LINE_BLOCK_COUNT {
            return Err(SelectedBsiDrcSyntaxError::BlockIndexOutOfRange(value));
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedBsiDrcSyntaxError {
    SourceGenerationMismatch,
    DialnormOutOfRange(i32),
    BlockIndexOutOfRange(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedBsiDrcFields {
    source_index: CodecSourceIndex,
    dialnorm: u8,
    rf_code: Option<u8>,
    line_codes: [Option<u8>; 6],
}

impl SelectedBsiDrcFields {
    pub const fn source_index(self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn dialnorm(self) -> u8 {
        self.dialnorm
    }

    pub const fn rf_code(self) -> Option<u8> {
        self.rf_code
    }

    pub fn line_code(self, block: CodecBlockIndex) -> Option<u8> {
        self.line_codes[usize::from(block.value())]
    }
}

/// Binds same-source F06ND controls to the selected F12 BSI field values.
///
/// Only RF and per-block line flags/words are consumed. Auxiliary and limiter
/// audit words have no destination in this selected BSI mapping.
pub fn bind_selected_bsi_drc_fields(
    frame_source: CodecSourceIndex,
    f06nd: &NoneDrcCallbackTransition,
) -> Result<SelectedBsiDrcFields, SelectedBsiDrcSyntaxError> {
    let clock = f06nd.clock();
    if clock.source_index != frame_source {
        return Err(SelectedBsiDrcSyntaxError::SourceGenerationMismatch);
    }

    let dialnorm = f06nd.dialnorm();
    if !(1..=31).contains(&dialnorm) {
        return Err(SelectedBsiDrcSyntaxError::DialnormOutOfRange(dialnorm));
    }

    let scheduler = &f06nd.frame().output.scheduler;
    let rf_code =
        (scheduler.rf_flag_u16 == 1).then(|| project_u16_high_byte(scheduler.rf_packed_word_u16));
    let line_codes = std::array::from_fn(|block| {
        (scheduler.line_flags_u16[block] == 1)
            .then(|| project_u16_high_byte(scheduler.line_packed_words_u16[block]))
    });

    Ok(SelectedBsiDrcFields {
        source_index: frame_source,
        dialnorm: u8::try_from(dialnorm)
            .map_err(|_| SelectedBsiDrcSyntaxError::DialnormOutOfRange(dialnorm))?,
        rf_code,
        line_codes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::bitio::{FrameBitCursor, FrameBitWriter};
    use crate::codec::frame_builder::SelectedBsiDrcWriter;
    use serde_json::Value;

    const OWNER_VECTORS: &[u8] =
        include_bytes!("../../tests/fixtures/codec_fixture/CODEC-OWNER-VECTORS.json");

    #[test]
    fn manual_codec_fixture_values_match_selected_drc_field_positions() {
        let document: Value = serde_json::from_slice(OWNER_VECTORS).unwrap();
        let case = document["success_cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| &record["case"])
            .find(|case| case["case_id"] == "mixed_short_actual_keyless_joc")
            .unwrap();
        let bsi = &case["input"]["bsi"];
        let line_values = bsi["line_dynamic_range_codes"].as_array().unwrap();
        let fields = SelectedBsiDrcFields {
            source_index: CodecSourceIndex::new(0),
            dialnorm: u8::try_from(bsi["dialnorm"].as_u64().unwrap()).unwrap(),
            rf_code: Some(u8::try_from(bsi["rf_compression_code"].as_u64().unwrap()).unwrap()),
            line_codes: std::array::from_fn(|block| {
                line_values[block]
                    .as_u64()
                    .map(|value| u8::try_from(value).unwrap())
            }),
        };
        let expected_frame = decode_hex(case["expected"]["frame_hex"].as_str().unwrap());
        let mut actual_frame = vec![0_u8; expected_frame.len()];
        let mut drc_writer = SelectedBsiDrcWriter::new(fields);
        let mut cursor = {
            let mut writer =
                FrameBitWriter::new(&mut actual_frame, FrameBitCursor::new(45)).unwrap();
            drc_writer.write_header_drc(&mut writer).unwrap();
            writer.cursor().bit_offset()
        };

        for label in ["dialnorm", "compre", "compr"] {
            let field = find_field(&case["expected"]["field_offsets"], label);
            let field_offset = u32::try_from(field["begin"].as_u64().unwrap()).unwrap();
            let field_width = u8::try_from(field["width"].as_u64().unwrap()).unwrap();
            let field_value = u32::try_from(field["value"].as_u64().unwrap()).unwrap();
            assert_eq!(
                read_bits(&actual_frame, field_offset, field_width),
                field_value,
                "{label} field",
            );
            assert_eq!(
                read_bits(&expected_frame, field_offset, field_width),
                field_value,
                "{label} fixture field",
            );
        }
        for block_value in 0..LINE_BLOCK_COUNT {
            let block = CodecBlockIndex::new(block_value).unwrap();
            let presence = find_field(
                &case["expected"]["field_offsets"],
                &format!("block[{block_value}].line_drc_present"),
            );
            let offset = u32::try_from(presence["begin"].as_u64().unwrap()).unwrap();
            let next_cursor = {
                let mut writer =
                    FrameBitWriter::new(&mut actual_frame, FrameBitCursor::new(cursor)).unwrap();
                let cursor_before_block = writer.cursor().bit_offset();
                assert!(
                    cursor_before_block <= offset,
                    "block {block_value} cursor precedes its field"
                );
                writer.skip_zeroes(offset - cursor_before_block).unwrap();
                drc_writer.write_line_drc(&mut writer, block).unwrap();
                writer.cursor().bit_offset()
            };

            let presence_value = u32::try_from(presence["value"].as_u64().unwrap()).unwrap();
            assert_eq!(read_bits(&actual_frame, offset, 1), presence_value);
            assert_eq!(read_bits(&expected_frame, offset, 1), presence_value);
            if let Some(code) = fields.line_code(block) {
                let code_field = find_field(
                    &case["expected"]["field_offsets"],
                    &format!("block[{block_value}].line_drc"),
                );
                let code_offset = u32::try_from(code_field["begin"].as_u64().unwrap()).unwrap();
                assert_eq!(code_offset, offset + 1);
                assert_eq!(read_bits(&actual_frame, code_offset, 8), u32::from(code));
                assert_eq!(read_bits(&expected_frame, code_offset, 8), u32::from(code),);
            }
            cursor = next_cursor;
        }
        assert!(drc_writer.is_complete());
        assert_eq!(
            drc_writer.finish().unwrap().source_index(),
            fields.source_index()
        );
    }

    fn find_field<'a>(fields: &'a Value, label: &str) -> &'a Value {
        fields
            .as_array()
            .unwrap()
            .iter()
            .find(|field| field["label"] == label)
            .unwrap_or_else(|| panic!("missing field {label}"))
    }

    fn decode_hex(value: &str) -> Vec<u8> {
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let digit = |byte| match byte {
                    b'0'..=b'9' => byte - b'0',
                    b'a'..=b'f' => byte - b'a' + 10,
                    _ => panic!("invalid fixture hex"),
                };
                (digit(pair[0]) << 4) | digit(pair[1])
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
}
