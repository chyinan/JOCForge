// pattern: Imperative Shell

use super::*;
use crate::{
    frontend::{
        PackageManifestEvidence, ProviderFileEvidence,
        job_configuration::{
            SelectedJobConfiguration, SelectedWaveDescriptor, parse_selected_job_configuration,
        },
    },
    joc::{
        dispatch_state::JocPayloadInput,
        emdf::{metadata_bodies::assemble_metadata_bodies, metadata_owner::MetadataPayloadBody},
        oamd::reader_target::{
            HistoryProducerGeneration, ReaderTargetCallbackInputs, ReaderTargetState,
            SELECTED_PROFILE_ID, stage_reader_target,
        },
    },
    spatial::{
        DescriptorMathematicalProvider, QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
        ValidatedWindowQ15Provider,
        spatial_audio_state::{
            fitting_mode0::Mode0FittingProviderInput,
            spatial_audio_composer::{
                ReaderAttemptBinding, SpatialAudioCallbackOutcome, SpatialAudioJocProviders,
                stage_spatial_audio_callback,
            },
            spatial_audio_receiver::SpatialAudioReceiverProviderInput,
        },
        validate_descriptor_mathematical_provider, validate_f04_parameter_package_manifest,
        validate_qmf_mathematical_fft_provider, validate_qmf_receiver_functional_provider,
        validate_window_q15_provider,
    },
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
const JOB_CONFIGURATION_VECTORS: &[u8] = include_bytes!(
    "../../../../../tests/fixtures/job_configuration/job-configuration-vectors.json"
);
const READER_ATTEMPT_ID: crate::transaction::AttemptId = crate::transaction::AttemptId::new(7);

struct ValidatedProviders {
    manifest_sha256: String,
    functional: QmfReceiverFunctionalProvider,
    blend: ValidatedWindowQ15Provider,
    fft: QmfMathematicalFftProvider,
    descriptor: DescriptorMathematicalProvider,
}

impl ValidatedProviders {
    fn input(&self) -> SpatialAudioJocProviders<'_> {
        SpatialAudioJocProviders {
            receiver: SpatialAudioReceiverProviderInput {
                functional: Some(&self.functional),
                blend: Some(&self.blend),
                fft: Some(&self.fft),
                descriptor: Some(&self.descriptor),
                selected_manifest_sha256: Some(&self.manifest_sha256),
            },
            fitting: Mode0FittingProviderInput::Ready {
                functional: &self.functional,
                fft: &self.fft,
                descriptor: &self.descriptor,
            },
        }
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
        manifest_sha256: manifest.manifest_sha256().to_hex(),
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

#[test]
fn full_parent_advances_priming_then_preserves_ready_target_lineage() {
    let mut context = context();
    let providers = validated_providers();
    let configuration = selected_job_configuration();
    let mut reader_state = ReaderTargetState::cold();
    let current_xyz_words = [0_u32; 15 * 3];
    let current_ids = [u32::MAX; 15];
    let current_attributes = [0_u32; 15];
    let assignment_active = [0_u8; 15];
    let source_flags = [0_u8; 48];
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
                let bodies = assemble_metadata_bodies(
                    &configuration,
                    &projection,
                    target_receipt,
                    MetadataPayloadBody {
                        writer_generation: projection.source_index(),
                        bytes: vec![0x40],
                    },
                )
                .expect("available ID11, ID2 and ID14 bodies share one source lineage");
                assert_eq!(bodies.lineage().source_index(), projection.source_index());
                assert_eq!(bodies.lineage().dispatch(), projection.dispatch());
                assert_eq!(
                    bodies.bodies().id11.as_ref().unwrap().bytes,
                    target_receipt.id11_body()
                );
                assert_eq!(bodies.bodies().id2.as_ref().unwrap().bytes.len(), 3);
                assert_eq!(bodies.bodies().id14.as_ref().unwrap().bytes, [0x40]);
            }
        }
        context.spatial_audio.admit_staged_for_test(staged);
        assert_eq!(context.spatial_audio.generation(), callback + 1);
        reader_state = reader_after;
    }
}
