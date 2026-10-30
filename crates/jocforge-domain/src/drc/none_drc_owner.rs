// pattern: Functional Core
//! Selected standalone DRC cold owner and same-source F06ND leaf proposal.
//!
//! The source proposal only stages statistics, scheduler, and zero-FIFO leaves.
//! The enclosing source parent still needs the F06NR secondary after-state
//! before it can make a complete DRC after-root or commit.

use crate::{
    analysis::source_frame::{SharedAnalysisIdentity, SharedAnalysisReceipt},
    drc::{
        none_drc_callback::{
            NoneDrcCallbackError, NoneDrcCallbackInput, NoneDrcCallbackTransition,
            transition_none_drc_callback,
        },
        none_drc_override_fifo::NoneDrcOverrideFifoState,
        none_drc_scheduler::NoneDrcSchedulerState,
        none_drc_statistics::{FiniteStatisticsPlanes, NoneDrcStatisticsState},
    },
    meter::{
        measurement_receipt::ProgrammeMeasurementReceipt,
        prepared_job_identity::PreparedJobInputIdentity,
        programme_measurement::{ProgrammeMeasurementError, dialnorm_from_selected_lufs},
    },
    timeline::{CodecSourceIndex, FrontendCallbackIndex, source_for_callback},
    transaction::{
        AttemptId, CallbackLineageIdentity, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt,
        GtcOwnerReceiptError, GtcOwnerStateSnapshot, ProgrammeId,
    },
    transform::{long_transform::LongTransformHistory, secondary_overlap::SecondaryOverlapCarry},
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const SECONDARY_LANE_COUNT: usize = 5;
#[allow(dead_code)] // Consumed by the future GTC owner-state envelope adapter.
const DRC_OWNER_ID: &str = "drc";
#[allow(dead_code)] // Consumed by the future GTC owner-state envelope adapter.
const DRC_STATE_REF: &str = "owners.drc";

/// The selected secondary state owned by the DRC partition.
///
/// Ordinary F07 input history remains owned by the shared Analysis root. This
/// value records only five independent forward histories, five inverse
/// overlap carries, and their common source-frame index.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct NoneDrcSecondaryState {
    forward_history: [LongTransformHistory; SECONDARY_LANE_COUNT],
    inverse_carry: [SecondaryOverlapCarry; SECONDARY_LANE_COUNT],
    frame_index: u64,
}

impl NoneDrcSecondaryState {
    pub(crate) fn cold() -> Self {
        Self {
            forward_history: std::array::from_fn(|_| LongTransformHistory::zeroed()),
            inverse_carry: std::array::from_fn(|_| SecondaryOverlapCarry::cold()),
            frame_index: 0,
        }
    }

    pub(crate) const fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub(crate) const fn forward_history(&self) -> &[LongTransformHistory; SECONDARY_LANE_COUNT] {
        &self.forward_history
    }

    pub(crate) const fn inverse_carry(&self) -> &[SecondaryOverlapCarry; SECONDARY_LANE_COUNT] {
        &self.inverse_carry
    }

    /// Used only by the future F06NR source-stage composer after it has run all
    /// five lanes and six blocks from this exact before-state.
    pub(crate) fn from_f06nr_after(
        forward_history: [LongTransformHistory; SECONDARY_LANE_COUNT],
        inverse_carry: [SecondaryOverlapCarry; SECONDARY_LANE_COUNT],
        frame_index: u64,
    ) -> Self {
        Self {
            forward_history,
            inverse_carry,
            frame_index,
        }
    }
}

/// Measurement identity and frozen dialnorm bound to one standalone DRC root.
#[allow(dead_code)] // The selected parent receipt path is not wired yet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NoneDrcMeasurementBinding {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    input_sha256: [u8; 32],
    config_sha256: [u8; 32],
    meter_terminal_sha256: [u8; 32],
    programme_dialnorm: u8,
    prepared_job_input_identity: Option<Arc<PreparedJobInputIdentity>>,
}

impl NoneDrcMeasurementBinding {
    pub(crate) fn from_receipt(
        receipt: &ProgrammeMeasurementReceipt,
    ) -> Result<Self, NoneDrcOwnerError> {
        let programme_dialnorm = dialnorm_from_selected_lufs(receipt.selected_lufs_word())
            .map_err(NoneDrcOwnerError::MeasurementDialnorm)?;
        Ok(Self {
            programme_id: receipt.programme_id().clone(),
            attempt: receipt.attempt(),
            input_sha256: *receipt.input_sha256(),
            config_sha256: *receipt.config_sha256(),
            meter_terminal_sha256: *receipt.meter_terminal_sha256(),
            programme_dialnorm,
            prepared_job_input_identity: None,
        })
    }

    pub(crate) fn from_receipt_for_prepared_input(
        receipt: &ProgrammeMeasurementReceipt,
        prepared_job_input_identity: Arc<PreparedJobInputIdentity>,
    ) -> Result<Self, NoneDrcOwnerError> {
        if receipt.input_sha256() != &prepared_job_input_identity.resource().input_sha256()
            || receipt.config_sha256()
                != &prepared_job_input_identity
                    .normalized_configuration()
                    .config_sha256()
        {
            return Err(NoneDrcOwnerError::PreparedJobInputIdentityMismatch);
        }
        let mut binding = Self::from_receipt(receipt)?;
        binding.prepared_job_input_identity = Some(prepared_job_input_identity);
        Ok(binding)
    }

    pub const fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> AttemptId {
        self.attempt
    }

    #[allow(dead_code)] // Retained for the future GTC receipt join.
    pub const fn input_sha256(&self) -> &[u8; 32] {
        &self.input_sha256
    }

    #[allow(dead_code)] // Retained for the future GTC receipt join.
    pub const fn config_sha256(&self) -> &[u8; 32] {
        &self.config_sha256
    }

    #[allow(dead_code)] // Retained for the future GTC receipt join.
    pub const fn meter_terminal_sha256(&self) -> &[u8; 32] {
        &self.meter_terminal_sha256
    }

    pub const fn programme_dialnorm(&self) -> u8 {
        self.programme_dialnorm
    }

    pub fn prepared_job_input_identity(&self) -> Option<&Arc<PreparedJobInputIdentity>> {
        self.prepared_job_input_identity.as_ref()
    }
}

/// Standalone DRC owner state composed from the existing selected leaf states.
///
/// It intentionally excludes ordinary F07 input history, which belongs to the
/// shared Analysis owner. It does not implement a state-ref serializer or a
/// top-level GTC owner receipt.
#[derive(Debug, PartialEq)]
pub struct NoneDrcOwnerState {
    secondary: NoneDrcSecondaryState,
    statistics: NoneDrcStatisticsState,
    scheduler: NoneDrcSchedulerState,
    override_fifo: NoneDrcOverrideFifoState,
    programme_dialnorm: Option<u8>,
}

impl NoneDrcOwnerState {
    fn cold(programme_dialnorm: Option<u8>) -> Self {
        Self {
            secondary: NoneDrcSecondaryState::cold(),
            statistics: NoneDrcStatisticsState::cold(),
            scheduler: NoneDrcSchedulerState::cold(),
            override_fifo: NoneDrcOverrideFifoState::cold(),
            programme_dialnorm,
        }
    }

    pub fn source_generation(&self) -> Result<CodecSourceIndex, NoneDrcOwnerError> {
        let generation = self.scheduler.generation();
        if self.override_fifo.frames_processed() != generation
            || self.secondary.frame_index() != generation
        {
            return Err(NoneDrcOwnerError::StateGenerationMismatch);
        }
        Ok(CodecSourceIndex::new(generation))
    }

    pub const fn programme_dialnorm(&self) -> Option<u8> {
        self.programme_dialnorm
    }

    pub(crate) const fn secondary(&self) -> &NoneDrcSecondaryState {
        &self.secondary
    }

    #[allow(dead_code)] // Read by the future source-root commit adapter.
    pub(crate) const fn statistics(&self) -> &NoneDrcStatisticsState {
        &self.statistics
    }

    #[allow(dead_code)] // Read by the future source-root commit adapter.
    pub(crate) const fn scheduler(&self) -> &NoneDrcSchedulerState {
        &self.scheduler
    }

    #[allow(dead_code)] // Read by the future source-root commit adapter.
    pub(crate) const fn override_fifo(&self) -> &NoneDrcOverrideFifoState {
        &self.override_fifo
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, NoneDrcOwnerError> {
        if self
            .programme_dialnorm
            .is_some_and(|dialnorm| !(1..=31).contains(&dialnorm))
        {
            return Err(NoneDrcOwnerError::CanonicalSerialization);
        }

        let forward_history = self
            .secondary
            .forward_history
            .iter()
            .map(|history| history.words().to_vec())
            .collect::<Vec<_>>();
        let inverse_carry = self
            .secondary
            .inverse_carry
            .iter()
            .map(|carry| carry.words().to_vec())
            .collect::<Vec<_>>();
        let filter_pairs = self
            .statistics
            .filter_pairs()
            .iter()
            .map(|pair| [pair[0].to_bits(), pair[1].to_bits()])
            .collect::<Vec<_>>();
        let previous_primary_peak = self
            .statistics
            .previous_primary_peak()
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>();
        let previous_secondary_peak = self
            .statistics
            .previous_secondary_peak()
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>();
        let (line_level, line_hold, line_log_gain) = self.scheduler.line_dynamics_words();
        let (rf_level, rf_hold, rf_log_gain) = self.scheduler.rf_dynamics_words();
        let (line_limiter_gain, line_limiter_hold) = self.scheduler.line_limiter_words();
        let (rf_limiter_gain, rf_limiter_hold) = self.scheduler.rf_limiter_words();
        let peak_logs = self.scheduler.peak_six_words();

        let mut finite_words = forward_history
            .iter()
            .flatten()
            .chain(inverse_carry.iter().flatten())
            .copied()
            .chain(filter_pairs.iter().flatten().copied())
            .chain(previous_primary_peak.iter().copied())
            .chain(previous_secondary_peak.iter().copied())
            .chain([self.statistics.previous_downmix_peak().to_bits()])
            .chain([line_level, line_log_gain, rf_level, rf_log_gain])
            .chain([line_limiter_gain, rf_limiter_gain])
            .chain(peak_logs);
        if finite_words.any(|word| !f32::from_bits(word).is_finite()) {
            return Err(NoneDrcOwnerError::CanonicalSerialization);
        }

        let root_value = serde_json::json!({
            "metadata_override_fifo": {
                "append_slot": self.override_fifo.append_slot(),
                "frames_processed_u64": self.override_fifo.frames_processed(),
                "metadata_delay_samples": self.override_fifo.metadata_delay_samples(),
                "records": self.override_fifo.records().iter().map(|_| 0_u8).collect::<Vec<_>>(),
            },
            "programme_dialnorm": self.programme_dialnorm,
            "scheduler": {
                "generation_u64": self.scheduler.generation(),
                "line_dynamics": {
                    "hold_u16": line_hold,
                    "level_f32_u32_word": line_level,
                    "log_gain_f32_u32_word": line_log_gain,
                },
                "line_limiter": {
                    "gain_f32_u32_word": line_limiter_gain,
                    "hold_u16": line_limiter_hold,
                },
                "peak_logs_f32_u32_words": peak_logs,
                "rf_dynamics": {
                    "hold_u16": rf_hold,
                    "level_f32_u32_word": rf_level,
                    "log_gain_f32_u32_word": rf_log_gain,
                },
                "rf_limiter": {
                    "gain_f32_u32_word": rf_limiter_gain,
                    "hold_u16": rf_limiter_hold,
                },
            },
            "secondary": {
                "forward_history_f32_u32_words": forward_history,
                "frame_index_u64": self.secondary.frame_index(),
                "inverse_carry_f32_u32_words": inverse_carry,
            },
            "statistics": {
                "filter_pairs_f32_u32_words": filter_pairs,
                "previous_downmix_peak_f32_u32_word": self.statistics.previous_downmix_peak().to_bits(),
                "previous_primary_peak_f32_u32_words": previous_primary_peak,
                "previous_secondary_peak_f32_u32_words": previous_secondary_peak,
            },
        });
        serde_json::to_vec(&root_value).map_err(|_| NoneDrcOwnerError::CanonicalSerialization)
    }
}

fn none_drc_state_reference(
    state: &NoneDrcOwnerState,
) -> Result<NoneDrcOwnerStateReference, NoneDrcOwnerError> {
    let generation = state.source_generation()?.value();
    let canonical_bytes = state.canonical_bytes()?;
    let state_sha256 = Sha256::digest(&canonical_bytes).into();
    let cold_bytes = NoneDrcOwnerState::cold(None).canonical_bytes()?;
    let cold_sha256 = Sha256::digest(&cold_bytes).into();
    Ok(NoneDrcOwnerStateReference {
        owner_id: DRC_OWNER_ID,
        state_ref: DRC_STATE_REF,
        generation,
        state_sha256,
        cold_sha256,
        canonical_bytes,
    })
}

/// Complete canonical owner-state reference for the standalone GTC DRC member.
#[allow(dead_code)] // Consumed by the future MeasurementTransaction/GTC parent.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct NoneDrcOwnerStateReference {
    owner_id: &'static str,
    state_ref: &'static str,
    generation: u64,
    state_sha256: [u8; 32],
    cold_sha256: [u8; 32],
    canonical_bytes: Vec<u8>,
}

#[allow(dead_code)] // Consumed by the future MeasurementTransaction/GTC parent.
impl NoneDrcOwnerStateReference {
    pub const fn owner_id(&self) -> &'static str {
        self.owner_id
    }

    pub const fn state_ref(&self) -> &'static str {
        self.state_ref
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn state_sha256(&self) -> &[u8; 32] {
        &self.state_sha256
    }

    pub const fn cold_sha256(&self) -> &[u8; 32] {
        &self.cold_sha256
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Projects the complete DRC owner preimage into the registered GTC owner slot.
    ///
    /// This is an owner snapshot only; it does not admit the receipt to a parent transaction.
    pub(crate) fn gtc_snapshot(&self) -> Result<GtcOwnerStateSnapshot<'_>, GtcOwnerReceiptError> {
        GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::Drc,
            self.generation,
            self.cold_sha256,
            &self.canonical_bytes,
        )
    }
}

/// Primary and secondary F06ND planes prepared for one source generation.
/// The parent binds both to the same SharedAnalysisReceipt before construction.
pub(crate) struct NoneDrcSourcePlanes<'a> {
    shared_analysis_identity: SharedAnalysisIdentity,
    primary: &'a FiniteStatisticsPlanes,
    secondary: &'a FiniteStatisticsPlanes,
}

impl<'a> NoneDrcSourcePlanes<'a> {
    pub(crate) fn from_parent_analysis(
        shared_analysis: &SharedAnalysisReceipt,
        primary: &'a FiniteStatisticsPlanes,
        secondary: &'a FiniteStatisticsPlanes,
    ) -> Self {
        Self {
            shared_analysis_identity: shared_analysis.identity().clone(),
            primary,
            secondary,
        }
    }
}

/// Selected standalone DRC owner initialized after meter finalization.
///
/// It owns the local DRC leaf states and immutable measurement binding. It
/// carries no ordinary history and has no local commit or restart authority.
#[derive(Debug, PartialEq)]
pub struct NoneDrcOwnerRoot {
    measurement: Option<NoneDrcMeasurementBinding>,
    state: NoneDrcOwnerState,
}

impl NoneDrcOwnerRoot {
    /// Creates the complete selected DRC root before programme measurement is finalized.
    #[allow(dead_code)] // Installed by the future MeasurementTransaction parent root.
    pub(crate) fn cold_before_meter_finalize() -> Self {
        Self {
            measurement: None,
            state: NoneDrcOwnerState::cold(None),
        }
    }

    pub fn cold_after_meter_finalize(
        receipt: &ProgrammeMeasurementReceipt,
    ) -> Result<Self, NoneDrcOwnerError> {
        let measurement = NoneDrcMeasurementBinding::from_receipt(receipt)?;
        let state = NoneDrcOwnerState::cold(Some(measurement.programme_dialnorm()));
        Ok(Self {
            measurement: Some(measurement),
            state,
        })
    }

    /// Creates a production-bound DRC root from the same input capability used by the meter,
    /// source Analysis, and PreparedJobInput. The receipt hashes must match that admitted input.
    pub fn cold_after_meter_finalize_for_prepared_input(
        receipt: &ProgrammeMeasurementReceipt,
        prepared_job_input_identity: Arc<PreparedJobInputIdentity>,
    ) -> Result<Self, NoneDrcOwnerError> {
        let measurement = NoneDrcMeasurementBinding::from_receipt_for_prepared_input(
            receipt,
            prepared_job_input_identity,
        )?;
        let state = NoneDrcOwnerState::cold(Some(measurement.programme_dialnorm()));
        Ok(Self {
            measurement: Some(measurement),
            state,
        })
    }

    pub(crate) fn matches_prepared_job_input_identity(
        &self,
        shared_analysis: &SharedAnalysisReceipt,
    ) -> bool {
        match (
            self.measurement
                .as_ref()
                .and_then(NoneDrcMeasurementBinding::prepared_job_input_identity),
            shared_analysis.identity().prepared_job_input_identity(),
        ) {
            (None, None) => true,
            (Some(drc), Some(analysis)) => Arc::ptr_eq(drc, analysis),
            _ => false,
        }
    }

    pub(crate) const fn measurement(&self) -> Option<&NoneDrcMeasurementBinding> {
        self.measurement.as_ref()
    }

    pub const fn state(&self) -> &NoneDrcOwnerState {
        &self.state
    }

    /// Produces the canonical standalone `owners.drc` state reference.
    ///
    /// The measurement receipt and immutable binding are shared parent inputs;
    /// the referenced preimage contains only the complete selected DRC root value.
    #[allow(dead_code)] // Consumed by the future MeasurementTransaction/GTC parent.
    pub(crate) fn state_reference(&self) -> Result<NoneDrcOwnerStateReference, NoneDrcOwnerError> {
        none_drc_state_reference(&self.state)
    }

    /// Projects an unchanged DRC owner receipt for a callback before codec source dispatch.
    ///
    /// The selected meter receipt must already have constructed this DRC root. Callbacks 0 and
    /// 1 do not advance F06ND/F06NR state; later callbacks require a source proposal instead.
    #[allow(dead_code)] // Consumed by the future 12-owner callback coordinator.
    pub(crate) fn gtc_unchanged_callback_receipt(
        &self,
        callback_index: FrontendCallbackIndex,
        transaction_id: CallbackTransactionId,
    ) -> Result<GtcOwnerReceipt, NoneDrcOwnerError> {
        if source_for_callback(callback_index).is_some() {
            return Err(NoneDrcOwnerError::SourceStageRequired);
        }
        if self.measurement.is_none() || self.state.programme_dialnorm.is_none() {
            return Err(NoneDrcOwnerError::MeasurementNotFinalized);
        }
        if self.state.source_generation()?.value() != 0 {
            return Err(NoneDrcOwnerError::UnchangedSourceAlreadyAdvanced);
        }
        let reference = self.state_reference()?;
        let snapshot = reference
            .gtc_snapshot()
            .map_err(NoneDrcOwnerError::GtcReceipt)?;
        GtcOwnerReceipt::from_transition(snapshot, snapshot, transaction_id)
            .map_err(NoneDrcOwnerError::GtcReceipt)
    }

    /// Prepares the F06ND statistics/scheduler/FIFO substage for the same source
    /// as the parent-issued SharedAnalysisReceipt. No state is installed here;
    /// the F06NR after-state is joined before returning the complete DRC proposal.
    pub(crate) fn prepare_f06nd_source(
        &self,
        shared_analysis: &SharedAnalysisReceipt,
        callback_identity: &CallbackLineageIdentity,
        source_planes: &NoneDrcSourcePlanes<'_>,
    ) -> Result<NoneDrcF06ndStageProposal, NoneDrcOwnerError> {
        let Some(measurement) = self.measurement.as_ref() else {
            return Err(NoneDrcOwnerError::MeasurementNotFinalized);
        };
        let Some(programme_dialnorm) = self.state.programme_dialnorm() else {
            return Err(NoneDrcOwnerError::MeasurementNotFinalized);
        };
        if programme_dialnorm != measurement.programme_dialnorm() {
            return Err(NoneDrcOwnerError::MeasurementOptionMismatch);
        }
        let shared_identity = shared_analysis.identity();
        let source_stage_key = shared_identity.source_stage_key();
        if callback_identity.programme_id() != measurement.programme_id() {
            return Err(NoneDrcOwnerError::ProgrammeMismatch);
        }
        if callback_identity.attempt_id() != measurement.attempt()
            || source_stage_key.attempt_id() != measurement.attempt()
        {
            return Err(NoneDrcOwnerError::AttemptMismatch);
        }
        if !self.matches_prepared_job_input_identity(shared_analysis) {
            return Err(NoneDrcOwnerError::PreparedJobInputIdentityMismatch);
        }
        if source_stage_key.analysis_owner_id()
            != crate::analysis::source_frame::AnalysisOwnerId::Analysis
            || shared_analysis.advancement_count() != 1
        {
            return Err(NoneDrcOwnerError::SharedAnalysisIdentity);
        }
        let callback_index = callback_identity.callback_index();
        let Some(source_from_callback) = source_for_callback(callback_index) else {
            return Err(NoneDrcOwnerError::PrimingHasNoSource);
        };
        let source_index = shared_identity.source_index();
        if source_from_callback != source_index
            || &source_planes.shared_analysis_identity != shared_identity
        {
            return Err(NoneDrcOwnerError::SourceMismatch);
        }
        if self.state.source_generation()? != source_index {
            return Err(NoneDrcOwnerError::StateGenerationMismatch);
        }

        let callback_input = NoneDrcCallbackInput {
            callback_index,
            primary_samples: source_planes.primary,
            secondary_samples: source_planes.secondary,
            dialnorm: i32::from(programme_dialnorm),
        };
        let Some(callback_transition) = transition_none_drc_callback(
            &self.state.statistics,
            &self.state.scheduler,
            &self.state.override_fifo,
            &callback_input,
        )
        .map_err(NoneDrcOwnerError::Callback)?
        else {
            return Err(NoneDrcOwnerError::PrimingHasNoSource);
        };
        let clock = callback_transition.clock();
        if clock.source_index != source_index {
            return Err(NoneDrcOwnerError::SourceMismatch);
        }

        Ok(NoneDrcF06ndStageProposal {
            measurement: measurement.clone(),
            shared_analysis_identity: shared_identity.clone(),
            callback_identity: callback_identity.clone(),
            callback_transition,
        })
    }
}

/// One noncommitting F06ND source proposal. It contains the exact shared-source
/// and callback identities plus the existing statistics/scheduler/FIFO leaves.
#[must_use]
#[derive(Debug, PartialEq)]
pub(crate) struct NoneDrcF06ndStageProposal {
    measurement: NoneDrcMeasurementBinding,
    shared_analysis_identity: SharedAnalysisIdentity,
    callback_identity: CallbackLineageIdentity,
    callback_transition: NoneDrcCallbackTransition,
}

impl NoneDrcF06ndStageProposal {
    #[cfg(test)]
    pub const fn shared_analysis_identity(&self) -> &SharedAnalysisIdentity {
        &self.shared_analysis_identity
    }

    #[cfg(test)]
    pub const fn callback_identity(&self) -> &CallbackLineageIdentity {
        &self.callback_identity
    }

    #[cfg(test)]
    pub const fn callback_transition(&self) -> &NoneDrcCallbackTransition {
        &self.callback_transition
    }

    #[cfg(test)]
    pub const fn bsi_projection(
        &self,
    ) -> &crate::drc::none_drc_bsi_projection::NoneDrcBsiProjection {
        &self.callback_transition.frame().output.bsi_projection
    }

    /// Joins the same-source F06NR after-state to the F06ND leaves. This is a
    /// complete noncommitting DRC after-root value; the source parent still
    /// owns the single commit and the GTC OwnerReceipt.
    pub(crate) fn join_f06nr_after(
        self,
        secondary_after: NoneDrcSecondaryState,
    ) -> Result<NoneDrcOwnerAfterProposal, NoneDrcOwnerError> {
        let source_index = self.shared_analysis_identity.source_index();
        let generation_after = source_index
            .value()
            .checked_add(1)
            .ok_or(NoneDrcOwnerError::GenerationOverflow)?;
        let (clock, frame, fifo_transition, dialnorm) = self.callback_transition.into_parts();
        let (_, fifo_after) = fifo_transition.into_parts();
        if clock.source_index != source_index
            || self.callback_identity.callback_index() != clock.callback_index
            || i32::from(self.measurement.programme_dialnorm()) != dialnorm
            || frame.next_scheduler_state.generation() != generation_after
            || fifo_after.frames_processed() != generation_after
            || secondary_after.frame_index() != generation_after
        {
            return Err(NoneDrcOwnerError::AfterStateGenerationMismatch);
        }

        let state = NoneDrcOwnerState {
            secondary: secondary_after,
            statistics: frame.next_statistics_state,
            scheduler: frame.next_scheduler_state,
            override_fifo: fifo_after,
            programme_dialnorm: Some(self.measurement.programme_dialnorm()),
        };
        Ok(NoneDrcOwnerAfterProposal {
            measurement: self.measurement,
            shared_analysis_identity: self.shared_analysis_identity,
            callback_identity: self.callback_identity,
            source_index,
            frame_output: frame.output,
            state,
        })
    }
}

/// Complete DRC after-state proposal for one source. It carries no commit,
/// reset, or fatal authority and has no locally minted OwnerReceipt.
#[must_use]
#[derive(Debug, PartialEq)]
pub struct NoneDrcOwnerAfterProposal {
    measurement: NoneDrcMeasurementBinding,
    shared_analysis_identity: SharedAnalysisIdentity,
    callback_identity: CallbackLineageIdentity,
    source_index: CodecSourceIndex,
    frame_output: crate::drc::none_drc_frame::NoneDrcFrameOutput,
    state: NoneDrcOwnerState,
}

impl NoneDrcOwnerAfterProposal {
    pub fn programme_id(&self) -> &ProgrammeId {
        self.measurement.programme_id()
    }

    pub const fn attempt(&self) -> AttemptId {
        self.measurement.attempt()
    }

    pub const fn programme_dialnorm(&self) -> u8 {
        self.measurement.programme_dialnorm()
    }

    pub const fn shared_analysis_identity(&self) -> &SharedAnalysisIdentity {
        &self.shared_analysis_identity
    }

    pub const fn callback_identity(&self) -> &CallbackLineageIdentity {
        &self.callback_identity
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    /// Returns the exact input-admission capability retained by the meter-finalized DRC owner.
    pub fn prepared_job_input_identity(&self) -> Option<&Arc<PreparedJobInputIdentity>> {
        self.measurement.prepared_job_input_identity()
    }

    pub const fn meter_terminal_sha256(&self) -> &[u8; 32] {
        self.measurement.meter_terminal_sha256()
    }

    pub const fn frame_output(&self) -> &crate::drc::none_drc_frame::NoneDrcFrameOutput {
        &self.frame_output
    }

    pub const fn state(&self) -> &NoneDrcOwnerState {
        &self.state
    }

    /// Projects this noncommitting DRC after-root into one owner-level GTC receipt.
    ///
    /// The caller supplies the exact committed before-root and shared callback transaction ID.
    /// The proposal must carry that same parent token through its shared-analysis source key;
    /// this method never installs either root or assigns top-level commit authority.
    #[allow(dead_code)] // Consumed by the future 12-owner source transaction preflight.
    pub(crate) fn gtc_owner_receipt(
        &self,
        before: &NoneDrcOwnerRoot,
        transaction_id: CallbackTransactionId,
    ) -> Result<GtcOwnerReceipt, NoneDrcOwnerError> {
        let Some(before_measurement) = before.measurement.as_ref() else {
            return Err(NoneDrcOwnerError::MeasurementNotFinalized);
        };
        if before_measurement != &self.measurement
            || before.state.programme_dialnorm != Some(before_measurement.programme_dialnorm())
            || self.state.programme_dialnorm != Some(self.measurement.programme_dialnorm())
        {
            return Err(NoneDrcOwnerError::MeasurementOptionMismatch);
        }

        let source_key = self.shared_analysis_identity.source_stage_key();
        if source_key.transaction_id() != &transaction_id {
            return Err(NoneDrcOwnerError::ParentTransactionMismatch);
        }
        if self.callback_identity.programme_id() != before_measurement.programme_id()
            || self.measurement.programme_id() != before_measurement.programme_id()
        {
            return Err(NoneDrcOwnerError::ProgrammeMismatch);
        }
        if self.callback_identity.attempt_id() != before_measurement.attempt()
            || self.measurement.attempt() != before_measurement.attempt()
            || source_key.attempt_id() != before_measurement.attempt()
        {
            return Err(NoneDrcOwnerError::AttemptMismatch);
        }
        if source_key.source_index() != self.source_index
            || self.shared_analysis_identity.source_index() != self.source_index
            || source_for_callback(self.callback_identity.callback_index())
                != Some(self.source_index)
        {
            return Err(NoneDrcOwnerError::SourceMismatch);
        }

        let before_generation = before.state.source_generation()?;
        if before_generation != self.source_index {
            return Err(NoneDrcOwnerError::BeforeStateMismatch);
        }
        let expected_after_generation = self
            .source_index
            .value()
            .checked_add(1)
            .ok_or(NoneDrcOwnerError::GenerationOverflow)?;
        if self.state.source_generation()? != CodecSourceIndex::new(expected_after_generation) {
            return Err(NoneDrcOwnerError::AfterStateMismatch);
        }

        match (
            before_measurement.prepared_job_input_identity(),
            self.measurement.prepared_job_input_identity(),
            self.shared_analysis_identity.prepared_job_input_identity(),
        ) {
            (None, None, None) => {}
            (Some(before), Some(proposal), Some(shared))
                if Arc::ptr_eq(before, proposal) && Arc::ptr_eq(proposal, shared) => {}
            _ => return Err(NoneDrcOwnerError::PreparedJobInputIdentityMismatch),
        }

        let before_reference = before.state_reference()?;
        let before_snapshot = before_reference
            .gtc_snapshot()
            .map_err(NoneDrcOwnerError::GtcReceipt)?;
        let after_reference = none_drc_state_reference(&self.state)?;
        let after_snapshot = after_reference
            .gtc_snapshot()
            .map_err(NoneDrcOwnerError::GtcReceipt)?;
        GtcOwnerReceipt::from_transition(before_snapshot, after_snapshot, transaction_id)
            .map_err(NoneDrcOwnerError::GtcReceipt)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoneDrcOwnerError {
    MeasurementDialnorm(ProgrammeMeasurementError),
    MeasurementNotFinalized,
    MeasurementOptionMismatch,
    PreparedJobInputIdentityMismatch,
    CanonicalSerialization,
    ProgrammeMismatch,
    AttemptMismatch,
    SharedAnalysisIdentity,
    PrimingHasNoSource,
    SourceMismatch,
    StateGenerationMismatch,
    GenerationOverflow,
    AfterStateGenerationMismatch,
    SourceStageRequired,
    UnchangedSourceAlreadyAdvanced,
    BeforeStateMismatch,
    AfterStateMismatch,
    ParentTransactionMismatch,
    GtcReceipt(GtcOwnerReceiptError),
    Callback(NoneDrcCallbackError),
}

impl NoneDrcOwnerError {
    pub const fn code(self) -> Option<&'static str> {
        match self {
            Self::MeasurementDialnorm(_)
            | Self::MeasurementNotFinalized
            | Self::MeasurementOptionMismatch
            | Self::PreparedJobInputIdentityMismatch
            | Self::CanonicalSerialization
            | Self::SharedAnalysisIdentity => None,
            Self::ProgrammeMismatch
            | Self::AttemptMismatch
            | Self::PrimingHasNoSource
            | Self::SourceMismatch
            | Self::StateGenerationMismatch
            | Self::AfterStateGenerationMismatch
            | Self::SourceStageRequired
            | Self::UnchangedSourceAlreadyAdvanced
            | Self::BeforeStateMismatch
            | Self::AfterStateMismatch
            | Self::ParentTransactionMismatch => Some("DRC_INPUT_ERROR"),
            Self::GtcReceipt(_) => Some("DRC_OWNER_RECEIPT_ERROR"),
            Self::GenerationOverflow => Some("NUMERIC_FAILURE"),
            Self::Callback(error) => error.outcome_code(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        analysis::source_frame::{
            AnalysisOwnerGeneration, zeroed_shared_analysis_receipt_for_identity_tests,
        },
        drc::none_drc_override_fifo::ZeroLogicalRecord,
        meter::measurement_receipt::ProgrammeMeasurementReceiptInput,
        meter::prepared_job_identity::{
            NormalizedJobConfigurationIdentity, PreparedAdmChunkSpan, PreparedAdmGraphProvenance,
            PreparedInputResourceIdentity, PreparedJobInputIdentity,
        },
        numeric::FiniteF32,
        timeline::FrontendCallbackIndex,
        transaction::CallbackTransactionId,
    };
    use serde_json::Value;
    use std::sync::Arc;

    const SOURCE_LANE_COUNT: usize = 6;
    const SOURCE_SAMPLE_COUNT: usize = 6 * 256;
    const OPTION_COLD_VECTORS: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/DRC-STANDALONE-OWNER-OPTIONAL-DIALNORM-V1/INPUT-ONLY-OPTION-COLD-VECTOR.json"
    );

    fn measurement_receipt() -> ProgrammeMeasurementReceipt {
        measurement_receipt_with_selected_lufs(-23.0)
    }

    fn measurement_receipt_with_selected_lufs(selected_lufs: f32) -> ProgrammeMeasurementReceipt {
        ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
            programme_id: ProgrammeId::new("drc-root-test").unwrap(),
            attempt: AttemptId::new(3),
            input_sha256: [0x11; 32],
            config_sha256: [0x22; 32],
            dialogue_word: (-23.0_f32).to_bits(),
            itu_word: (-23.0_f32).to_bits(),
            true_peak_word: (-1.0_f32).to_bits(),
            gating_kind: "LEVEL",
            selected_lufs_word: selected_lufs.to_bits(),
            meter_terminal_sha256: [0x33; 32],
            normal_callbacks: 4_247,
            zero_drains: 3,
            energy_windows: 6_794,
        })
        .unwrap()
    }

    fn prepared_job_identity(
        input_sha256: [u8; 32],
        config_sha256: [u8; 32],
    ) -> Arc<PreparedJobInputIdentity> {
        let resource = PreparedInputResourceIdentity::new(input_sha256, 1_024);
        let graph = PreparedAdmGraphProvenance::new(
            resource,
            PreparedAdmChunkSpan::new(*b"axml", 10, 10, [0x41; 32]),
            PreparedAdmChunkSpan::new(*b"chna", 20, 10, [0x42; 32]),
            PreparedAdmChunkSpan::new(*b"dbmd", 30, 10, [0x43; 32]),
        );
        Arc::new(
            PreparedJobInputIdentity::new(
                resource,
                NormalizedJobConfigurationIdentity::new(
                    config_sha256,
                    [0x44; 32],
                    crate::frontend::job_configuration::SelectedInputKind::Adm,
                ),
                graph,
            )
            .unwrap(),
        )
    }

    fn hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut encoded = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            encoded.push(char::from(HEX[usize::from(*byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(*byte & 0x0f)]));
        }
        encoded
    }

    fn callback_identity(callback_index: u32, attempt: u64) -> CallbackLineageIdentity {
        CallbackLineageIdentity::new(
            ProgrammeId::new("drc-root-test").unwrap(),
            AttemptId::new(attempt),
            FrontendCallbackIndex::new(callback_index),
        )
    }

    fn zero_planes() -> Box<FiniteStatisticsPlanes> {
        let zero = FiniteF32::new(0.0).unwrap();
        let rows = vec![[zero; SOURCE_SAMPLE_COUNT]; SOURCE_LANE_COUNT].into_boxed_slice();
        Box::<FiniteStatisticsPlanes>::try_from(rows).unwrap()
    }

    fn source_zero_after(
        root: &NoneDrcOwnerRoot,
        transaction_id: CallbackTransactionId,
    ) -> NoneDrcOwnerAfterProposal {
        let attempt = root.measurement().unwrap().attempt();
        let shared_analysis = zeroed_shared_analysis_receipt_for_identity_tests(
            attempt,
            transaction_id,
            CodecSourceIndex::new(0),
            7,
            8,
        );
        let planes = zero_planes();
        let source_planes = NoneDrcSourcePlanes::from_parent_analysis(
            &shared_analysis,
            planes.as_ref(),
            planes.as_ref(),
        );
        let f06nd = root
            .prepare_f06nd_source(
                &shared_analysis,
                &callback_identity(2, attempt.value()),
                &source_planes,
            )
            .unwrap();
        let secondary_after = NoneDrcSecondaryState::from_f06nr_after(
            std::array::from_fn(|_| LongTransformHistory::zeroed()),
            std::array::from_fn(|_| SecondaryOverlapCarry::cold()),
            1,
        );
        f06nd.join_f06nr_after(secondary_after).unwrap()
    }

    #[test]
    fn cold_root_owns_selected_drc_leaves_and_frozen_measurement_binding() {
        let receipt = measurement_receipt();
        let root = NoneDrcOwnerRoot::cold_after_meter_finalize(&receipt).unwrap();
        let measurement = root.measurement().expect("finalized measurement is bound");
        assert_eq!(measurement.programme_id(), receipt.programme_id());
        assert_eq!(measurement.attempt(), receipt.attempt());
        assert_eq!(
            measurement.meter_terminal_sha256(),
            receipt.meter_terminal_sha256()
        );
        assert_eq!(
            root.state().programme_dialnorm(),
            Some(dialnorm_from_selected_lufs(receipt.selected_lufs_word()).unwrap())
        );
        assert_eq!(
            root.state().source_generation().unwrap(),
            CodecSourceIndex::new(0)
        );
        assert_eq!(root.state().secondary().frame_index(), 0);
        assert!(
            root.state()
                .secondary()
                .forward_history()
                .iter()
                .all(|history| history.words().iter().all(|word| *word == 0))
        );
        assert!(
            root.state()
                .secondary()
                .inverse_carry()
                .iter()
                .all(|carry| carry.words().iter().all(|word| *word == 0))
        );
        assert_eq!(root.state().statistics().filter_pairs(), &[[0.0; 2]; 5]);
        assert_eq!(root.state().scheduler().generation(), 0);
        assert_eq!(root.state().override_fifo().frames_processed(), 0);
        assert_eq!(root.state().override_fifo().metadata_delay_samples(), 3072);
        assert_eq!(root.state().override_fifo().append_slot(), 3);
        assert_eq!(
            root.state().override_fifo().records(),
            &[ZeroLogicalRecord; 4]
        );
    }

    #[test]
    fn prepared_input_identity_is_retained_only_when_receipt_hashes_match() {
        let receipt = measurement_receipt();
        let identity = prepared_job_identity(*receipt.input_sha256(), *receipt.config_sha256());
        let root = NoneDrcOwnerRoot::cold_after_meter_finalize_for_prepared_input(
            &receipt,
            Arc::clone(&identity),
        )
        .unwrap();
        let stored = root
            .measurement()
            .unwrap()
            .prepared_job_input_identity()
            .unwrap();
        assert!(Arc::ptr_eq(&identity, stored));

        let different_resource = prepared_job_identity([0x77; 32], *receipt.config_sha256());
        assert_eq!(
            NoneDrcOwnerRoot::cold_after_meter_finalize_for_prepared_input(
                &receipt,
                different_resource,
            ),
            Err(NoneDrcOwnerError::PreparedJobInputIdentityMismatch)
        );

        let different_configuration = prepared_job_identity(*receipt.input_sha256(), [0x88; 32]);
        assert_eq!(
            NoneDrcOwnerRoot::cold_after_meter_finalize_for_prepared_input(
                &receipt,
                different_configuration,
            ),
            Err(NoneDrcOwnerError::PreparedJobInputIdentityMismatch)
        );
    }

    #[test]
    fn optional_dialnorm_owner_reference_matches_both_approved_root_preimages() {
        let vectors: Value = serde_json::from_slice(OPTION_COLD_VECTORS).unwrap();
        let expected_none = &vectors["null_dialnorm_cold_root"];
        let expected_some = &vectors["some_dialnorm_cold_root"];

        let none_root = NoneDrcOwnerRoot::cold_before_meter_finalize();
        assert!(none_root.measurement().is_none());
        assert_eq!(none_root.state().programme_dialnorm(), None);
        let none_reference = none_root.state_reference().unwrap();
        assert_eq!(none_reference.owner_id(), "drc");
        assert_eq!(none_reference.state_ref(), "owners.drc");
        assert_eq!(none_reference.generation(), 0);
        assert_eq!(
            none_reference.canonical_bytes().len(),
            expected_none["canonical_bytes"].as_u64().unwrap() as usize
        );
        assert_eq!(
            hex(none_reference.canonical_bytes()),
            expected_none["canonical_preimage_utf8_hex"]
        );
        assert_eq!(
            serde_json::to_vec(&expected_none["root_value"]).unwrap(),
            none_reference.canonical_bytes()
        );
        assert_eq!(
            hex(none_reference.state_sha256()),
            expected_none["state_sha256"]
        );
        assert_eq!(
            hex(none_reference.cold_sha256()),
            expected_none["cold_sha256"]
        );

        let receipt = measurement_receipt_with_selected_lufs(-24.0);
        let some_root = NoneDrcOwnerRoot::cold_after_meter_finalize(&receipt).unwrap();
        assert_eq!(some_root.state().programme_dialnorm(), Some(24));
        let some_reference = some_root.state_reference().unwrap();
        assert_eq!(some_reference.owner_id(), "drc");
        assert_eq!(some_reference.state_ref(), "owners.drc");
        assert_eq!(some_reference.generation(), 0);
        assert_eq!(
            some_reference.canonical_bytes().len(),
            expected_some["canonical_bytes"].as_u64().unwrap() as usize
        );
        assert_eq!(
            hex(some_reference.canonical_bytes()),
            expected_some["canonical_preimage_utf8_hex"]
        );
        assert_eq!(
            serde_json::to_vec(&expected_some["root_value"]).unwrap(),
            some_reference.canonical_bytes()
        );
        assert_eq!(
            hex(some_reference.state_sha256()),
            expected_some["state_sha256"]
        );
        assert_eq!(
            hex(some_reference.cold_sha256()),
            expected_some["cold_sha256"]
        );
        assert_eq!(some_reference.cold_sha256(), none_reference.state_sha256());
        assert_eq!(
            some_reference.generation(),
            none_reference.generation(),
            "measurement finalize changes only the optional measurement projection"
        );
        let mut expected_none_value = expected_none["root_value"].clone();
        expected_none_value["programme_dialnorm"] =
            expected_some["root_value"]["programme_dialnorm"].clone();
        assert_eq!(expected_none_value, expected_some["root_value"]);
    }

    #[test]
    fn drc_source_stage_rejects_the_pre_measurement_none_root() {
        let identity_receipt = measurement_receipt();
        let root = NoneDrcOwnerRoot::cold_before_meter_finalize();
        let shared_analysis = zeroed_shared_analysis_receipt_for_identity_tests(
            identity_receipt.attempt(),
            CallbackTransactionId::new("drc-before-measurement").unwrap(),
            CodecSourceIndex::new(0),
            7,
            8,
        );
        let planes = zero_planes();
        let source_planes = NoneDrcSourcePlanes::from_parent_analysis(
            &shared_analysis,
            planes.as_ref(),
            planes.as_ref(),
        );

        assert_eq!(
            root.prepare_f06nd_source(
                &shared_analysis,
                &callback_identity(2, identity_receipt.attempt().value()),
                &source_planes,
            ),
            Err(NoneDrcOwnerError::MeasurementNotFinalized)
        );
        assert_eq!(
            root.state().source_generation().unwrap(),
            CodecSourceIndex::new(0)
        );
    }

    #[test]
    fn f06nd_stage_joins_one_shared_source_without_installing_leaf_after_states() {
        let receipt = measurement_receipt();
        let root = NoneDrcOwnerRoot::cold_after_meter_finalize(&receipt).unwrap();
        let transaction_id = CallbackTransactionId::new("drc-source-0").unwrap();
        let shared_analysis = zeroed_shared_analysis_receipt_for_identity_tests(
            receipt.attempt(),
            transaction_id,
            CodecSourceIndex::new(0),
            7,
            8,
        );
        let callback = callback_identity(2, receipt.attempt().value());
        let secondary = zero_planes();
        let source_planes = NoneDrcSourcePlanes::from_parent_analysis(
            &shared_analysis,
            secondary.as_ref(),
            secondary.as_ref(),
        );
        let proposal = root
            .prepare_f06nd_source(&shared_analysis, &callback, &source_planes)
            .unwrap();

        assert_eq!(
            proposal.shared_analysis_identity().source_index(),
            CodecSourceIndex::new(0)
        );
        assert_eq!(
            proposal
                .shared_analysis_identity()
                .analysis_after_generation(),
            AnalysisOwnerGeneration::from_parent_snapshot(8)
        );
        assert_eq!(proposal.callback_identity(), &callback);
        assert_eq!(
            proposal.callback_transition().clock().source_index,
            CodecSourceIndex::new(0)
        );
        assert_eq!(
            proposal.callback_transition().clock().publication,
            crate::timeline::PublicationDisposition::Suppressed { joc_popped: None }
        );
        assert_eq!(
            proposal.callback_transition().frame().output.bsi_projection,
            *proposal.bsi_projection()
        );
        assert_eq!(
            root.state().source_generation().unwrap(),
            CodecSourceIndex::new(0)
        );
        assert_eq!(root.state().override_fifo().frames_processed(), 0);

        let secondary_after = NoneDrcSecondaryState::from_f06nr_after(
            std::array::from_fn(|_| LongTransformHistory::zeroed()),
            std::array::from_fn(|_| SecondaryOverlapCarry::cold()),
            1,
        );
        let after = proposal.join_f06nr_after(secondary_after).unwrap();
        assert_eq!(after.source_index(), CodecSourceIndex::new(0));
        assert_eq!(
            after.state().source_generation().unwrap(),
            CodecSourceIndex::new(1)
        );
        assert_eq!(after.state().secondary().frame_index(), 1);
        assert_eq!(after.state().override_fifo().frames_processed(), 1);
        assert_eq!(
            root.state().source_generation().unwrap(),
            CodecSourceIndex::new(0)
        );
    }

    #[test]
    fn f06nd_stage_rejects_attempt_programme_source_and_secondary_clock_mismatches() {
        let receipt = measurement_receipt();
        let root = NoneDrcOwnerRoot::cold_after_meter_finalize(&receipt).unwrap();
        let transaction_id = CallbackTransactionId::new("drc-source-0").unwrap();
        let shared_analysis = zeroed_shared_analysis_receipt_for_identity_tests(
            receipt.attempt(),
            transaction_id,
            CodecSourceIndex::new(0),
            7,
            8,
        );
        let secondary = zero_planes();
        let source_planes = NoneDrcSourcePlanes::from_parent_analysis(
            &shared_analysis,
            secondary.as_ref(),
            secondary.as_ref(),
        );

        assert_eq!(
            root.prepare_f06nd_source(
                &shared_analysis,
                &CallbackLineageIdentity::new(
                    ProgrammeId::new("other-programme").unwrap(),
                    receipt.attempt(),
                    FrontendCallbackIndex::new(2),
                ),
                &source_planes,
            ),
            Err(NoneDrcOwnerError::ProgrammeMismatch)
        );
        assert_eq!(
            root.prepare_f06nd_source(&shared_analysis, &callback_identity(2, 4), &source_planes,),
            Err(NoneDrcOwnerError::AttemptMismatch)
        );
        assert_eq!(
            root.prepare_f06nd_source(
                &shared_analysis,
                &callback_identity(3, receipt.attempt().value()),
                &source_planes,
            ),
            Err(NoneDrcOwnerError::SourceMismatch)
        );
        let other_source_analysis = zeroed_shared_analysis_receipt_for_identity_tests(
            receipt.attempt(),
            CallbackTransactionId::new("drc-source-1").unwrap(),
            CodecSourceIndex::new(1),
            8,
            9,
        );
        assert_eq!(
            root.prepare_f06nd_source(
                &shared_analysis,
                &callback_identity(2, receipt.attempt().value()),
                &NoneDrcSourcePlanes::from_parent_analysis(
                    &other_source_analysis,
                    secondary.as_ref(),
                    secondary.as_ref(),
                ),
            ),
            Err(NoneDrcOwnerError::SourceMismatch)
        );
        assert_eq!(
            root.state().source_generation().unwrap(),
            CodecSourceIndex::new(0)
        );
    }

    #[test]
    fn f06nd_f06nr_join_rejects_a_secondary_after_state_from_another_source() {
        let receipt = measurement_receipt();
        let root = NoneDrcOwnerRoot::cold_after_meter_finalize(&receipt).unwrap();
        let shared_analysis = zeroed_shared_analysis_receipt_for_identity_tests(
            receipt.attempt(),
            CallbackTransactionId::new("drc-source-0").unwrap(),
            CodecSourceIndex::new(0),
            7,
            8,
        );
        let planes = zero_planes();
        let source_planes = NoneDrcSourcePlanes::from_parent_analysis(
            &shared_analysis,
            planes.as_ref(),
            planes.as_ref(),
        );
        let stage = root
            .prepare_f06nd_source(
                &shared_analysis,
                &callback_identity(2, receipt.attempt().value()),
                &source_planes,
            )
            .unwrap();
        let wrong_secondary_after = NoneDrcSecondaryState::from_f06nr_after(
            std::array::from_fn(|_| LongTransformHistory::zeroed()),
            std::array::from_fn(|_| SecondaryOverlapCarry::cold()),
            2,
        );

        assert_eq!(
            stage.join_f06nr_after(wrong_secondary_after),
            Err(NoneDrcOwnerError::AfterStateGenerationMismatch)
        );
        assert_eq!(
            root.state().source_generation().unwrap(),
            CodecSourceIndex::new(0)
        );
    }

    #[test]
    fn gtc_state_reference_preserves_none_some_option_and_the_same_cold_digest() {
        let before_meter = NoneDrcOwnerRoot::cold_before_meter_finalize();
        let none_reference = before_meter.state_reference().unwrap();
        let none_snapshot = none_reference.gtc_snapshot().unwrap();
        let finalized =
            NoneDrcOwnerRoot::cold_after_meter_finalize(&measurement_receipt()).unwrap();
        let some_reference = finalized.state_reference().unwrap();
        let some_snapshot = some_reference.gtc_snapshot().unwrap();
        let none_value: Value = serde_json::from_slice(none_snapshot.canonical_bytes()).unwrap();
        let some_value: Value = serde_json::from_slice(some_snapshot.canonical_bytes()).unwrap();

        assert_eq!(none_snapshot.owner_id(), GtcOwnerId::Drc);
        assert_eq!(none_snapshot.state_ref().as_str(), "owners.drc");
        assert_eq!(none_snapshot.generation(), 0);
        assert_eq!(some_snapshot.generation(), 0);
        assert_eq!(none_value["programme_dialnorm"], Value::Null);
        assert!(some_value["programme_dialnorm"].as_u64().is_some());
        assert_ne!(none_snapshot.state_sha256(), some_snapshot.state_sha256());
        assert_eq!(none_snapshot.cold_sha256(), some_snapshot.cold_sha256());
        assert_eq!(
            none_snapshot.canonical_bytes(),
            none_reference.canonical_bytes()
        );
        assert_eq!(
            some_snapshot.canonical_bytes(),
            some_reference.canonical_bytes()
        );
    }

    #[test]
    fn meter_finalized_drc_owner_emits_unchanged_receipts_only_for_callbacks_zero_and_one() {
        let root = NoneDrcOwnerRoot::cold_after_meter_finalize(&measurement_receipt()).unwrap();
        assert!(root.state().programme_dialnorm().is_some());

        for callback in [0, 1] {
            let transaction_id =
                CallbackTransactionId::new(&format!("drc-callback-{callback}")).unwrap();
            let receipt = root
                .gtc_unchanged_callback_receipt(
                    FrontendCallbackIndex::new(callback),
                    transaction_id.clone(),
                )
                .unwrap();

            assert_eq!(receipt.owner_id(), GtcOwnerId::Drc);
            assert_eq!(receipt.before_generation(), 0);
            assert_eq!(receipt.after_generation(), 0);
            assert_eq!(receipt.before_sha256(), receipt.after_sha256());
            assert!(!receipt.changed());
            assert_eq!(receipt.commit_count(), 0);
            assert_eq!(receipt.transaction_id(), &transaction_id);
        }

        assert_eq!(
            root.gtc_unchanged_callback_receipt(
                FrontendCallbackIndex::new(2),
                CallbackTransactionId::new("drc-callback-2").unwrap(),
            ),
            Err(NoneDrcOwnerError::SourceStageRequired)
        );
    }

    #[test]
    fn source_proposal_gtc_receipt_binds_parent_transaction_and_exact_after_root() {
        let root = NoneDrcOwnerRoot::cold_after_meter_finalize(&measurement_receipt()).unwrap();
        let before_reference = root.state_reference().unwrap();
        let transaction_id = CallbackTransactionId::new("drc-source-zero-transaction").unwrap();
        let proposal = source_zero_after(&root, transaction_id.clone());

        let receipt = proposal
            .gtc_owner_receipt(&root, transaction_id.clone())
            .unwrap();
        let after_reference = none_drc_state_reference(proposal.state()).unwrap();
        let before_snapshot = before_reference.gtc_snapshot().unwrap();
        let after_snapshot = after_reference.gtc_snapshot().unwrap();

        assert_eq!(receipt.owner_id(), GtcOwnerId::Drc);
        assert_eq!(receipt.before_generation(), before_snapshot.generation());
        assert_eq!(receipt.after_generation(), after_snapshot.generation());
        assert_eq!(receipt.before_sha256(), before_snapshot.state_sha256());
        assert_eq!(receipt.after_sha256(), after_snapshot.state_sha256());
        assert!(receipt.changed());
        assert_eq!(receipt.commit_count(), 1);
        assert_eq!(receipt.transaction_id(), &transaction_id);
        assert_eq!(before_snapshot.cold_sha256(), after_snapshot.cold_sha256());
        assert!(
            serde_json::from_slice::<Value>(after_snapshot.canonical_bytes()).unwrap()
                ["programme_dialnorm"]
                .as_u64()
                .is_some()
        );
        assert_eq!(
            root.state().source_generation().unwrap(),
            CodecSourceIndex::new(0)
        );
    }

    #[test]
    fn source_proposal_gtc_receipt_rejects_wrong_parent_token_and_stale_before_root() {
        let receipt = measurement_receipt();
        let root = NoneDrcOwnerRoot::cold_after_meter_finalize(&receipt).unwrap();
        let source_transaction = CallbackTransactionId::new("drc-source-zero-transaction").unwrap();
        let proposal = source_zero_after(&root, source_transaction.clone());

        assert_eq!(
            proposal.gtc_owner_receipt(
                &root,
                CallbackTransactionId::new("foreign-parent-transaction").unwrap(),
            ),
            Err(NoneDrcOwnerError::ParentTransactionMismatch)
        );

        let mut stale_before = NoneDrcOwnerRoot::cold_after_meter_finalize(&receipt).unwrap();
        let stale_proposal = source_zero_after(&root, source_transaction.clone());
        stale_before.state = stale_proposal.state;
        assert_eq!(
            proposal.gtc_owner_receipt(&stale_before, source_transaction),
            Err(NoneDrcOwnerError::BeforeStateMismatch)
        );
        assert_eq!(
            root.state().source_generation().unwrap(),
            CodecSourceIndex::new(0)
        );
    }
}
