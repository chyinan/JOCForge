// pattern: Functional Core
//! Selected standalone DRC cold owner and same-source F06ND leaf proposal.
//!
//! The source proposal only stages statistics, scheduler, and zero-FIFO leaves.
//! The enclosing source parent still needs the F06NR secondary after-state
//! before it can make a complete DRC after-root or commit.

use crate::{
    analysis::source_frame::{SharedAnalysisIdentity, SharedAnalysisReceipt},
    drc::{
        none_drc_bsi_projection::NoneDrcBsiProjection,
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
        programme_measurement::{ProgrammeMeasurementError, dialnorm_from_selected_lufs},
    },
    timeline::{CodecSourceIndex, source_for_callback},
    transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
    transform::{long_transform::LongTransformHistory, secondary_overlap::SecondaryOverlapCarry},
};

const SECONDARY_LANE_COUNT: usize = 5;

/// The selected secondary state owned by the DRC partition.
///
/// Ordinary F07 input history remains owned by the shared Analysis root. This
/// value records only five independent forward histories, five inverse
/// overlap carries, and their common source-frame index.
#[allow(dead_code)] // F06NR will consume this owner state in the future DRC source stage.
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

    #[allow(dead_code)] // Exposed to the future F06NR composition in the same crate.
    pub(crate) const fn forward_history(&self) -> &[LongTransformHistory; SECONDARY_LANE_COUNT] {
        &self.forward_history
    }

    #[allow(dead_code)] // Exposed to the future F06NR composition in the same crate.
    pub(crate) const fn inverse_carry(&self) -> &[SecondaryOverlapCarry; SECONDARY_LANE_COUNT] {
        &self.inverse_carry
    }

    /// Used only by the future F06NR source-stage composer after it has run all
    /// five lanes and six blocks from this exact before-state.
    #[allow(dead_code)]
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
        })
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
}

/// Standalone DRC owner state composed from the existing selected leaf states.
///
/// It intentionally excludes ordinary F07 input history, which belongs to the
/// shared Analysis owner. It does not implement a state-ref serializer or a
/// top-level GTC owner receipt.
#[allow(dead_code)] // Its full after-state is retained for the future DRC parent.
#[derive(Debug, PartialEq)]
pub(crate) struct NoneDrcOwnerState {
    secondary: NoneDrcSecondaryState,
    statistics: NoneDrcStatisticsState,
    scheduler: NoneDrcSchedulerState,
    override_fifo: NoneDrcOverrideFifoState,
    programme_dialnorm: u8,
}

#[allow(dead_code)] // No top-level source parent installs this root yet.
impl NoneDrcOwnerState {
    fn cold(programme_dialnorm: u8) -> Self {
        Self {
            secondary: NoneDrcSecondaryState::cold(),
            statistics: NoneDrcStatisticsState::cold(),
            scheduler: NoneDrcSchedulerState::cold(),
            override_fifo: NoneDrcOverrideFifoState::cold(),
            programme_dialnorm,
        }
    }

    pub(crate) fn source_generation(&self) -> Result<CodecSourceIndex, NoneDrcOwnerError> {
        let generation = self.scheduler.generation();
        if self.override_fifo.frames_processed() != generation
            || self.secondary.frame_index() != generation
        {
            return Err(NoneDrcOwnerError::StateGenerationMismatch);
        }
        Ok(CodecSourceIndex::new(generation))
    }

    pub(crate) const fn programme_dialnorm(&self) -> u8 {
        self.programme_dialnorm
    }

    #[allow(dead_code)] // Read by the future complete DRC source-stage composer.
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
}

/// Primary and secondary F06ND planes prepared for one source generation.
/// The parent binds both to the same SharedAnalysisReceipt before construction.
pub(crate) struct NoneDrcSourcePlanes<'a> {
    shared_analysis_identity: SharedAnalysisIdentity,
    primary: &'a FiniteStatisticsPlanes,
    secondary: &'a FiniteStatisticsPlanes,
}

impl<'a> NoneDrcSourcePlanes<'a> {
    #[allow(dead_code)] // Called by the future same-source DRC parent composition.
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
#[allow(dead_code)] // The P1.3-C parent source root is not connected yet.
#[derive(Debug, PartialEq)]
pub(crate) struct NoneDrcOwnerRoot {
    measurement: NoneDrcMeasurementBinding,
    state: NoneDrcOwnerState,
}

impl NoneDrcOwnerRoot {
    #[allow(dead_code)] // Called by the future programme/source parent.
    pub(crate) fn cold_after_meter_finalize(
        receipt: &ProgrammeMeasurementReceipt,
    ) -> Result<Self, NoneDrcOwnerError> {
        let measurement = NoneDrcMeasurementBinding::from_receipt(receipt)?;
        let state = NoneDrcOwnerState::cold(measurement.programme_dialnorm());
        Ok(Self { measurement, state })
    }

    #[allow(dead_code)] // Used by the future owner receipt constructor.
    pub(crate) const fn measurement(&self) -> &NoneDrcMeasurementBinding {
        &self.measurement
    }

    #[allow(dead_code)] // Read by the future source parent; no mutable access is exposed.
    pub(crate) const fn state(&self) -> &NoneDrcOwnerState {
        &self.state
    }

    /// Prepares the spec-closed F06ND statistics/scheduler/FIFO substage for the
    /// same source as the parent-issued SharedAnalysisReceipt. No state is
    /// installed here; a complete DRC after-root also needs the F06NR secondary
    /// after-state before the enclosing source parent can commit.
    #[allow(dead_code)] // The source parent and F06NR after-state join are not connected yet.
    pub(crate) fn prepare_f06nd_source(
        &self,
        shared_analysis: &SharedAnalysisReceipt,
        callback_identity: &CallbackLineageIdentity,
        source_planes: &NoneDrcSourcePlanes<'_>,
    ) -> Result<NoneDrcF06ndStageProposal, NoneDrcOwnerError> {
        let shared_identity = shared_analysis.identity();
        let source_stage_key = shared_identity.source_stage_key();
        if callback_identity.programme_id() != self.measurement.programme_id() {
            return Err(NoneDrcOwnerError::ProgrammeMismatch);
        }
        if callback_identity.attempt_id() != self.measurement.attempt()
            || source_stage_key.attempt_id() != self.measurement.attempt()
        {
            return Err(NoneDrcOwnerError::AttemptMismatch);
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
            dialnorm: i32::from(self.state.programme_dialnorm()),
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
            measurement: self.measurement.clone(),
            shared_analysis_identity: shared_identity.clone(),
            callback_identity: callback_identity.clone(),
            callback_transition,
        })
    }
}

/// One noncommitting F06ND source proposal. It contains the exact shared-source
/// and callback identities plus the existing statistics/scheduler/FIFO leaves.
#[allow(dead_code)] // Returned to the future source parent; it has no commit authority.
#[must_use]
#[derive(Debug, PartialEq)]
pub(crate) struct NoneDrcF06ndStageProposal {
    measurement: NoneDrcMeasurementBinding,
    shared_analysis_identity: SharedAnalysisIdentity,
    callback_identity: CallbackLineageIdentity,
    callback_transition: NoneDrcCallbackTransition,
}

#[allow(dead_code)] // No parent source coordinator consumes this proposal yet.
impl NoneDrcF06ndStageProposal {
    pub const fn measurement(&self) -> &NoneDrcMeasurementBinding {
        &self.measurement
    }

    pub const fn shared_analysis_identity(&self) -> &SharedAnalysisIdentity {
        &self.shared_analysis_identity
    }

    pub const fn callback_identity(&self) -> &CallbackLineageIdentity {
        &self.callback_identity
    }

    pub const fn callback_transition(&self) -> &NoneDrcCallbackTransition {
        &self.callback_transition
    }

    pub const fn bsi_projection(&self) -> &NoneDrcBsiProjection {
        &self.callback_transition.frame().output.bsi_projection
    }

    /// Joins the same-source F06NR after-state to the F06ND leaves. This is a
    /// complete noncommitting DRC after-root value; the source parent still
    /// owns the single commit and the GTC OwnerReceipt.
    #[allow(dead_code)] // The source parent and F06NR transition are not wired yet.
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
            programme_dialnorm: self.measurement.programme_dialnorm(),
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

    pub fn into_parts(
        self,
    ) -> (
        NoneDrcMeasurementBinding,
        SharedAnalysisIdentity,
        CallbackLineageIdentity,
        NoneDrcCallbackTransition,
    ) {
        (
            self.measurement,
            self.shared_analysis_identity,
            self.callback_identity,
            self.callback_transition,
        )
    }
}

/// Complete DRC after-state proposal for one source. It carries no commit,
/// reset, or fatal authority and has no locally minted OwnerReceipt.
#[allow(dead_code)] // Returned to the future source parent after the F06NR join.
#[must_use]
#[derive(Debug, PartialEq)]
pub(crate) struct NoneDrcOwnerAfterProposal {
    measurement: NoneDrcMeasurementBinding,
    shared_analysis_identity: SharedAnalysisIdentity,
    callback_identity: CallbackLineageIdentity,
    source_index: CodecSourceIndex,
    frame_output: crate::drc::none_drc_frame::NoneDrcFrameOutput,
    state: NoneDrcOwnerState,
}

#[allow(dead_code)] // No parent source coordinator consumes the after-root yet.
impl NoneDrcOwnerAfterProposal {
    pub const fn measurement(&self) -> &NoneDrcMeasurementBinding {
        &self.measurement
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

    pub const fn frame_output(&self) -> &crate::drc::none_drc_frame::NoneDrcFrameOutput {
        &self.frame_output
    }

    pub const fn state(&self) -> &NoneDrcOwnerState {
        &self.state
    }

    pub fn into_parts(
        self,
    ) -> (
        NoneDrcMeasurementBinding,
        SharedAnalysisIdentity,
        CallbackLineageIdentity,
        CodecSourceIndex,
        crate::drc::none_drc_frame::NoneDrcFrameOutput,
        NoneDrcOwnerState,
    ) {
        (
            self.measurement,
            self.shared_analysis_identity,
            self.callback_identity,
            self.source_index,
            self.frame_output,
            self.state,
        )
    }
}

#[allow(dead_code)] // Used only by the unconnected DRC root/source stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NoneDrcOwnerError {
    MeasurementDialnorm(ProgrammeMeasurementError),
    ProgrammeMismatch,
    AttemptMismatch,
    SharedAnalysisIdentity,
    PrimingHasNoSource,
    SourceMismatch,
    StateGenerationMismatch,
    GenerationOverflow,
    AfterStateGenerationMismatch,
    Callback(NoneDrcCallbackError),
}

#[allow(dead_code)] // Product error translation is owned by the future parent.
impl NoneDrcOwnerError {
    pub const fn code(self) -> Option<&'static str> {
        match self {
            Self::MeasurementDialnorm(_) | Self::SharedAnalysisIdentity => None,
            Self::ProgrammeMismatch
            | Self::AttemptMismatch
            | Self::PrimingHasNoSource
            | Self::SourceMismatch
            | Self::StateGenerationMismatch
            | Self::AfterStateGenerationMismatch => Some("DRC_INPUT_ERROR"),
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
        numeric::FiniteF32,
        timeline::FrontendCallbackIndex,
        transaction::CallbackTransactionId,
    };

    const SOURCE_LANE_COUNT: usize = 6;
    const SOURCE_SAMPLE_COUNT: usize = 6 * 256;

    fn measurement_receipt() -> ProgrammeMeasurementReceipt {
        ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
            programme_id: ProgrammeId::new("drc-root-test").unwrap(),
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
        .unwrap()
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

    #[test]
    fn cold_root_owns_selected_drc_leaves_and_frozen_measurement_binding() {
        let receipt = measurement_receipt();
        let root = NoneDrcOwnerRoot::cold_after_meter_finalize(&receipt).unwrap();
        assert_eq!(root.measurement().programme_id(), receipt.programme_id());
        assert_eq!(root.measurement().attempt(), receipt.attempt());
        assert_eq!(
            root.measurement().meter_terminal_sha256(),
            receipt.meter_terminal_sha256()
        );
        assert_eq!(
            root.state().programme_dialnorm(),
            dialnorm_from_selected_lufs(receipt.selected_lufs_word()).unwrap()
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
}
