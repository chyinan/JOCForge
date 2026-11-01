// pattern: Functional Core
//! Connects one F07-F10 analysis source receipt to the selected F06ND/F06NR DRC proposal.

use std::sync::Arc;

pub use jocforge_domain::drc::{NoneDrcOwnerError, NoneDrcOwnerRoot, NoneDrcSourceFrameProposal};
use jocforge_domain::meter::measurement_receipt::ProgrammeMeasurementReceipt;
pub use jocforge_domain::transaction::CallbackLineageIdentity;
use jocforge_domain::{
    analysis::{owner::AnalysisNumericPlanStage, source_frame::SharedAnalysisReceipt},
    drc::SecondarySourceError,
    timeline::source_for_callback,
};

use crate::PreparedJobInput;

/// Failures while binding F06ND/F06NR to the exact F07-F10 analysis source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrcSourceBridgeError {
    MissingSharedAnalysis,
    PreparedJobIdentityUnavailable,
    PreparedJobIdentityMismatch,
    ProgrammeIdentityMismatch,
    AttemptIdentityMismatch,
    CallbackSourceMismatch,
    AnalysisAfterGenerationMismatch,
    Drc(SecondarySourceError),
}

impl std::fmt::Display for DrcSourceBridgeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSharedAnalysis => formatter.write_str("DRC_SHARED_ANALYSIS_MISSING"),
            Self::PreparedJobIdentityUnavailable => {
                formatter.write_str("DRC_PREPARED_IDENTITY_UNAVAILABLE")
            }
            Self::PreparedJobIdentityMismatch => {
                formatter.write_str("DRC_PREPARED_IDENTITY_MISMATCH")
            }
            Self::ProgrammeIdentityMismatch => {
                formatter.write_str("DRC_PROGRAMME_IDENTITY_MISMATCH")
            }
            Self::AttemptIdentityMismatch => formatter.write_str("DRC_ATTEMPT_IDENTITY_MISMATCH"),
            Self::CallbackSourceMismatch => formatter.write_str("DRC_CALLBACK_SOURCE_MISMATCH"),
            Self::AnalysisAfterGenerationMismatch => {
                formatter.write_str("DRC_ANALYSIS_AFTER_GENERATION_MISMATCH")
            }
            Self::Drc(error) => write!(formatter, "{error:?}"),
        }
    }
}

impl std::error::Error for DrcSourceBridgeError {}

/// Builds the selected DRC cold root from the terminal receipt and exact admitted input identity.
pub fn cold_drc_owner_after_measurement(
    prepared: &PreparedJobInput,
    receipt: &ProgrammeMeasurementReceipt,
) -> Result<NoneDrcOwnerRoot, NoneDrcOwnerError> {
    NoneDrcOwnerRoot::cold_after_meter_finalize_for_prepared_input(
        receipt,
        prepared.domain_identity_capability(),
    )
}

/// Validates the admitted identity and schedule for one analysis source receipt.
///
/// This preflight leaves both owner roots unchanged and does not infer the source index from a
/// byte count or sample value.
pub fn validate_drc_source_lineage<'a>(
    prepared: &PreparedJobInput,
    analysis_stage: &'a AnalysisNumericPlanStage,
    callback_identity: &CallbackLineageIdentity,
) -> Result<&'a SharedAnalysisReceipt, DrcSourceBridgeError> {
    let shared = analysis_stage
        .shared_analysis()
        .ok_or(DrcSourceBridgeError::MissingSharedAnalysis)?;
    let prepared_identity = prepared.domain_identity_capability();
    let shared_prepared_identity = shared
        .identity()
        .prepared_job_input_identity()
        .ok_or(DrcSourceBridgeError::PreparedJobIdentityUnavailable)?;
    let after_prepared_identity = analysis_stage
        .after_state()
        .prepared_job_input_identity()
        .ok_or(DrcSourceBridgeError::PreparedJobIdentityUnavailable)?;
    if !Arc::ptr_eq(shared_prepared_identity, &prepared_identity)
        || !Arc::ptr_eq(after_prepared_identity, &prepared_identity)
    {
        return Err(DrcSourceBridgeError::PreparedJobIdentityMismatch);
    }
    if callback_identity.programme_id() != prepared.programme_id() {
        return Err(DrcSourceBridgeError::ProgrammeIdentityMismatch);
    }

    let source_key = shared.identity().source_stage_key();
    if callback_identity.attempt_id() != source_key.attempt_id() {
        return Err(DrcSourceBridgeError::AttemptIdentityMismatch);
    }
    let callback_source = source_for_callback(callback_identity.callback_index());
    if callback_source != Some(shared.identity().source_index()) {
        return Err(DrcSourceBridgeError::CallbackSourceMismatch);
    }

    let after_state = analysis_stage.after_state();
    let expected_generation = shared.identity().analysis_after_generation().value();
    if !after_state.is_commit_eligible()
        || after_state.input_history_generation() != expected_generation
    {
        return Err(DrcSourceBridgeError::AnalysisAfterGenerationMismatch);
    }

    Ok(shared)
}

/// Stages the same-source F06NR/F06ND proposal without installing either owner root.
pub fn prepare_drc_source_from_analysis(
    prepared: &PreparedJobInput,
    drc_before: &NoneDrcOwnerRoot,
    analysis_stage: &AnalysisNumericPlanStage,
    callback_identity: &CallbackLineageIdentity,
) -> Result<NoneDrcSourceFrameProposal, DrcSourceBridgeError> {
    let shared = validate_drc_source_lineage(prepared, analysis_stage, callback_identity)?;
    drc_before
        .prepare_source_frame(shared, callback_identity)
        .map_err(DrcSourceBridgeError::Drc)
}
