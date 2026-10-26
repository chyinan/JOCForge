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
use crate::timeline::CodecSourceIndex;
use crate::transaction::{AttemptId, CallbackTransactionId};

const EXTERNAL_AUDIO_LANES: usize = 6;
const FBW_EXTERNAL_ROWS: [usize; F07_FULLBAND_LANES] = [0, 2, 1, 4, 5];

pub type InstalledAudioSourceFrame =
    [[[u32; INPUT_HISTORY_BLOCK_SAMPLES]; F07_BLOCKS_PER_FRAME]; EXTERNAL_AUDIO_LANES];
pub type DelayedAudioSourceFrame =
    Box<[[[u32; INPUT_HISTORY_BLOCK_SAMPLES]; F07_BLOCKS_PER_FRAME]]>;

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
            f07_proposal,
        }
    }

    pub const fn identity(&self) -> &SharedAnalysisIdentity {
        &self.identity
    }

    pub const fn f07_proposal(&self) -> &F07SourceFrameProposal {
        &self.f07_proposal
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
