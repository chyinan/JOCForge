// pattern: Functional Core
//! Local parent adapter for one source-stage shared-analysis and numeric-plan receipt.
//!
//! This does not implement the top-level callback transaction, owner swap, restart, or EOF
//! coordinator. The parent supplies the canonical GTC owner generations; this adapter never
//! computes them from source index, elapsed time, staged bytes, or a new counter.

use crate::{
    analysis::source_frame::{
        AnalysisOwnerGeneration, F07SourceFrameProposal, SharedAnalysisReceipt, SourceStageKey,
    },
    meter::prepared_job_identity::PreparedJobInputIdentity,
    timeline::CodecSourceIndex,
    transform::fullband_exponent_pipeline::NumericPlanReceipt,
};

use super::{AttemptId, CallbackTransactionId};
use std::sync::Arc;

/// One local parent source stage. It owns no top-level callback/commit authority.
#[must_use]
#[allow(dead_code)] // The top-level callback parent is not yet wired to this local source stage.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SourceTransaction {
    source_stage_key: SourceStageKey,
    shared_analysis: Option<SharedAnalysisReceipt>,
    numeric_plan_prepared: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceTransactionStageError {
    AnalysisDoubleAdvance,
    AnalysisNotPrepared,
    NumericPlanAlreadyPrepared,
}

#[allow(dead_code)] // Called only by the future parent source coordinator.
impl SourceTransaction {
    /// Begins one source stage using the parent-owned attempt, transaction, source, and snapshot.
    pub(crate) fn new(
        attempt_id: AttemptId,
        transaction_id: CallbackTransactionId,
        source_index: CodecSourceIndex,
        analysis_before_generation: AnalysisOwnerGeneration,
    ) -> Self {
        Self {
            source_stage_key: SourceStageKey::from_parent_snapshot(
                attempt_id,
                transaction_id,
                source_index,
                analysis_before_generation,
            ),
            shared_analysis: None,
            numeric_plan_prepared: false,
        }
    }

    /// Begins a source stage bound to the exact immutable identity admitted by F01E.
    pub(crate) fn new_for_prepared_input(
        attempt_id: AttemptId,
        transaction_id: CallbackTransactionId,
        source_index: CodecSourceIndex,
        analysis_before_generation: AnalysisOwnerGeneration,
        prepared_job_input_identity: Arc<PreparedJobInputIdentity>,
    ) -> Self {
        Self {
            source_stage_key: SourceStageKey::from_prepared_input_snapshot(
                attempt_id,
                transaction_id,
                source_index,
                analysis_before_generation,
                prepared_job_input_identity,
            ),
            shared_analysis: None,
            numeric_plan_prepared: false,
        }
    }

    /// Stages one F07 proposal into the source's shared-analysis receipt.
    ///
    /// `analysis_after_generation` is the already-staged GTC owner value. No generation advance
    /// is inferred here. A second call is rejected before replacing the first staged proposal.
    pub(crate) fn prepare_shared_analysis(
        &mut self,
        analysis_after_generation: AnalysisOwnerGeneration,
        f07_proposal: F07SourceFrameProposal,
    ) -> Result<(), SourceTransactionStageError> {
        if self.shared_analysis.is_some() {
            return Err(SourceTransactionStageError::AnalysisDoubleAdvance);
        }
        self.shared_analysis = Some(SharedAnalysisReceipt::from_parent(
            self.source_stage_key.clone(),
            analysis_after_generation,
            f07_proposal,
        ));
        Ok(())
    }

    pub(crate) fn shared_analysis(&self) -> Option<&SharedAnalysisReceipt> {
        self.shared_analysis.as_ref()
    }

    /// Produces exactly one immutable numeric-plan receipt from the same shared result.
    pub(crate) fn prepare_numeric_plan<T>(
        &mut self,
        plan: T,
    ) -> Result<NumericPlanReceipt<'_, T>, SourceTransactionStageError> {
        if self.shared_analysis.is_none() {
            return Err(SourceTransactionStageError::AnalysisNotPrepared);
        }
        if self.numeric_plan_prepared {
            return Err(SourceTransactionStageError::NumericPlanAlreadyPrepared);
        }
        self.numeric_plan_prepared = true;
        let shared_analysis = self
            .shared_analysis
            .as_ref()
            .expect("shared analysis is preflighted above");
        Ok(NumericPlanReceipt::from_parent(shared_analysis, plan))
    }
}
