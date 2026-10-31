// pattern: Functional Core
//! Boundary for the approved frame schema. It consumes validated field proposals only.

use crate::analysis::source_frame::{SharedAnalysisIdentity, SharedAnalysisReceipt};
use crate::drc::NoneDrcOwnerAfterProposal;
use crate::drc::none_drc_bsi_projection::project_u16_high_byte;
use crate::timeline::CodecSourceIndex;

const LINE_BLOCK_COUNT: u8 = 6;

/// The selected F12 additional-BSI complexity U8 (`complexity_index_type_a`).
///
/// The value is supplied by the clean F13JO model path; this type does not
/// establish native BSI source identity or whole-frame readiness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedComplexityIndexTypeA(u8);

impl SelectedComplexityIndexTypeA {
    pub const fn from_model_value(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedBsiDrcFields {
    source_identity: SharedAnalysisIdentity,
    dialnorm: u8,
    rf_code: Option<u8>,
    line_codes: [Option<u8>; 6],
}

impl SelectedBsiDrcFields {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_identity.source_index()
    }

    pub const fn source_identity(&self) -> &SharedAnalysisIdentity {
        &self.source_identity
    }

    pub const fn dialnorm(&self) -> u8 {
        self.dialnorm
    }

    pub const fn rf_code(&self) -> Option<u8> {
        self.rf_code
    }

    pub fn line_code(&self, block: CodecBlockIndex) -> Option<u8> {
        self.line_codes[usize::from(block.value())]
    }
}

/// Selected-profile mixing and information values supplied by the normalized
/// configuration provider. Width validation is performed before frame writes.
///
/// This value carries no producer identity, source generation, or persistence;
/// those joins remain the caller's responsibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedBsiMixingInformationValues {
    pub dmixmod: u8,
    pub ltrtcmixlev: u8,
    pub lorocmixlev: u8,
    pub ltrtsurmixlev: u8,
    pub lorosurmixlev: u8,
    pub bsmod: u8,
    pub copyrightb: bool,
    pub origbs: bool,
    pub dsurexmod: u8,
    pub sourcefscod: bool,
}

/// Binds same-source F06ND controls to the selected F12 BSI field values.
///
/// Only RF and per-block line flags/words are consumed. Auxiliary and limiter
/// audit words have no destination in this selected BSI mapping.
pub fn bind_selected_bsi_drc_fields(
    shared_analysis: &SharedAnalysisReceipt,
    f06nd: &NoneDrcOwnerAfterProposal,
) -> Result<SelectedBsiDrcFields, SelectedBsiDrcSyntaxError> {
    let identity = shared_analysis.identity();
    if f06nd.shared_analysis_identity() != identity
        || f06nd.source_index() != identity.source_index()
    {
        return Err(SelectedBsiDrcSyntaxError::SourceGenerationMismatch);
    }

    let dialnorm = i32::from(f06nd.programme_dialnorm());
    if !(1..=31).contains(&dialnorm) {
        return Err(SelectedBsiDrcSyntaxError::DialnormOutOfRange(dialnorm));
    }

    let scheduler = &f06nd.frame_output().scheduler;
    let rf_code =
        (scheduler.rf_flag_u16 == 1).then(|| project_u16_high_byte(scheduler.rf_packed_word_u16));
    let line_codes = std::array::from_fn(|block| {
        (scheduler.line_flags_u16[block] == 1)
            .then(|| project_u16_high_byte(scheduler.line_packed_words_u16[block]))
    });

    Ok(SelectedBsiDrcFields {
        source_identity: identity.clone(),
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
    use crate::codec::frame_builder::{SelectedBsiDrcWriter, write_selected_eac3_fixed_prefix};
    use crate::{
        analysis::{
            owner::AnalysisOwnerRoot,
            source_frame::{
                InstalledAudioSourceFrame, zeroed_shared_analysis_receipt_for_identity_tests,
            },
            transient_parameters::load_transient_parameters,
        },
        drc::NoneDrcOwnerRoot,
        meter::measurement_receipt::{
            ProgrammeMeasurementReceipt, ProgrammeMeasurementReceiptInput,
        },
        timeline::FrontendCallbackIndex,
        transaction::{AttemptId, CallbackLineageIdentity, CallbackTransactionId, ProgrammeId},
    };
    use serde_json::Value;

    const OWNER_VECTORS: &[u8] =
        include_bytes!("../../tests/fixtures/codec_fixture/CODEC-OWNER-VECTORS.json");
    const TRANSIENT_PARAMETERS: &[u8] =
        include_bytes!("../../tests/fixtures/transient_channel/PARAMETERS.json");

    #[test]
    fn selected_bsi_drc_fields_require_exact_shared_analysis_identity() {
        let source = CodecSourceIndex::new(0);
        let originating_analysis =
            crate::analysis::source_frame::zeroed_shared_analysis_receipt_for_identity_tests(
                crate::transaction::AttemptId::new(1),
                crate::transaction::CallbackTransactionId::new("bsi-origin").unwrap(),
                source,
                0,
                1,
            );
        let current_analysis =
            crate::analysis::source_frame::zeroed_shared_analysis_receipt_for_identity_tests(
                crate::transaction::AttemptId::new(1),
                crate::transaction::CallbackTransactionId::new("bsi-current").unwrap(),
                source,
                1,
                2,
            );
        let drc_fields = SelectedBsiDrcFields {
            source_identity: originating_analysis.identity().clone(),
            dialnorm: 1,
            rf_code: Some(0),
            line_codes: [Some(0); 6],
        };

        assert_eq!(
            crate::codec::selected_frame_trial::SelectedBsiInput::from_drc_fields(
                &current_analysis,
                &drc_fields,
                crate::codec::selected_frame_trial::SelectedBsiMixingInput {
                    preferred_downmix_mode: 0,
                    ltrt_center_mix_level: 0,
                    loro_center_mix_level: 0,
                    ltrt_surround_mix_level: 0,
                    loro_surround_mix_level: 0,
                },
                crate::codec::selected_frame_trial::SelectedBsiInformationInput {
                    bitstream_mode: 0,
                    copyright: false,
                    original_bitstream: false,
                    surround_ex_mode: 0,
                    source_sample_rate_code: 0,
                },
            ),
            Err(crate::codec::selected_frame_trial::SelectedFramePlanError::SourceGenerationMismatch)
        );
    }

    #[test]
    fn selected_drc_binding_consumes_the_same_f07_and_drc_owner_proposals() {
        let receipt =
            ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
                programme_id: ProgrammeId::new("frame-syntax-test").unwrap(),
                attempt: AttemptId::new(3),
                input_sha256: [0x11; 32],
                config_sha256: [0x22; 32],
                dialogue_word: (-23.0_f32).to_bits(),
                itu_word: (-23.0_f32).to_bits(),
                true_peak_word: (-1.0_f32).to_bits(),
                gating_kind: "LEVEL",
                selected_lufs_word: (-23.0_f32).to_bits(),
                meter_terminal_sha256: [0x33; 32],
                normal_callbacks: 4_247,
                zero_drains: 3,
                energy_windows: 6_794,
            })
            .unwrap();
        let attempt = receipt.attempt();
        let owner = NoneDrcOwnerRoot::cold_after_meter_finalize(&receipt).unwrap();
        let installed_audio: Box<InstalledAudioSourceFrame> = vec![[[0_u32; 256]; 6]; 6]
            .into_boxed_slice()
            .try_into()
            .unwrap();
        let transient_parameters = load_transient_parameters(Some(TRANSIENT_PARAMETERS)).unwrap();
        let f07_stage = AnalysisOwnerRoot::cold()
            .prepare_f07_source(
                attempt,
                CallbackTransactionId::new("frame-syntax-f07-source-0").unwrap(),
                CodecSourceIndex::new(0),
                &installed_audio,
                [false; 5],
                &transient_parameters,
            )
            .unwrap();
        let shared_analysis = f07_stage.shared_analysis().unwrap();
        let callback_identity = CallbackLineageIdentity::new(
            receipt.programme_id().clone(),
            attempt,
            FrontendCallbackIndex::new(2),
        );
        let drc_proposal = owner
            .prepare_source_frame_from_f07_stage(&f07_stage, &callback_identity)
            .unwrap();
        let fields =
            bind_selected_bsi_drc_fields(shared_analysis, drc_proposal.after_root()).unwrap();

        assert_eq!(fields.source_index(), CodecSourceIndex::new(0));
        assert_eq!(fields.source_identity(), shared_analysis.identity());
        assert_eq!(
            fields.dialnorm(),
            drc_proposal.after_root().programme_dialnorm()
        );

        let different_identity = zeroed_shared_analysis_receipt_for_identity_tests(
            attempt,
            CallbackTransactionId::new("different-f07-source-0").unwrap(),
            CodecSourceIndex::new(0),
            0,
            1,
        );
        assert_eq!(
            bind_selected_bsi_drc_fields(&different_identity, drc_proposal.after_root()),
            Err(SelectedBsiDrcSyntaxError::SourceGenerationMismatch),
        );
    }

    #[test]
    fn selected_bsi_writer_rejects_cursor_order_capacity_and_incomplete_fields() {
        let fields = fixture_drc_fields();
        let mut bytes = [0xA5_u8; 8];
        let mut writer = SelectedBsiDrcWriter::new(fields.clone());
        let mut wrong_cursor_bytes = [0xA5_u8; 8];
        let before_wrong_cursor = wrong_cursor_bytes;
        let cursor =
            {
                let mut bit_writer =
                    FrameBitWriter::new(&mut wrong_cursor_bytes, FrameBitCursor::new(44)).unwrap();
                assert_eq!(
                writer.write_header_drc(&mut bit_writer),
                Err(crate::codec::frame_builder::SelectedBsiDrcWriterError::HeaderCursorMismatch {
                    actual: 44,
                }),
            );
                bit_writer.cursor().bit_offset()
            };
        assert_eq!(cursor, 44);
        assert_eq!(wrong_cursor_bytes, before_wrong_cursor);

        let header_cursor = {
            let mut bit_writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
            write_selected_eac3_fixed_prefix(&mut bit_writer).unwrap();
            writer.write_header_drc(&mut bit_writer).unwrap();
            bit_writer.cursor().bit_offset()
        };
        assert_eq!(header_cursor, 59);
        let before_failed_line = bytes;
        let cursor = {
            let mut bit_writer =
                FrameBitWriter::new(&mut bytes, FrameBitCursor::new(header_cursor)).unwrap();
            assert_eq!(
                writer.write_line_drc(&mut bit_writer, CodecBlockIndex::new(1).unwrap()),
                Err(
                    crate::codec::frame_builder::SelectedBsiDrcWriterError::BlockOutOfOrder {
                        expected: 0,
                        actual: 1,
                    }
                ),
            );
            bit_writer.cursor().bit_offset()
        };
        assert_eq!(cursor, header_cursor);
        assert_eq!(bytes, before_failed_line);

        let mut short_bytes = [0xA5_u8; 8];
        let mut short_writer = SelectedBsiDrcWriter::new(fields);
        {
            let mut bit_writer =
                FrameBitWriter::new(&mut short_bytes, FrameBitCursor::new(0)).unwrap();
            write_selected_eac3_fixed_prefix(&mut bit_writer).unwrap();
            short_writer.write_header_drc(&mut bit_writer).unwrap();
            assert_eq!(
                short_writer.write_line_drc(&mut bit_writer, CodecBlockIndex::new(0).unwrap()),
                Err(
                    crate::codec::frame_builder::SelectedBsiDrcWriterError::BitWriter(
                        crate::codec::bitio::FrameBitWriterError::BufferTooSmall,
                    )
                ),
            );
            assert_eq!(bit_writer.cursor().bit_offset(), header_cursor);
        }
        assert!(!short_writer.is_complete());
        assert_eq!(
            short_writer.finish(),
            Err(
                crate::codec::frame_builder::SelectedBsiDrcWriterError::IncompleteBlockFields {
                    written: 0,
                }
            ),
        );
    }

    #[test]
    fn selected_bsi_writer_rejects_missing_or_duplicate_header() {
        let fields = fixture_drc_fields();
        let mut bytes = [0_u8; 32];
        let mut bit_writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(45)).unwrap();
        let mut writer = SelectedBsiDrcWriter::new(fields.clone());

        assert_eq!(
            writer.write_line_drc(&mut bit_writer, CodecBlockIndex::new(0).unwrap()),
            Err(crate::codec::frame_builder::SelectedBsiDrcWriterError::HeaderNotWritten),
        );
        assert_eq!(bit_writer.cursor().bit_offset(), 45);
        assert_eq!(
            writer.finish(),
            Err(crate::codec::frame_builder::SelectedBsiDrcWriterError::HeaderNotWritten),
        );

        let mut writer = SelectedBsiDrcWriter::new(fields);
        writer.write_header_drc(&mut bit_writer).unwrap();
        assert_eq!(
            writer.write_header_drc(&mut bit_writer),
            Err(crate::codec::frame_builder::SelectedBsiDrcWriterError::HeaderAlreadyWritten),
        );
        assert_eq!(
            writer.finish(),
            Err(
                crate::codec::frame_builder::SelectedBsiDrcWriterError::IncompleteBlockFields {
                    written: 0,
                }
            ),
        );
    }

    fn fixture_drc_fields() -> SelectedBsiDrcFields {
        let analysis = zeroed_shared_analysis_receipt_for_identity_tests(
            AttemptId::new(3),
            CallbackTransactionId::new("drc-writer-test").unwrap(),
            CodecSourceIndex::new(0),
            0,
            1,
        );
        SelectedBsiDrcFields {
            source_identity: analysis.identity().clone(),
            dialnorm: 24,
            rf_code: Some(0x31),
            line_codes: [Some(0x11), None, Some(0x22), None, Some(0x33), Some(0x44)],
        }
    }

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
        let shared_analysis =
            crate::analysis::source_frame::zeroed_shared_analysis_receipt_for_identity_tests(
                crate::transaction::AttemptId::new(1),
                crate::transaction::CallbackTransactionId::new("codec-fixture").unwrap(),
                CodecSourceIndex::new(0),
                0,
                1,
            );
        let fields = SelectedBsiDrcFields {
            source_identity: shared_analysis.identity().clone(),
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
        let source_index = fields.source_index();
        let mut drc_writer = SelectedBsiDrcWriter::new(fields.clone());
        let mut cursor = {
            let mut writer =
                FrameBitWriter::new(&mut actual_frame, FrameBitCursor::new(0)).unwrap();
            assert_eq!(
                write_selected_eac3_fixed_prefix(&mut writer)
                    .unwrap()
                    .bit_offset(),
                45,
            );
            drc_writer.write_header_drc(&mut writer).unwrap();
            writer.cursor().bit_offset()
        };

        for label in [
            "syncword",
            "strmtyp",
            "substreamid",
            "frmsiz",
            "fscod",
            "numblkscod",
            "acmod",
            "lfeon",
            "bsid",
            "dialnorm",
            "compre",
            "compr",
        ] {
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
        assert_eq!(drc_writer.finish().unwrap().source_index(), source_index);
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
