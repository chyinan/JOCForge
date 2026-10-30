// pattern: Functional Core
//! Selected one-source-frame F07.1/F07A/F07.4/F07.5 pure composition proposal.

use super::dither::plan_selected_dither_flags;
use super::input_history::{INPUT_HISTORY_BLOCK_SAMPLES, InputHistory};
use super::structure_refresh::{
    F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES, selected_structure_refresh_flags,
};
use super::transient_channel::{
    TransientChannelError, TransientChannelState, process_transient_channel_block,
};
use super::transient_parameters::TransientParameters;
use crate::meter::prepared_job_identity::PreparedJobInputIdentity;
use crate::transaction::{AttemptId, CallbackTransactionId};
use crate::{
    numeric::FiniteF32,
    spatial::spatial_audio_state::SelectedJocQmfProjection,
    timeline::{CodecSourceIndex, source_for_callback},
};
use std::sync::Arc;

const EXTERNAL_AUDIO_LANES: usize = 6;
pub(crate) const FBW_EXTERNAL_ROWS: [usize; F07_FULLBAND_LANES] = [0, 2, 1, 4, 5];
pub(crate) const LFE_EXTERNAL_ROW: usize = 3;
const SOURCE_FRAME_SAMPLE_COUNT: usize = INPUT_HISTORY_BLOCK_SAMPLES * F07_BLOCKS_PER_FRAME;

pub type InstalledAudioSourceFrame =
    [[[u32; INPUT_HISTORY_BLOCK_SAMPLES]; F07_BLOCKS_PER_FRAME]; EXTERNAL_AUDIO_LANES];
pub type DelayedAudioSourceFrame =
    Box<[[[u32; INPUT_HISTORY_BLOCK_SAMPLES]; F07_BLOCKS_PER_FRAME]]>;

/// MAIN source audio copied from one admitted spatial receiver projection.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct SourcedInstalledAudioFrame {
    source_index: CodecSourceIndex,
    audio: Box<InstalledAudioSourceFrame>,
}

impl SourcedInstalledAudioFrame {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn audio(&self) -> &InstalledAudioSourceFrame {
        &self.audio
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstalledAudioProjectionError {
    CallbackHasNoCodecSource,
    SourceIndexMismatch,
    UnexpectedFinalSixSampleCount { lane: u8, actual_samples: usize },
}

/// Copies the admitted F04R final six into the six externally ordered F07 source rows.
///
/// The receiver projection carries the typed source identity. Its callback must map to that
/// identity through the shared programme clock; each `[L, R, C, LFE, Ls, Rs]` row is copied
/// bit-for-bit and partitioned into six consecutive 256-sample blocks.
pub fn installed_audio_from_final_six(
    projection: &SelectedJocQmfProjection,
) -> Result<SourcedInstalledAudioFrame, InstalledAudioProjectionError> {
    let source_index = source_for_callback(projection.callback_index())
        .ok_or(InstalledAudioProjectionError::CallbackHasNoCodecSource)?;
    if source_index != projection.source_index() {
        return Err(InstalledAudioProjectionError::SourceIndexMismatch);
    }

    let audio = pack_final_six(projection.final_six())?;
    Ok(SourcedInstalledAudioFrame {
        source_index,
        audio,
    })
}

fn pack_final_six(
    final_six: &[Box<[FiniteF32]>; EXTERNAL_AUDIO_LANES],
) -> Result<Box<InstalledAudioSourceFrame>, InstalledAudioProjectionError> {
    let mut audio: Box<InstalledAudioSourceFrame> =
        vec![[[0_u32; INPUT_HISTORY_BLOCK_SAMPLES]; F07_BLOCKS_PER_FRAME]; EXTERNAL_AUDIO_LANES]
            .into_boxed_slice()
            .try_into()
            .expect("the fixed lane vector converts to the selected audio frame");
    for (lane, source_row) in final_six.iter().enumerate() {
        if source_row.len() != SOURCE_FRAME_SAMPLE_COUNT {
            return Err(
                InstalledAudioProjectionError::UnexpectedFinalSixSampleCount {
                    lane: u8::try_from(lane).unwrap_or(u8::MAX),
                    actual_samples: source_row.len(),
                },
            );
        }
        for block in 0..F07_BLOCKS_PER_FRAME {
            let source_offset = block * INPUT_HISTORY_BLOCK_SAMPLES;
            for sample in 0..INPUT_HISTORY_BLOCK_SAMPLES {
                audio[lane][block][sample] = source_row[source_offset + sample].to_bits();
            }
        }
    }
    Ok(audio)
}

#[cfg(test)]
mod installed_audio_projection_tests {
    use super::{
        EXTERNAL_AUDIO_LANES, F07_BLOCKS_PER_FRAME, INPUT_HISTORY_BLOCK_SAMPLES,
        InstalledAudioProjectionError, SOURCE_FRAME_SAMPLE_COUNT, pack_final_six,
    };
    use crate::numeric::FiniteF32;

    #[test]
    fn final_six_is_packed_by_external_lane_and_consecutive_sample() {
        let final_six = std::array::from_fn(|lane| {
            (0..SOURCE_FRAME_SAMPLE_COUNT)
                .map(|sample| {
                    let value = if lane == 0 && sample == 0 {
                        -0.0
                    } else {
                        (lane * SOURCE_FRAME_SAMPLE_COUNT + sample) as f32
                    };
                    FiniteF32::new(value).unwrap()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        });

        let installed = pack_final_six(&final_six).unwrap();
        assert_eq!(installed[0][0][0], (-0.0_f32).to_bits());
        for lane in 0..EXTERNAL_AUDIO_LANES {
            for block in 0..F07_BLOCKS_PER_FRAME {
                for sample in [0, INPUT_HISTORY_BLOCK_SAMPLES - 1] {
                    let source_sample = block * INPUT_HISTORY_BLOCK_SAMPLES + sample;
                    let expected = if lane == 0 && source_sample == 0 {
                        (-0.0_f32).to_bits()
                    } else {
                        ((lane * SOURCE_FRAME_SAMPLE_COUNT + source_sample) as f32).to_bits()
                    };
                    assert_eq!(installed[lane][block][sample], expected);
                }
            }
        }
    }

    #[test]
    fn final_six_rejects_a_malformed_row_before_returning_audio() {
        let final_six = std::array::from_fn(|lane| {
            let length = if lane == 4 {
                SOURCE_FRAME_SAMPLE_COUNT - 1
            } else {
                SOURCE_FRAME_SAMPLE_COUNT
            };
            vec![FiniteF32::new(0.0).unwrap(); length].into_boxed_slice()
        });

        assert_eq!(
            pack_final_six(&final_six),
            Err(
                InstalledAudioProjectionError::UnexpectedFinalSixSampleCount {
                    lane: 4,
                    actual_samples: SOURCE_FRAME_SAMPLE_COUNT - 1,
                }
            )
        );
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct F07SourceFrameBeforeState {
    input_histories: [InputHistory; EXTERNAL_AUDIO_LANES],
    transient_states: [TransientChannelState; F07_FULLBAND_LANES],
    previous_short: [bool; F07_FULLBAND_LANES],
}

impl F07SourceFrameBeforeState {
    pub fn new(
        input_histories: [InputHistory; EXTERNAL_AUDIO_LANES],
        transient_states: [TransientChannelState; F07_FULLBAND_LANES],
        previous_short: [bool; F07_FULLBAND_LANES],
    ) -> Self {
        Self {
            input_histories,
            transient_states,
            previous_short,
        }
    }

    pub const fn input_history_by_external_lane(&self) -> &[InputHistory; EXTERNAL_AUDIO_LANES] {
        &self.input_histories
    }

    pub const fn transient_state_by_fbw_lane(
        &self,
    ) -> &[TransientChannelState; F07_FULLBAND_LANES] {
        &self.transient_states
    }

    pub const fn previous_short_by_fbw_lane(&self) -> &[bool; F07_FULLBAND_LANES] {
        &self.previous_short
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct F07SourceFrameAfterState {
    input_histories: [InputHistory; EXTERNAL_AUDIO_LANES],
    transient_states: [TransientChannelState; F07_FULLBAND_LANES],
    previous_short: [bool; F07_FULLBAND_LANES],
}

impl F07SourceFrameAfterState {
    pub const fn input_history_by_external_lane(&self) -> &[InputHistory; EXTERNAL_AUDIO_LANES] {
        &self.input_histories
    }

    pub const fn transient_state_by_fbw_lane(
        &self,
    ) -> &[TransientChannelState; F07_FULLBAND_LANES] {
        &self.transient_states
    }

    pub const fn previous_short_by_fbw_lane(&self) -> &[bool; F07_FULLBAND_LANES] {
        &self.previous_short
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum F07SourceFrameError {
    Transient(TransientChannelError),
}

#[derive(Debug, Eq, PartialEq)]
pub struct F07SourceFrameOutput {
    delayed_audio: DelayedAudioSourceFrame,
    previous_short_before: [bool; F07_FULLBAND_LANES],
    short_source: [[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    other_source: [[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    short_consumed: [[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    other_consumed: [[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    structure_refresh: [[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    dither_flags: [[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    dither_syntax_present: bool,
}

impl F07SourceFrameOutput {
    pub const fn delayed_audio(&self) -> &DelayedAudioSourceFrame {
        &self.delayed_audio
    }

    pub const fn previous_short_before(&self) -> &[bool; F07_FULLBAND_LANES] {
        &self.previous_short_before
    }

    pub const fn short_source(&self) -> &[[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.short_source
    }

    pub const fn other_source(&self) -> &[[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.other_source
    }

    pub const fn short_consumed(&self) -> &[[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.short_consumed
    }

    pub const fn other_consumed(&self) -> &[[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.other_consumed
    }

    pub const fn structure_refresh(&self) -> &[[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.structure_refresh
    }

    pub const fn dither_flags(&self) -> &[[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.dither_flags
    }

    pub const fn dither_syntax_present(&self) -> bool {
        self.dither_syntax_present
    }
}

#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct F07SourceFrameProposal {
    output: F07SourceFrameOutput,
    after_state: F07SourceFrameAfterState,
}

impl F07SourceFrameProposal {
    pub const fn output(&self) -> &F07SourceFrameOutput {
        &self.output
    }

    pub const fn after_state(&self) -> &F07SourceFrameAfterState {
        &self.after_state
    }

    pub fn into_parts(self) -> (F07SourceFrameOutput, F07SourceFrameAfterState) {
        (self.output, self.after_state)
    }
}

/// The existing GTC analysis-owner generation value. This wrapper does not
/// advance or synthesize a generation; the parent supplies the committed or
/// staged snapshot value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AnalysisOwnerGeneration(u64);

impl AnalysisOwnerGeneration {
    #[allow(dead_code)] // The analysis root generation is not wired into Rust yet.
    pub(crate) const fn from_parent_snapshot(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// The selected analysis root is the sole owner in this source-stage key.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AnalysisOwnerId {
    Analysis,
}

impl AnalysisOwnerId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Analysis => "analysis",
        }
    }
}

/// Parent-owned source transaction identity required by the approved GTC join.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceStageKey {
    attempt_id: AttemptId,
    transaction_id: CallbackTransactionId,
    source_index: CodecSourceIndex,
    analysis_owner_id: AnalysisOwnerId,
    analysis_before_generation: AnalysisOwnerGeneration,
    prepared_job_input_identity: Option<Arc<PreparedJobInputIdentity>>,
}

impl SourceStageKey {
    /// Builds the key only inside this receipt constructor from canonical parent values.
    pub(crate) fn from_parent_snapshot(
        attempt_id: AttemptId,
        transaction_id: CallbackTransactionId,
        source_index: CodecSourceIndex,
        analysis_before_generation: AnalysisOwnerGeneration,
    ) -> Self {
        Self {
            attempt_id,
            transaction_id,
            source_index,
            analysis_owner_id: AnalysisOwnerId::Analysis,
            analysis_before_generation,
            prepared_job_input_identity: None,
        }
    }

    /// Production source key carrying the exact admitted PreparedJobInput capability.
    pub(crate) fn from_prepared_input_snapshot(
        attempt_id: AttemptId,
        transaction_id: CallbackTransactionId,
        source_index: CodecSourceIndex,
        analysis_before_generation: AnalysisOwnerGeneration,
        prepared_job_input_identity: Arc<PreparedJobInputIdentity>,
    ) -> Self {
        Self {
            attempt_id,
            transaction_id,
            source_index,
            analysis_owner_id: AnalysisOwnerId::Analysis,
            analysis_before_generation,
            prepared_job_input_identity: Some(prepared_job_input_identity),
        }
    }

    pub const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }

    pub fn transaction_id(&self) -> &CallbackTransactionId {
        &self.transaction_id
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn analysis_owner_id(&self) -> AnalysisOwnerId {
        self.analysis_owner_id
    }

    pub const fn analysis_before_generation(&self) -> AnalysisOwnerGeneration {
        self.analysis_before_generation
    }

    /// Returns the input-admission capability, if this is a production-bound source key.
    pub fn prepared_job_input_identity(&self) -> Option<&Arc<PreparedJobInputIdentity>> {
        self.prepared_job_input_identity.as_ref()
    }
}

/// Identity of one parent's staged analysis result, including its exact after generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedAnalysisIdentity {
    source_stage_key: SourceStageKey,
    analysis_after_generation: AnalysisOwnerGeneration,
}

impl SharedAnalysisIdentity {
    pub const fn source_stage_key(&self) -> &SourceStageKey {
        &self.source_stage_key
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_stage_key.source_index()
    }

    pub const fn analysis_after_generation(&self) -> AnalysisOwnerGeneration {
        self.analysis_after_generation
    }

    /// Carries the same admission-created capability used to construct this source transaction.
    pub fn prepared_job_input_identity(&self) -> Option<&Arc<PreparedJobInputIdentity>> {
        self.source_stage_key.prepared_job_input_identity()
    }
}

/// Exact selected provider lineage from the approved F07-F12 provider manifest.
///
/// These are existing package identities; this value computes no digest and defines no
/// serialized receipt format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnalysisProviderLineage {
    transient: &'static str,
    transform: &'static str,
    policy: &'static str,
    difference_map: &'static str,
}

impl AnalysisProviderLineage {
    const fn selected() -> Self {
        Self {
            transient: "130cd1a446ea8413a1d0d4c5eea631e890b7f1cd675975efa92697e258402991",
            transform: "f1fb1a16b3caedab6ddc583201b9454755fd8871ecdf7881a6a3ffbad10fddf1",
            policy: "a99d077900058eb4a862f5ea29bc87408bf6d7a4f4cc42e3da66ae6c68a93c6a",
            difference_map: "6cf3ae8d899b334d5aa87e0744e4c58b7dba1d92177ca07738726f8ea0ee2c6d",
        }
    }

    pub const fn transient_sha256(self) -> &'static str {
        self.transient
    }

    pub const fn transform_sha256(self) -> &'static str {
        self.transform
    }

    pub const fn policy_sha256(self) -> &'static str {
        self.policy
    }

    pub const fn difference_map_sha256(self) -> &'static str {
        self.difference_map
    }
}

/// Parent-issued source identity plus the staged F07 proposal used by F06/F08-F12 consumers.
///
/// The parent retains commit authority and owns the complete shared-analysis after-state. This
/// local receipt keeps the existing F07 proposal and key; it is not the complete analysis root.
/// The fixed advancement count records the contract's single `SharedAnalysisPrepare`; it is not
/// caller-selected or inferred from sample values.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct SharedAnalysisReceipt {
    identity: SharedAnalysisIdentity,
    provider_lineage: AnalysisProviderLineage,
    f07_proposal: F07SourceFrameProposal,
}

impl SharedAnalysisReceipt {
    /// The source parent may construct one receipt only after staging F07 exactly once.
    ///
    /// Generation values are the existing GTC owner snapshot fields. This function never
    /// increments, derives, hashes, or otherwise synthesizes either generation.
    #[allow(dead_code)] // Reserved for the selected parent source transaction, which is not wired yet.
    pub(crate) fn from_parent(
        source_stage_key: SourceStageKey,
        analysis_after_generation: AnalysisOwnerGeneration,
        f07_proposal: F07SourceFrameProposal,
    ) -> Self {
        Self {
            identity: SharedAnalysisIdentity {
                source_stage_key,
                analysis_after_generation,
            },
            provider_lineage: AnalysisProviderLineage::selected(),
            f07_proposal,
        }
    }

    pub const fn identity(&self) -> &SharedAnalysisIdentity {
        &self.identity
    }

    pub const fn f07_proposal(&self) -> &F07SourceFrameProposal {
        &self.f07_proposal
    }

    pub const fn provider_lineage(&self) -> AnalysisProviderLineage {
        self.provider_lineage
    }

    pub const fn advancement_count(&self) -> u8 {
        1
    }

    pub fn into_parts(self) -> (SharedAnalysisIdentity, F07SourceFrameProposal) {
        (self.identity, self.f07_proposal)
    }
}

/// Stages one 6x1536 installed-audio frame through F07.1, five F07A FBW lanes,
/// F07.4 structure-refresh flags, and F07.5 dither flags.
///
/// F07A reads installed FBW rows before the F07.1 delay. F07.1 independently
/// advances all six external rows, including LFE. The returned delayed frame is
/// suitable for a later F08 caller. This proposal owns no absolute source index,
/// parent success/fatal/reset/EOF authority, or committed programme root.
pub fn stage_f07a_source_frame(
    before: &F07SourceFrameBeforeState,
    installed_audio: &InstalledAudioSourceFrame,
    transient_parameters: &TransientParameters,
) -> Result<F07SourceFrameProposal, F07SourceFrameError> {
    let mut delayed_audio: DelayedAudioSourceFrame =
        vec![[[0_u32; INPUT_HISTORY_BLOCK_SAMPLES]; F07_BLOCKS_PER_FRAME]; EXTERNAL_AUDIO_LANES]
            .into_boxed_slice();
    let mut input_history_after: [InputHistory; EXTERNAL_AUDIO_LANES] =
        std::array::from_fn(|lane| before.input_histories[lane].staged_copy());

    for external_lane in 0..EXTERNAL_AUDIO_LANES {
        let mut history = before.input_histories[external_lane].staged_copy();
        for block in 0..F07_BLOCKS_PER_FRAME {
            let current = &installed_audio[external_lane][block];
            let (delayed, next_history) = history.advance_block(current);
            delayed_audio[external_lane][block] = *delayed.words();
            history = next_history;
        }
        input_history_after[external_lane] = history;
    }

    let mut transient_state_after: [TransientChannelState; F07_FULLBAND_LANES] =
        std::array::from_fn(|lane| before.transient_states[lane].staged_copy());
    let mut short_source = [[0_u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];
    let mut other_source = [[0_u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];
    let mut short_consumed = [[0_u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];
    let mut other_consumed = [[0_u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];
    let mut short_consumed_flags = [[false; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];
    let mut other_consumed_flags = [[false; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];

    for fbw_lane in 0..F07_FULLBAND_LANES {
        let external_lane = FBW_EXTERNAL_ROWS[fbw_lane];
        let mut state = before.transient_states[fbw_lane].staged_copy();
        for block in 0..F07_BLOCKS_PER_FRAME {
            let input = &installed_audio[external_lane][block];
            let output = process_transient_channel_block(state, input, transient_parameters)
                .map_err(F07SourceFrameError::Transient)?;
            let short_source_value = output.short_source();
            let other_source_value = output.other_source();
            let short_consumed_value = output.short_consumed();
            let other_consumed_value = output.other_consumed();
            let short_flag = short_consumed_value != 0;
            let other_flag = other_consumed_value != 0;
            short_source[block][fbw_lane] = short_source_value;
            other_source[block][fbw_lane] = other_source_value;
            short_consumed[block][fbw_lane] = short_consumed_value;
            other_consumed[block][fbw_lane] = other_consumed_value;
            short_consumed_flags[block][fbw_lane] = short_flag;
            other_consumed_flags[block][fbw_lane] = other_flag;
            state = output.into_next_state();
        }
        transient_state_after[fbw_lane] = state;
    }

    let mut structure_refresh = [[false; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];
    for fbw_lane in 0..F07_FULLBAND_LANES {
        let short_lane = std::array::from_fn(|block| short_consumed_flags[block][fbw_lane]);
        let other_lane = std::array::from_fn(|block| other_consumed_flags[block][fbw_lane]);
        let lane_flags = selected_structure_refresh_flags(&short_lane, &other_lane);
        for (refresh_row, lane_flag) in structure_refresh.iter_mut().zip(lane_flags.as_array()) {
            refresh_row[fbw_lane] = *lane_flag;
        }
    }
    let dither_plan = plan_selected_dither_flags(before.previous_short, &short_consumed_flags);
    let next_previous_short = *dither_plan.next_previous_short();
    let dither_flags = *dither_plan.flags().as_array();

    Ok(F07SourceFrameProposal {
        output: F07SourceFrameOutput {
            delayed_audio,
            previous_short_before: *before.previous_short_by_fbw_lane(),
            short_source,
            other_source,
            short_consumed,
            other_consumed,
            structure_refresh,
            dither_flags,
            dither_syntax_present: dither_plan.syntax_present(),
        },
        after_state: F07SourceFrameAfterState {
            input_histories: input_history_after,
            transient_states: transient_state_after,
            previous_short: next_previous_short,
        },
    })
}

/// Synthetic identity-only receipt fixture for cross-owner staging tests.
///
/// The zero output is not an F07 vector or a production cold constructor.
#[cfg(test)]
pub(crate) fn zeroed_shared_analysis_receipt_for_identity_tests(
    attempt_id: AttemptId,
    transaction_id: CallbackTransactionId,
    source_index: CodecSourceIndex,
    before_generation: u64,
    after_generation: u64,
) -> SharedAnalysisReceipt {
    let proposal = F07SourceFrameProposal {
        output: F07SourceFrameOutput {
            delayed_audio: vec![
                [[0_u32; INPUT_HISTORY_BLOCK_SAMPLES]; F07_BLOCKS_PER_FRAME];
                EXTERNAL_AUDIO_LANES
            ]
            .into_boxed_slice(),
            previous_short_before: [false; F07_FULLBAND_LANES],
            short_source: [[0; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
            other_source: [[0; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
            short_consumed: [[0; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
            other_consumed: [[0; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
            structure_refresh: [[false; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
            dither_flags: [[false; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
            dither_syntax_present: false,
        },
        after_state: F07SourceFrameAfterState {
            input_histories: std::array::from_fn(|_| InputHistory::cold()),
            transient_states: std::array::from_fn(|_| TransientChannelState::cold()),
            previous_short: [false; F07_FULLBAND_LANES],
        },
    };
    let source_stage_key = SourceStageKey::from_parent_snapshot(
        attempt_id,
        transaction_id,
        source_index,
        AnalysisOwnerGeneration::from_parent_snapshot(before_generation),
    );
    SharedAnalysisReceipt::from_parent(
        source_stage_key,
        AnalysisOwnerGeneration::from_parent_snapshot(after_generation),
        proposal,
    )
}
