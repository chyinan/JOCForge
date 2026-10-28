// pattern: Imperative Shell
//! Explicit user-authorized real-resource callback probe. It stores no PCM or path output.

use jocforge::meter_source_bridge::{
    prepare_selected_adm_bwf_input, stage_meter_source_oar_callback, view_for_source_proposal,
};
use jocforge_application::{
    ProgrammeId,
    meter_source_bridge::{
        IndependentMeterPrepass, KMatrixMode, KMatrixProfile, MeterCallbackIndex,
        MeterSourceSampleCount, SharedGainDelayParameters, generate_k_matrix,
    },
};
use jocforge_domain::{
    frontend::{
        FrontendControlMachine, PackageManifestEvidence, ProviderFileEvidence,
        ValidatedFeatureFft4096MathProvider, ValidatedFeatureProcessorMathProvider,
        normal_control_snapshot::{
            NormalControlBeforeState, NormalControlPreparedSnapshot,
            propose_normal_control_snapshot,
        },
        validate_current_features_package_manifest, validate_f01e_package_manifest,
        validate_f01e_provider, validate_f03_package_manifest, validate_f03_target_cold_provider,
        validate_feature_fft4096_math_provider, validate_feature_processor_math_provider,
    },
    joc::oamd::reader_target::ReaderTargetState,
    meter::installed48_renderer::{
        MeterInstalled48Input, MeterInstalled48State, stage_meter_installed48,
    },
    profile::{EncodingProfile, resolve_selected_profile},
    spatial::{
        SpatialControlMachine, ValidatedComponentParameterProvider, ValidatedWindowQ15Provider,
        diffusion_filter::{
            DiffusionFilterParameters, validate_diffusion_filter_package_manifest,
            validate_diffusion_filter_parameters,
        },
        frontend_stages::stage_frontend_features_from_installed48,
        installed48_activity::{PrimaryActivityInput, PrimaryInstalled48ActivityProducer},
        installed48_source_map::build_meter_installed48_source_map,
        post_window_fanout::CallbackPhase,
        production_callback::{
            SpatialProductionNormalCallbackInputs,
            continue_spatial_production_callback_from_normal_stage,
        },
        selected_p11_geometry::generate_selected_p11_geometry,
        selected_p15_geometry::generate_selected_p15_geometry,
        source_flags::{
            BeforeRegularHistory, CurrentHostPcm, PrefixWriterSet, SourceFlagsInput,
            SourceFlagsParentLineage, project_normal_control_snapshot_source_controls,
        },
        source_geometry_outer::stage_normal_source_geometry,
        spatial_audio_state::SpatialAudioMachine,
        validate_component_parameter_provider, validate_f04_parameter_package_manifest,
        validate_window_q15_provider,
    },
    timeline::{AdmSampleIndex, FrontendCallbackIndex},
    transaction::{AttemptId, CallbackLineageIdentity},
};
use jocforge_io::pcm::{
    meter_source_cursor::{
        MeterPcmSourceCursor, MeterPcmSourceIdentity, stage_meter_pcm_source_frame,
    },
    riff_pcm24::parse_selected_riff_pcm24,
};
use serde_json::Value;
use std::{env, fmt::Write as _, fs, sync::Arc};

const F01E_MANIFEST: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/f01e_provider/MANIFEST.json");
const F01E_PROVIDER: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/f01e_provider/f01e-selected-constructor-v1.json"
);
const F03_PROVIDER: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/target_cold/f03-selected-target-cold-v1.json"
);
const FEATURE_PROCESSOR: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/current_features/feature-processor-math-v1.json"
);
const FEATURE_FFT: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/current_features/feature-fft4096-math-v1.json"
);
const WINDOW_Q15: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/f04_parameters/window-q15.json");
const COMPONENT_PARAMETERS: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/f04_parameters/component-parameter-mathematical-v1.json"
);
const DIFFUSION_PARAMETERS: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/diffusion_filter/diffusion-mathematical-v1.json"
);
const DIFFUSION_CONSTRUCTOR: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/diffusion_filter/diffusion-constructor-mathematical-v1.json"
);

fn lowercase_sha256(bytes: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("formatting into a String is infallible");
    }
    output
}

fn validated_f01e_provider() -> jocforge_domain::frontend::ValidatedF01EColdProvider {
    let manifest = validate_f01e_package_manifest(&PackageManifestEvidence::new(
        Arc::from(F01E_MANIFEST),
        None,
    ))
    .unwrap();
    validate_f01e_provider(
        &ProviderFileEvidence::new(Arc::from(F01E_PROVIDER), None),
        &manifest,
    )
    .unwrap()
}

fn selected_diffusion_parameters() -> DiffusionFilterParameters {
    let manifest = validate_diffusion_filter_package_manifest(&PackageManifestEvidence::new(
        Arc::from(F01E_MANIFEST),
        None,
    ))
    .unwrap();
    validate_diffusion_filter_parameters(
        &ProviderFileEvidence::new(Arc::from(DIFFUSION_PARAMETERS), None),
        &ProviderFileEvidence::new(Arc::from(DIFFUSION_CONSTRUCTOR), None),
        &manifest,
    )
    .unwrap()
}

fn selected_meter_prepass(source_samples: MeterSourceSampleCount) -> IndependentMeterPrepass {
    const SELECTED_KERNEL_Q15: [u16; 32] = [
        20, 177, 491, 958, 1_573, 2_331, 3_224, 4_244, 5_381, 6_624, 7_961, 9_379, 10_864, 12_403,
        13_980, 15_580, 17_188, 18_788, 20_365, 21_904, 23_389, 24_807, 26_144, 27_387, 28_524,
        29_544, 30_437, 31_195, 31_810, 32_277, 32_591, 32_748,
    ];
    let gain = SharedGainDelayParameters::selected(&SELECTED_KERNEL_Q15).unwrap();
    let matrix = generate_k_matrix(KMatrixProfile {
        sample_rate_hz: 48_000,
        mode: KMatrixMode::Modern,
        stage_count: 2,
    })
    .unwrap();
    IndependentMeterPrepass::cold(source_samples, gain, matrix).unwrap()
}

struct SpatialCallbackOwners {
    spatial_control: SpatialControlMachine,
    spatial_audio: SpatialAudioMachine,
    reader_target: ReaderTargetState,
    processor: ValidatedFeatureProcessorMathProvider,
    fft: ValidatedFeatureFft4096MathProvider,
    window: ValidatedWindowQ15Provider,
    parameters: ValidatedComponentParameterProvider,
}

fn spatial_callback_owners() -> SpatialCallbackOwners {
    let profile = EncodingProfile::selected();
    let manifest = PackageManifestEvidence::new(Arc::from(F01E_MANIFEST), None);
    let f01e_manifest = validate_f01e_package_manifest(&manifest).unwrap();
    let f01e = validate_f01e_provider(
        &ProviderFileEvidence::new(Arc::from(F01E_PROVIDER), None),
        &f01e_manifest,
    )
    .unwrap();
    let f03_manifest = validate_f03_package_manifest(&manifest).unwrap();
    let f03 = validate_f03_target_cold_provider(
        &ProviderFileEvidence::new(Arc::from(F03_PROVIDER), None),
        &f03_manifest,
    )
    .unwrap();
    let current_features_manifest = validate_current_features_package_manifest(&manifest).unwrap();
    let processor = validate_feature_processor_math_provider(
        &ProviderFileEvidence::new(Arc::from(FEATURE_PROCESSOR), None),
        &current_features_manifest,
    )
    .unwrap();
    let fft = validate_feature_fft4096_math_provider(
        &ProviderFileEvidence::new(Arc::from(FEATURE_FFT), None),
        &current_features_manifest,
    )
    .unwrap();
    let f04_manifest = validate_f04_parameter_package_manifest(&manifest).unwrap();
    let window = validate_window_q15_provider(
        &ProviderFileEvidence::new(Arc::from(WINDOW_Q15), None),
        &f04_manifest,
    )
    .unwrap();
    let parameters = validate_component_parameter_provider(
        &ProviderFileEvidence::new(Arc::from(COMPONENT_PARAMETERS), None),
        &f04_manifest,
        &profile,
    )
    .unwrap();
    SpatialCallbackOwners {
        spatial_control: SpatialControlMachine::new_cold(profile, &f01e, &f03).unwrap(),
        spatial_audio: SpatialAudioMachine::new_cold(profile).unwrap(),
        reader_target: ReaderTargetState::cold(),
        processor,
        fft,
        window,
        parameters,
    }
}

#[test]
#[ignore = "requires an explicitly supplied user-authorized 38-channel ADM BWF path"]
fn authorized_adm_bwf_produces_one_bound_normal_geometry_and_pcm_callback() {
    let path = env::var_os("JOCFORGE_AUTHORIZED_WAV")
        .expect("set JOCFORGE_AUTHORIZED_WAV to the user-authorized ADM BWF file");
    let resource: Arc<[u8]> = Arc::from(fs::read(path).unwrap().into_boxed_slice());
    let parsed = parse_selected_riff_pcm24(&resource, None).unwrap();
    let expected_bytes = env::var("JOCFORGE_AUTHORIZED_WAV_BYTES")
        .expect("set the independently measured WAV byte count")
        .parse::<u64>()
        .unwrap();
    let expected_sha = env::var("JOCFORGE_AUTHORIZED_WAV_SHA256")
        .expect("set the independently measured WAV SHA-256");
    assert_eq!(u64::try_from(resource.len()).unwrap(), expected_bytes);
    assert_eq!(lowercase_sha256(&parsed.source_sha256()), expected_sha);
    assert_eq!(parsed.sample_rate_hz(), 48_000);
    assert_eq!(parsed.channels(), 38);
    assert_eq!(parsed.block_align(), 114);
    assert_eq!(parsed.total_samples(), 6_522_906);

    let job_vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let job_xml = job_vectors["config_cases"][0]["input_xml"]
        .as_str()
        .unwrap()
        .as_bytes();
    let programme_id = ProgrammeId::new("authorized-adm-normal-callback").unwrap();
    let prepared = prepare_selected_adm_bwf_input(
        programme_id.clone(),
        job_xml,
        Arc::clone(&resource),
        &parsed,
    )
    .unwrap();
    assert_eq!(
        prepared.primary_metadata_controls().unwrap().rows().len(),
        38
    );

    let frontend = FrontendControlMachine::from_provider(
        validated_f01e_provider(),
        programme_id.clone(),
        AttemptId::new(0),
    )
    .unwrap();
    let snapshot: Arc<NormalControlPreparedSnapshot> = prepared
        .normal_control_prepared_snapshot(frontend.committed())
        .unwrap();
    let before = NormalControlBeforeState::from_frontend_state(
        frontend.committed(),
        frontend.generation(),
        None,
    )
    .unwrap();
    let transaction_id = CallbackLineageIdentity::new(
        programme_id,
        AttemptId::new(0),
        FrontendCallbackIndex::new(0),
    );
    let normal = propose_normal_control_snapshot(&before, &snapshot, transaction_id).unwrap();
    assert_eq!(normal.control38_rows().len(), 38);
    assert_eq!(normal.invocation_hosts().len(), 28);

    let p11 = generate_selected_p11_geometry(0x111b, true).unwrap();
    let source_map =
        build_meter_installed48_source_map(prepared.primary_metadata_controls().unwrap(), &p11)
            .unwrap();
    let combined =
        stage_normal_source_geometry(prepared.graph().graph(), &source_map, normal).unwrap();
    assert_eq!(combined.source_geometry().rows().len(), 48);
    assert_eq!(combined.source_geometry().value_generation().value(), 0);

    let source_identity = MeterPcmSourceIdentity::from_selected_resource_sha256(
        prepared.resource_identity().input_sha256(),
    );
    let source_samples = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
    let mut cursor = MeterPcmSourceCursor::cold(&parsed, source_identity, source_samples).unwrap();
    let pcm_proposal =
        stage_meter_pcm_source_frame(&cursor, MeterCallbackIndex::new(0), &parsed).unwrap();
    let callback = view_for_source_proposal(&prepared, &pcm_proposal).unwrap();
    assert_eq!(callback.physical_rows().len(), 38);
    assert_eq!(callback.physical_rows()[0].len(), 1_536);
    assert_eq!(callback.host_input_count().value(), 1_536);
    assert_eq!(callback.source_padding_count().value(), 0);
    assert_eq!(
        callback.source_identity().sha256(),
        prepared.resource_identity().input_sha256()
    );

    let activity_producer = PrimaryInstalled48ActivityProducer::new(&source_map).unwrap();
    let activity_before = activity_producer.cold_state().unwrap();
    let activity = activity_producer
        .produce_next(
            &source_map,
            &activity_before,
            PrimaryActivityInput::NormalRead {
                reader_relative_sample: AdmSampleIndex::new(0),
                pcm_word_rows: callback.physical_rows().to_vec(),
            },
        )
        .unwrap();
    let source_binding = source_map.source_binding();
    let diffusion_parameters = selected_diffusion_parameters();
    let meter_before =
        MeterInstalled48State::cold(*source_binding.source_receipt_sha256(), source_samples);
    let installed48 = stage_meter_installed48(
        &meter_before,
        MeterInstalled48Input {
            source_sha256: *source_binding.source_receipt_sha256(),
            callback_index: MeterCallbackIndex::new(0),
            source_cursor_before: AdmSampleIndex::new(callback.source_cursor_before().value()),
            host_input_samples: u16::try_from(callback.host_input_count().value()).unwrap(),
            source_padding_samples: u16::try_from(callback.source_padding_count().value()).unwrap(),
            raw_pcm_rows: callback.physical_rows(),
            activity: activity.output(),
            membership_route: activity.membership_route(),
            source_map: &source_map,
            controls: prepared.primary_metadata_controls().unwrap(),
            diffusion_parameters: Some(&diffusion_parameters),
        },
    )
    .unwrap();
    assert_eq!(installed48.frame().rows().len(), 48);
    assert!(
        installed48
            .frame()
            .rows()
            .iter()
            .all(|row| row.len() == 3_072)
    );

    let owners = spatial_callback_owners();
    let normal_control = combined.normal_control();
    let resolved_profile = resolve_selected_profile(&EncodingProfile::selected()).unwrap();
    let source_projection = project_normal_control_snapshot_source_controls(
        normal_control,
        prepared.graph().graph(),
        resolved_profile,
    )
    .unwrap();
    let source_control_clock = source_projection.clock();
    let before_history =
        BeforeRegularHistory::from_owner(&owners.spatial_control, source_projection.route())
            .unwrap();
    let prefix_writers =
        PrefixWriterSet::new(activity.output().prefix_writer_indices().to_vec()).unwrap();
    let source_flags_parent = SourceFlagsParentLineage::new(
        normal_control.transaction_id().clone(),
        &owners.spatial_control,
    )
    .unwrap();
    let current_pcm = callback
        .physical_rows()
        .iter()
        .map(|row| row.iter().copied().map(f32::from_bits).collect())
        .collect();
    let source_flags = SourceFlagsInput::from_primary_regular_source_controls(
        CurrentHostPcm::new(current_pcm).unwrap(),
        Some(before_history),
        source_projection,
        prefix_writers,
        source_flags_parent,
    )
    .unwrap();
    let frontend = stage_frontend_features_from_installed48(
        &owners.spatial_control,
        normal_control.transaction_id().clone(),
        activity,
        installed48,
        source_flags,
        CallbackPhase::Normal,
        &owners.processor,
        &owners.fft,
    )
    .unwrap();
    let p15 = generate_selected_p15_geometry(0x159b, true).unwrap();
    let spatial_control_before = owners.spatial_control.digest();
    let spatial_audio_before = owners.spatial_audio.digest();
    let reader_target_before = owners.reader_target.canonical_sha256();
    let spatial_stage = continue_spatial_production_callback_from_normal_stage(
        SpatialProductionNormalCallbackInputs {
            frontend,
            normal_source_geometry: combined,
            source_control_clock,
            source_map: &source_map,
            p15_geometry: &p15,
            spatial_control: &owners.spatial_control,
            spatial_audio: &owners.spatial_audio,
            reader_target: &owners.reader_target,
            window: &owners.window,
            parameters: &owners.parameters,
        },
    )
    .unwrap();
    assert_eq!(spatial_stage.callback_index().value(), 0);
    assert_eq!(spatial_stage.component_rows().callback_index().value(), 0);
    assert_eq!(spatial_stage.canonical_dense_d_f32_bits().len(), 15 * 48);
    assert_eq!(owners.spatial_control.digest(), spatial_control_before);
    assert_eq!(owners.spatial_audio.digest(), spatial_audio_before);
    assert_eq!(
        owners.reader_target.canonical_sha256(),
        reader_target_before
    );

    let handoff = callback
        .bind_spatial_outputs(
            spatial_stage.component_rows(),
            spatial_stage.reader_target_proposal().delivered_controls(),
        )
        .unwrap();
    let mut meter_prepass = selected_meter_prepass(source_samples);
    let cursor_before = cursor.callback_index();
    let meter_callback_before = meter_prepass.next_callback_index();
    let oar_stage =
        stage_meter_source_oar_callback(&mut cursor, &mut meter_prepass, pcm_proposal, handoff)
            .unwrap();
    drop(oar_stage);
    assert_eq!(cursor.callback_index(), cursor_before);
    assert_eq!(meter_prepass.next_callback_index(), meter_callback_before);

    println!(
        "authorized one-callback check passed: 38 PCM rows, 48 geometry/installed rows, spatial stages 5-14, OAR stage pending, {} samples",
        parsed.total_samples()
    );
}
