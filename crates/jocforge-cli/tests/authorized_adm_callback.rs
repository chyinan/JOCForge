// pattern: Imperative Shell
//! Explicit user-authorized real-resource callback probe. It stores no PCM or path output.

use jocforge::independent_meter_callback::{
    stage_independent_meter_g0_callback, stage_independent_meter_normal_callback,
};
use jocforge::meter_callback_stage::{
    MainOutputOarMeterProbeInput, stage_main_output_oar_meter_probe,
};
use jocforge::meter_source_bridge::{prepare_selected_adm_bwf_input, view_for_source_proposal};
use jocforge_application::{
    PreparedJobInput, ProgrammeId,
    independent_meter_callback::{IndependentMeterG0Providers, MeterCallbackGtcBinding},
    metadata_source_bridge::{
        MainMetadataBodiesInput, prepare_main_metadata_bodies_from_spatial_callback,
        prepare_metadata_owner_from_measurement,
    },
    meter_source_admission::admit_meter_source_owner,
    meter_source_bridge::{
        FrontendControlReplacementCandidate, KMatrixMode, KMatrixProfile, MeterCallbackIndex,
        MeterCallbackPhase, MeterComponentOutputRows, MeterKFilterExponentState,
        MeterProviderIdentity, MeterProviderSet, MeterRootCallbackProgress, MeterSourcePosition,
        MeterSourceSampleCount, NormalControlSnapshotOutput, SharedGainDelayParameters,
        SpatialAudioCallbackOutcome, SpatialAudioReplacementCandidate,
        SpatialControlReplacementCandidate, StandaloneMeterRoot, generate_k_matrix,
    },
    source_analysis_bridge::{
        SourceAnalysisBridgeError, SpatialSourceAnalysisBridgeInput,
        prepare_source_analysis_from_spatial_callback,
    },
    spatial_callback::{MainSpatialCallbackInput, stage_main_spatial_callback},
};
use jocforge_domain::{
    analysis::{
        owner::AnalysisOwnerRoot, source_analysis::SourceAnalysisProviders,
        transient_parameters::load_transient_parameters,
    },
    codec::policy_parameters::{PolicyProfile, load_policy_parameters},
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
    joc::{
        coefficient_representation::RepresentationRetryProvider, fit_score::FitScoreProvider,
        programme_state::SelectedJocState,
    },
    profile::EncodingProfile,
    spatial::{
        SpatialControlMachine, ValidatedComponentParameterProvider, ValidatedWindowQ15Provider,
        diffusion_filter::{
            DiffusionFilterParameters, validate_diffusion_filter_package_manifest,
            validate_diffusion_filter_parameters,
        },
        selected_p11_geometry::generate_selected_p11_geometry,
        selected_p15_geometry::generate_selected_p15_geometry,
        spatial_audio_state::SpatialAudioMachine,
        validate_component_parameter_provider, validate_f04_parameter_package_manifest,
        validate_window_q15_provider,
    },
    state::{Control38Snapshot, Host38Frame, ReaderEofMachine, ReaderEofPhase, ReaderEofState},
    timeline::{FrontendCallbackIndex, NormalCallbackCount},
    transaction::{AttemptId, CallbackLineageIdentity, CallbackTransactionId, GtcOwnerId},
    transform::math_generated_parameters::MathGeneratedTransformParameters,
};
use jocforge_io::pcm::{
    meter_source_cursor::{
        MeterPcmSourceCursor, MeterPcmSourceIdentity, MeterPcmSourceProposal,
        stage_meter_pcm_source_frame,
    },
    riff_pcm24::parse_selected_riff_pcm24,
};
use jocforge_io::provider::SelectedSpatialAudioProviderBundle;
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
const METER_ROOT_V4_COLD: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-ROOT-CANONICAL-STATE-REFERENCE-V4/input-only-cold-roots/N6522906_pre_callback0_cold_root.json"
);
const F07_TRANSIENT_PARAMETERS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/ANALYSIS-ROOT-APPROVED-SOURCE-LEAVES-V1/approved/parameters/transient-d05-authorized-v1.json"
);
const F08_TRANSFORM_PARAMETERS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/ANALYSIS-ROOT-APPROVED-SOURCE-LEAVES-V1/approved/parameters/ordinary-transform-math-v1.json"
);
const F09_POLICY_PARAMETERS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/ANALYSIS-ROOT-APPROVED-SOURCE-LEAVES-V1/approved/parameters/policy-authorized-v1.json"
);
const JOC_FUNCTIONAL_PARAMETERS: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-functional.json"
);
const JOC_MATH_PARAMETERS: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-math.json"
);
const SOURCE_ANALYSIS_TEST_STACK_BYTES: usize = 16 * 1024 * 1024;

fn lowercase_sha256(bytes: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("formatting into a String is infallible");
    }
    output
}

fn sha256_from_hex(value: &str) -> [u8; 32] {
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap();
    }
    bytes
}

fn selected_meter_provider_set() -> MeterProviderSet {
    let value: Value = serde_json::from_slice(METER_ROOT_V4_COLD).unwrap();
    let providers = value["binding"]["provider_ids_hash_sorted"]
        .as_array()
        .unwrap()
        .iter()
        .map(|provider| {
            MeterProviderIdentity::new(
                provider["id"].as_str().unwrap(),
                sha256_from_hex(provider["sha256"].as_str().unwrap()),
            )
            .unwrap()
        })
        .collect();
    MeterProviderSet::new(
        sha256_from_hex(value["binding"]["provider_set_sha256"].as_str().unwrap()),
        providers,
    )
    .unwrap()
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

struct SpatialCallbackOwners<'a> {
    spatial_control: SpatialControlMachine,
    spatial_audio: SpatialAudioMachine,
    processor: &'a ValidatedFeatureProcessorMathProvider,
    fft: &'a ValidatedFeatureFft4096MathProvider,
    window: &'a ValidatedWindowQ15Provider,
    parameters: &'a ValidatedComponentParameterProvider,
}

fn spatial_callback_owners(providers: &IndependentMeterG0Providers) -> SpatialCallbackOwners<'_> {
    let profile = EncodingProfile::selected();
    SpatialCallbackOwners {
        spatial_control: SpatialControlMachine::new_cold(
            profile,
            &providers.f01e,
            &providers.f03_target,
        )
        .unwrap(),
        spatial_audio: SpatialAudioMachine::new_cold(profile).unwrap(),
        processor: &providers.feature_processor,
        fft: &providers.feature_fft,
        window: &providers.window,
        parameters: &providers.component_parameters,
    }
}

fn independent_meter_g0_providers() -> IndependentMeterG0Providers {
    // These identities come from the repository's locally validated provider packages. They do
    // not establish native encoder identity or equivalence to any external implementation.
    let manifest = PackageManifestEvidence::new(Arc::from(F01E_MANIFEST), None);
    let f01e_manifest = validate_f01e_package_manifest(&manifest).unwrap();
    let f01e = validate_f01e_provider(
        &ProviderFileEvidence::new(Arc::from(F01E_PROVIDER), None),
        &f01e_manifest,
    )
    .unwrap();
    let f03_manifest = validate_f03_package_manifest(&manifest).unwrap();
    let f03_target = validate_f03_target_cold_provider(
        &ProviderFileEvidence::new(Arc::from(F03_PROVIDER), None),
        &f03_manifest,
    )
    .unwrap();
    let current_features_manifest = validate_current_features_package_manifest(&manifest).unwrap();
    let feature_processor = validate_feature_processor_math_provider(
        &ProviderFileEvidence::new(Arc::from(FEATURE_PROCESSOR), None),
        &current_features_manifest,
    )
    .unwrap();
    let feature_fft = validate_feature_fft4096_math_provider(
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
    let component_parameters = validate_component_parameter_provider(
        &ProviderFileEvidence::new(Arc::from(COMPONENT_PARAMETERS), None),
        &f04_manifest,
        &EncodingProfile::selected(),
    )
    .unwrap();
    IndependentMeterG0Providers {
        f01e,
        f03_target,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        diffusion: selected_diffusion_parameters(),
        p11_geometry: generate_selected_p11_geometry(0x111b, true).unwrap(),
        p15_geometry: generate_selected_p15_geometry(0x159b, true).unwrap(),
    }
}

fn selected_main_spatial_callback_provider_inputs()
-> jocforge::selected_spatial_callback_providers::SelectedMainSpatialCallbackProviders {
    let supplements = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../outputs/IMPLEMENTER_SUPPLEMENTS");
    jocforge::selected_spatial_callback_providers::load_selected_main_spatial_callback_providers(
        &supplements.join("ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1"),
        &supplements.join("F01B-F03W-COLD-STATE-ASSETS-V1"),
        &supplements.join("METER-APPROVED-PARAMETER-ASSETS-V1"),
        &supplements.join("F04-RECEIVER-APPROVED-PARAMETERS-V1"),
        jocforge_application::independent_meter_callback::generate_selected_spatial_math_geometry()
            .expect("selected P11/P15 mathematical geometry is generated"),
    )
    .expect("approved MAIN and F04 receiver provider values are loaded")
}

fn selected_standalone_meter_root(
    prepared: &PreparedJobInput,
    source_samples: MeterSourceSampleCount,
) -> StandaloneMeterRoot {
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
    let admission = admit_meter_source_owner(prepared, AttemptId::new(0), source_samples).unwrap();
    StandaloneMeterRoot::cold_selected(
        admission.standalone_meter_identity(),
        source_samples,
        admission.owner_id().clone(),
        selected_meter_provider_set(),
        MeterKFilterExponentState::new([0; 6]),
        gain,
        matrix,
    )
    .unwrap()
}

struct AuthorizedInitialSpatialCandidate {
    cursor: MeterPcmSourceCursor,
    source_proposal: MeterPcmSourceProposal,
    frontend_candidate: FrontendControlReplacementCandidate,
    spatial_control_candidate: SpatialControlReplacementCandidate,
    spatial_audio_candidate: SpatialAudioReplacementCandidate,
    component_rows: MeterComponentOutputRows,
    spatial_audio_outcome: SpatialAudioCallbackOutcome,
}

struct AuthorizedMainSpatialContinuationCandidates {
    source_proposal: MeterPcmSourceProposal,
    frontend_candidate: FrontendControlReplacementCandidate,
    spatial_control_candidate: SpatialControlReplacementCandidate,
    spatial_audio_candidate: SpatialAudioReplacementCandidate,
}

#[test]
#[ignore = "requires an explicitly supplied user-authorized 38-channel ADM BWF path"]
fn authorized_adm_bwf_stages_one_main_output_oar_meter_probe() {
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
    let before =
        NormalControlBeforeState::from_frontend_state(frontend.committed(), frontend.generation())
            .unwrap();
    let transaction_id = CallbackLineageIdentity::new(
        programme_id,
        AttemptId::new(0),
        FrontendCallbackIndex::new(0),
    );
    let normal = propose_normal_control_snapshot(&before, &snapshot, transaction_id).unwrap();
    assert_eq!(normal.control38_rows().len(), 38);
    assert_eq!(normal.invocation_hosts().len(), 28);

    let frontend_before = frontend.state_reference();
    let parent_transaction_id =
        CallbackTransactionId::new("authorized-g0-parent-callback").unwrap();
    let frontend_stage = frontend.stage_normal_snapshot(normal).unwrap();
    let (frontend_staged, normal_output) = frontend_stage.into_parts();
    let frontend_candidate = frontend
        .replacement_candidate(frontend_staged, parent_transaction_id.clone())
        .unwrap();
    let frontend_receipt = frontend_candidate.receipt();
    let next_frontend = frontend_candidate.next_machine();
    assert_eq!(frontend_receipt.owner_id(), GtcOwnerId::FrontendControl);
    assert_eq!(frontend_receipt.before_generation(), 0);
    assert_eq!(frontend_receipt.after_generation(), 1);
    assert!(frontend_receipt.changed());
    assert_eq!(frontend_receipt.commit_count(), 1);
    assert_eq!(
        frontend_receipt.transaction_id().as_str(),
        "authorized-g0-parent-callback"
    );
    assert_eq!(next_frontend.generation(), 1);
    assert_eq!(
        next_frontend.committed().schema(),
        "FRONTEND_CONTROL_AFTER_STATE_V1"
    );
    assert_eq!(next_frontend.committed().callback_index().value(), 1);
    assert_eq!(next_frontend.committed().normal_callbacks().value(), 1);
    let frontend_state_reference = next_frontend.state_reference();
    let frontend_state_bytes = frontend_state_reference.canonical_bytes();
    let frontend_state_value: Value = serde_json::from_slice(frontend_state_bytes).unwrap();
    assert_eq!(
        serde_json::to_vec(&frontend_state_value).unwrap(),
        frontend_state_bytes
    );
    assert_eq!(frontend_state_value["owner_id"], "frontend_control");
    assert_eq!(frontend_state_value["owner_generation"], 1);
    assert_eq!(frontend_state_value["callback_index"], 1);
    assert_eq!(
        frontend_state_value["objects"].as_array().unwrap().len(),
        28
    );
    assert_eq!(
        frontend_state_value["last_control38_snapshot"]
            .as_array()
            .unwrap()
            .len(),
        38
    );
    assert_eq!(
        frontend_state_value["last_regular37_snapshot"]
            .as_array()
            .unwrap()
            .len(),
        37
    );
    assert_eq!(
        frontend_state_value["prepared_identity"]["physical_tracks"]
            .as_array()
            .unwrap()
            .len(),
        38
    );
    assert_eq!(
        frontend_state_value["prepared_identity"]["object_events"]
            .as_array()
            .unwrap()
            .len(),
        28
    );
    assert_eq!(
        next_frontend
            .committed()
            .last_control38_snapshot()
            .unwrap()
            .as_ref(),
        normal_output.control38_carrier_words()
    );
    assert_eq!(
        next_frontend
            .committed()
            .last_regular37_snapshot()
            .unwrap()
            .as_ref(),
        normal_output.regular37_carrier_words()
    );
    assert!(next_frontend.committed().prepared_identity().is_some());
    assert_eq!(frontend.generation(), 0);
    assert_eq!(
        frontend.state_reference().state_sha256(),
        frontend_before.state_sha256()
    );

    let callback_provider_inputs = selected_main_spatial_callback_provider_inputs();
    run_authorized_initial_meter_callback(
        &prepared,
        &resource,
        &parsed,
        frontend_candidate,
        normal_output,
        &parent_transaction_id,
        callback_provider_inputs.providers(),
        callback_provider_inputs.spatial_audio(),
    );
}

#[test]
#[ignore = "requires an explicitly supplied user-authorized 38-channel ADM BWF path"]
fn authorized_adm_bwf_stages_independent_meter_g0_and_g1_oar_candidates() {
    std::thread::Builder::new()
        .name("authorized-independent-meter-callback-test".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(authorized_adm_bwf_stages_independent_meter_g0_and_g1_oar_candidates_inner)
        .expect("start the meter callback test on its selected stack")
        .join()
        .expect("the meter callback test thread should not panic");
}

fn authorized_adm_bwf_stages_independent_meter_g0_and_g1_oar_candidates_inner() {
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
    let programme_id = ProgrammeId::new("authorized-independent-meter-g0").unwrap();
    let prepared = prepare_selected_adm_bwf_input(
        programme_id.clone(),
        job_xml,
        Arc::clone(&resource),
        &parsed,
    )
    .unwrap();
    // This probe invokes only the independent Meter entry point. Its programme identity is
    // deliberately separate from the MAIN-output probe's identity above.
    assert_eq!(
        prepared.programme_id().as_str(),
        "authorized-independent-meter-g0"
    );
    assert_ne!(
        prepared.programme_id().as_str(),
        "authorized-adm-normal-callback"
    );
    let source_samples = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
    let source_identity = MeterPcmSourceIdentity::from_selected_resource_sha256(
        prepared.resource_identity().input_sha256(),
    );
    let source_cursor =
        MeterPcmSourceCursor::cold(&parsed, source_identity, source_samples).unwrap();
    let meter_root = selected_standalone_meter_root(&prepared, source_samples);
    let meter_root_before = meter_root.canonical_sha256().unwrap();
    let parent_transaction_id =
        CallbackTransactionId::new("authorized-independent-meter-g0-parent").unwrap();
    let meter_binding = MeterCallbackGtcBinding::new(
        parent_transaction_id,
        programme_id,
        AttemptId::new(0),
        MeterCallbackIndex::new(0),
    );

    let candidate = stage_independent_meter_g0_callback(
        &prepared,
        &source_cursor,
        &parsed,
        &meter_root,
        meter_binding,
        MeterKFilterExponentState::new(*meter_root.k_filter_exponents_i32_by_channel()),
        &independent_meter_g0_providers(),
    )
    .unwrap();
    let spatial = candidate.application_candidate();
    assert_eq!(candidate.source_cursor_after().callback_index().value(), 1);
    assert_eq!(
        candidate.source_cursor_after().source_cursor().value(),
        1_536
    );
    assert_eq!(candidate.source_cursor_after().read_index().value(), 1);
    assert_eq!(source_cursor.callback_index().value(), 0);
    assert_eq!(source_cursor.source_cursor().value(), 0);
    assert_eq!(source_cursor.read_index().value(), 0);
    assert_eq!(
        spatial.frontend_candidate().receipt().owner_id(),
        GtcOwnerId::FrontendControl
    );
    assert_eq!(
        spatial.frontend_candidate().receipt().before_generation(),
        0
    );
    assert_eq!(spatial.frontend_candidate().receipt().after_generation(), 1);
    assert_eq!(
        spatial
            .frontend_candidate()
            .next_machine()
            .committed()
            .callback_index()
            .value(),
        1
    );
    assert_eq!(spatial.primary_activity_after().next_callback().value(), 1);
    assert_eq!(
        spatial.spatial_control_candidate().receipt().owner_id(),
        GtcOwnerId::SpatialControl
    );
    assert_eq!(
        spatial
            .spatial_control_candidate()
            .receipt()
            .before_generation(),
        0
    );
    assert_eq!(
        spatial
            .spatial_control_candidate()
            .receipt()
            .after_generation(),
        1
    );
    assert_eq!(
        spatial.spatial_audio_candidate().receipt().owner_id(),
        GtcOwnerId::SpatialAudio
    );
    assert_eq!(
        spatial
            .spatial_audio_candidate()
            .receipt()
            .before_generation(),
        0
    );
    assert_eq!(
        spatial
            .spatial_audio_candidate()
            .receipt()
            .after_generation(),
        1
    );
    assert_eq!(spatial.component_rows().callback_index().value(), 0);
    assert_eq!(
        spatial.component_rows().canonical_rows_f32_bits().len(),
        15 * 1_536
    );
    assert_eq!(spatial.reader_target_controls().callback_index().value(), 0);
    assert_eq!(spatial.reader_target_controls().rows().len(), 15);
    assert_eq!(
        spatial.source_receipt().spatial_callback_index(),
        Some(FrontendCallbackIndex::new(0))
    );
    assert_eq!(
        spatial
            .source_receipt()
            .target_control_lineages()
            .unwrap()
            .len(),
        15
    );
    let meter_receipt = spatial.meter_proposal().gtc_receipt().unwrap();
    assert_eq!(meter_receipt.owner_id(), GtcOwnerId::Meter);
    assert_eq!(meter_receipt.before_generation(), 0);
    assert_eq!(meter_receipt.after_generation(), 1);
    assert_eq!(meter_receipt.commit_count(), 1);
    assert_eq!(
        spatial
            .meter_proposal()
            .after_root()
            .meter_callback_index()
            .value(),
        1
    );
    assert_eq!(meter_root.meter_callback_index().value(), 0);
    assert_eq!(meter_root.canonical_sha256().unwrap(), meter_root_before);

    stage_authorized_independent_meter_g1(
        &prepared,
        &parsed,
        &source_cursor,
        candidate,
        &meter_root,
        meter_root_before,
    );
}

#[inline(never)]
fn stage_authorized_independent_meter_g1(
    prepared: &PreparedJobInput,
    parsed: &jocforge_io::pcm::riff_pcm24::ParsedRiffPcm24<'_>,
    source_cursor: &MeterPcmSourceCursor,
    candidate: jocforge::independent_meter_callback::IndependentMeterG0Candidate,
    meter_root: &StandaloneMeterRoot,
    meter_root_before: [u8; 32],
) {
    let next = stage_independent_meter_normal_callback(
        prepared,
        candidate,
        parsed,
        MeterCallbackGtcBinding::new(
            CallbackTransactionId::new("authorized-independent-meter-g1-parent").unwrap(),
            prepared.programme_id().clone(),
            AttemptId::new(0),
            MeterCallbackIndex::new(1),
        ),
        MeterKFilterExponentState::new(*meter_root.k_filter_exponents_i32_by_channel()),
        &independent_meter_g0_providers(),
    )
    .unwrap();
    assert_eq!(next.source_cursor_after().callback_index().value(), 2);
    assert_eq!(next.source_cursor_after().source_cursor().value(), 3_072);
    assert_eq!(next.source_cursor_after().read_index().value(), 2);
    assert_eq!(source_cursor.callback_index().value(), 0);
    assert_eq!(source_cursor.source_cursor().value(), 0);
    assert_eq!(source_cursor.read_index().value(), 0);
    let continued = next.application_candidate();
    assert_eq!(continued.component_rows().callback_index().value(), 1);
    assert_eq!(
        continued.reader_target_controls().callback_index().value(),
        1
    );
    assert_eq!(
        continued.primary_activity_after().next_callback().value(),
        2
    );
    assert!(continued.fresh_zero_controls().is_none());
    let continued_receipt = continued.meter_proposal().gtc_receipt().unwrap();
    assert_eq!(continued_receipt.before_generation(), 1);
    assert_eq!(continued_receipt.after_generation(), 2);
    assert_eq!(continued_receipt.commit_count(), 1);
    assert_eq!(meter_root.meter_callback_index().value(), 0);
    assert_eq!(meter_root.canonical_sha256().unwrap(), meter_root_before);
}

#[test]
#[ignore = "runs the selected full Meter prepass on the explicitly authorized ADM BWF input"]
fn authorized_adm_bwf_stages_complete_independent_meter_prepass_receipt() {
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
    let programme_id = ProgrammeId::new("authorized-independent-meter-prepass").unwrap();
    let prepared =
        prepare_selected_adm_bwf_input(programme_id, job_xml, Arc::clone(&resource), &parsed)
            .unwrap();
    let source_samples = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
    let source_identity = MeterPcmSourceIdentity::from_selected_resource_sha256(
        prepared.resource_identity().input_sha256(),
    );
    let source_cursor =
        MeterPcmSourceCursor::cold(&parsed, source_identity, source_samples).unwrap();
    let meter_root = selected_standalone_meter_root(&prepared, source_samples);
    let meter_root_before = meter_root.canonical_sha256().unwrap();
    let supplements = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../outputs/IMPLEMENTER_SUPPLEMENTS");
    let spatial_provider_root = supplements.join("ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1");
    let cold_state_root = supplements.join("F01B-F03W-COLD-STATE-ASSETS-V1");
    let current_features_root = supplements.join("METER-APPROVED-PARAMETER-ASSETS-V1");
    let callback_providers =
        jocforge::selected_spatial_callback_providers::load_selected_independent_meter_callback_providers(
            &spatial_provider_root,
            &cold_state_root,
            &current_features_root,
            jocforge_application::independent_meter_callback::
                generate_selected_spatial_math_geometry()
                .unwrap(),
        )
        .unwrap();
    let prepass = callback_providers
        .stage_prepass(&prepared, &source_cursor, &parsed, &meter_root)
        .unwrap();
    let measurement_dependents = prepass.prepare_measurement_dependents(&prepared).unwrap();
    assert_eq!(
        measurement_dependents.receipt(),
        prepass.terminal_measurement().receipt()
    );
    assert_eq!(
        *measurement_dependents
            .id1_configuration()
            .measurement_input_sha256(),
        prepared.domain_identity().resource().input_sha256()
    );
    assert!(
        measurement_dependents
            .drc_root()
            .state()
            .programme_dialnorm()
            .is_some()
    );
    let metadata_provider_bundle = jocforge_io::provider::load_metadata_owner_provider_bundle(
        supplements.join("METADATA-OWNER-ROOT-V1"),
    )
    .unwrap();
    let metadata_owner = prepare_metadata_owner_from_measurement(
        &prepared,
        &measurement_dependents,
        metadata_provider_bundle.provider_file(),
        metadata_provider_bundle.wrapper_file(),
    )
    .unwrap();
    assert_eq!(metadata_owner.state_reference().generation(), 0);
    let schedule = prepass.terminal_measurement().schedule();
    assert_eq!(schedule.normal_callbacks().value(), 4_247);
    assert_eq!(schedule.zero_drains().value(), 3);
    assert_eq!(prepass.meter_owner_receipts().len(), 4_250);
    assert_eq!(
        prepass
            .meter_owner_receipts()
            .first()
            .unwrap()
            .before_generation(),
        0
    );
    assert_eq!(
        prepass
            .meter_owner_receipts()
            .last()
            .unwrap()
            .after_generation(),
        4_250
    );
    assert_eq!(
        prepass
            .final_callback()
            .source_cursor_after()
            .callback_index()
            .value(),
        4_250
    );
    assert_eq!(
        prepass
            .final_callback()
            .source_cursor_after()
            .source_cursor()
            .value(),
        parsed.total_samples()
    );
    assert_eq!(
        prepass
            .final_callback()
            .source_cursor_after()
            .read_index()
            .value(),
        4_247
    );
    assert_eq!(
        prepass.terminal_measurement().receipt().normal_callbacks(),
        4_247
    );
    assert_eq!(prepass.terminal_measurement().receipt().zero_drains(), 3);
    assert_eq!(meter_root.meter_callback_index().value(), 0);
    assert_eq!(meter_root.canonical_sha256().unwrap(), meter_root_before);
}

// The callback probe keeps each separately supplied provider and input explicit.
#[allow(clippy::too_many_arguments)]
fn run_authorized_initial_meter_callback(
    prepared: &PreparedJobInput,
    resource: &Arc<[u8]>,
    parsed: &jocforge_io::pcm::riff_pcm24::ParsedRiffPcm24<'_>,
    frontend_candidate: FrontendControlReplacementCandidate,
    normal_output: NormalControlSnapshotOutput,
    parent_transaction_id: &CallbackTransactionId,
    providers: &IndependentMeterG0Providers,
    spatial_audio_providers: &SelectedSpatialAudioProviderBundle,
) {
    let spatial = stage_authorized_initial_spatial_callback(
        prepared,
        parsed,
        frontend_candidate,
        normal_output,
        parent_transaction_id,
        providers,
        spatial_audio_providers,
    );
    let g1_candidates = stage_authorized_main_g1_current_features_candidate(
        prepared,
        parsed,
        &spatial,
        providers,
        spatial_audio_providers,
    );
    stage_authorized_main_g2_receiver_candidate(
        prepared,
        resource,
        parsed,
        &g1_candidates,
        providers,
        spatial_audio_providers,
    );
    finish_authorized_initial_meter_callback(prepared, spatial);
}

fn stage_authorized_main_g1_current_features_candidate(
    prepared: &PreparedJobInput,
    parsed: &jocforge_io::pcm::riff_pcm24::ParsedRiffPcm24<'_>,
    g0: &AuthorizedInitialSpatialCandidate,
    providers: &IndependentMeterG0Providers,
    spatial_audio_providers: &SelectedSpatialAudioProviderBundle,
) -> AuthorizedMainSpatialContinuationCandidates {
    let g1_source = stage_meter_pcm_source_frame(
        g0.source_proposal.after(),
        MeterCallbackIndex::new(1),
        parsed,
    )
    .unwrap();
    let g1_callback = view_for_source_proposal(prepared, &g1_source).unwrap();
    assert_eq!(g1_callback.host_input_count().value(), 1_536);
    assert_eq!(g1_callback.source_padding_count().value(), 0);
    assert_eq!(
        g1_callback.source_identity().sha256(),
        prepared.resource_identity().input_sha256()
    );
    assert_eq!(g1_source.frame().source_cursor_before().value(), 1_536);
    assert_eq!(g1_source.frame().source_cursor_after().value(), 3_072);

    let frontend_after_g0 = g0.frontend_candidate.next_machine();
    let spatial_control_after_g0 = g0.spatial_control_candidate.next_machine();
    let spatial_audio_after_g0 = g0.spatial_audio_candidate.next_machine();
    let spatial_control_before_g1 = spatial_control_after_g0.digest();
    let spatial_audio_before_g1 = spatial_audio_after_g0.digest();

    let failed_shape = stage_main_spatial_callback(MainSpatialCallbackInput {
        prepared,
        controls: normal_controls_for_frontend(prepared, frontend_after_g0),
        physical_pcm_rows: &g1_callback.physical_rows()[..37],
        spatial_control: spatial_control_after_g0,
        spatial_audio: spatial_audio_after_g0,
        p11_geometry: &providers.p11_geometry,
        p15_geometry: &providers.p15_geometry,
        diffusion_parameters: &providers.diffusion,
        feature_processor: &providers.feature_processor,
        feature_fft: &providers.feature_fft,
        window: &providers.window,
        component_parameters: &providers.component_parameters,
        spatial_audio_providers: Some(spatial_audio_providers),
    });
    assert!(
        failed_shape.is_err(),
        "a short physical-row input must be rejected"
    );
    assert_eq!(spatial_control_after_g0.digest(), spatial_control_before_g1);
    assert_eq!(spatial_audio_after_g0.digest(), spatial_audio_before_g1);

    let (frontend_candidate_g1, g1_controls) =
        frontend_candidate_and_controls(prepared, frontend_after_g0, "authorized-main-f01e-g1");
    let g1_proposal = stage_main_spatial_callback(MainSpatialCallbackInput {
        prepared,
        controls: g1_controls,
        physical_pcm_rows: g1_callback.physical_rows(),
        spatial_control: spatial_control_after_g0,
        spatial_audio: spatial_audio_after_g0,
        p11_geometry: &providers.p11_geometry,
        p15_geometry: &providers.p15_geometry,
        diffusion_parameters: &providers.diffusion,
        feature_processor: &providers.feature_processor,
        feature_fft: &providers.feature_fft,
        window: &providers.window,
        component_parameters: &providers.component_parameters,
        spatial_audio_providers: Some(spatial_audio_providers),
    })
    .unwrap();
    assert_eq!(g1_proposal.callback_index().value(), 1);
    let spatial_control_after_g1 = g1_proposal.spatial_control_after_proposal().after_state();
    assert_eq!(spatial_control_after_g1.callback_index().value(), 2);
    assert_eq!(
        spatial_control_after_g1.source_cursor_samples().value(),
        3_072
    );
    assert_eq!(spatial_control_after_g1.source_vector_bits().len(), 48 * 24);
    assert_eq!(spatial_control_after_g1.source_scalar_bits().len(), 48);
    assert!(
        spatial_control_after_g1
            .source_vector_bits()
            .iter()
            .chain(spatial_control_after_g1.source_scalar_bits())
            .all(|word| f32::from_bits(*word).is_finite())
    );

    let g1_parent_transaction = CallbackTransactionId::new("authorized-main-spatial-g1").unwrap();
    let g1_parts = g1_proposal.into_parent_parts().into_parts().1;
    let spatial_control_candidate_g1 = spatial_control_after_g0
        .replacement_candidate(
            spatial_control_after_g0
                .stage(
                    spatial_control_after_g0
                        .prepare(g1_parts.spatial_control_after)
                        .unwrap(),
                )
                .unwrap(),
            g1_parent_transaction.clone(),
        )
        .unwrap();
    let spatial_audio_candidate_g1 = spatial_audio_after_g0
        .replacement_candidate(
            spatial_audio_after_g0
                .stage(
                    spatial_audio_after_g0
                        .prepare(g1_parts.spatial_audio_after)
                        .unwrap(),
                )
                .unwrap(),
            g1_parent_transaction,
        )
        .unwrap();
    assert_eq!(
        spatial_control_candidate_g1.receipt().before_generation(),
        1
    );
    assert_eq!(spatial_control_candidate_g1.receipt().after_generation(), 2);
    assert_eq!(
        spatial_control_candidate_g1
            .next_machine()
            .committed()
            .callback_index()
            .value(),
        2
    );
    assert_eq!(spatial_audio_candidate_g1.receipt().before_generation(), 1);
    assert_eq!(spatial_audio_candidate_g1.receipt().after_generation(), 2);

    let spatial_control_after_g1 = spatial_control_candidate_g1.next_machine();
    let spatial_audio_after_g1 = spatial_audio_candidate_g1.next_machine();
    let spatial_control_before_repeated = spatial_control_after_g1.digest();
    let spatial_audio_before_repeated = spatial_audio_after_g1.digest();
    let repeated_g1 = stage_main_spatial_callback(MainSpatialCallbackInput {
        prepared,
        controls: normal_controls_for_frontend(prepared, frontend_after_g0),
        physical_pcm_rows: g1_callback.physical_rows(),
        spatial_control: spatial_control_after_g1,
        spatial_audio: spatial_audio_after_g1,
        p11_geometry: &providers.p11_geometry,
        p15_geometry: &providers.p15_geometry,
        diffusion_parameters: &providers.diffusion,
        feature_processor: &providers.feature_processor,
        feature_fft: &providers.feature_fft,
        window: &providers.window,
        component_parameters: &providers.component_parameters,
        spatial_audio_providers: Some(spatial_audio_providers),
    });
    assert!(
        repeated_g1.is_err(),
        "a repeated callback index must be rejected"
    );
    assert_eq!(
        spatial_control_after_g1.digest(),
        spatial_control_before_repeated
    );
    assert_eq!(
        spatial_audio_after_g1.digest(),
        spatial_audio_before_repeated
    );

    AuthorizedMainSpatialContinuationCandidates {
        source_proposal: g1_source,
        frontend_candidate: frontend_candidate_g1,
        spatial_control_candidate: spatial_control_candidate_g1,
        spatial_audio_candidate: spatial_audio_candidate_g1,
    }
}

fn normal_controls_for_frontend(
    prepared: &PreparedJobInput,
    frontend: &FrontendControlMachine,
) -> NormalControlSnapshotOutput {
    let transaction_id = format!(
        "authorized-main-f01e-replay-{}",
        frontend.committed().callback_index().value()
    );
    frontend_candidate_and_controls(prepared, frontend, &transaction_id).1
}

fn frontend_candidate_and_controls(
    prepared: &PreparedJobInput,
    frontend: &FrontendControlMachine,
    transaction_id: &str,
) -> (
    FrontendControlReplacementCandidate,
    NormalControlSnapshotOutput,
) {
    let snapshot: Arc<NormalControlPreparedSnapshot> = prepared
        .normal_control_prepared_snapshot(frontend.committed())
        .unwrap();
    let before =
        NormalControlBeforeState::from_frontend_state(frontend.committed(), frontend.generation())
            .unwrap();
    let lineage = CallbackLineageIdentity::new(
        prepared.programme_id().clone(),
        AttemptId::new(0),
        frontend.committed().callback_index(),
    );
    let proposal = propose_normal_control_snapshot(&before, &snapshot, lineage).unwrap();
    let (staged, controls) = frontend
        .stage_normal_snapshot(proposal)
        .unwrap()
        .into_parts();
    let candidate = frontend
        .replacement_candidate(staged, CallbackTransactionId::new(transaction_id).unwrap())
        .unwrap();
    (candidate, controls)
}

fn cold_analysis_root_for_prepared_input(prepared: &PreparedJobInput) -> Box<AnalysisOwnerRoot> {
    let identity = prepared.domain_identity_capability();
    std::thread::Builder::new()
        .name("jocforge-authorized-analysis-cold-root".to_owned())
        .stack_size(SOURCE_ANALYSIS_TEST_STACK_BYTES)
        .spawn(move || Box::new(AnalysisOwnerRoot::cold_for_prepared_input(identity)))
        .unwrap()
        .join()
        .unwrap()
}

fn stage_authorized_main_g2_receiver_candidate(
    prepared: &PreparedJobInput,
    resource: &Arc<[u8]>,
    parsed: &jocforge_io::pcm::riff_pcm24::ParsedRiffPcm24<'_>,
    g1: &AuthorizedMainSpatialContinuationCandidates,
    providers: &IndependentMeterG0Providers,
    spatial_audio_providers: &SelectedSpatialAudioProviderBundle,
) {
    let g2_source = stage_meter_pcm_source_frame(
        g1.source_proposal.after(),
        MeterCallbackIndex::new(2),
        parsed,
    )
    .unwrap();
    let g2_callback = view_for_source_proposal(prepared, &g2_source).unwrap();
    assert_eq!(g2_callback.host_input_count().value(), 1_536);
    assert_eq!(g2_callback.source_padding_count().value(), 0);
    assert_eq!(g2_source.frame().source_cursor_before().value(), 3_072);
    assert_eq!(g2_source.frame().source_cursor_after().value(), 4_608);

    let frontend_after_g1 = g1.frontend_candidate.next_machine();
    let (frontend_candidate_g2, g2_controls) =
        frontend_candidate_and_controls(prepared, frontend_after_g1, "authorized-main-f01e-g2");
    let spatial_control_after_g1 = g1.spatial_control_candidate.next_machine();
    let spatial_audio_after_g1 = g1.spatial_audio_candidate.next_machine();
    let spatial_control_before_g2 = spatial_control_after_g1.digest();
    let spatial_audio_before_g2 = spatial_audio_after_g1.digest();

    let g2_proposal = stage_main_spatial_callback(MainSpatialCallbackInput {
        prepared,
        controls: g2_controls,
        physical_pcm_rows: g2_callback.physical_rows(),
        spatial_control: spatial_control_after_g1,
        spatial_audio: spatial_audio_after_g1,
        p11_geometry: &providers.p11_geometry,
        p15_geometry: &providers.p15_geometry,
        diffusion_parameters: &providers.diffusion,
        feature_processor: &providers.feature_processor,
        feature_fft: &providers.feature_fft,
        window: &providers.window,
        component_parameters: &providers.component_parameters,
        spatial_audio_providers: Some(spatial_audio_providers),
    })
    .unwrap();
    assert_eq!(g2_proposal.callback_index().value(), 2);
    assert!(matches!(
        g2_proposal.spatial_audio_outcome(),
        SpatialAudioCallbackOutcome::Joc(_)
    ));

    let analysis_before = cold_analysis_root_for_prepared_input(prepared);
    let transient = load_transient_parameters(Some(F07_TRANSIENT_PARAMETERS)).unwrap();
    let transform =
        MathGeneratedTransformParameters::from_provider_bytes(F08_TRANSFORM_PARAMETERS).unwrap();
    let policy =
        load_policy_parameters(PolicyProfile::selected(), Some(F09_POLICY_PARAMETERS)).unwrap();
    let source_transaction_id = CallbackTransactionId::new("authorized-main-source0").unwrap();
    let analysis_stage =
        prepare_source_analysis_from_spatial_callback(SpatialSourceAnalysisBridgeInput {
            prepared,
            analysis_before: &analysis_before,
            spatial_callback: &g2_proposal,
            attempt_id: AttemptId::new(0),
            source_transaction_id: source_transaction_id.clone(),
            previous_short: [false; 5],
            providers: SourceAnalysisProviders {
                transient: &transient,
                transform: &transform,
                policy: &policy,
            },
        })
        .unwrap();
    let joc_before = SelectedJocState::cold();
    let fit_score =
        FitScoreProvider::from_approved_bytes(JOC_FUNCTIONAL_PARAMETERS, JOC_MATH_PARAMETERS)
            .unwrap();
    let representation_retry =
        RepresentationRetryProvider::from_approved_bytes(JOC_FUNCTIONAL_PARAMETERS).unwrap();
    let metadata_bodies =
        prepare_main_metadata_bodies_from_spatial_callback(MainMetadataBodiesInput {
            prepared,
            spatial_callback: &g2_proposal,
            joc_before: &joc_before,
            source_transaction_id: source_transaction_id.clone(),
            qmf_functional: spatial_audio_providers.qmf_receiver_functional(),
            fit_score: &fit_score,
            retry: &representation_retry,
        })
        .unwrap();
    assert_eq!(metadata_bodies.joc_receipt().owner_id(), GtcOwnerId::Joc);
    assert_eq!(metadata_bodies.joc_receipt().before_generation(), 0);
    assert_eq!(metadata_bodies.joc_receipt().after_generation(), 1);
    assert_eq!(
        metadata_bodies.joc_receipt().transaction_id(),
        &source_transaction_id
    );
    assert_eq!(metadata_bodies.joc_after().after().generation(), 1);
    assert_eq!(
        metadata_bodies
            .metadata_bodies()
            .lineage()
            .source_index()
            .value(),
        0
    );
    assert_eq!(
        metadata_bodies
            .metadata_bodies()
            .lineage()
            .dispatch()
            .value(),
        1
    );
    assert_eq!(
        metadata_bodies.metadata_bodies().lineage().transaction_id(),
        g2_proposal.transaction_id()
    );
    let id14_provenance = metadata_bodies
        .metadata_bodies()
        .id14_provenance()
        .expect("the F05JOC source proposal carries ID14 provenance");
    assert_eq!(id14_provenance.producer_generation().value(), 0);
    assert_eq!(id14_provenance.dispatch().value(), 1);
    assert_eq!(id14_provenance.writer().value(), 0);
    assert_eq!(id14_provenance.object_count().value(), 15);
    assert_eq!(
        id14_provenance.producer_transaction_id(),
        Some(&source_transaction_id)
    );
    assert_eq!(joc_before.generation(), 0);
    assert_eq!(
        joc_before.state_sha256().unwrap(),
        metadata_bodies.joc_receipt().before_sha256()
    );

    let job_vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let original_job_xml = job_vectors["config_cases"][0]["input_xml"]
        .as_str()
        .unwrap();
    let alternate_job_xml = original_job_xml.replace(
        "<file_name>out.ec3</file_name>",
        "<file_name>different-output.ec3</file_name>",
    );
    let alternate_prepared = prepare_selected_adm_bwf_input(
        prepared.programme_id().clone(),
        alternate_job_xml.as_bytes(),
        Arc::clone(resource),
        parsed,
    )
    .unwrap();
    assert_ne!(alternate_prepared.config_sha256(), prepared.config_sha256());
    let alternate_analysis_before = cold_analysis_root_for_prepared_input(&alternate_prepared);
    let mismatched_spatial_input =
        prepare_source_analysis_from_spatial_callback(SpatialSourceAnalysisBridgeInput {
            prepared: &alternate_prepared,
            analysis_before: &alternate_analysis_before,
            spatial_callback: &g2_proposal,
            attempt_id: AttemptId::new(0),
            source_transaction_id: CallbackTransactionId::new(
                "authorized-main-source0-mismatched-job",
            )
            .unwrap(),
            previous_short: [false; 5],
            providers: SourceAnalysisProviders {
                transient: &transient,
                transform: &transform,
                policy: &policy,
            },
        });
    assert!(matches!(
        mismatched_spatial_input,
        Err(SourceAnalysisBridgeError::SpatialCallbackIdentityMismatch)
    ));
    assert_eq!(alternate_analysis_before.input_history_generation(), 0);

    let shared_analysis = analysis_stage.shared_analysis().unwrap();
    assert_eq!(
        shared_analysis
            .identity()
            .source_stage_key()
            .transaction_id(),
        &source_transaction_id
    );
    assert_eq!(shared_analysis.identity().source_index().value(), 0);
    assert_eq!(
        shared_analysis
            .identity()
            .source_index()
            .dispatch()
            .unwrap()
            .value(),
        1
    );
    assert_eq!(
        shared_analysis
            .identity()
            .analysis_after_generation()
            .value(),
        1
    );
    assert_eq!(shared_analysis.advancement_count(), 1);
    assert!(analysis_stage.plan().is_some());
    assert_eq!(analysis_before.input_history_generation(), 0);
    assert_eq!(analysis_before.transient_generation(), 0);
    assert_eq!(analysis_before.transform_generation(), 0);
    assert_eq!(analysis_stage.after_state().input_history_generation(), 1);
    assert_eq!(analysis_stage.after_state().transient_generation(), 1);
    assert_eq!(analysis_stage.after_state().transform_generation(), 1);

    assert_eq!(
        g2_proposal
            .spatial_control_after_proposal()
            .after_state()
            .callback_index()
            .value(),
        3
    );
    assert_eq!(
        g2_proposal
            .spatial_control_after_proposal()
            .after_state()
            .source_cursor_samples()
            .value(),
        4_608
    );
    assert_eq!(
        g2_proposal
            .spatial_audio_after_proposal()
            .after_state()
            .callback_index()
            .value(),
        3
    );

    let g2_parent_transaction = CallbackTransactionId::new("authorized-main-spatial-g2").unwrap();
    let g2_parts = g2_proposal.into_parent_parts().into_parts().1;
    let spatial_control_candidate_g2 = spatial_control_after_g1
        .replacement_candidate(
            spatial_control_after_g1
                .stage(
                    spatial_control_after_g1
                        .prepare(g2_parts.spatial_control_after)
                        .unwrap(),
                )
                .unwrap(),
            g2_parent_transaction.clone(),
        )
        .unwrap();
    let spatial_audio_candidate_g2 = spatial_audio_after_g1
        .replacement_candidate(
            spatial_audio_after_g1
                .stage(
                    spatial_audio_after_g1
                        .prepare(g2_parts.spatial_audio_after)
                        .unwrap(),
                )
                .unwrap(),
            g2_parent_transaction,
        )
        .unwrap();
    assert_eq!(frontend_candidate_g2.receipt().before_generation(), 2);
    assert_eq!(frontend_candidate_g2.receipt().after_generation(), 3);
    assert_eq!(
        spatial_control_candidate_g2.receipt().before_generation(),
        2
    );
    assert_eq!(spatial_control_candidate_g2.receipt().after_generation(), 3);
    assert_eq!(spatial_audio_candidate_g2.receipt().before_generation(), 2);
    assert_eq!(spatial_audio_candidate_g2.receipt().after_generation(), 3);
    assert_eq!(spatial_control_after_g1.digest(), spatial_control_before_g2);
    assert_eq!(spatial_audio_after_g1.digest(), spatial_audio_before_g2);
}

fn stage_authorized_initial_spatial_callback(
    prepared: &PreparedJobInput,
    parsed: &jocforge_io::pcm::riff_pcm24::ParsedRiffPcm24<'_>,
    frontend_candidate: FrontendControlReplacementCandidate,
    normal_output: NormalControlSnapshotOutput,
    parent_transaction_id: &CallbackTransactionId,
    providers: &IndependentMeterG0Providers,
    spatial_audio_providers: &SelectedSpatialAudioProviderBundle,
) -> AuthorizedInitialSpatialCandidate {
    let source_identity = MeterPcmSourceIdentity::from_selected_resource_sha256(
        prepared.resource_identity().input_sha256(),
    );
    let source_samples = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
    let cursor = MeterPcmSourceCursor::cold(parsed, source_identity, source_samples).unwrap();
    let pcm_proposal =
        stage_meter_pcm_source_frame(&cursor, MeterCallbackIndex::new(0), parsed).unwrap();
    let callback = view_for_source_proposal(prepared, &pcm_proposal).unwrap();
    assert_eq!(callback.physical_rows().len(), 38);
    assert_eq!(callback.physical_rows()[0].len(), 1_536);
    assert_eq!(callback.host_input_count().value(), 1_536);
    assert_eq!(callback.source_padding_count().value(), 0);
    assert_eq!(
        callback.source_identity().sha256(),
        prepared.resource_identity().input_sha256()
    );

    let reader_eof = ReaderEofMachine::cold();
    let reader_eof_before = reader_eof.state_reference();
    let reader_host38 = Host38Frame::new(
        callback
            .physical_rows()
            .iter()
            .flat_map(|row| row.iter().copied().map(f32::from_bits))
            .collect(),
    )
    .unwrap();
    let reader_after = ReaderEofState::from_parts(
        FrontendCallbackIndex::new(1),
        1_536,
        NormalCallbackCount::new(1),
        4,
        Some(reader_host38),
        Some(Control38Snapshot::new(normal_output.control38_carrier_words().to_vec()).unwrap()),
        ReaderEofPhase::Normal,
    )
    .unwrap();
    let reader_after_bytes = reader_after.canonical_bytes();
    let reader_staged = reader_eof
        .stage(reader_eof.prepare("reader-g0", reader_after).unwrap())
        .unwrap();
    let reader_candidate = reader_eof
        .replacement_candidate(reader_staged, parent_transaction_id.clone())
        .unwrap();
    assert_eq!(reader_candidate.next_machine().generation(), 1);
    assert_eq!(
        reader_candidate
            .next_machine()
            .committed()
            .canonical_bytes(),
        reader_after_bytes
    );
    assert_eq!(
        reader_candidate.gtc_receipt().owner_id(),
        GtcOwnerId::ReaderEof
    );
    assert_eq!(reader_candidate.gtc_receipt().after_generation(), 1);
    assert_eq!(reader_candidate.gtc_receipt().commit_count(), 1);
    assert_eq!(reader_eof.generation(), 0);
    assert_eq!(
        reader_eof.state_reference().canonical_bytes(),
        reader_eof_before.canonical_bytes()
    );

    let owners = spatial_callback_owners(providers);
    let spatial_control_before = owners.spatial_control.digest();
    let spatial_audio_before = owners.spatial_audio.digest();
    let reader_target_before = owners
        .spatial_control
        .committed()
        .reader_target_state()
        .canonical_sha256();
    let spatial_stage = stage_main_spatial_callback(MainSpatialCallbackInput {
        prepared,
        controls: normal_output,
        physical_pcm_rows: callback.physical_rows(),
        spatial_control: &owners.spatial_control,
        spatial_audio: &owners.spatial_audio,
        p11_geometry: &providers.p11_geometry,
        p15_geometry: &providers.p15_geometry,
        diffusion_parameters: &providers.diffusion,
        feature_processor: owners.processor,
        feature_fft: owners.fft,
        window: owners.window,
        component_parameters: owners.parameters,
        spatial_audio_providers: Some(spatial_audio_providers),
    })
    .unwrap();
    assert_eq!(spatial_stage.callback_index().value(), 0);
    assert_eq!(spatial_stage.component_rows().callback_index().value(), 0);
    assert_eq!(spatial_stage.canonical_dense_d_f32_bits().len(), 15 * 48);
    let spatial_after = spatial_stage.spatial_control_after_proposal().after_state();
    let audio_after = spatial_stage.spatial_audio_after_proposal().after_state();
    assert_eq!(spatial_after.callback_index().value(), 1);
    assert_eq!(spatial_after.source_cursor_samples().value(), 1_536);
    assert_eq!(spatial_after.normal_callbacks().value(), 1);
    assert!(!spatial_after.source_cache_is_cold());
    assert_eq!(spatial_after.regular_raw_history_bits().len(), 37 * 1_536);
    assert_eq!(spatial_after.previous_pcm_bits().len(), 38 * 1_536);
    assert_eq!(spatial_after.previous_current_rows_bits().len(), 48 * 1_536);
    assert_eq!(spatial_after.matrix_f32_bits().len(), 15 * 48);
    assert_eq!(audio_after.callback_index().value(), 1);
    assert!(!audio_after.canonical_bytes().is_empty());
    assert_ne!(audio_after.digest(), spatial_audio_before);
    assert_eq!(
        spatial_stage
            .spatial_audio_after_proposal()
            .after_state()
            .callback_index()
            .value(),
        1
    );
    assert_eq!(
        spatial_stage
            .spatial_control_after_proposal()
            .join_metadata()
            .after_digest(),
        spatial_after.digest()
    );
    assert!(
        spatial_after
            .previous_marker_active_u8()
            .unwrap()
            .iter()
            .all(|active| *active <= 1)
    );
    assert_eq!(owners.spatial_control.digest(), spatial_control_before);
    assert_eq!(owners.spatial_audio.digest(), spatial_audio_before);
    assert_eq!(
        owners
            .spatial_control
            .committed()
            .reader_target_state()
            .canonical_sha256(),
        reader_target_before
    );

    let spatial_parts = spatial_stage.into_parent_parts().into_parts().1;
    assert!(matches!(
        spatial_parts.spatial_audio_outcome,
        jocforge_domain::spatial::SpatialAudioCallbackOutcome::Priming
    ));
    let spatial_control_candidate = owners
        .spatial_control
        .replacement_candidate(
            owners
                .spatial_control
                .stage(
                    owners
                        .spatial_control
                        .prepare(spatial_parts.spatial_control_after)
                        .unwrap(),
                )
                .unwrap(),
            parent_transaction_id.clone(),
        )
        .unwrap();
    let spatial_audio_candidate = owners
        .spatial_audio
        .replacement_candidate(
            owners
                .spatial_audio
                .stage(
                    owners
                        .spatial_audio
                        .prepare(spatial_parts.spatial_audio_after)
                        .unwrap(),
                )
                .unwrap(),
            parent_transaction_id.clone(),
        )
        .unwrap();
    assert_eq!(
        spatial_control_candidate.receipt().owner_id(),
        GtcOwnerId::SpatialControl
    );
    assert_eq!(spatial_control_candidate.receipt().before_generation(), 0);
    assert_eq!(spatial_control_candidate.receipt().after_generation(), 1);
    assert_eq!(spatial_control_candidate.receipt().commit_count(), 1);
    assert_eq!(
        spatial_audio_candidate.receipt().owner_id(),
        GtcOwnerId::SpatialAudio
    );
    assert_eq!(spatial_audio_candidate.receipt().before_generation(), 0);
    assert_eq!(spatial_audio_candidate.receipt().after_generation(), 1);
    assert_eq!(spatial_audio_candidate.receipt().commit_count(), 1);
    assert_eq!(owners.spatial_control.generation(), 0);
    assert_eq!(owners.spatial_control.digest(), spatial_control_before);
    assert_eq!(owners.spatial_audio.generation(), 0);
    assert_eq!(owners.spatial_audio.digest(), spatial_audio_before);

    AuthorizedInitialSpatialCandidate {
        cursor,
        source_proposal: pcm_proposal,
        frontend_candidate,
        spatial_control_candidate,
        spatial_audio_candidate,
        component_rows: spatial_parts.component_rows,
        spatial_audio_outcome: spatial_parts.spatial_audio_outcome,
    }
}

fn finish_authorized_initial_meter_callback(
    prepared: &PreparedJobInput,
    spatial: AuthorizedInitialSpatialCandidate,
) {
    let cursor_before = spatial.cursor.callback_index();
    let source_samples = spatial.cursor.source_samples();
    let meter_root = selected_standalone_meter_root(prepared, source_samples);
    let source_frame = spatial.source_proposal.frame();
    let meter_progress = MeterRootCallbackProgress::new(
        source_frame.callback_index(),
        MeterSourcePosition::new(source_frame.source_cursor_before().value()),
        MeterSourcePosition::new(source_frame.source_cursor_after().value()),
        source_frame.source_read_index_before(),
        source_frame.source_read_index_after(),
        source_frame.host_input_count(),
        MeterKFilterExponentState::new(*meter_root.k_filter_exponents_i32_by_channel()),
    );
    let callback_candidate = stage_main_output_oar_meter_probe(MainOutputOarMeterProbeInput {
        prepared,
        source_cursor_before: &spatial.cursor,
        source_proposal: spatial.source_proposal,
        frontend_candidate: spatial.frontend_candidate,
        spatial_control_candidate: spatial.spatial_control_candidate,
        spatial_audio_candidate: spatial.spatial_audio_candidate,
        component_rows: spatial.component_rows,
        spatial_audio_outcome: spatial.spatial_audio_outcome,
        meter_before: &meter_root,
        meter_progress,
    })
    .unwrap();
    assert_eq!(
        callback_candidate.source_receipt().callback_index().value(),
        0
    );
    assert_eq!(
        callback_candidate
            .source_receipt()
            .upstream_generation()
            .value(),
        1
    );
    assert_eq!(
        callback_candidate.source_receipt().phase(),
        MeterCallbackPhase::NormalSource
    );
    assert_eq!(
        callback_candidate.meter_receipt().callback_index().value(),
        0
    );
    assert_eq!(
        callback_candidate
            .meter_receipt()
            .next_callback_index()
            .value(),
        1
    );
    assert_eq!(
        callback_candidate
            .meter_after()
            .meter_callback_index()
            .value(),
        1
    );
    assert_eq!(meter_root.meter_callback_index().value(), 0);
    assert_eq!(spatial.cursor.callback_index(), cursor_before);
    assert_eq!(spatial.cursor.callback_index().value(), 0);
    eprintln!(
        "authorized initial Meter callback staged without parent commit: {} samples",
        source_samples.value()
    );
}
