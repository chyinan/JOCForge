// pattern: Imperative Shell
//! Connects one ready MAIN spatial callback proposal to the existing F07-F10 source stages.

use std::{sync::Arc, thread};

use jocforge_domain::{
    analysis::transient_parameters::{
        TransientParameterError, TransientParameters, load_transient_parameters,
    },
    analysis::{
        owner::AnalysisNumericPlanStage,
        source_analysis::{
            SourceAnalysisError, SpatialSourceAnalysisInput, prepare_source_analysis,
            prepare_source_analysis_with_installed_audio,
        },
        source_frame::SourcedInstalledAudioFrame,
        structure_refresh::F07_FULLBAND_LANES,
    },
    codec::policy_parameters::{
        PolicyParameterError, PolicyParameters, PolicyProfile, load_policy_parameters,
    },
    spatial::SpatialAudioCallbackOutcome,
    transaction::{AttemptId, CallbackTransactionId},
    transform::math_generated_parameters::{
        MathGeneratedTransformParameterError, MathGeneratedTransformParameters,
    },
};

pub use jocforge_domain::analysis::{
    owner::AnalysisOwnerRoot, source_analysis::SourceAnalysisProviders,
};

use crate::{PreparedJobInput, spatial_callback::MainSpatialSource};

const SOURCE_ANALYSIS_STACK_BYTES: usize = 16 * 1024 * 1024;
const TRANSIENT_PARAMETERS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/ANALYSIS-ROOT-APPROVED-SOURCE-LEAVES-V1/approved/parameters/transient-d05-authorized-v1.json"
);
const POLICY_PARAMETERS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/ANALYSIS-ROOT-APPROVED-SOURCE-LEAVES-V1/approved/parameters/policy-authorized-v1.json"
);

/// Hash-validated F07-F10 values: transient and policy rows from the approved source-leaves package,
/// with the transform table loaded through the domain's selected provider identity gate.
pub struct SelectedSourceAnalysisProviders {
    transient: TransientParameters,
    transform: MathGeneratedTransformParameters,
    policy: PolicyParameters,
}

impl SelectedSourceAnalysisProviders {
    pub fn load_approved() -> Result<Self, SelectedSourceAnalysisProviderError> {
        let transient = load_transient_parameters(Some(TRANSIENT_PARAMETERS))
            .map_err(SelectedSourceAnalysisProviderError::Transient)?;
        let transform = MathGeneratedTransformParameters::selected_math_generated_v1()
            .map_err(SelectedSourceAnalysisProviderError::Transform)?;
        let policy = load_policy_parameters(PolicyProfile::selected(), Some(POLICY_PARAMETERS))
            .map_err(SelectedSourceAnalysisProviderError::Policy)?;
        Ok(Self {
            transient,
            transform,
            policy,
        })
    }

    pub const fn providers(&self) -> SourceAnalysisProviders<'_> {
        SourceAnalysisProviders {
            transient: &self.transient,
            transform: &self.transform,
            policy: &self.policy,
        }
    }
}

#[derive(Debug)]
pub enum SelectedSourceAnalysisProviderError {
    Transient(TransientParameterError),
    Transform(MathGeneratedTransformParameterError),
    Policy(PolicyParameterError),
}

/// Constructs the selected cold Analysis root on a worker with an explicit stack budget.
pub fn cold_analysis_root_for_prepared_input(
    prepared: &PreparedJobInput,
) -> Result<Box<AnalysisOwnerRoot>, SourceAnalysisBridgeError> {
    let identity = prepared.domain_identity_capability();
    thread::Builder::new()
        .name("jocforge-main-analysis-cold-root".to_owned())
        .stack_size(SOURCE_ANALYSIS_STACK_BYTES)
        .spawn(move || Box::new(AnalysisOwnerRoot::cold_for_prepared_input(identity)))
        .map_err(|_| SourceAnalysisBridgeError::WorkerUnavailable)?
        .join()
        .map_err(|_| SourceAnalysisBridgeError::WorkerPanicked)
}

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
    pub spatial_callback: &'a dyn MainSpatialSource,
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
    prepare_source_analysis_from_spatial_callback_with_installed_audio(input, None)
}

/// Stages F07-F10 using a profile-selected source frame while retaining the same ready F04
/// callback and source identity checks. The `None` route remains the selected 5.1 behavior.
pub fn prepare_source_analysis_from_spatial_callback_with_installed_audio(
    input: SpatialSourceAnalysisBridgeInput<'_>,
    installed_audio: Option<&SourcedInstalledAudioFrame>,
) -> Result<Box<AnalysisNumericPlanStage>, SourceAnalysisBridgeError> {
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name("jocforge-main-source-analysis".to_owned())
            .stack_size(SOURCE_ANALYSIS_STACK_BYTES)
            .spawn_scoped(scope, move || {
                prepare_source_analysis_on_current_thread(input, installed_audio)
            })
            .map_err(|_| SourceAnalysisBridgeError::WorkerUnavailable)?;
        worker
            .join()
            .map_err(|_| SourceAnalysisBridgeError::WorkerPanicked)?
    })
}

/// Stages the analysis on an encoder worker that already owns a large DSP stack.
pub fn prepare_source_analysis_on_current_thread(
    input: SpatialSourceAnalysisBridgeInput<'_>,
    installed_audio: Option<&SourcedInstalledAudioFrame>,
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
    let frame = spatial_callback.source_view();
    if !std::sync::Arc::ptr_eq(
        frame.prepared_identity,
        &prepared.domain_identity_capability(),
    ) {
        return Err(SourceAnalysisBridgeError::SpatialCallbackIdentityMismatch);
    }
    let callback_identity = frame.lineage;
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
    let projection = match frame.outcome {
        SpatialAudioCallbackOutcome::Priming => {
            return Err(SourceAnalysisBridgeError::SpatialAudioStillPriming);
        }
        SpatialAudioCallbackOutcome::Joc(projection) => projection.as_ref(),
    };

    let analysis_stage = if let Some(installed_audio) = installed_audio {
        prepare_source_analysis_with_installed_audio(
            SpatialSourceAnalysisInput {
                analysis_before,
                attempt_id,
                transaction_id: source_transaction_id,
                projection,
                previous_short,
                providers,
            },
            installed_audio,
        )
    } else {
        prepare_source_analysis(SpatialSourceAnalysisInput {
            analysis_before,
            attempt_id,
            transaction_id: source_transaction_id,
            projection,
            previous_short,
            providers,
        })
    }
    .map_err(SourceAnalysisBridgeError::SourceAnalysis)?;
    Ok(Box::new(analysis_stage))
}

#[cfg(test)]
mod tests {
    use super::SelectedSourceAnalysisProviders;
    use jocforge_domain::{
        analysis::{
            owner::AnalysisOwnerRoot,
            source_analysis::{SourceAnalysisInput, prepare_source_analysis_frame},
            source_frame::installed_audio_from_external_pcm_rows,
            structure_refresh::F07_FULLBAND_LANES,
        },
        timeline::CodecSourceIndex,
        transaction::{AttemptId, CallbackTransactionId},
    };

    #[test]
    fn source_analysis_loads_selected_dee_window_from_domain_provider() {
        let providers = SelectedSourceAnalysisProviders::load_approved().unwrap();

        assert_eq!(providers.transform.window_f32_bits()[21], 0x3c0e_a5aa);
    }

    #[test]
    fn nonzero_six_lane_source_reaches_f08_fullband_and_lfe_inputs() {
        let providers = SelectedSourceAnalysisProviders::load_approved().unwrap();
        let rows: [Vec<u32>; 6] = std::array::from_fn(|lane| {
            let mut row = vec![0.0_f32.to_bits(); 1_536];
            row[128] = (0.25_f32 + lane as f32 * 0.03125).to_bits();
            row
        });
        let source_index = CodecSourceIndex::new(2);
        let installed_audio = installed_audio_from_external_pcm_rows(
            source_index,
            std::array::from_fn(|lane| rows[lane].as_slice()),
        )
        .unwrap();
        let stage = prepare_source_analysis_frame(SourceAnalysisInput {
            analysis_before: &AnalysisOwnerRoot::cold(),
            attempt_id: AttemptId::new(3),
            transaction_id: CallbackTransactionId::new("flat7x-i0-analysis-test").unwrap(),
            installed_audio: &installed_audio,
            previous_short: [false; F07_FULLBAND_LANES],
            providers: providers.providers(),
        })
        .unwrap();
        let numeric_plan = stage.plan().expect("F07-F10 plan is staged");
        let f08 = numeric_plan.f08();

        for lane in 0..F07_FULLBAND_LANES {
            assert!(
                (0..6).any(|block| {
                    f08.fullband_exponent_inputs()[block * F07_FULLBAND_LANES + lane]
                        .primary_clipped_f32_bits()
                        .iter()
                        .any(|word| *word != 0)
                }),
                "F08 fullband lane {lane} receives nonzero source coefficients"
            );
        }
        assert!(f08.lfe_exponent_inputs().iter().any(|input| {
            input
                .primary_clipped_f32_bits()
                .iter()
                .any(|word| *word != 0)
        }));
    }
}
