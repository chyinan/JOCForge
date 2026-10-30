// pattern: Functional Core
//! Noncommitting MAIN receiver-projection to F07-F10 source analysis composition.

use crate::{
    codec::policy_parameters::PolicyParameters,
    spatial::spatial_audio_state::SelectedJocQmfProjection,
    timeline::CodecSourceIndex,
    transaction::{AttemptId, CallbackTransactionId},
    transform::math_generated_parameters::MathGeneratedTransformParameters,
};

use super::{
    owner::{
        AnalysisGainDbStage, AnalysisNumericPlanStage, AnalysisOwnerRoot, AnalysisOwnerStageError,
        F07AnalysisStage, F08AnalysisStage,
    },
    source_frame::{
        InstalledAudioProjectionError, SourcedInstalledAudioFrame, installed_audio_from_final_six,
    },
    structure_refresh::F07_FULLBAND_LANES,
    transient_parameters::TransientParameters,
};

/// Failures while composing one admitted MAIN source into a noncommitting analysis after-state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceAnalysisError {
    Projection(InstalledAudioProjectionError),
    ProjectionAttemptMismatch,
    Analysis(AnalysisOwnerStageError),
}

pub struct SourceAnalysisProviders<'a> {
    pub transient: &'a TransientParameters,
    pub transform: &'a MathGeneratedTransformParameters,
    pub policy: &'a PolicyParameters,
}

pub struct SpatialSourceAnalysisInput<'a> {
    pub analysis_before: &'a AnalysisOwnerRoot,
    pub attempt_id: AttemptId,
    pub transaction_id: CallbackTransactionId,
    pub projection: &'a SelectedJocQmfProjection,
    pub previous_short: [bool; F07_FULLBAND_LANES],
    pub providers: SourceAnalysisProviders<'a>,
}

pub struct SourceAnalysisInput<'a> {
    pub analysis_before: &'a AnalysisOwnerRoot,
    pub attempt_id: AttemptId,
    pub transaction_id: CallbackTransactionId,
    pub installed_audio: &'a SourcedInstalledAudioFrame,
    pub previous_short: [bool; F07_FULLBAND_LANES],
    pub providers: SourceAnalysisProviders<'a>,
}

/// Runs the already specified F07, F08, F09G and F09/F10 owner stages for one MAIN source.
///
/// The F04R projection is converted by `installed_audio_from_final_six`, which checks the
/// callback-to-source clock and preserves the six external lanes. The supplied analysis root
/// remains unchanged; the returned F07-F10 after-root and source identity are still pending for
/// the enclosing source transaction. This function provides no DSP algorithms or commit authority.
#[inline(never)]
pub fn prepare_source_analysis(
    input: SpatialSourceAnalysisInput<'_>,
) -> Result<AnalysisNumericPlanStage, SourceAnalysisError> {
    let SpatialSourceAnalysisInput {
        analysis_before,
        attempt_id,
        transaction_id,
        projection,
        previous_short,
        providers,
    } = input;
    if projection.attempt() != attempt_id {
        return Err(SourceAnalysisError::ProjectionAttemptMismatch);
    }
    let installed_audio =
        installed_audio_from_final_six(projection).map_err(SourceAnalysisError::Projection)?;
    prepare_source_analysis_frame(SourceAnalysisInput {
        analysis_before,
        attempt_id,
        transaction_id,
        installed_audio: &installed_audio,
        previous_short,
        providers,
    })
}

/// Composes an already validated six-lane source frame with the existing F07-F10 producers.
///
/// Keeping the sourced frame as an input lets a parent coordinator use the exact same typed
/// source identity when it builds sibling F11/F12 proposals.
#[inline(never)]
pub fn prepare_source_analysis_frame(
    input: SourceAnalysisInput<'_>,
) -> Result<AnalysisNumericPlanStage, SourceAnalysisError> {
    let SourceAnalysisInput {
        analysis_before,
        attempt_id,
        transaction_id,
        installed_audio,
        previous_short,
        providers,
    } = input;
    let f07 = prepare_f07_stage(
        analysis_before,
        attempt_id,
        transaction_id,
        installed_audio,
        previous_short,
        providers.transient,
    )?;
    let f08 = complete_f08_stage(f07, providers.transform)?;
    let gain_db = complete_gain_db_stage(f08, providers.policy)?;
    complete_f09_f10_stage(gain_db, providers.policy)
}

// The owner after-stages contain sizeable fixed histories. Keep one pipeline stage in each
// stack frame and pass the owned after-value on the heap between calls.
#[inline(never)]
fn prepare_f07_stage(
    analysis_before: &AnalysisOwnerRoot,
    attempt_id: AttemptId,
    transaction_id: CallbackTransactionId,
    installed_audio: &SourcedInstalledAudioFrame,
    previous_short: [bool; F07_FULLBAND_LANES],
    transient_parameters: &TransientParameters,
) -> Result<Box<F07AnalysisStage>, SourceAnalysisError> {
    let source_index: CodecSourceIndex = installed_audio.source_index();
    let stage = analysis_before
        .prepare_f07_source(
            attempt_id,
            transaction_id,
            source_index,
            installed_audio.audio(),
            previous_short,
            transient_parameters,
        )
        .map_err(SourceAnalysisError::Analysis)?;
    Ok(Box::new(stage))
}

#[inline(never)]
fn complete_f08_stage(
    f07: Box<F07AnalysisStage>,
    transform_parameters: &MathGeneratedTransformParameters,
) -> Result<Box<F08AnalysisStage>, SourceAnalysisError> {
    let stage = (*f07)
        .complete_f08(transform_parameters)
        .map_err(SourceAnalysisError::Analysis)?;
    Ok(Box::new(stage))
}

#[inline(never)]
fn complete_gain_db_stage(
    f08: Box<F08AnalysisStage>,
    policy_parameters: &PolicyParameters,
) -> Result<Box<AnalysisGainDbStage>, SourceAnalysisError> {
    let stage = (*f08)
        .complete_gain_db(policy_parameters)
        .map_err(SourceAnalysisError::Analysis)?;
    Ok(Box::new(stage))
}

#[inline(never)]
fn complete_f09_f10_stage(
    gain_db: Box<AnalysisGainDbStage>,
    policy_parameters: &PolicyParameters,
) -> Result<AnalysisNumericPlanStage, SourceAnalysisError> {
    (*gain_db)
        .complete_f09_f10(policy_parameters)
        .map_err(SourceAnalysisError::Analysis)
}
