// pattern: Imperative Shell

use super::*;
use crate::{
    analysis::{
        owner::AnalysisOwnerRoot, source_frame::SourcedInstalledAudioFrame,
        transient_parameters::load_transient_parameters,
    },
    codec::policy_parameters::{PolicyProfile, load_policy_parameters},
    frontend::{
        PackageManifestEvidence, ProviderFileEvidence,
        job_configuration::{
            SelectedJobConfiguration, SelectedWaveDescriptor, parse_selected_job_configuration,
        },
    },
    joc::{
        coefficient_fitting::CoefficientFitState,
        coefficient_representation::RepresentationRetryProvider,
        dispatch_state::{JocDispatchState, JocPayloadInput},
        fit_score::FitScoreProvider,
        oamd::reader_target::{
            HistoryProducerGeneration, ReaderTargetCallbackInputs, ReaderTargetState,
            SELECTED_PROFILE_ID, stage_reader_target,
        },
        source_stage::stage_joc_source_with_transaction,
    },
    spatial::{
        DescriptorMathematicalProvider, QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
        SpatialAudioJocProviders, ValidatedWindowQ15Provider,
        spatial_audio_state::spatial_audio_composer::{
            ReaderAttemptBinding, SpatialAudioCallbackOutcome, stage_spatial_audio_callback,
        },
        validate_descriptor_mathematical_provider, validate_f04_parameter_package_manifest,
        validate_qmf_mathematical_fft_provider, validate_qmf_receiver_functional_provider,
        validate_window_q15_provider,
    },
    transaction::{AttemptId, CallbackTransactionId},
    transform::math_generated_parameters::MathGeneratedTransformParameters,
};

const MANIFEST_BYTES: &[u8] =
    include_bytes!("../../../../../tests/fixtures/f01e_provider/MANIFEST.json");
const QMF_FUNCTIONAL_BYTES: &[u8] =
    include_bytes!("../../../../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");
const QMF_FFT_BYTES: &[u8] =
    include_bytes!("../../../../../tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json");
const WINDOW_Q15_BYTES: &[u8] =
    include_bytes!("../../../../../tests/fixtures/f04_parameters/window-q15.json");
const DESCRIPTOR_BYTES: &[u8] =
    include_bytes!("../../../../../tests/fixtures/f04_parameters/descriptor-mathematical.json");
const TRANSIENT_PARAMETERS: &[u8] =
    include_bytes!("../../../../../tests/fixtures/transient_channel/PARAMETERS.json");
const MATH_PARAMETERS: &[u8] =
    include_bytes!("../../../../../src/transform/ordinary_transform_math_parameters.json");
const POLICY_PARAMETERS: &[u8] =
    include_bytes!("../../../../../tests/fixtures/policy_authorized/PARAMETERS.json");
const JOB_CONFIGURATION_VECTORS: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/job_configuration/job-configuration-vectors.json"
);
const JOC_FUNCTIONAL_BYTES: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-functional.json"
);
const JOC_MATH_BYTES: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-math.json"
);
const READER_ATTEMPT_ID: crate::transaction::AttemptId = crate::transaction::AttemptId::new(7);

struct ValidatedProviders {
    functional: QmfReceiverFunctionalProvider,
    blend: ValidatedWindowQ15Provider,
    fft: QmfMathematicalFftProvider,
    descriptor: DescriptorMathematicalProvider,
}

impl ValidatedProviders {
    fn input(&self) -> SpatialAudioJocProviders<'_> {
        SpatialAudioJocProviders::new(&self.functional, &self.blend, &self.fft, &self.descriptor)
    }
}

fn validated_providers() -> ValidatedProviders {
    let manifest = validate_f04_parameter_package_manifest(&PackageManifestEvidence::new(
        Arc::from(MANIFEST_BYTES),
        Some("shared parameter manifest".to_owned()),
    ))
    .expect("shared parameter manifest validates");
    let functional = validate_qmf_receiver_functional_provider(
        &ProviderFileEvidence::new(
            Arc::from(QMF_FUNCTIONAL_BYTES),
            Some("receiver functional provider".to_owned()),
        ),
        &manifest,
    )
    .expect("receiver functional provider validates");
    let blend = validate_window_q15_provider(
        &ProviderFileEvidence::new(
            Arc::from(WINDOW_Q15_BYTES),
            Some("window provider".to_owned()),
        ),
        &manifest,
    )
    .expect("window provider validates");
    let fft = validate_qmf_mathematical_fft_provider(
        &ProviderFileEvidence::new(Arc::from(QMF_FFT_BYTES), Some("FFT provider".to_owned())),
        &manifest,
    )
    .expect("FFT provider validates");
    let descriptor = validate_descriptor_mathematical_provider(
        &ProviderFileEvidence::new(
            Arc::from(DESCRIPTOR_BYTES),
            Some("descriptor provider".to_owned()),
        ),
        &manifest,
    )
    .expect("descriptor provider validates");
    ValidatedProviders {
        functional,
        blend,
        fft,
        descriptor,
    }
}

fn selected_job_configuration() -> SelectedJobConfiguration {
    let vectors: serde_json::Value =
        serde_json::from_slice(JOB_CONFIGURATION_VECTORS).expect("job vectors are valid JSON");
    let xml = vectors["config_cases"][0]["input_xml"]
        .as_str()
        .expect("accepted job fixture has XML");
    parse_selected_job_configuration(
        xml.as_bytes(),
        SelectedWaveDescriptor::new(48_000, 38, 24),
        6_522_906,
    )
    .expect("selected job fixture admits the supported profile")
}

fn assert_final_six_stages_one_f07_to_f10_source(
    attempt_id: AttemptId,
    installed_audio: &SourcedInstalledAudioFrame,
) {
    let expected_source_index = installed_audio.source_index();
    let analysis_before = AnalysisOwnerRoot::cold();
    let before_reference = analysis_before.state_reference().unwrap();
    let transient = load_transient_parameters(Some(TRANSIENT_PARAMETERS)).unwrap();
    let transform = MathGeneratedTransformParameters::from_provider_bytes(MATH_PARAMETERS).unwrap();
    let policy =
        load_policy_parameters(PolicyProfile::selected(), Some(POLICY_PARAMETERS)).unwrap();

    let after = crate::analysis::source_analysis::prepare_source_analysis_frame(
        crate::analysis::source_analysis::SourceAnalysisInput {
            analysis_before: &analysis_before,
            attempt_id,
            transaction_id: CallbackTransactionId::new("spatial-final-six-source-analysis")
                .unwrap(),
            installed_audio,
            previous_short: [false; crate::analysis::structure_refresh::F07_FULLBAND_LANES],
            providers: crate::analysis::source_analysis::SourceAnalysisProviders {
                transient: &transient,
                transform: &transform,
                policy: &policy,
            },
        },
    )
    .expect("the admitted final-six value stages one selected F07-F10 source");

    let shared = after
        .shared_analysis()
        .expect("F07 issues the same-source analysis receipt");
    assert_eq!(shared.identity().source_index(), expected_source_index);
    assert_eq!(
        shared.identity().source_stage_key().attempt_id(),
        attempt_id
    );
    assert_eq!(shared.advancement_count(), 1);
    assert_eq!(after.after_state().input_history_generation(), 1);
    assert_eq!(after.after_state().transient_generation(), 1);
    assert_eq!(after.after_state().transform_generation(), 1);
    assert_eq!(after.after_state().policy_generation(), 1);
    assert!(after.after_state().is_commit_eligible());
    let plan = after.plan().expect("F07-F10 retain one source-local plan");
    assert!(plan.f08().is_six_block_complete());
    assert!(plan.exponents().is_six_block_complete());
    assert_eq!(analysis_before.state_reference().unwrap(), before_reference);
    assert_eq!(analysis_before.input_history_generation(), 0);
}

#[test]
fn full_parent_advances_priming_then_preserves_ready_target_lineage() {
    let source = compose_full_parent_source();
    assert_final_six_stages_one_f07_to_f10_source(source.attempt_id, &source.installed_audio);
}

struct ComposedSourceFixture {
    attempt_id: AttemptId,
    installed_audio: SourcedInstalledAudioFrame,
}

fn compose_full_parent_source() -> ComposedSourceFixture {
    let mut context = context();
    let providers = validated_providers();
    let configuration = selected_job_configuration();
    let mut reader_state = ReaderTargetState::cold();
    let current_xyz_words = [0_u32; 15 * 3];
    let current_ids = [u32::MAX; 15];
    let current_attributes = [0_u32; 15];
    let assignment_active = [0_u8; 15];
    let source_flags = [0_u8; 48];
    let mut source_audio = None;
    let silent_row = [0.0_f32; FRAME_SAMPLES];
    let previous_overlap_rows: [&[f32]; 15] = std::array::from_fn(|_| silent_row.as_slice());
    let current_mixer_rows: [&[f32]; 15] = std::array::from_fn(|_| silent_row.as_slice());

    for callback in 0..=2 {
        let (parent, phase, component, lfe) = valid_pair(&context, callback);
        let type10 = assemble_type10(&parent, phase, component, lfe)
            .expect("same-callback component and LFE stages assemble");
        let reader = stage_reader_target(
            &reader_state,
            &ReaderTargetCallbackInputs {
                callback_index: FrontendCallbackIndex::new(u32::try_from(callback).unwrap()),
                reader_enabled: callback >= 2,
                snapshot_keys_complete: true,
                programme_id: parent.programme_id().clone(),
                attempt_id: READER_ATTEMPT_ID,
                transaction_id: parent.transaction_id().clone(),
                profile_id: SELECTED_PROFILE_ID,
                current_xyz_words: &current_xyz_words,
                current_ids: &current_ids,
                current_attributes: &current_attributes,
                assignment_active: &assignment_active,
                active_target_indices: &[],
                matrix: &[],
                source_flags: &source_flags,
                previous_overlap_rows,
                current_mixer_rows,
            },
        )
        .expect("reader state advances on the same frontend callback");
        let (reader_after, target_receipt) = reader.into_parts_for_test();
        let attempt_binding = (callback >= 2)
            .then(|| ReaderAttemptBinding::new(parent.attempt(), READER_ATTEMPT_ID).unwrap());
        let proposal = stage_spatial_audio_callback(
            &context.spatial_audio,
            parent,
            phase,
            type10,
            target_receipt.as_ref(),
            attempt_binding,
            (callback >= 2).then(|| providers.input()),
        )
        .expect("the whole spatial-audio parent stages exactly one full root");
        let (after, outcome) = proposal.into_parts();
        let prepared = context
            .spatial_audio
            .prepare(after)
            .expect("parent after proposal still matches the committed before root");
        let staged = context
            .spatial_audio
            .stage(prepared)
            .expect("full-root staging computes the canonical after digest");
        assert_eq!(
            u64::from(staged.after_state().callback_index().value()),
            callback + 1
        );
        assert_eq!(
            staged.after_state().receiver_callback_count().value(),
            callback.saturating_sub(1)
        );

        match outcome {
            SpatialAudioCallbackOutcome::Priming => assert!(callback < 2),
            SpatialAudioCallbackOutcome::Joc(projection) => {
                assert_eq!(callback, 2);
                let installed_audio =
                    crate::analysis::source_frame::installed_audio_from_final_six(&projection)
                        .expect("the receiver's final six is the same typed F07 source");
                assert_eq!(installed_audio.source_index(), projection.source_index());
                for lane in 0..6 {
                    for block in 0..6 {
                        for sample in [0, 255] {
                            let source_sample = block * 256 + sample;
                            assert_eq!(
                                installed_audio.audio()[lane][block][sample],
                                projection.final_six()[lane][source_sample].to_bits(),
                            );
                        }
                    }
                }
                source_audio = Some(ComposedSourceFixture {
                    attempt_id: projection.attempt(),
                    installed_audio,
                });
                let payload_input = JocPayloadInput::from_projection(&projection)
                    .expect("the selected QMF projection binds a writer input");
                assert_eq!(
                    payload_input.clip_gain_index_for_test(),
                    projection.descriptor_selected_index()
                );
                let target_receipt = target_receipt
                    .as_ref()
                    .expect("ready callback carries a target receipt");
                assert_eq!(projection.callback_index().value(), 2);
                assert_eq!(projection.source_index().value(), 0);
                assert_eq!(projection.dispatch().value(), 1);
                assert_eq!(projection.component_origin_generation().value(), 2);
                assert_eq!(projection.lfe_origin_generation().value(), 0);
                assert_eq!(projection.target_reader_invocation().value(), 0);
                assert_eq!(
                    projection.target_history_producer_generations(),
                    target_receipt.history_producer_generations()
                );
                assert!(
                    projection
                        .target_history_producer_generations()
                        .contains(&HistoryProducerGeneration::Cold)
                );
                assert_eq!(
                    projection.target_update_words(),
                    &target_receipt.target_words()
                );
                assert_eq!(
                    projection.target_state_before_sha256(),
                    target_receipt.state_before_sha256()
                );
                assert_eq!(
                    projection.target_state_after_sha256(),
                    target_receipt.state_after_sha256()
                );
                assert_eq!(projection.target_sha256(), target_receipt.target_sha256());
                assert_eq!(
                    projection.target_semantic_sha256(),
                    target_receipt.semantic_sha256()
                );
                assert_eq!(projection.receiver_generation_after().value(), 1);
                let score_provider =
                    FitScoreProvider::from_approved_bytes(JOC_FUNCTIONAL_BYTES, JOC_MATH_BYTES)
                        .expect("selected JOC fit-score providers validate");
                let retry_provider =
                    RepresentationRetryProvider::from_approved_bytes(JOC_FUNCTIONAL_BYTES)
                        .expect("selected ID14 representation provider validates");
                let joc_source = stage_joc_source_with_transaction(
                    &CoefficientFitState::cold(),
                    &JocDispatchState::cold(),
                    &projection,
                    CallbackTransactionId::new("spatial-final-six-source-analysis")
                        .expect("the parent callback transaction ID is nonempty"),
                    &providers.functional,
                    &score_provider,
                    &retry_provider,
                )
                .expect("same-source F05JL stage supplies its complete ID14 body");
                assert_eq!(
                    joc_source.lineage().source_index(),
                    projection.source_index()
                );
                assert_eq!(joc_source.dispatch_after().sequence_counter(), 0);
                assert_eq!(joc_source.dispatch_after().successful_dispatches(), 1);
                assert_eq!(
                    joc_source.id14_body().body().writer_generation,
                    projection.source_index()
                );
                let bodies = joc_source
                    .assemble_metadata_bodies(&configuration, &projection, target_receipt)
                    .expect("same-dispatch ID14 writer body joins the F13 body barrier");
                assert_eq!(bodies.lineage().source_index(), projection.source_index());
                assert_eq!(bodies.lineage().dispatch(), projection.dispatch());
                assert_eq!(
                    bodies.bodies().id11.as_ref().unwrap().bytes,
                    target_receipt.id11_body()
                );
                assert_eq!(bodies.bodies().id2.as_ref().unwrap().bytes.len(), 3);
                let id14 = bodies.bodies().id14.as_ref().unwrap();
                assert_eq!(id14.writer_generation, projection.source_index());
                assert!(!id14.bytes.is_empty());
            }
        }
        context.spatial_audio.admit_staged_for_test(staged);
        assert_eq!(context.spatial_audio.generation(), callback + 1);
        reader_state = reader_after;
    }
    source_audio.expect("the ready g2 projection supplies source s=0")
}
