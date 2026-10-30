// pattern: Imperative Shell
//! Connects one ready MAIN spatial callback proposal to the existing F07-F10 source stages.

use std::{sync::Arc, thread};

use jocforge_domain::{
    analysis::{
        owner::{AnalysisNumericPlanStage, AnalysisOwnerRoot},
        source_analysis::{
            SourceAnalysisError, SourceAnalysisProviders, SpatialSourceAnalysisInput,
            prepare_source_analysis,
        },
        structure_refresh::F07_FULLBAND_LANES,
    },
    spatial::SpatialAudioCallbackOutcome,
    transaction::{AttemptId, CallbackTransactionId},
};

use crate::{PreparedJobInput, spatial_callback::MainSpatialCallbackCandidate};

const SOURCE_ANALYSIS_STACK_BYTES: usize = 16 * 1024 * 1024;

/// Failures while binding a ready MAIN spatial proposal to its admitted job and analysis owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceAnalysisBridgeError {
    PreparedJobIdentityUnavailable,
    PreparedJobIdentityMismatch,
    SpatialCallbackIdentityMismatch,
    ProgrammeIdentityMismatch,
    AttemptIdentityMismatch,
    SpatialAudioStillPriming,
    WorkerUnavailable,
    WorkerPanicked,
    SourceAnalysis(SourceAnalysisError),
}

/// Inputs for one source-scoped, noncommitting F07-F10 proposal.
pub struct SpatialSourceAnalysisBridgeInput<'a> {
    pub prepared: &'a PreparedJobInput,
    pub analysis_before: &'a AnalysisOwnerRoot,
    pub spatial_callback: &'a MainSpatialCallbackCandidate,
    pub attempt_id: AttemptId,
    pub source_transaction_id: CallbackTransactionId,
    /// Before-source F07 dither flags in the selected fullband lane order.
    pub previous_short: [bool; F07_FULLBAND_LANES],
    pub providers: SourceAnalysisProviders<'a>,
}

/// Stages F07-F10 from the same admitted input and ready F04 callback proposal.
///
/// The resulting AnalysisNumericPlanStage retains a source transaction and shared-analysis
/// receipt, but installs no owner root and grants no AU or publication authority.
pub fn prepare_source_analysis_from_spatial_callback(
    input: SpatialSourceAnalysisBridgeInput<'_>,
) -> Result<Box<AnalysisNumericPlanStage>, SourceAnalysisBridgeError> {
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name("jocforge-main-source-analysis".to_owned())
            .stack_size(SOURCE_ANALYSIS_STACK_BYTES)
            .spawn_scoped(scope, move || prepare_source_analysis_inner(input))
            .map_err(|_| SourceAnalysisBridgeError::WorkerUnavailable)?;
        worker
            .join()
            .map_err(|_| SourceAnalysisBridgeError::WorkerPanicked)?
    })
}

fn prepare_source_analysis_inner(
    input: SpatialSourceAnalysisBridgeInput<'_>,
) -> Result<Box<AnalysisNumericPlanStage>, SourceAnalysisBridgeError> {
    let SpatialSourceAnalysisBridgeInput {
        prepared,
        analysis_before,
        spatial_callback,
        attempt_id,
        source_transaction_id,
        previous_short,
        providers,
    } = input;
    if !spatial_callback.is_bound_to(prepared) {
        return Err(SourceAnalysisBridgeError::SpatialCallbackIdentityMismatch);
    }
    let proposal = spatial_callback.proposal();
    let callback_identity = proposal.transaction_id();
    if callback_identity.programme_id() != prepared.programme_id() {
        return Err(SourceAnalysisBridgeError::ProgrammeIdentityMismatch);
    }
    if callback_identity.attempt_id() != attempt_id {
        return Err(SourceAnalysisBridgeError::AttemptIdentityMismatch);
    }
    let owner_identity = analysis_before
        .prepared_job_input_identity()
        .ok_or(SourceAnalysisBridgeError::PreparedJobIdentityUnavailable)?;
    let prepared_identity = prepared.domain_identity_capability();
    if !Arc::ptr_eq(owner_identity, &prepared_identity) {
        return Err(SourceAnalysisBridgeError::PreparedJobIdentityMismatch);
    }
    let projection = match proposal.spatial_audio_outcome() {
        SpatialAudioCallbackOutcome::Priming => {
            return Err(SourceAnalysisBridgeError::SpatialAudioStillPriming);
        }
        SpatialAudioCallbackOutcome::Joc(projection) => projection.as_ref(),
    };

    let analysis_stage = prepare_source_analysis(SpatialSourceAnalysisInput {
        analysis_before,
        attempt_id,
        transaction_id: source_transaction_id,
        projection,
        previous_short,
        providers,
    })
    .map_err(SourceAnalysisBridgeError::SourceAnalysis)?;
    Ok(Box::new(analysis_stage))
}
