// pattern: Functional Core
//! One-source, one-q selected 48 kHz / 1024 kbps / 5.1 frame trial.
//!
//! This module consumes an immutable upstream plan. It does not run transforms,
//! exponent normalization, F06/F11/F13 producers, the q search, or any commit.

use super::bap::{
    BapError, BapFrameInput, BapProfileInput, FULLBAND_BIN_COUNT, FULLBAND_CHANNEL_COUNT,
    LFE_BIN_COUNT, MAIN_FAST_GAIN_CODE, allocate_bap_frame, load_bap_parameters,
};
use super::bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError};
use super::exponent_packing::PackedExponentGroup;
use super::frame_builder::{SelectedEac3PrefixError, write_selected_eac3_fixed_prefix};
use super::frame_syntax::{CodecBlockIndex, SelectedBsiDrcFields, SelectedComplexityIndexTypeA};
use super::grouped_quantizer::{
    MantissaBlockError, MantissaBlockItemInput, MantissaChannel, pack_block_mantissas,
};
use crate::analysis::source_frame::{SharedAnalysisIdentity, SharedAnalysisReceipt};
use crate::integrity::crc16::eac3_crc16;
use crate::timeline::CodecSourceIndex;
use crate::transform::fullband_exponent_pipeline::NumericPlanReceipt;

const FRAME_BYTES: usize = 4_096;
const FRAME_BITS: u32 = 32_768;
const PRE_TAIL_BIT_LIMIT: u32 = 32_734;
const FULLBAND_LANE_COUNT: usize = FULLBAND_CHANNEL_COUNT;
const BLOCK_COUNT: usize = 6;
const MANTISSA_ITEM_COUNT: usize = FULLBAND_LANE_COUNT * FULLBAND_BIN_COUNT + LFE_BIN_COUNT;
const MAIN_SLOW_DECAY_CODE: u8 = 2;
const MAIN_FAST_DECAY_CODE: u8 = 1;
const MAIN_SLOW_GAIN_CODE: u8 = 1;
const MAIN_FLOOR_CODE: u8 = 7;
const CONVERTER_CARRIER_BLOCK: usize = 2;

const CHANNEL_ORDER: [MantissaChannel; 6] = [
    MantissaChannel::Left,
    MantissaChannel::Center,
    MantissaChannel::Right,
    MantissaChannel::LeftSurround,
    MantissaChannel::RightSurround,
    MantissaChannel::Lfe,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoefficientBlockInput {
    /// F09LB-staged clipped PRIMARY words in `[L,C,R,Ls,Rs]` order.
    pub fullband_words: [[u32; FULLBAND_BIN_COUNT]; FULLBAND_LANE_COUNT],
    /// F09LB-staged clipped MAIN words for LFE.
    pub lfe_words: [u32; LFE_BIN_COUNT],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FullbandExponentLanePlan<'a> {
    /// Expanded F10 exponent row consumed by both F12A and F12S.5.
    pub decoded_exponents: &'a [u8; FULLBAND_BIN_COUNT],
    /// F09/F10 refresh strategy and syntax inputs for this lane and block.
    pub strategy: u8,
    pub bandwidth_code: u8,
    pub base_exponent: u8,
    pub packed_groups: &'a [PackedExponentGroup],
    pub gain_range: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LfeExponentPlan<'a> {
    pub decoded_exponents: &'a [u8; LFE_BIN_COUNT],
    pub strategy: u8,
    pub base_exponent: u8,
    pub packed_groups: [PackedExponentGroup; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockExponentPlan<'a> {
    pub fullband: [FullbandExponentLanePlan<'a>; FULLBAND_LANE_COUNT],
    pub lfe: LfeExponentPlan<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MainExponentSyntax {
    /// Format 0: five U5 main frame codes.
    FrameCodes([u8; FULLBAND_LANE_COUNT]),
    /// Format 1: six blocks of five U2 strategies, read from `BlockExponentPlan`.
    BlockStrategies,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConverterHeaderInput {
    /// Independent F11 converter plan. These are never replaced with main codes.
    pub frame_codes: [u8; FULLBAND_LANE_COUNT],
    pub snr_offset: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedBsiMixingInput {
    pub preferred_downmix_mode: u8,
    pub ltrt_center_mix_level: u8,
    pub loro_center_mix_level: u8,
    pub ltrt_surround_mix_level: u8,
    pub loro_surround_mix_level: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedBsiInformationInput {
    pub bitstream_mode: u8,
    pub copyright: bool,
    pub original_bitstream: bool,
    pub surround_ex_mode: u8,
    pub source_sample_rate_code: u8,
}

/// Same-source selected BSI values needed by the F12 wire trial.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedBsiInput {
    source_identity: SharedAnalysisIdentity,
    dialnorm: u8,
    rf_compression_code: Option<u8>,
    line_dynamic_range_codes: [Option<u8>; BLOCK_COUNT],
    mixing: SelectedBsiMixingInput,
    information: SelectedBsiInformationInput,
}

impl SelectedBsiInput {
    /// Admits explicit F06 values for a typed trial plan.
    #[allow(dead_code)] // The parent F06/codec source adapter is not wired yet.
    pub(crate) fn from_parent_values(
        shared_analysis: &SharedAnalysisReceipt,
        dialnorm: u8,
        rf_compression_code: Option<u8>,
        line_dynamic_range_codes: [Option<u8>; BLOCK_COUNT],
        mixing: SelectedBsiMixingInput,
        information: SelectedBsiInformationInput,
    ) -> Result<Self, SelectedFramePlanError> {
        let input = Self {
            source_identity: shared_analysis.identity().clone(),
            dialnorm,
            rf_compression_code,
            line_dynamic_range_codes,
            mixing,
            information,
        };
        validate_bsi_input(&input)?;
        Ok(input)
    }

    /// Projects the already-bound DRC receipt while keeping its source key.
    pub fn from_drc_fields(
        shared_analysis: &SharedAnalysisReceipt,
        drc: SelectedBsiDrcFields,
        mixing: SelectedBsiMixingInput,
        information: SelectedBsiInformationInput,
    ) -> Result<Self, SelectedFramePlanError> {
        let mut line_dynamic_range_codes = [None; BLOCK_COUNT];
        for (index, destination) in line_dynamic_range_codes.iter_mut().enumerate() {
            let index = u8::try_from(index).map_err(|_| SelectedFramePlanError::InvalidBsiInput)?;
            let block =
                CodecBlockIndex::new(index).map_err(|_| SelectedFramePlanError::InvalidBsiInput)?;
            *destination = drc.line_code(block);
        }
        if drc.source_index() != shared_analysis.identity().source_index() {
            return Err(SelectedFramePlanError::SourceGenerationMismatch);
        }
        Self::from_parent_values(
            shared_analysis,
            drc.dialnorm(),
            drc.rf_code(),
            line_dynamic_range_codes,
            mixing,
            information,
        )
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_identity.source_index()
    }

    pub const fn source_identity(&self) -> &SharedAnalysisIdentity {
        &self.source_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedEmdfInput<'a> {
    source_identity: SharedAnalysisIdentity,
    /// Complete F13 container after its public-integrity stage.
    pub complete_container: &'a [u8],
    /// Parsed by F13JO from this same container; F12 does not derive it.
    pub complexity: SelectedComplexityIndexTypeA,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmdfFrameInput<'a> {
    /// Typed ABSENT_PUBLISHED_PAYLOAD_V1 for a suppressed source.
    AbsentPublishedPayloadV1,
    Published(PublishedEmdfInput<'a>),
}

impl<'a> PublishedEmdfInput<'a> {
    #[allow(dead_code)] // The parent F13/codec source adapter is not wired yet.
    pub(crate) fn from_parent_values(
        shared_analysis: &SharedAnalysisReceipt,
        complete_container: &'a [u8],
        complexity: SelectedComplexityIndexTypeA,
    ) -> Self {
        Self {
            source_identity: shared_analysis.identity().clone(),
            complete_container,
            complexity,
        }
    }

    pub const fn source_identity(&self) -> &SharedAnalysisIdentity {
        &self.source_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedSourceNumericPlan<'a> {
    /// F09LB-clipped PRIMARY words and F09/F10 expanded exponent rows are
    /// frozen together by the parent for every q trial in this source.
    pub bap_provider_file: Option<&'a [u8]>,
    pub coefficients_by_block: &'a [CoefficientBlockInput; BLOCK_COUNT],
    pub exponent_plan_by_block: &'a [BlockExponentPlan<'a>; BLOCK_COUNT],
    pub db_per_bit_codes: [u8; BLOCK_COUNT],
    pub short_flags: [[bool; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    pub previous_short: [bool; FULLBAND_LANE_COUNT],
    pub main_exponent_syntax: MainExponentSyntax,
    pub converter: ConverterHeaderInput,
}

pub struct SelectedFrameTrialPlanInput<'a> {
    /// Parent-issued receipt binds the complete fixed source plan to one shared analysis result.
    pub numeric_plan: &'a NumericPlanReceipt<'a, SelectedSourceNumericPlan<'a>>,
    pub bsi: SelectedBsiInput,
    pub emdf: EmdfFrameInput<'a>,
}

/// Immutable, preflighted inputs shared by every q trial for one source.
pub struct SelectedFrameTrialPlan<'a> {
    source_identity: SharedAnalysisIdentity,
    bap_provider_file: &'a [u8],
    coefficients_by_block: &'a [CoefficientBlockInput; BLOCK_COUNT],
    exponent_plan_by_block: &'a [BlockExponentPlan<'a>; BLOCK_COUNT],
    db_per_bit_codes: [u8; BLOCK_COUNT],
    short_flags: [[bool; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    main_exponent_syntax: MainExponentSyntax,
    converter: ConverterHeaderInput,
    bsi: SelectedBsiInput,
    emdf: EmdfFrameInput<'a>,
    short_syntax: bool,
    dither_flags: [[bool; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    dither_syntax: bool,
    ba_syntax: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedFramePlanError {
    BapProvider(BapError),
    SourceGenerationMismatch,
    InvalidBsiInput,
    InvalidConverterInput,
    InvalidExponentInput {
        block: usize,
        lane: usize,
    },
    NonFiniteCoefficient {
        block: usize,
        lane: usize,
        bin: usize,
    },
    CoefficientOutsideClipDomain {
        block: usize,
        lane: usize,
        bin: usize,
    },
    EmdfContainerLengthOutOfRange(usize),
}

impl<'a> SelectedFrameTrialPlan<'a> {
    pub fn new(input: SelectedFrameTrialPlanInput<'a>) -> Result<Self, SelectedFramePlanError> {
        let numeric_plan = input.numeric_plan.plan();
        let source_identity = input.numeric_plan.identity();
        let bap_provider_file =
            numeric_plan
                .bap_provider_file
                .ok_or(SelectedFramePlanError::BapProvider(
                    BapError::ParameterUnavailable,
                ))?;
        load_bap_parameters(Some(bap_provider_file))
            .map_err(SelectedFramePlanError::BapProvider)?;

        if input.numeric_plan.shared_analysis().advancement_count() != 1 {
            return Err(SelectedFramePlanError::SourceGenerationMismatch);
        }
        validate_source_generation_binding(source_identity, &input.bsi, &input.emdf)?;
        validate_bsi_input(&input.bsi)?;
        if numeric_plan.db_per_bit_codes.iter().any(|code| *code > 3) {
            return Err(SelectedFramePlanError::InvalidBsiInput);
        }
        if numeric_plan
            .converter
            .frame_codes
            .iter()
            .any(|code| *code > 31)
            || numeric_plan.converter.snr_offset > 1_023
        {
            return Err(SelectedFramePlanError::InvalidConverterInput);
        }
        validate_exponent_and_coefficient_plan(
            numeric_plan.coefficients_by_block,
            numeric_plan.exponent_plan_by_block,
            numeric_plan.main_exponent_syntax,
        )?;
        match &input.emdf {
            EmdfFrameInput::AbsentPublishedPayloadV1 => {}
            EmdfFrameInput::Published(payload) => {
                if !(4..=511).contains(&payload.complete_container.len()) {
                    return Err(SelectedFramePlanError::EmdfContainerLengthOutOfRange(
                        payload.complete_container.len(),
                    ));
                }
            }
        }

        let short_syntax = numeric_plan
            .short_flags
            .iter()
            .flatten()
            .any(|is_short| *is_short);
        let dither_flags = std::array::from_fn(|block| {
            std::array::from_fn(|lane| {
                !numeric_plan.short_flags[block][lane] && !numeric_plan.previous_short[lane]
            })
        });
        let dither_syntax = dither_flags
            .iter()
            .flatten()
            .any(|is_dithered| !*is_dithered);
        let ba_syntax = ba_syntax_required(numeric_plan.db_per_bit_codes);

        Ok(Self {
            source_identity: source_identity.clone(),
            bap_provider_file,
            coefficients_by_block: numeric_plan.coefficients_by_block,
            exponent_plan_by_block: numeric_plan.exponent_plan_by_block,
            db_per_bit_codes: numeric_plan.db_per_bit_codes,
            short_flags: numeric_plan.short_flags,
            main_exponent_syntax: numeric_plan.main_exponent_syntax,
            converter: numeric_plan.converter,
            bsi: input.bsi,
            emdf: input.emdf,
            short_syntax,
            dither_flags,
            dither_syntax,
            ba_syntax,
        })
    }
}

fn validate_source_generation_binding(
    expected: &SharedAnalysisIdentity,
    bsi: &SelectedBsiInput,
    emdf: &EmdfFrameInput<'_>,
) -> Result<(), SelectedFramePlanError> {
    if bsi.source_identity() != expected {
        return Err(SelectedFramePlanError::SourceGenerationMismatch);
    }
    if let EmdfFrameInput::Published(payload) = emdf {
        if payload.source_identity() != expected {
            return Err(SelectedFramePlanError::SourceGenerationMismatch);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateInfeasibleReason {
    CandidateOutsideU10(u16),
    BapInput,
    Mantissa {
        block: usize,
        error: MantissaBlockError,
    },
    MantissaBitStream,
    FrameCapacity,
    FrameField(FrameBitWriterError),
    CrcResidual(u16),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedFrameTrialFatalError {
    BapProvider(BapError),
    BapCandidateJoin { requested: u16, received: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedMainFrameCandidate {
    source_identity: SharedAnalysisIdentity,
    q: u16,
    frame_bytes: [u8; FRAME_BYTES],
    pre_tail_body_bits: u32,
    mantissa_bits_by_block: [u64; BLOCK_COUNT],
}

impl SelectedMainFrameCandidate {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_identity.source_index()
    }

    pub const fn source_generation_identity(&self) -> &SharedAnalysisIdentity {
        &self.source_identity
    }

    pub const fn q(&self) -> u16 {
        self.q
    }

    pub const fn frame_bytes(&self) -> &[u8; FRAME_BYTES] {
        &self.frame_bytes
    }

    pub const fn pre_tail_body_bits(&self) -> u32 {
        self.pre_tail_body_bits
    }

    pub const fn mantissa_bits_by_block(&self) -> &[u64; BLOCK_COUNT] {
        &self.mantissa_bits_by_block
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedFrameTrialOutcome {
    Infeasible(CandidateInfeasibleReason),
    Built(Box<SelectedMainFrameCandidate>),
}

/// Builds a fresh 4096-byte candidate from one immutable plan and one U10 q.
///
/// BAP, block-local groupers, frame scratch, and the bit cursor are recreated
/// for every call. BAP input failures are candidate-local after preflight;
/// provider failures remain fatal. Converter-plan production is an upstream
/// input and is never converted into a main-trial retry result.
pub fn build_selected_frame_trial(
    plan: &SelectedFrameTrialPlan<'_>,
    q: u16,
) -> Result<SelectedFrameTrialOutcome, SelectedFrameTrialFatalError> {
    if q > 1_023 {
        return Ok(SelectedFrameTrialOutcome::Infeasible(
            CandidateInfeasibleReason::CandidateOutsideU10(q),
        ));
    }

    let exponent_rows_by_block: [[&[u8]; 6]; BLOCK_COUNT] = std::array::from_fn(|block| {
        std::array::from_fn(|channel| {
            if channel < FULLBAND_LANE_COUNT {
                plan.exponent_plan_by_block[block].fullband[channel]
                    .decoded_exponents
                    .as_slice()
            } else {
                plan.exponent_plan_by_block[block]
                    .lfe
                    .decoded_exponents
                    .as_slice()
            }
        })
    });
    let fine_zero = q.trailing_zeros() >= 4;
    let bap_input = BapFrameInput {
        decoded_exponents_by_block: &exponent_rows_by_block,
        end_exclusive_by_channel: [
            FULLBAND_BIN_COUNT,
            FULLBAND_BIN_COUNT,
            FULLBAND_BIN_COUNT,
            FULLBAND_BIN_COUNT,
            FULLBAND_BIN_COUNT,
            LFE_BIN_COUNT,
        ],
        candidate: q,
        slow_decay_code: MAIN_SLOW_DECAY_CODE,
        fast_decay_code: MAIN_FAST_DECAY_CODE,
        slow_gain_code: MAIN_SLOW_GAIN_CODE,
        db_per_bit_codes: plan.db_per_bit_codes,
        floor_code: MAIN_FLOOR_CODE,
        fast_gain_code: MAIN_FAST_GAIN_CODE,
        caller_all_active_fine_zero_by_block: [fine_zero; BLOCK_COUNT],
        profile: BapProfileInput::selected(),
    };
    let bap_frame = match allocate_bap_frame(Some(plan.bap_provider_file), &bap_input) {
        Ok(frame) => frame,
        Err(BapError::InputError) => {
            return Ok(SelectedFrameTrialOutcome::Infeasible(
                CandidateInfeasibleReason::BapInput,
            ));
        }
        Err(
            error @ (BapError::ParameterUnavailable
            | BapError::ParameterError
            | BapError::UnsupportedProfile),
        ) => {
            return Err(SelectedFrameTrialFatalError::BapProvider(error));
        }
    };
    if bap_frame.candidate_q() != q {
        return Err(SelectedFrameTrialFatalError::BapCandidateJoin {
            requested: q,
            received: bap_frame.candidate_q(),
        });
    }

    let mut frame_bytes = [0_u8; FRAME_BYTES];
    let (pre_tail_body_bits, mantissa_bits_by_block) = {
        let mut writer = match FrameBitWriter::new(&mut frame_bytes, FrameBitCursor::new(0)) {
            Ok(writer) => writer,
            Err(error) => {
                return Ok(SelectedFrameTrialOutcome::Infeasible(
                    CandidateInfeasibleReason::FrameField(error),
                ));
            }
        };
        let mantissa_bits_by_block = match write_frame_body(&mut writer, plan, q, &bap_frame) {
            Ok(bits) => bits,
            Err(BuildError::Writer(error)) => {
                return Ok(SelectedFrameTrialOutcome::Infeasible(
                    CandidateInfeasibleReason::FrameField(error),
                ));
            }
            Err(BuildError::Mantissa { block, error }) => {
                return Ok(SelectedFrameTrialOutcome::Infeasible(
                    CandidateInfeasibleReason::Mantissa { block, error },
                ));
            }
            Err(BuildError::MantissaBitStream) => {
                return Ok(SelectedFrameTrialOutcome::Infeasible(
                    CandidateInfeasibleReason::MantissaBitStream,
                ));
            }
            Err(BuildError::PrefixCursorMismatch) => {
                return Ok(SelectedFrameTrialOutcome::Infeasible(
                    CandidateInfeasibleReason::FrameField(FrameBitWriterError::CursorOverflow),
                ));
            }
        };

        let pre_tail_body_bits = writer.cursor().bit_offset();
        if pre_tail_body_bits > PRE_TAIL_BIT_LIMIT {
            return Ok(SelectedFrameTrialOutcome::Infeasible(
                CandidateInfeasibleReason::FrameCapacity,
            ));
        }
        let pad_bits = PRE_TAIL_BIT_LIMIT - pre_tail_body_bits;
        if let Err(error) = writer.skip_zeroes(pad_bits) {
            return Ok(SelectedFrameTrialOutcome::Infeasible(
                CandidateInfeasibleReason::FrameField(error),
            ));
        }
        if let Err(error) = writer.write_unsigned(0, 16) {
            return Ok(SelectedFrameTrialOutcome::Infeasible(
                CandidateInfeasibleReason::FrameField(error),
            ));
        }
        if let Err(error) = writer
            .write_bit(false)
            .and_then(|()| writer.write_bit(false))
        {
            return Ok(SelectedFrameTrialOutcome::Infeasible(
                CandidateInfeasibleReason::FrameField(error),
            ));
        }
        if writer.cursor().bit_offset() != FRAME_BITS - 16 {
            return Ok(SelectedFrameTrialOutcome::Infeasible(
                CandidateInfeasibleReason::FrameField(FrameBitWriterError::CursorOverflow),
            ));
        }
        (pre_tail_body_bits, mantissa_bits_by_block)
    };

    let crc16 = eac3_crc16(&frame_bytes[2..FRAME_BYTES - 2]);
    {
        let mut crc_writer =
            match FrameBitWriter::new(&mut frame_bytes, FrameBitCursor::new(FRAME_BITS - 16)) {
                Ok(writer) => writer,
                Err(error) => {
                    return Ok(SelectedFrameTrialOutcome::Infeasible(
                        CandidateInfeasibleReason::FrameField(error),
                    ));
                }
            };
        if let Err(error) = crc_writer.write_unsigned(u32::from(crc16), 16) {
            return Ok(SelectedFrameTrialOutcome::Infeasible(
                CandidateInfeasibleReason::FrameField(error),
            ));
        }
        if crc_writer.cursor().bit_offset() != FRAME_BITS {
            return Ok(SelectedFrameTrialOutcome::Infeasible(
                CandidateInfeasibleReason::FrameField(FrameBitWriterError::CursorOverflow),
            ));
        }
    }

    let residual = eac3_crc16(&frame_bytes[2..]);
    if residual != 0 {
        return Ok(SelectedFrameTrialOutcome::Infeasible(
            CandidateInfeasibleReason::CrcResidual(residual),
        ));
    }

    Ok(SelectedFrameTrialOutcome::Built(Box::new(
        SelectedMainFrameCandidate {
            source_identity: plan.source_identity.clone(),
            q,
            frame_bytes,
            pre_tail_body_bits,
            mantissa_bits_by_block,
        },
    )))
}

fn validate_bsi_input(input: &SelectedBsiInput) -> Result<(), SelectedFramePlanError> {
    if !(1..=31).contains(&input.dialnorm)
        || input.rf_compression_code.is_none()
        || input.line_dynamic_range_codes.iter().any(Option::is_none)
        || input.mixing.preferred_downmix_mode > 3
        || input.mixing.ltrt_center_mix_level > 7
        || input.mixing.loro_center_mix_level > 7
        || input.mixing.ltrt_surround_mix_level > 7
        || input.mixing.loro_surround_mix_level > 7
        || input.information.bitstream_mode > 7
        || input.information.surround_ex_mode > 3
        || input.information.source_sample_rate_code > 1
    {
        return Err(SelectedFramePlanError::InvalidBsiInput);
    }
    Ok(())
}

fn validate_exponent_and_coefficient_plan(
    coefficients_by_block: &[CoefficientBlockInput; BLOCK_COUNT],
    exponent_plan_by_block: &[BlockExponentPlan<'_>; BLOCK_COUNT],
    main_exponent_syntax: MainExponentSyntax,
) -> Result<(), SelectedFramePlanError> {
    if let MainExponentSyntax::FrameCodes(codes) = main_exponent_syntax
        && codes.iter().any(|code| *code > 31)
    {
        return Err(SelectedFramePlanError::InvalidExponentInput { block: 0, lane: 0 });
    }

    for (block_index, (coefficients, exponent_plan)) in coefficients_by_block
        .iter()
        .zip(exponent_plan_by_block)
        .enumerate()
    {
        for (lane_index, lane) in exponent_plan.fullband.iter().enumerate() {
            if lane.strategy > 3
                || lane.bandwidth_code > 63
                || lane.base_exponent > 15
                || lane.gain_range > 3
                || lane.decoded_exponents.iter().any(|exponent| *exponent > 24)
                || (lane.strategy != 0
                    && lane.packed_groups.iter().any(|group| group.value() > 127))
            {
                return Err(SelectedFramePlanError::InvalidExponentInput {
                    block: block_index,
                    lane: lane_index,
                });
            }
            for (bin, word) in coefficients.fullband_words[lane_index]
                .iter()
                .copied()
                .enumerate()
            {
                let coefficient = f32::from_bits(word);
                if !coefficient.is_finite() {
                    return Err(SelectedFramePlanError::NonFiniteCoefficient {
                        block: block_index,
                        lane: lane_index,
                        bin,
                    });
                }
                if !(-1.0..=1.0).contains(&coefficient) {
                    return Err(SelectedFramePlanError::CoefficientOutsideClipDomain {
                        block: block_index,
                        lane: lane_index,
                        bin,
                    });
                }
            }
        }
        if exponent_plan.lfe.strategy > 1
            || exponent_plan.lfe.base_exponent > 15
            || exponent_plan
                .lfe
                .decoded_exponents
                .iter()
                .any(|exponent| *exponent > 24)
            || (exponent_plan.lfe.strategy != 0
                && exponent_plan
                    .lfe
                    .packed_groups
                    .iter()
                    .any(|group| group.value() > 127))
        {
            return Err(SelectedFramePlanError::InvalidExponentInput {
                block: block_index,
                lane: FULLBAND_LANE_COUNT,
            });
        }
        for (bin, word) in coefficients.lfe_words.iter().copied().enumerate() {
            let coefficient = f32::from_bits(word);
            if !coefficient.is_finite() {
                return Err(SelectedFramePlanError::NonFiniteCoefficient {
                    block: block_index,
                    lane: FULLBAND_LANE_COUNT,
                    bin,
                });
            }
            if !(-1.0..=1.0).contains(&coefficient) {
                return Err(SelectedFramePlanError::CoefficientOutsideClipDomain {
                    block: block_index,
                    lane: FULLBAND_LANE_COUNT,
                    bin,
                });
            }
        }
    }
    Ok(())
}

fn write_frame_body(
    writer: &mut FrameBitWriter<'_>,
    plan: &SelectedFrameTrialPlan<'_>,
    q: u16,
    bap_frame: &super::bap::BapFrame,
) -> Result<[u64; BLOCK_COUNT], BuildError> {
    match write_selected_eac3_fixed_prefix(writer) {
        Ok(_) => {}
        Err(SelectedEac3PrefixError::BitWriter(error)) => return Err(BuildError::Writer(error)),
        Err(SelectedEac3PrefixError::CursorMismatch { .. }) => {
            return Err(BuildError::PrefixCursorMismatch);
        }
    }

    write_bsi_header(writer, plan)?;
    write_frame_controls(writer, plan, q)?;
    let mut mantissa_bits_by_block = [0_u64; BLOCK_COUNT];
    let tuples = plan
        .db_per_bit_codes
        .map(|db_per_bit| [2, 1, 1, db_per_bit, 7]);

    for (block, mantissa_bit_count) in mantissa_bits_by_block.iter_mut().enumerate() {
        write_block_short_and_dither(writer, plan, block)?;
        write_line_drc(writer, plan.bsi.line_dynamic_range_codes[block])?;
        writer.write_bit(false).map_err(BuildError::Writer)?;
        write_fullband_exponents(writer, &plan.exponent_plan_by_block[block])?;
        write_lfe_exponents(writer, &plan.exponent_plan_by_block[block].lfe)?;
        write_ba_update(writer, plan.ba_syntax, block, &tuples)?;
        write_fast_gain(writer)?;
        write_converter_snr(writer, plan.converter.snr_offset, block)?;
        write_emdf_skip(writer, &plan.emdf, block)?;

        let mantissas = pack_block_mantissas(&mantissa_items_for_block(plan, bap_frame, block)?)
            .map_err(|error| BuildError::Mantissa { block, error })?;
        *mantissa_bit_count = mantissas.bit_count();
        append_mantissa_bits(writer, &mantissas)?;
    }
    Ok(mantissa_bits_by_block)
}

fn write_bsi_header(
    writer: &mut FrameBitWriter<'_>,
    plan: &SelectedFrameTrialPlan<'_>,
) -> Result<(), BuildError> {
    let bsi = &plan.bsi;
    writer
        .write_unsigned(u32::from(bsi.dialnorm), 5)
        .map_err(BuildError::Writer)?;
    writer
        .write_bit(bsi.rf_compression_code.is_some())
        .map_err(BuildError::Writer)?;
    if let Some(code) = bsi.rf_compression_code {
        writer
            .write_unsigned(u32::from(code), 8)
            .map_err(BuildError::Writer)?;
    }

    // F12S.7 fixes mixmdate=1 for the selected profile.
    writer.write_bit(true).map_err(BuildError::Writer)?;
    writer
        .write_unsigned(u32::from(bsi.mixing.preferred_downmix_mode), 2)
        .map_err(BuildError::Writer)?;
    for level in [
        bsi.mixing.ltrt_center_mix_level,
        bsi.mixing.loro_center_mix_level,
        bsi.mixing.ltrt_surround_mix_level,
        bsi.mixing.loro_surround_mix_level,
    ] {
        writer
            .write_unsigned(u32::from(level), 3)
            .map_err(BuildError::Writer)?;
    }
    writer.write_bit(false).map_err(BuildError::Writer)?;
    writer.write_bit(false).map_err(BuildError::Writer)?;
    writer.write_bit(false).map_err(BuildError::Writer)?;
    writer.write_unsigned(0, 2).map_err(BuildError::Writer)?;
    writer.write_bit(false).map_err(BuildError::Writer)?;

    // F12S.7 fixes infomdate=1 for the selected profile.
    writer.write_bit(true).map_err(BuildError::Writer)?;
    writer
        .write_unsigned(u32::from(bsi.information.bitstream_mode), 3)
        .map_err(BuildError::Writer)?;
    writer
        .write_bit(bsi.information.copyright)
        .map_err(BuildError::Writer)?;
    writer
        .write_bit(bsi.information.original_bitstream)
        .map_err(BuildError::Writer)?;
    writer
        .write_unsigned(u32::from(bsi.information.surround_ex_mode), 2)
        .map_err(BuildError::Writer)?;
    writer.write_bit(false).map_err(BuildError::Writer)?;
    writer
        .write_unsigned(u32::from(bsi.information.source_sample_rate_code), 1)
        .map_err(BuildError::Writer)?;

    match &plan.emdf {
        EmdfFrameInput::AbsentPublishedPayloadV1 => {
            writer.write_bit(false).map_err(BuildError::Writer)?;
        }
        EmdfFrameInput::Published(payload) => {
            writer.write_bit(true).map_err(BuildError::Writer)?;
            writer.write_unsigned(1, 6).map_err(BuildError::Writer)?;
            writer.write_unsigned(1, 8).map_err(BuildError::Writer)?;
            writer
                .write_unsigned(u32::from(payload.complexity.value()), 8)
                .map_err(BuildError::Writer)?;
        }
    }
    Ok(())
}

fn write_frame_controls(
    writer: &mut FrameBitWriter<'_>,
    plan: &SelectedFrameTrialPlan<'_>,
    q: u16,
) -> Result<(), BuildError> {
    let exponent_format = match plan.main_exponent_syntax {
        MainExponentSyntax::FrameCodes(_) => 0,
        MainExponentSyntax::BlockStrategies => 1,
    };
    writer
        .write_unsigned(exponent_format, 1)
        .map_err(BuildError::Writer)?;
    writer.write_bit(false).map_err(BuildError::Writer)?;
    writer.write_unsigned(0, 2).map_err(BuildError::Writer)?;
    writer.write_bit(false).map_err(BuildError::Writer)?;

    let emdf_skip_syntax = matches!(&plan.emdf, EmdfFrameInput::Published(_));
    let syntax_mask = u32::from(plan.short_syntax) * 64
        + u32::from(plan.dither_syntax) * 32
        + u32::from(plan.ba_syntax) * 16
        + 8
        + u32::from(emdf_skip_syntax) * 2;
    writer
        .write_unsigned(syntax_mask, 7)
        .map_err(BuildError::Writer)?;
    writer.skip_zeroes(6).map_err(BuildError::Writer)?;

    match plan.main_exponent_syntax {
        MainExponentSyntax::FrameCodes(codes) => {
            for code in codes {
                writer
                    .write_unsigned(u32::from(code), 5)
                    .map_err(BuildError::Writer)?;
            }
        }
        MainExponentSyntax::BlockStrategies => {
            for block in plan.exponent_plan_by_block {
                for lane in block.fullband {
                    writer
                        .write_unsigned(u32::from(lane.strategy), 2)
                        .map_err(BuildError::Writer)?;
                }
            }
        }
    }
    for block in plan.exponent_plan_by_block {
        writer
            .write_unsigned(u32::from(block.lfe.strategy), 1)
            .map_err(BuildError::Writer)?;
    }
    for code in plan.converter.frame_codes {
        writer
            .write_unsigned(u32::from(code), 5)
            .map_err(BuildError::Writer)?;
    }
    writer
        .write_unsigned(u32::from(q >> 4), 6)
        .map_err(BuildError::Writer)?;
    writer
        .write_unsigned(u32::from(q & 0x0f), 4)
        .map_err(BuildError::Writer)?;
    writer.write_bit(false).map_err(BuildError::Writer)?;
    Ok(())
}

fn write_block_short_and_dither(
    writer: &mut FrameBitWriter<'_>,
    plan: &SelectedFrameTrialPlan<'_>,
    block: usize,
) -> Result<(), BuildError> {
    if plan.short_syntax {
        for short in plan.short_flags[block] {
            writer.write_bit(short).map_err(BuildError::Writer)?;
        }
    }
    if plan.dither_syntax {
        for dither in plan.dither_flags[block] {
            writer.write_bit(dither).map_err(BuildError::Writer)?;
        }
    }
    Ok(())
}

fn write_line_drc(
    writer: &mut FrameBitWriter<'_>,
    line_code: Option<u8>,
) -> Result<(), BuildError> {
    writer
        .write_bit(line_code.is_some())
        .map_err(BuildError::Writer)?;
    if let Some(code) = line_code {
        writer
            .write_unsigned(u32::from(code), 8)
            .map_err(BuildError::Writer)?;
    }
    Ok(())
}

fn write_fullband_exponents(
    writer: &mut FrameBitWriter<'_>,
    block: &BlockExponentPlan<'_>,
) -> Result<(), BuildError> {
    for lane in block.fullband {
        if lane.strategy != 0 {
            writer
                .write_unsigned(u32::from(lane.bandwidth_code), 6)
                .map_err(BuildError::Writer)?;
        }
    }
    for lane in block.fullband {
        if lane.strategy == 0 {
            continue;
        }
        writer
            .write_unsigned(u32::from(lane.base_exponent), 4)
            .map_err(BuildError::Writer)?;
        for group in lane.packed_groups {
            writer
                .write_unsigned(u32::from(group.value()), 7)
                .map_err(BuildError::Writer)?;
        }
        writer
            .write_unsigned(u32::from(lane.gain_range), 2)
            .map_err(BuildError::Writer)?;
    }
    Ok(())
}

fn write_lfe_exponents(
    writer: &mut FrameBitWriter<'_>,
    lfe: &LfeExponentPlan<'_>,
) -> Result<(), BuildError> {
    if lfe.strategy == 0 {
        return Ok(());
    }
    writer
        .write_unsigned(u32::from(lfe.base_exponent), 4)
        .map_err(BuildError::Writer)?;
    for group in lfe.packed_groups {
        writer
            .write_unsigned(u32::from(group.value()), 7)
            .map_err(BuildError::Writer)?;
    }
    Ok(())
}

fn write_ba_update(
    writer: &mut FrameBitWriter<'_>,
    ba_syntax: bool,
    block: usize,
    tuples: &[[u8; 5]; BLOCK_COUNT],
) -> Result<(), BuildError> {
    if !ba_syntax {
        return Ok(());
    }
    let update = block == 0 || tuples[block] != tuples[block - 1];
    writer.write_bit(update).map_err(BuildError::Writer)?;
    if update {
        for (value, width) in tuples[block].into_iter().zip([2_u8, 2, 2, 2, 3]) {
            writer
                .write_unsigned(u32::from(value), width)
                .map_err(BuildError::Writer)?;
        }
    }
    Ok(())
}

fn write_fast_gain(writer: &mut FrameBitWriter<'_>) -> Result<(), BuildError> {
    writer.write_bit(true).map_err(BuildError::Writer)?;
    for _ in 0..6 {
        writer
            .write_unsigned(u32::from(MAIN_FAST_GAIN_CODE), 3)
            .map_err(BuildError::Writer)?;
    }
    Ok(())
}

fn write_converter_snr(
    writer: &mut FrameBitWriter<'_>,
    snr_offset: u16,
    block: usize,
) -> Result<(), BuildError> {
    writer.write_bit(block == 0).map_err(BuildError::Writer)?;
    if block == 0 {
        writer
            .write_unsigned(u32::from(snr_offset), 10)
            .map_err(BuildError::Writer)?;
    }
    Ok(())
}

fn write_emdf_skip(
    writer: &mut FrameBitWriter<'_>,
    emdf: &EmdfFrameInput<'_>,
    block: usize,
) -> Result<(), BuildError> {
    let EmdfFrameInput::Published(payload) = emdf else {
        return Ok(());
    };
    let carrier = block == CONVERTER_CARRIER_BLOCK;
    writer.write_bit(carrier).map_err(BuildError::Writer)?;
    if carrier {
        let length = u32::try_from(payload.complete_container.len())
            .map_err(|_| BuildError::Writer(FrameBitWriterError::CursorOverflow))?;
        writer
            .write_unsigned(length, 9)
            .map_err(BuildError::Writer)?;
        writer
            .write_bytes(payload.complete_container)
            .map_err(BuildError::Writer)?;
    }
    Ok(())
}

fn mantissa_items_for_block(
    plan: &SelectedFrameTrialPlan<'_>,
    bap_frame: &super::bap::BapFrame,
    block_index: usize,
) -> Result<Vec<MantissaBlockItemInput>, BuildError> {
    let coefficients = &plan.coefficients_by_block[block_index];
    let exponents = &plan.exponent_plan_by_block[block_index];
    let baps = &bap_frame.blocks()[block_index];
    let mut items = Vec::with_capacity(MANTISSA_ITEM_COUNT);

    for (lane, channel) in CHANNEL_ORDER
        .iter()
        .copied()
        .take(FULLBAND_LANE_COUNT)
        .enumerate()
    {
        for bin in 0..FULLBAND_BIN_COUNT {
            let bin = u16::try_from(bin).map_err(|_| BuildError::MantissaBitStream)?;
            let bin_index = usize::from(bin);
            items.push(MantissaBlockItemInput {
                channel,
                bin,
                coefficient_word: coefficients.fullband_words[lane][bin_index],
                decoded_exponent: i16::from(exponents.fullband[lane].decoded_exponents[bin_index]),
                bap: baps.fullband_rows()[lane][bin_index],
            });
        }
    }
    for bin in 0..LFE_BIN_COUNT {
        let bin_u16 = u16::try_from(bin).map_err(|_| BuildError::MantissaBitStream)?;
        items.push(MantissaBlockItemInput {
            channel: MantissaChannel::Lfe,
            bin: bin_u16,
            coefficient_word: coefficients.lfe_words[bin],
            decoded_exponent: i16::from(exponents.lfe.decoded_exponents[bin]),
            bap: baps.lfe_row()[bin],
        });
    }
    Ok(items)
}

fn append_mantissa_bits(
    writer: &mut FrameBitWriter<'_>,
    output: &super::grouped_quantizer::MantissaBlockOutput,
) -> Result<(), BuildError> {
    for bit_index in 0..output.bit_count() {
        let byte_index =
            usize::try_from(bit_index / 8).map_err(|_| BuildError::MantissaBitStream)?;
        let byte = *output
            .packed_bytes()
            .get(byte_index)
            .ok_or(BuildError::MantissaBitStream)?;
        let shift = u8::try_from(7 - (bit_index % 8)).map_err(|_| BuildError::MantissaBitStream)?;
        writer
            .write_bit((byte >> shift) & 1 != 0)
            .map_err(BuildError::Writer)?;
    }
    Ok(())
}

fn ba_syntax_required(db_per_bit_codes: [u8; BLOCK_COUNT]) -> bool {
    let default_tuple = [2_u8, 1, 1, 2, 7];
    let tuples = db_per_bit_codes.map(|db_per_bit| [2, 1, 1, db_per_bit, 7]);
    tuples[0] != default_tuple || tuples.windows(2).any(|pair| pair[0] != pair[1])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BuildError {
    Writer(FrameBitWriterError),
    Mantissa {
        block: usize,
        error: MantissaBlockError,
    },
    MantissaBitStream,
    PrefixCursorMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::input_history::{INPUT_HISTORY_BLOCK_SAMPLES, InputHistory};
    use crate::analysis::source_frame::{
        AnalysisOwnerGeneration, F07SourceFrameBeforeState, F07SourceFrameProposal,
        InstalledAudioSourceFrame, SharedAnalysisReceipt, stage_f07a_source_frame,
    };
    use crate::analysis::transient_channel::TransientChannelState;
    use crate::analysis::transient_parameters::load_transient_parameters;
    use crate::timeline::CodecSourceIndex;
    use crate::transaction::{
        AttemptId, CallbackTransactionId, SourceTransaction, SourceTransactionStageError,
    };
    use serde_json::Value;
    use sha2::{Digest, Sha256};

    const FRAME_PACKAGE_MANIFEST: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1/MANIFEST.json"
    );
    const INPUT_ONLY_PLANS: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1/INPUT-ONLY-PLANS.json"
    );
    const BAP_PROVIDER: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-PUBLIC-BAP-PROVIDER-V1/public-bap-48k-v1.json"
    );
    const FRAME_PACKAGE_MANIFEST_SHA256: &str =
        "d9085785d5484d236d4b24925a52954fc0bb01153c2e173e2d07d8f6e1b4a5f6";
    const INPUT_ONLY_PLANS_SHA256: &str =
        "2a5b7a11cc2d429435a2b0c2fe88255f5eeffed295c0954239e8b2a6791d7d74";
    const TRANSIENT_PARAMETERS: &[u8] =
        include_bytes!("../../tests/fixtures/transient_channel/PARAMETERS.json");

    fn f07_proposal() -> F07SourceFrameProposal {
        let before = F07SourceFrameBeforeState::new(
            std::array::from_fn(|_| InputHistory::cold()),
            std::array::from_fn(|_| TransientChannelState::cold()),
            [false; FULLBAND_LANE_COUNT],
        );
        let installed: Box<InstalledAudioSourceFrame> =
            vec![[[0_u32; INPUT_HISTORY_BLOCK_SAMPLES]; BLOCK_COUNT]; 6]
                .into_boxed_slice()
                .try_into()
                .unwrap();
        let parameters = load_transient_parameters(Some(TRANSIENT_PARAMETERS)).unwrap();
        stage_f07a_source_frame(&before, &installed, &parameters).unwrap()
    }

    fn source_transaction(
        source_index: CodecSourceIndex,
        attempt: u64,
        before_generation: u64,
        after_generation: u64,
    ) -> SourceTransaction {
        source_transaction_with_transaction(
            source_index,
            attempt,
            before_generation,
            after_generation,
            &format!("source-{attempt}-{}", source_index.value()),
        )
    }

    fn source_transaction_with_transaction(
        source_index: CodecSourceIndex,
        attempt: u64,
        before_generation: u64,
        after_generation: u64,
        transaction_label: &str,
    ) -> SourceTransaction {
        let transaction_id = CallbackTransactionId::new(transaction_label).unwrap();
        let mut source_transaction = SourceTransaction::new(
            AttemptId::new(attempt),
            transaction_id,
            source_index,
            AnalysisOwnerGeneration::from_parent_snapshot(before_generation),
        );
        source_transaction
            .prepare_shared_analysis(
                AnalysisOwnerGeneration::from_parent_snapshot(after_generation),
                f07_proposal(),
            )
            .unwrap();
        source_transaction
    }

    fn numeric_plan_values<'a>(
        coefficients_by_block: &'a [CoefficientBlockInput; BLOCK_COUNT],
        exponent_plan_by_block: &'a [BlockExponentPlan<'a>; BLOCK_COUNT],
    ) -> SelectedSourceNumericPlan<'a> {
        SelectedSourceNumericPlan {
            bap_provider_file: Some(BAP_PROVIDER),
            coefficients_by_block,
            exponent_plan_by_block,
            db_per_bit_codes: [2; BLOCK_COUNT],
            short_flags: [[false; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
            previous_short: [false; FULLBAND_LANE_COUNT],
            main_exponent_syntax: MainExponentSyntax::FrameCodes([0; FULLBAND_LANE_COUNT]),
            converter: ConverterHeaderInput {
                frame_codes: [0; FULLBAND_LANE_COUNT],
                snr_offset: 0,
            },
        }
    }

    fn bsi_values(shared_analysis: &SharedAnalysisReceipt) -> SelectedBsiInput {
        SelectedBsiInput::from_parent_values(
            shared_analysis,
            1,
            Some(0),
            [Some(0); BLOCK_COUNT],
            SelectedBsiMixingInput {
                preferred_downmix_mode: 0,
                ltrt_center_mix_level: 0,
                loro_center_mix_level: 0,
                ltrt_surround_mix_level: 0,
                loro_surround_mix_level: 0,
            },
            SelectedBsiInformationInput {
                bitstream_mode: 0,
                copyright: false,
                original_bitstream: false,
                surround_ex_mode: 0,
                source_sample_rate_code: 0,
            },
        )
        .unwrap()
    }

    #[test]
    fn approved_input_plan_is_hash_bound_and_has_no_expected_frame_or_audio() {
        assert_eq!(
            sha256(FRAME_PACKAGE_MANIFEST),
            FRAME_PACKAGE_MANIFEST_SHA256
        );
        let manifest: Value = serde_json::from_slice(FRAME_PACKAGE_MANIFEST).unwrap();
        assert_eq!(
            manifest["package_id"],
            "CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1"
        );
        assert_eq!(sha256(INPUT_ONLY_PLANS), INPUT_ONLY_PLANS_SHA256);

        let plans: Value = serde_json::from_slice(INPUT_ONLY_PLANS).unwrap();
        assert_eq!(plans["input_only"], true);
        assert_eq!(plans["expected_frame_or_au_bytes_included"], false);
        assert_eq!(plans["pcm_or_audio_samples_included"], false);
        assert_eq!(plans["prototype_expected_outputs_used_as_seed"], false);
        assert_eq!(plans["common_plan_shape"]["frame_bytes"], 4_096);
    }

    #[test]
    fn supplied_absent_payload_plan_builds_only_a_complete_keyless_scratch_frame() {
        let plans: Value = serde_json::from_slice(INPUT_ONLY_PLANS).unwrap();
        let source0 = plans["plans"]
            .as_array()
            .unwrap()
            .iter()
            .find(|plan| plan["id"] == "source0_suppressed")
            .unwrap();
        assert_eq!(source0["inputs"]["payload"], "ABSENT_PUBLISHED_PAYLOAD_V1");
        assert_eq!(source0["inputs"]["build_full_frame_scratch"], true);
        assert_eq!(source0["inputs"]["frame_bytes"], 4_096);
        assert_eq!(source0["inputs"]["eac3_crc16"], true);
        assert_eq!(source0["inputs"]["publish_au"], false);

        let zero_fullband_exponents = [0_u8; FULLBAND_BIN_COUNT];
        let zero_lfe_exponents = [0_u8; LFE_BIN_COUNT];
        let zero_group =
            super::super::exponent_packing::pack_exponent_difference_group(&[0, 0, 0]).unwrap();
        let exponent_blocks = std::array::from_fn(|_| BlockExponentPlan {
            fullband: std::array::from_fn(|_| FullbandExponentLanePlan {
                decoded_exponents: &zero_fullband_exponents,
                strategy: 0,
                bandwidth_code: 0,
                base_exponent: 0,
                packed_groups: &[],
                gain_range: 0,
            }),
            lfe: LfeExponentPlan {
                decoded_exponents: &zero_lfe_exponents,
                strategy: 0,
                base_exponent: 0,
                packed_groups: [zero_group; 2],
            },
        });
        let coefficient_blocks = std::array::from_fn(|_| CoefficientBlockInput {
            fullband_words: [[0_u32; FULLBAND_BIN_COUNT]; FULLBAND_LANE_COUNT],
            lfe_words: [0_u32; LFE_BIN_COUNT],
        });
        let source_index = CodecSourceIndex::new(0);
        let mut source_transaction = source_transaction(source_index, 1, 7, 7);
        let shared = source_transaction.shared_analysis().unwrap();
        let shared_identity = shared.identity().clone();
        let source_stage_key = shared_identity.source_stage_key();
        assert_eq!(source_stage_key.attempt_id(), AttemptId::new(1));
        assert_eq!(source_stage_key.transaction_id().as_str(), "source-1-0");
        assert_eq!(source_stage_key.source_index(), source_index);
        assert_eq!(source_stage_key.analysis_owner_id().as_str(), "analysis");
        assert_eq!(source_stage_key.analysis_before_generation().value(), 7);
        assert_eq!(shared.identity().analysis_after_generation().value(), 7);
        assert_eq!(shared.advancement_count(), 1);
        let bsi = bsi_values(shared);
        let numeric_plan = source_transaction
            .prepare_numeric_plan(numeric_plan_values(&coefficient_blocks, &exponent_blocks))
            .unwrap();
        let plan = SelectedFrameTrialPlan::new(SelectedFrameTrialPlanInput {
            numeric_plan: &numeric_plan,
            bsi,
            emdf: EmdfFrameInput::AbsentPublishedPayloadV1,
        })
        .unwrap();

        let first = build_selected_frame_trial(&plan, 16).unwrap();
        let repeated = build_selected_frame_trial(&plan, 16).unwrap();
        let SelectedFrameTrialOutcome::Built(candidate) = first else {
            panic!("the input-only source0 candidate should produce scratch");
        };
        let SelectedFrameTrialOutcome::Built(repeated) = repeated else {
            panic!("a repeated candidate should start from fresh scratch");
        };

        assert_eq!(candidate.q(), 16);
        assert_eq!(candidate.source_index(), source_index);
        assert_eq!(candidate.source_generation_identity(), &shared_identity);
        assert_eq!(candidate.frame_bytes().len(), FRAME_BYTES);
        assert_eq!(candidate.frame_bytes()[0..2], [0x0b, 0x77]);
        assert_eq!(read_bits(candidate.frame_bytes(), 59, 1), 1);
        assert_eq!(read_bits(candidate.frame_bytes(), 80, 1), 1);
        assert_eq!(read_bits(candidate.frame_bytes(), 90, 1), 0);
        assert_eq!(read_bits(candidate.frame_bytes(), 96, 7), 8);
        assert_eq!(read_bits(candidate.frame_bytes(), 165, 6), 1);
        assert_eq!(read_bits(candidate.frame_bytes(), 171, 4), 0);
        assert_eq!(read_bits(candidate.frame_bytes(), 175, 1), 0);
        assert_eq!(read_bits(candidate.frame_bytes(), 176, 1), 1);
        assert_eq!(read_bits(candidate.frame_bytes(), 177, 8), 0);
        assert_eq!(read_bits(candidate.frame_bytes(), 185, 1), 0);
        assert!(candidate.pre_tail_body_bits() <= PRE_TAIL_BIT_LIMIT);
        assert_eq!(candidate.frame_bytes(), repeated.frame_bytes());
        assert_eq!(
            read_bits(candidate.frame_bytes(), PRE_TAIL_BIT_LIMIT, 16),
            0
        );
        assert_eq!(read_bits(candidate.frame_bytes(), 32_750, 1), 0);
        assert_eq!(read_bits(candidate.frame_bytes(), 32_751, 1), 0);
        assert_eq!(eac3_crc16(&candidate.frame_bytes()[2..]), 0);
        let outcome = build_selected_frame_trial(&plan, 1_024).unwrap();
        assert_eq!(
            outcome,
            SelectedFrameTrialOutcome::Infeasible(CandidateInfeasibleReason::CandidateOutsideU10(
                1_024
            ))
        );
    }

    #[test]
    fn source_generation_binding_rejects_same_source_from_another_attempt() {
        let source_index = CodecSourceIndex::new(0);
        let expected =
            source_transaction_with_transaction(source_index, 1, 7, 7, "same-transaction");
        let other_attempt =
            source_transaction_with_transaction(source_index, 2, 7, 7, "same-transaction");
        let expected_identity = expected.shared_analysis().unwrap().identity().clone();
        let bsi = bsi_values(other_attempt.shared_analysis().unwrap());

        assert_eq!(
            validate_source_generation_binding(
                &expected_identity,
                &bsi,
                &EmdfFrameInput::AbsentPublishedPayloadV1,
            ),
            Err(SelectedFramePlanError::SourceGenerationMismatch)
        );
    }

    #[test]
    fn source_generation_binding_rejects_same_source_attempt_with_different_after_generation() {
        let source_index = CodecSourceIndex::new(0);
        let expected = source_transaction(source_index, 1, 7, 7);
        let other_generation = source_transaction(source_index, 1, 7, 8);
        let expected_identity = expected.shared_analysis().unwrap().identity().clone();
        let bsi = bsi_values(expected.shared_analysis().unwrap());
        let emdf = EmdfFrameInput::Published(PublishedEmdfInput::from_parent_values(
            other_generation.shared_analysis().unwrap(),
            &[0xA5; 4],
            SelectedComplexityIndexTypeA::from_model_value(16),
        ));

        assert_eq!(
            validate_source_generation_binding(&expected_identity, &bsi, &emdf),
            Err(SelectedFramePlanError::SourceGenerationMismatch)
        );
    }

    #[test]
    fn source_transaction_rejects_second_shared_analysis_prepare_without_replacement() {
        let source_index = CodecSourceIndex::new(0);
        let mut transaction = source_transaction(source_index, 1, 7, 7);
        let before_identity = transaction.shared_analysis().unwrap().identity().clone();

        assert_eq!(
            transaction.prepare_shared_analysis(
                AnalysisOwnerGeneration::from_parent_snapshot(8),
                f07_proposal(),
            ),
            Err(SourceTransactionStageError::AnalysisDoubleAdvance)
        );
        assert_eq!(
            transaction.shared_analysis().unwrap().identity(),
            &before_identity
        );
    }

    #[test]
    fn source_transaction_prepares_only_one_numeric_plan_receipt() {
        let zero_fullband_exponents = [0_u8; FULLBAND_BIN_COUNT];
        let zero_lfe_exponents = [0_u8; LFE_BIN_COUNT];
        let zero_group =
            super::super::exponent_packing::pack_exponent_difference_group(&[0, 0, 0]).unwrap();
        let exponent_blocks = std::array::from_fn(|_| BlockExponentPlan {
            fullband: std::array::from_fn(|_| FullbandExponentLanePlan {
                decoded_exponents: &zero_fullband_exponents,
                strategy: 0,
                bandwidth_code: 0,
                base_exponent: 0,
                packed_groups: &[],
                gain_range: 0,
            }),
            lfe: LfeExponentPlan {
                decoded_exponents: &zero_lfe_exponents,
                strategy: 0,
                base_exponent: 0,
                packed_groups: [zero_group; 2],
            },
        });
        let coefficient_blocks = std::array::from_fn(|_| CoefficientBlockInput {
            fullband_words: [[0_u32; FULLBAND_BIN_COUNT]; FULLBAND_LANE_COUNT],
            lfe_words: [0_u32; LFE_BIN_COUNT],
        });
        let source_index = CodecSourceIndex::new(0);
        let mut transaction = source_transaction(source_index, 1, 7, 7);
        let plan_values = numeric_plan_values(&coefficient_blocks, &exponent_blocks);
        let receipt = transaction.prepare_numeric_plan(plan_values).unwrap();
        assert_eq!(receipt.identity().source_index(), source_index);
        drop(receipt);

        assert_eq!(
            transaction.prepare_numeric_plan(plan_values).unwrap_err(),
            SourceTransactionStageError::NumericPlanAlreadyPrepared
        );
    }

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn read_bits(bytes: &[u8], offset: u32, width: u8) -> u32 {
        (0..u32::from(width)).fold(0_u32, |value, relative| {
            let absolute = offset + relative;
            let byte = bytes[usize::try_from(absolute / 8).unwrap()];
            let shift = 7 - (absolute % 8);
            (value << 1) | u32::from((byte >> shift) & 1)
        })
    }
}
