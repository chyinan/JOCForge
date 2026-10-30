// pattern: Functional Core
//! Noncommitting F06NR primary-bank source frame and same-source F06ND handoff.
//!
//! The stage consumes already-delayed primary rows and F07 `short_consumed` flags
//! from one parent-issued source identity. It owns no callback commit, receipt
//! admission, reset, EOF, file, or product authority. The F08/F06NI numerical
//! after-state has input-only vectors but no independent after-state oracle.

use crate::{
    analysis::owner::F07AnalysisStage,
    analysis::source_frame::{AnalysisOwnerId, SharedAnalysisReceipt},
    drc::{
        none_drc_owner::{
            NoneDrcOwnerAfterProposal, NoneDrcOwnerError, NoneDrcOwnerRoot, NoneDrcSecondaryState,
            NoneDrcSourcePlanes,
        },
        none_drc_statistics::FiniteStatisticsPlanes,
    },
    numeric::FiniteF32,
    timeline::{CodecSourceIndex, JocDispatchIndex, source_for_callback},
    transaction::CallbackLineageIdentity,
    transform::{
        fbw_transform::{F08FbwTransformError, f08_fbw_transform_block},
        long_transform::LongTransformHistory,
        math_generated_parameters::{
            MathGeneratedTransformParameterError, MathGeneratedTransformParameters,
        },
        secondary_carrier::{SecondaryCarrier, SecondaryCarrierError, assemble_secondary_carrier},
        secondary_fft::{SecondaryFftError, SecondaryFftParameters},
        secondary_inverse::{SecondaryInverseError, execute_secondary_inverse256},
        secondary_overlap::{
            SecondaryOverlapCarry, SecondaryOverlapError, SecondaryOverlapParameters,
            overlap_secondary_block,
        },
        window_fold::{F08WindowFoldError, f08_window_fold_block},
    },
};

const EXTERNAL_ROW_COUNT: usize = 6;
const SECONDARY_LANE_COUNT: usize = 5;
const SOURCE_SAMPLE_COUNT: usize = 1_536;
const BLOCK_COUNT: usize = 6;
const BLOCK_SAMPLE_COUNT: usize = 256;
const F06NI_ZERO_TAIL_START: usize = 217;
const SECONDARY_ROW_FOR_LANE: [usize; SECONDARY_LANE_COUNT] = [0, 2, 1, 4, 5];
const ZERO_COMPANION_HISTORY: [u32; 128] = [0; 128];

/// F07-delayed external rows in `[L,R,C,LFE,Ls,Rs]` order.
pub(crate) type DelayedPrimaryRows = [[u32; SOURCE_SAMPLE_COUNT]; EXTERNAL_ROW_COUNT];

/// F07 `short_consumed` flags in block-major `[L,C,R,Ls,Rs]` order.
pub(crate) type ShortConsumedFlags = [[u8; SECONDARY_LANE_COUNT]; BLOCK_COUNT];

/// Errors from a local secondary source proposal or its F06ND handoff.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecondarySourceError {
    Owner(NoneDrcOwnerError),
    InputNonFinite,
    ShortFlagOutOfRange,
    F07OutputShape,
    SourceClockOverflow,
    ForwardHistoryNonFinite,
    MathParameters(MathGeneratedTransformParameterError),
    InverseParameters(SecondaryFftError),
    OverlapParameters(SecondaryOverlapError),
    WindowFold(F08WindowFoldError),
    FbwTransform(F08FbwTransformError),
    Inverse(SecondaryInverseError),
    Overlap(SecondaryOverlapError),
    Carrier(SecondaryCarrierError),
}

/// Complete, noncommitting DRC source proposal carrying F06NR output and the
/// after-state joined with same-source F06ND statistics/scheduler/FIFO state.
#[must_use]
#[derive(Debug, PartialEq)]
pub struct NoneDrcSourceFrameProposal {
    source_index: CodecSourceIndex,
    dispatch_index: JocDispatchIndex,
    callback_identity: CallbackLineageIdentity,
    secondary_carrier: SecondaryCarrier,
    after_root: NoneDrcOwnerAfterProposal,
}

impl NoneDrcSourceFrameProposal {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn dispatch_index(&self) -> JocDispatchIndex {
        self.dispatch_index
    }

    pub const fn callback_identity(&self) -> &CallbackLineageIdentity {
        &self.callback_identity
    }

    pub const fn secondary_carrier(&self) -> &SecondaryCarrier {
        &self.secondary_carrier
    }

    pub const fn after_root(&self) -> &NoneDrcOwnerAfterProposal {
        &self.after_root
    }

    pub fn into_parts(
        self,
    ) -> (
        CodecSourceIndex,
        JocDispatchIndex,
        CallbackLineageIdentity,
        SecondaryCarrier,
        NoneDrcOwnerAfterProposal,
    ) {
        (
            self.source_index,
            self.dispatch_index,
            self.callback_identity,
            self.secondary_carrier,
            self.after_root,
        )
    }
}

/// Stages one F06NR PRIMARY source frame and the same-source F06ND transition.
/// It borrows the committed root; returned owner and carrier values are proposals.
impl NoneDrcOwnerRoot {
    /// Stages F06NR/F06ND directly from the public F07 owner proposal for this source.
    ///
    /// This keeps the handoff on the owner-produced SharedAnalysisReceipt and gives the
    /// source coordinator a production entry point without exposing row/flag arrays.
    pub fn prepare_source_frame_from_f07_stage(
        &self,
        f07_stage: &F07AnalysisStage,
        callback_identity: &CallbackLineageIdentity,
    ) -> Result<NoneDrcSourceFrameProposal, SecondarySourceError> {
        let shared_analysis = f07_stage
            .shared_analysis()
            .ok_or(SecondarySourceError::Owner(
                NoneDrcOwnerError::SharedAnalysisIdentity,
            ))?;
        self.prepare_source_frame(shared_analysis, callback_identity)
    }

    pub fn prepare_source_frame(
        &self,
        shared_analysis: &SharedAnalysisReceipt,
        callback_identity: &CallbackLineageIdentity,
    ) -> Result<NoneDrcSourceFrameProposal, SecondarySourceError> {
        prepare_secondary_source_with_f06nd(self, shared_analysis, callback_identity)
    }
}

fn prepare_secondary_source_with_f06nd(
    root: &NoneDrcOwnerRoot,
    shared_analysis: &SharedAnalysisReceipt,
    callback_identity: &CallbackLineageIdentity,
) -> Result<NoneDrcSourceFrameProposal, SecondarySourceError> {
    let (source_index, dispatch_index) =
        preflight_source_identity(root, shared_analysis, callback_identity)?;
    let (delayed_primary_rows, short_consumed) = source_inputs_from_receipt(shared_analysis)?;
    validate_inputs(&delayed_primary_rows, &short_consumed)?;

    let parameters = SecondarySourceParameters::selected()?;
    let secondary_stage = stage_secondary_frame(
        root.state().secondary(),
        source_index,
        &delayed_primary_rows,
        &short_consumed,
        &parameters,
    )?;
    let primary_planes = finite_planes_from_words(&delayed_primary_rows)?;
    let secondary_planes = finite_planes_from_carrier(&secondary_stage.secondary_carrier)?;
    let source_planes = NoneDrcSourcePlanes::from_parent_analysis(
        shared_analysis,
        &primary_planes,
        &secondary_planes,
    );
    let f06nd_stage = root
        .prepare_f06nd_source(shared_analysis, callback_identity, &source_planes)
        .map_err(SecondarySourceError::Owner)?;
    let after_root = f06nd_stage
        .join_f06nr_after(secondary_stage.after_state)
        .map_err(SecondarySourceError::Owner)?;

    Ok(NoneDrcSourceFrameProposal {
        source_index,
        dispatch_index,
        callback_identity: callback_identity.clone(),
        secondary_carrier: secondary_stage.secondary_carrier,
        after_root,
    })
}

fn source_inputs_from_receipt(
    shared_analysis: &SharedAnalysisReceipt,
) -> Result<(Box<DelayedPrimaryRows>, ShortConsumedFlags), SecondarySourceError> {
    let output = shared_analysis.f07_proposal().output();
    let delayed_audio = output.delayed_audio();
    if delayed_audio.len() != EXTERNAL_ROW_COUNT {
        return Err(SecondarySourceError::F07OutputShape);
    }

    let mut delayed_primary_rows = Vec::with_capacity(EXTERNAL_ROW_COUNT);
    for blocks in delayed_audio {
        let mut row_words = [0_u32; SOURCE_SAMPLE_COUNT];
        for (block, block_words) in blocks.iter().enumerate() {
            let start = block * BLOCK_SAMPLE_COUNT;
            row_words[start..start + BLOCK_SAMPLE_COUNT].copy_from_slice(block_words);
        }
        delayed_primary_rows.push(row_words);
    }
    let delayed_primary_rows =
        Box::<DelayedPrimaryRows>::try_from(delayed_primary_rows.into_boxed_slice())
            .map_err(|_| SecondarySourceError::F07OutputShape)?;
    Ok((delayed_primary_rows, *output.short_consumed()))
}

fn preflight_source_identity(
    root: &NoneDrcOwnerRoot,
    shared_analysis: &SharedAnalysisReceipt,
    callback_identity: &CallbackLineageIdentity,
) -> Result<(CodecSourceIndex, JocDispatchIndex), SecondarySourceError> {
    let measurement = root.measurement().ok_or(SecondarySourceError::Owner(
        NoneDrcOwnerError::MeasurementNotFinalized,
    ))?;
    let programme_dialnorm =
        root.state()
            .programme_dialnorm()
            .ok_or(SecondarySourceError::Owner(
                NoneDrcOwnerError::MeasurementNotFinalized,
            ))?;
    if programme_dialnorm != measurement.programme_dialnorm() {
        return Err(SecondarySourceError::Owner(
            NoneDrcOwnerError::MeasurementOptionMismatch,
        ));
    }

    let shared_identity = shared_analysis.identity();
    let source_stage_key = shared_identity.source_stage_key();
    if callback_identity.programme_id() != measurement.programme_id() {
        return Err(SecondarySourceError::Owner(
            NoneDrcOwnerError::ProgrammeMismatch,
        ));
    }
    if callback_identity.attempt_id() != measurement.attempt()
        || source_stage_key.attempt_id() != measurement.attempt()
    {
        return Err(SecondarySourceError::Owner(
            NoneDrcOwnerError::AttemptMismatch,
        ));
    }
    if !root.matches_prepared_job_input_identity(shared_analysis) {
        return Err(SecondarySourceError::Owner(
            NoneDrcOwnerError::PreparedJobInputIdentityMismatch,
        ));
    }
    if source_stage_key.analysis_owner_id() != AnalysisOwnerId::Analysis
        || shared_analysis.advancement_count() != 1
    {
        return Err(SecondarySourceError::Owner(
            NoneDrcOwnerError::SharedAnalysisIdentity,
        ));
    }

    let callback_index = callback_identity.callback_index();
    let source_index = shared_identity.source_index();
    if source_for_callback(callback_index) != Some(source_index) {
        return Err(SecondarySourceError::Owner(
            NoneDrcOwnerError::SourceMismatch,
        ));
    }
    if root
        .state()
        .source_generation()
        .map_err(SecondarySourceError::Owner)?
        != source_index
    {
        return Err(SecondarySourceError::Owner(
            NoneDrcOwnerError::StateGenerationMismatch,
        ));
    }

    let dispatch_index = source_index
        .dispatch()
        .map_err(|_| SecondarySourceError::SourceClockOverflow)?;
    Ok((source_index, dispatch_index))
}

fn validate_inputs(
    delayed_primary_rows: &DelayedPrimaryRows,
    short_consumed: &ShortConsumedFlags,
) -> Result<(), SecondarySourceError> {
    if delayed_primary_rows
        .iter()
        .flatten()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SecondarySourceError::InputNonFinite);
    }
    if short_consumed.iter().flatten().any(|flag| *flag > 1) {
        return Err(SecondarySourceError::ShortFlagOutOfRange);
    }
    Ok(())
}

struct SecondarySourceParameters {
    f08: MathGeneratedTransformParameters,
    f06nf: SecondaryFftParameters,
    overlap: SecondaryOverlapParameters,
}

impl SecondarySourceParameters {
    fn selected() -> Result<Self, SecondarySourceError> {
        Ok(Self {
            f08: MathGeneratedTransformParameters::selected_math_generated_v1()
                .map_err(SecondarySourceError::MathParameters)?,
            f06nf: SecondaryFftParameters::selected()
                .map_err(SecondarySourceError::InverseParameters)?,
            overlap: SecondaryOverlapParameters::selected()
                .map_err(SecondarySourceError::OverlapParameters)?,
        })
    }
}

struct SecondaryFrameProposal {
    after_state: NoneDrcSecondaryState,
    secondary_carrier: SecondaryCarrier,
}

fn stage_secondary_frame(
    before: &NoneDrcSecondaryState,
    source_index: CodecSourceIndex,
    delayed_primary_rows: &DelayedPrimaryRows,
    short_consumed: &ShortConsumedFlags,
    parameters: &SecondarySourceParameters,
) -> Result<SecondaryFrameProposal, SecondarySourceError> {
    if before.frame_index() != source_index.value() {
        return Err(SecondarySourceError::Owner(
            NoneDrcOwnerError::StateGenerationMismatch,
        ));
    }
    let frame_index_after = source_index
        .value()
        .checked_add(1)
        .ok_or(SecondarySourceError::SourceClockOverflow)?;
    let mut forward_history_after = before.forward_history().clone();
    let mut inverse_carry_after = before.inverse_carry().clone();
    let mut lane_blocks = vec![vec![Vec::<u32>::new(); BLOCK_COUNT]; SECONDARY_LANE_COUNT];

    for lane_index in 0..SECONDARY_LANE_COUNT {
        let external_row = SECONDARY_ROW_FOR_LANE[lane_index];
        for (block_index, flags_for_block) in short_consumed.iter().enumerate() {
            let start = block_index * BLOCK_SAMPLE_COUNT;
            let mut input_block = [0_u32; BLOCK_SAMPLE_COUNT];
            input_block.copy_from_slice(
                &delayed_primary_rows[external_row][start..start + BLOCK_SAMPLE_COUNT],
            );
            let block_proposal = stage_secondary_lane_block(
                &input_block,
                &forward_history_after[lane_index],
                &inverse_carry_after[lane_index],
                flags_for_block[lane_index],
                block_index,
                parameters,
            )?;
            forward_history_after[lane_index] = block_proposal.forward_history_after;
            inverse_carry_after[lane_index] = block_proposal.inverse_carry_after;
            let assembler_input_block = match block_proposal.disposition {
                CarrierDisposition::SuppressWarmup => 0,
                CarrierDisposition::WriteBlock(carrier_block) => carrier_block + 1,
            };
            // The carrier assembler accepts all six overlap results and drops
            // input block 0 itself; it writes only source blocks 1 through 5.
            lane_blocks[lane_index][assembler_input_block] = block_proposal.overlap_words.to_vec();
        }
    }

    let secondary_carrier =
        assemble_secondary_carrier(&lane_blocks).map_err(SecondarySourceError::Carrier)?;
    let after_state = NoneDrcSecondaryState::from_f06nr_after(
        forward_history_after,
        inverse_carry_after,
        frame_index_after,
    );
    Ok(SecondaryFrameProposal {
        after_state,
        secondary_carrier,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CarrierDisposition {
    SuppressWarmup,
    WriteBlock(usize),
}

struct SecondaryLaneBlockProposal {
    forward_history_after: LongTransformHistory,
    inverse_carry_after: SecondaryOverlapCarry,
    overlap_words: [u32; BLOCK_SAMPLE_COUNT],
    disposition: CarrierDisposition,
}

fn stage_secondary_lane_block(
    input_block: &[u32; BLOCK_SAMPLE_COUNT],
    forward_history_before: &LongTransformHistory,
    inverse_carry_before: &SecondaryOverlapCarry,
    short_consumed: u8,
    block_index: usize,
    parameters: &SecondarySourceParameters,
) -> Result<SecondaryLaneBlockProposal, SecondarySourceError> {
    if short_consumed > 1 || block_index >= BLOCK_COUNT {
        return Err(SecondarySourceError::ShortFlagOutOfRange);
    }
    let folded = f08_window_fold_block(
        input_block,
        forward_history_before.words(),
        &ZERO_COMPANION_HISTORY,
        &parameters.f08,
    )
    .map_err(SecondarySourceError::WindowFold)?;
    let forward_history_after =
        LongTransformHistory::try_from_words(folded.primary_history_after())
            .ok_or(SecondarySourceError::ForwardHistoryNonFinite)?;
    let transformed = f08_fbw_transform_block(&folded, i32::from(short_consumed), &parameters.f08)
        .map_err(SecondarySourceError::FbwTransform)?;
    let mut primary_spectrum = *transformed.primary_spectrum_bits();
    primary_spectrum[F06NI_ZERO_TAIL_START..].fill(0);
    let inverse =
        execute_secondary_inverse256(&primary_spectrum, short_consumed == 1, &parameters.f06nf)
            .map_err(SecondarySourceError::Inverse)?;
    let overlap = overlap_secondary_block(&inverse, inverse_carry_before, &parameters.overlap)
        .map_err(SecondarySourceError::Overlap)?;
    let (overlap_words, inverse_carry_after) = overlap.into_parts();
    let disposition = carrier_disposition(block_index);
    Ok(SecondaryLaneBlockProposal {
        forward_history_after,
        inverse_carry_after,
        overlap_words,
        disposition,
    })
}

fn carrier_disposition(block_index: usize) -> CarrierDisposition {
    if block_index == 0 {
        CarrierDisposition::SuppressWarmup
    } else {
        CarrierDisposition::WriteBlock(block_index - 1)
    }
}

fn finite_planes_from_words(
    words: &DelayedPrimaryRows,
) -> Result<Box<FiniteStatisticsPlanes>, SecondarySourceError> {
    let zero = FiniteF32::new(0.0).map_err(|_| SecondarySourceError::InputNonFinite)?;
    let mut planes = Vec::with_capacity(EXTERNAL_ROW_COUNT);
    for row_words in words {
        let mut finite_row = [zero; SOURCE_SAMPLE_COUNT];
        for (sample, word) in row_words.iter().enumerate() {
            finite_row[sample] = FiniteF32::new(f32::from_bits(*word))
                .map_err(|_| SecondarySourceError::InputNonFinite)?;
        }
        planes.push(finite_row);
    }
    Box::<FiniteStatisticsPlanes>::try_from(planes.into_boxed_slice())
        .map_err(|_| SecondarySourceError::InputNonFinite)
}

fn finite_planes_from_carrier(
    carrier: &SecondaryCarrier,
) -> Result<Box<FiniteStatisticsPlanes>, SecondarySourceError> {
    let zero = FiniteF32::new(0.0).map_err(|_| SecondarySourceError::InputNonFinite)?;
    let mut planes = Vec::with_capacity(EXTERNAL_ROW_COUNT);
    for words in carrier.rows() {
        if words.len() != SOURCE_SAMPLE_COUNT {
            return Err(SecondarySourceError::Carrier(
                SecondaryCarrierError::InputShape,
            ));
        }
        let mut finite_row = [zero; SOURCE_SAMPLE_COUNT];
        for (sample, word) in words.iter().enumerate() {
            finite_row[sample] = FiniteF32::new(f32::from_bits(*word))
                .map_err(|_| SecondarySourceError::InputNonFinite)?;
        }
        planes.push(finite_row);
    }
    Box::<FiniteStatisticsPlanes>::try_from(planes.into_boxed_slice())
        .map_err(|_| SecondarySourceError::Carrier(SecondaryCarrierError::InputShape))
}

#[cfg(test)]
mod tests;
