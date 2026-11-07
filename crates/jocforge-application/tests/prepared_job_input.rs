// pattern: Functional Core
use jocforge_application::{
    EncodeError, PreparedJobInputError, SelectedAdmResourcePayloads,
    independent_meter_callback::{
        IndependentMeterG0Providers, MeterCallbackGtcBinding,
        stage_independent_meter_fresh_zero_drain_callback, stage_independent_meter_g0_callback,
        stage_independent_meter_normal_callback,
    },
    meter_source_admission::{MeterSourceAdmissionError, admit_meter_source_owner},
    meter_source_bridge::{
        KMatrixMode, KMatrixProfile, MeterCallbackCount, MeterCallbackIndex, MeterCallbackPhase,
        MeterKFilterExponentState, MeterProviderIdentity, MeterProviderSet, MeterSampleCount,
        MeterSourceCallbackFrame, MeterSourceCallbackFrameParts, MeterSourceCallbackView,
        MeterSourceIdentity, MeterSourcePosition, MeterSourceReadIndex, MeterSourceSampleCount,
        MeterUpstreamGeneration, SharedGainDelayParameters, StandaloneMeterRoot, generate_k_matrix,
    },
    output_publication::{OutputStagingOwnerId, admit_selected_f15_plan},
    prepare_selected_job_input,
    reader_eof_callback::{
        ReaderEofActualSampleCount, ReaderEofMachine, ReaderEofNormalCallbackClock,
        ReaderEofPaddingSampleCount, stage_reader_eof_adm_input_normal_callback,
        stage_reader_eof_cached_drain,
    },
    spatial_callback::{
        MainHeldControlOutput, MainNormalControlOutput, MainSpatialCachedCallbackInput,
        MainSpatialCallbackInput, MainSpatialColdRoots, stage_main_frontend_adm_input_snapshot,
        stage_main_spatial_cached_callback, stage_main_spatial_callback,
    },
};
use jocforge_domain::frontend::adm_file_storage_map::AdmFileStorageTrackSlot;
use jocforge_domain::frontend::dbmd_sections::parse_selected_dbmd_sections;
use jocforge_domain::frontend::selected_adm_graph::SelectedTrackKind;
use jocforge_domain::frontend::{
    FrontendControlMachine, PackageManifestEvidence, ProviderFileEvidence,
    job_configuration::SelectedWaveDescriptor,
    normal_control_snapshot::{
        NormalControlAdmInputOutput, NormalControlBeforeState, NormalControlRow,
        NormalControlSnapshotError, propose_normal_control_snapshot,
    },
    validate_current_features_package_manifest, validate_f01e_package_manifest,
    validate_f01e_provider, validate_f03_package_manifest, validate_f03_target_cold_provider,
    validate_feature_fft4096_math_provider, validate_feature_processor_math_provider,
};
use jocforge_domain::profile::{CodecRateProfile, EncodingProfile, resolve_selected_profile};
use jocforge_domain::source_axis::{SelectedSourceRowDimensions, SelectedSourceRowShapeError};
use jocforge_domain::spatial::{
    diffusion_filter::{
        DiffusionFilterParameters, validate_diffusion_filter_package_manifest,
        validate_diffusion_filter_parameters,
    },
    installed48_activity::{PrimaryActivityInput, PrimaryInstalled48ActivityProducer},
    installed48_source_map::{
        build_meter_installed48_source_map, build_selected_installed48_source_map, field_masks_hex,
    },
    point_response::{
        PointResponseGateInput, prepare_point_response_sources, produce_point_response_columns,
    },
    prefix_membership::PrefixMembershipProducer,
    production_callback::selected_audio_provider_bundle::{
        SelectedSpatialAudioProviderBundle, SelectedSpatialAudioProviderEvidence,
    },
    selected_p11_geometry::generate_selected_p11_geometry,
    selected_p15_geometry::generate_selected_p15_geometry,
    source_flags::{
        SourceControlSnapshotClock, project_normal_control_adm_input_output_source_controls,
    },
    source_geometry_outer::{
        build_global_source_geometry_snapshot_from_adm_input_output, produce_compact_outer_matrix,
    },
    spatial_audio_state::SpatialAudioMachine,
    spatial_control::{
        SpatialControlInstalled48Callback, SpatialControlInstalled48Input, SpatialControlMachine,
        stage_spatial_control_installed48,
    },
    target_context::build_target_context_projection,
    target_slot_install::{SelectedTargetSlots, TargetSlotState, install_target_slots},
    validate_component_parameter_provider, validate_f04_parameter_package_manifest,
    validate_window_q15_provider,
};
use jocforge_domain::timeline::{
    AdmSampleIndex, CodecSourceIndex, FrontendCallbackIndex, NormalCallbackCount,
};
use jocforge_domain::transaction::{
    AttemptId, CallbackLineageIdentity, CallbackTransactionId, ProgrammeId,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{ops::Range, sync::Arc};

const AXML: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.xml");
const CHNA: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_track_associations.bin"
);
const DBMD: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.dbmd");
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
const QMF_RECEIVER_FUNCTIONAL: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json"
);
const QMF_MATHEMATICAL_FFT: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json"
);
const DESCRIPTOR_MATHEMATICAL: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/f04_parameters/descriptor-mathematical.json"
);
const METER_ROOT_V4_COLD: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-ROOT-CANONICAL-STATE-REFERENCE-V4/input-only-cold-roots/N6522906_pre_callback0_cold_root.json"
);
const DBMD_PREFIX_BYTES: usize = 4;
const DBMD_SECTION_HEADER_BYTES: usize = 3;
const DBMD_SECTION_10_ID: u8 = 10;
const DBMD_SECTION_10_FIXED_BYTES: usize = 142;

fn test_programme_id() -> ProgrammeId {
    ProgrammeId::new("JOCForgeTestID01").unwrap()
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

fn sha256_from_hex(value: &str) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
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

fn independent_meter_g0_providers() -> IndependentMeterG0Providers {
    let manifest = PackageManifestEvidence::new(Arc::from(F01E_MANIFEST), None);
    let f03_manifest = validate_f03_package_manifest(&manifest).unwrap();
    let current_features_manifest = validate_current_features_package_manifest(&manifest).unwrap();
    let f04_manifest = validate_f04_parameter_package_manifest(&manifest).unwrap();
    IndependentMeterG0Providers {
        f01e: validated_f01e_provider(),
        f03_target: validate_f03_target_cold_provider(
            &ProviderFileEvidence::new(Arc::from(F03_PROVIDER), None),
            &f03_manifest,
        )
        .unwrap(),
        feature_processor: validate_feature_processor_math_provider(
            &ProviderFileEvidence::new(Arc::from(FEATURE_PROCESSOR), None),
            &current_features_manifest,
        )
        .unwrap(),
        feature_fft: validate_feature_fft4096_math_provider(
            &ProviderFileEvidence::new(Arc::from(FEATURE_FFT), None),
            &current_features_manifest,
        )
        .unwrap(),
        window: validate_window_q15_provider(
            &ProviderFileEvidence::new(Arc::from(WINDOW_Q15), None),
            &f04_manifest,
        )
        .unwrap(),
        component_parameters: validate_component_parameter_provider(
            &ProviderFileEvidence::new(Arc::from(COMPONENT_PARAMETERS), None),
            &f04_manifest,
            &EncodingProfile::selected(),
        )
        .unwrap(),
        diffusion: selected_diffusion_parameters(),
        p11_geometry: generate_selected_p11_geometry(0x111b, true).unwrap(),
        p15_geometry: generate_selected_p15_geometry(0x159b, true).unwrap(),
    }
}

fn selected_spatial_audio_provider_bundle() -> SelectedSpatialAudioProviderBundle {
    let manifest = PackageManifestEvidence::new(Arc::from(F01E_MANIFEST), None);
    let component = ProviderFileEvidence::new(Arc::from(COMPONENT_PARAMETERS), None);
    let window = ProviderFileEvidence::new(Arc::from(WINDOW_Q15), None);
    let qmf_receiver = ProviderFileEvidence::new(Arc::from(QMF_RECEIVER_FUNCTIONAL), None);
    let qmf_fft = ProviderFileEvidence::new(Arc::from(QMF_MATHEMATICAL_FFT), None);
    let descriptor = ProviderFileEvidence::new(Arc::from(DESCRIPTOR_MATHEMATICAL), None);
    SelectedSpatialAudioProviderBundle::validate_selected(
        &SelectedSpatialAudioProviderEvidence::new(
            &manifest,
            &component,
            &window,
            &qmf_receiver,
            &qmf_fft,
            &descriptor,
        ),
    )
    .unwrap()
}

fn selected_meter_root(
    prepared: &jocforge_application::PreparedJobInput,
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

#[test]
fn bound_prepared_input_rejects_unselected_extent_before_f01e_snapshot() {
    let (resource, spans) = selected_resource();
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = &vectors["config_cases"][0];
    let descriptor = &case["pcm_descriptor"];
    let prepared = prepare_selected_job_input(
        test_programme_id(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(
            descriptor["sample_rate"].as_u64().unwrap() as u32,
            descriptor["channels"].as_u64().unwrap() as u16,
            descriptor["pcm_bits"].as_u64().unwrap() as u16,
        ),
        1,
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap()
    .bind_primary_control_rows(114)
    .unwrap();
    let unbound_prepared = prepare_selected_job_input(
        test_programme_id(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(
            descriptor["sample_rate"].as_u64().unwrap() as u32,
            descriptor["channels"].as_u64().unwrap() as u16,
            descriptor["pcm_bits"].as_u64().unwrap() as u16,
        ),
        1,
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap();
    let machine = FrontendControlMachine::from_provider(
        validated_f01e_provider(),
        test_programme_id(),
        AttemptId::new(0),
    )
    .unwrap();

    assert_eq!(
        unbound_prepared
            .normal_control_prepared_snapshot(machine.committed())
            .unwrap_err(),
        PreparedJobInputError::PrimaryControlBindingUnavailable
    );

    assert_eq!(
        prepared
            .normal_control_prepared_snapshot(machine.committed())
            .unwrap_err(),
        PreparedJobInputError::NormalControlSnapshot(
            NormalControlSnapshotError::SourceConfigurationMismatch
        )
    );

    let other_programme = FrontendControlMachine::from_provider(
        validated_f01e_provider(),
        ProgrammeId::new("other-programme").unwrap(),
        AttemptId::new(0),
    )
    .unwrap();
    assert_eq!(
        prepared
            .normal_control_prepared_snapshot(other_programme.committed())
            .unwrap_err(),
        PreparedJobInputError::ProgrammeIdentityMismatch
    );
}

#[test]
fn value_preparation_binds_graph_and_identity_to_one_immutable_resource() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    // This synthetic container exercises identity and chunk binding only; it is
    // not evidence of selected-profile source admission or audio processing.
    let (resource, spans) = selected_resource();
    let expected_digest: [u8; 32] = Sha256::digest(resource.as_ref()).into();
    let mut accepted = 0;
    let mut rejected = 0;
    for case in vectors["config_cases"].as_array().unwrap() {
        let descriptor = &case["pcm_descriptor"];
        let wave = SelectedWaveDescriptor::new(
            descriptor["sample_rate"].as_u64().unwrap() as u32,
            descriptor["channels"].as_u64().unwrap() as u16,
            descriptor["pcm_bits"].as_u64().unwrap() as u16,
        );
        let result = prepare_selected_job_input(
            test_programme_id(),
            case["input_xml"].as_str().unwrap().as_bytes(),
            wave,
            descriptor["source_samples"].as_u64().unwrap(),
            Arc::clone(&resource),
            metadata_payloads(&resource, &spans),
        );
        if case["expected"]["status"] == "ACCEPT" {
            let prepared = result.unwrap();
            assert_eq!(prepared.resource_identity().input_sha256(), expected_digest);
            assert_eq!(
                prepared.resource_identity().resource_bytes(),
                resource.len() as u64
            );
            let domain_identity = prepared.domain_identity();
            assert_eq!(
                domain_identity.resource().input_sha256(),
                prepared.resource_identity().input_sha256()
            );
            assert_eq!(
                domain_identity.resource().resource_bytes(),
                prepared.resource_identity().resource_bytes()
            );
            assert_eq!(
                domain_identity.normalized_configuration().config_sha256(),
                prepared.config_sha256()
            );
            assert_eq!(
                domain_identity.normalized_configuration().job_xml_sha256(),
                prepared.job_xml_sha256()
            );
            assert_eq!(
                domain_identity
                    .normalized_configuration()
                    .selected_input_label(),
                prepared.configuration().input_kind().selected_label()
            );
            assert_eq!(
                domain_identity.graph_provenance().parent_resource(),
                domain_identity.resource()
            );
            assert_eq!(prepared.resource(), resource.as_ref());
            assert_eq!(prepared.resource().as_ptr(), resource.as_ptr());
            assert_eq!(prepared.programme_id(), &test_programme_id());
            assert_eq!(
                prepared.configuration().source_samples(),
                descriptor["source_samples"].as_u64().unwrap()
            );
            assert_eq!(prepared.admitted_object_events().events().len(), 28);
            assert_eq!(
                prepared.graph().graph().receipt().physical_track_count(),
                38
            );
            assert_eq!(
                prepared.graph().provenance().parent_resource(),
                prepared.resource_identity()
            );
            assert_eq!(prepared.graph().provenance().axml().chunk_id(), *b"axml");
            assert_eq!(prepared.graph().provenance().chna().chunk_id(), *b"chna");
            assert_eq!(prepared.graph().provenance().dbmd().chunk_id(), *b"dbmd");
            let converter_map = prepared.converter_storage_map().unwrap();
            assert_eq!(
                converter_map.resource_identity(),
                prepared.resource_identity()
            );
            assert_eq!(converter_map.axml(), prepared.graph().provenance().axml());
            assert_eq!(converter_map.chna(), prepared.graph().provenance().chna());
            for track in prepared.graph().graph().tracks() {
                let slot = AdmFileStorageTrackSlot::new(
                    u8::try_from(track.track_index().value() - 1).unwrap(),
                );
                let entry = converter_map.channels().entry_for_slot(slot).unwrap();
                let expected_channel_input = match track.kind() {
                    SelectedTrackKind::DirectSpeakersBed { bed_input_id, .. } => bed_input_id,
                    SelectedTrackKind::Object { semantic_key } => semantic_key,
                };
                assert_eq!(entry.track_slot(), slot);
                assert_eq!(entry.channel_input_id().value(), expected_channel_input);
            }
            let xml_sha256: [u8; 32] =
                Sha256::digest(case["input_xml"].as_str().unwrap().as_bytes()).into();
            assert_eq!(prepared.job_xml_sha256(), xml_sha256);
            assert_eq!(
                prepared.identity_envelope()["job_xml_sha256"],
                hex_lower(&xml_sha256)
            );
            let config_bytes = serde_json::to_vec(prepared.identity_envelope()).unwrap();
            assert_eq!(
                prepared.config_sha256(),
                <[u8; 32]>::from(Sha256::digest(config_bytes))
            );
            accepted += 1;
        } else {
            assert!(result.is_err());
            rejected += 1;
        }
    }
    assert_eq!((accepted, rejected), (3, 8));

    let accepted_case = &vectors["config_cases"][0];
    let error = prepare_selected_job_input(
        test_programme_id(),
        accepted_case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(48_000, 38, 24),
        13_888_072,
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap_err();
    assert!(matches!(error, PreparedJobInputError::GraphAdmission(_)));
}

#[test]
fn metadata_from_a_different_resource_allocation_is_rejected() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let (resource, _) = selected_resource();
    let foreign_axml = AXML.to_vec();
    let foreign_chna = CHNA.to_vec();
    let foreign_dbmd = DBMD.to_vec();
    let metadata = SelectedAdmResourcePayloads::new(&foreign_axml, &foreign_chna, &foreign_dbmd);
    let accepted_case = &vectors["config_cases"][0];
    let result = prepare_selected_job_input(
        test_programme_id(),
        accepted_case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(48_000, 38, 24),
        accepted_case["pcm_descriptor"]["source_samples"]
            .as_u64()
            .unwrap(),
        resource,
        metadata,
    );
    assert_eq!(
        result.unwrap_err(),
        PreparedJobInputError::MetadataPayloadBinding
    );
}

#[test]
fn programme_id_is_caller_supplied_and_separate_from_content_identity() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = &vectors["config_cases"][0];
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let (resource, spans) = selected_resource();
    let first = prepare_selected_job_input(
        ProgrammeId::new("JOCForgeTestID01").unwrap(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        wave,
        descriptor["source_samples"].as_u64().unwrap(),
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap();
    let second = prepare_selected_job_input(
        ProgrammeId::new("JOCForgeOtherID1").unwrap(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        wave,
        descriptor["source_samples"].as_u64().unwrap(),
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap();

    assert_ne!(first.programme_id(), second.programme_id());
    assert_eq!(first.resource_identity(), second.resource_identity());
    assert_eq!(first.job_xml_sha256(), second.job_xml_sha256());
    assert_eq!(first.config_sha256(), second.config_sha256());
    assert!(first.identity_envelope().get("programme_id").is_none());
}

#[test]
fn meter_source_admission_binds_owner_identity_to_prepared_input_and_attempt() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = &vectors["config_cases"][0];
    let descriptor = &case["pcm_descriptor"];
    let (resource, spans) = selected_resource();
    let prepared = prepare_selected_job_input(
        test_programme_id(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(
            descriptor["sample_rate"].as_u64().unwrap() as u32,
            descriptor["channels"].as_u64().unwrap() as u16,
            descriptor["pcm_bits"].as_u64().unwrap() as u16,
        ),
        descriptor["source_samples"].as_u64().unwrap(),
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap();
    let samples = MeterSourceSampleCount::new(prepared.configuration().source_samples()).unwrap();
    let first = admit_meter_source_owner(&prepared, AttemptId::new(0), samples).unwrap();
    let repeated = admit_meter_source_owner(&prepared, AttemptId::new(0), samples).unwrap();
    let restarted = admit_meter_source_owner(&prepared, AttemptId::new(1), samples).unwrap();

    assert!(first.matches(&prepared, AttemptId::new(0), samples));
    assert_eq!(first.owner_id(), repeated.owner_id());
    assert_ne!(first.owner_id(), restarted.owner_id());
    assert_eq!(first.programme_id(), prepared.programme_id());
    assert_eq!(first.resource_identity(), prepared.resource_identity());
    assert_eq!(first.prepared_identity(), prepared.domain_identity());
    assert_eq!(first.config_sha256(), prepared.config_sha256());
    assert_eq!(
        first.selected_profile(),
        prepared.configuration().selected_profile()
    );
    assert_eq!(first.wave(), prepared.configuration().wave());
    assert_eq!(first.source_samples(), samples);
    assert_ne!(
        first.standalone_meter_identity(),
        restarted.standalone_meter_identity()
    );
    assert!(!first.matches(&prepared, AttemptId::new(1), samples));
}

#[test]
fn meter_source_admission_rejects_a_mismatched_typed_extent() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = &vectors["config_cases"][0];
    let descriptor = &case["pcm_descriptor"];
    let (resource, spans) = selected_resource();
    let prepared = prepare_selected_job_input(
        test_programme_id(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(
            descriptor["sample_rate"].as_u64().unwrap() as u32,
            descriptor["channels"].as_u64().unwrap() as u16,
            descriptor["pcm_bits"].as_u64().unwrap() as u16,
        ),
        descriptor["source_samples"].as_u64().unwrap(),
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap();
    let wrong_samples =
        MeterSourceSampleCount::new(prepared.configuration().source_samples() + 1).unwrap();

    assert_eq!(
        admit_meter_source_owner(&prepared, AttemptId::new(0), wrong_samples),
        Err(MeterSourceAdmissionError::SourceExtentMismatch)
    );
}

#[test]
fn meter_source_admission_carries_dynamic_adm_physical_row_count() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "adm"
        })
        .unwrap();
    let (resource, spans) = selected_resource_with_channels(36);
    let prepared = prepare_selected_job_input(
        test_programme_id(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(48_000, 36, 24),
        1,
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap();
    let samples = MeterSourceSampleCount::new(1).unwrap();
    let admission = admit_meter_source_owner(&prepared, AttemptId::new(0), samples).unwrap();

    assert_eq!(admission.wave().channels(), 36);
    assert_eq!(prepared.graph().graph().tracks().len(), 36);
    assert!(
        admission
            .selected_profile_identity()
            .unwrap()
            .contains("dynamic-rows")
    );
}

#[test]
fn meter_source_map_materializes_the_n_plus_ten_active_axis_without_padding() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "adm"
        })
        .unwrap();
    let geometry = generate_selected_p11_geometry(0x111b, true).unwrap();
    let membership = PrefixMembershipProducer::for_selected_primary_p11(&geometry).unwrap();

    for channels in [36_u16, 38] {
        let (resource, spans) = selected_resource_with_channels(channels);
        let prepared = prepare_selected_job_input(
            test_programme_id(),
            case["input_xml"].as_str().unwrap().as_bytes(),
            SelectedWaveDescriptor::new(48_000, channels, 24),
            1,
            Arc::clone(&resource),
            metadata_payloads(&resource, &spans),
        )
        .unwrap()
        .bind_primary_control_rows(u64::from(channels) * 3)
        .unwrap();
        let controls = prepared.primary_metadata_controls().unwrap();
        let source_map = build_meter_installed48_source_map(controls, &geometry).unwrap();
        let active_sources = usize::from(channels) + 10;

        assert_eq!(source_map.physical_host_count(), channels);
        assert_eq!(
            usize::from(source_map.regular_count()),
            usize::from(channels) - 1
        );
        assert_eq!(usize::from(source_map.source_count()), active_sources);
        assert_eq!(source_map.rows().len(), active_sources);
        assert_eq!(
            source_map
                .rows()
                .iter()
                .map(|row| row.source_index().value())
                .collect::<Vec<_>>(),
            (0..active_sources as u64).collect::<Vec<_>>()
        );

        let quiet_rows = vec![1_u8; usize::from(channels) - 1];
        let route = source_map
            .produce_primary_prefix_route(&quiet_rows, &membership)
            .unwrap();
        let postroute_rows = source_map
            .postroute_rows(&quiet_rows, route.writers())
            .unwrap();
        assert_eq!(postroute_rows.len(), active_sources);
        let (field0_mask, field4_mask) = field_masks_hex(&postroute_rows).unwrap();
        assert_eq!(field0_mask.len(), active_sources.div_ceil(4));
        assert_eq!(field4_mask.len(), active_sources.div_ceil(4));

        if channels == 36 {
            assert_eq!(source_map.row_for_source(CodecSourceIndex::new(46)), None);
            assert_eq!(source_map.row_for_source(CodecSourceIndex::new(47)), None);
        }
    }
}

#[test]
fn meter_source_map_materializes_n_plus_ten_active_rows_without_padding() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "adm"
        })
        .unwrap();
    let geometry = generate_selected_p11_geometry(0x111b, true).unwrap();
    let membership = PrefixMembershipProducer::for_selected_primary_p11(&geometry).unwrap();

    for channels in [36_u16, 38] {
        let (resource, spans) = selected_resource_with_channels(channels);
        let prepared = prepare_selected_job_input(
            test_programme_id(),
            case["input_xml"].as_str().unwrap().as_bytes(),
            SelectedWaveDescriptor::new(48_000, channels, 24),
            1,
            Arc::clone(&resource),
            metadata_payloads(&resource, &spans),
        )
        .unwrap()
        .bind_primary_control_rows(u64::from(channels) * 3)
        .unwrap();
        let source_map = build_meter_installed48_source_map(
            prepared.primary_metadata_controls().unwrap(),
            &geometry,
        )
        .unwrap();
        let active_sources = usize::from(channels) + 10;

        assert_eq!(source_map.physical_host_count(), channels);
        assert_eq!(
            usize::from(source_map.regular_count()),
            usize::from(channels) - 1
        );
        assert_eq!(usize::from(source_map.source_count()), active_sources);
        assert_eq!(source_map.rows().len(), active_sources);
        assert_eq!(
            source_map
                .rows()
                .iter()
                .map(|row| row.source_index().value())
                .collect::<Vec<_>>(),
            (0..active_sources as u64).collect::<Vec<_>>()
        );

        let quiet_rows = vec![1_u8; usize::from(channels) - 1];
        let route = source_map
            .produce_primary_prefix_route(&quiet_rows, &membership)
            .unwrap();
        let postroute_rows = source_map
            .postroute_rows(&quiet_rows, route.writers())
            .unwrap();
        assert_eq!(postroute_rows.len(), active_sources);
        let (field0_mask, field4_mask) = field_masks_hex(&postroute_rows).unwrap();
        assert_eq!(field0_mask.len(), active_sources.div_ceil(4));
        assert_eq!(field4_mask.len(), active_sources.div_ceil(4));

        let activity_producer = PrimaryInstalled48ActivityProducer::new_meter(&source_map).unwrap();
        let activity_before = activity_producer.cold_state().unwrap();
        let activity = activity_producer
            .produce_next(
                &source_map,
                &activity_before,
                PrimaryActivityInput::NormalRead {
                    reader_relative_sample: AdmSampleIndex::new(0),
                    pcm_word_rows: vec![vec![0_u32]; usize::from(channels)],
                },
            )
            .unwrap();
        assert_eq!(
            activity.output().active_source_count(),
            active_sources as u16
        );
        assert_eq!(
            activity.output().quiet_regular_rows().len(),
            usize::from(channels) - 1
        );
        assert_eq!(activity.output().postroute_rows().len(), active_sources);

        if channels == 36 {
            assert_eq!(source_map.row_for_source(CodecSourceIndex::new(46)), None);
            assert_eq!(source_map.row_for_source(CodecSourceIndex::new(47)), None);
        }
    }
}

#[test]
fn f01e_cold_provider_projects_adm_rows_and_selected_drc_profiles() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "adm"
        })
        .unwrap();
    let provider = validated_f01e_provider();
    let original_canonical = provider.canonical_bytes().to_vec();
    let mut input_snapshot_hashes = Vec::new();
    let input_xml = case["input_xml"]
        .as_str()
        .unwrap()
        .replace(
            "<line_mode_drc_profile>none</line_mode_drc_profile>",
            "<line_mode_drc_profile>film_standard</line_mode_drc_profile>",
        )
        .replace(
            "<rf_mode_drc_profile>none</rf_mode_drc_profile>",
            "<rf_mode_drc_profile>music_light</rf_mode_drc_profile>",
        );

    for (channels, expected_objects) in [(36_u16, 26_u64), (38, 28)] {
        let (resource, spans) = selected_resource_with_channels(channels);
        let prepared = prepare_selected_job_input(
            test_programme_id(),
            input_xml.as_bytes(),
            SelectedWaveDescriptor::new(48_000, channels, 24),
            1,
            Arc::clone(&resource),
            metadata_payloads(&resource, &spans),
        )
        .unwrap();

        let projected = provider
            .for_admitted_adm_input(prepared.configuration(), prepared.graph().graph())
            .unwrap();
        let cold: Value = serde_json::from_slice(projected.canonical_bytes()).unwrap();

        assert_eq!(projected.callback_index(), 0);
        assert_eq!(
            projected.object_count(),
            u32::try_from(expected_objects).unwrap()
        );
        assert_eq!(cold["schema"], "F01E_ADM_INPUT_COLD_PROJECTION_V1");
        assert_eq!(cold["phase"], "BEFORE_FIRST_NORMAL");
        assert_eq!(cold["remaining_cached_drains"], 4);
        assert_eq!(cold["normal_callbacks"], 0);
        assert_eq!(cold["normalized_config"]["channels"], channels);
        assert_eq!(cold["normalized_config"]["bed_count"], 10);
        assert_eq!(cold["normalized_config"]["object_count"], expected_objects);
        assert_eq!(cold["normalized_config"]["frame_samples"], 1_536);
        assert_eq!(cold["normalized_config"]["line_drc"], "film_standard");
        assert_eq!(cold["normalized_config"]["rf_drc"], "music_light");
        assert_eq!(
            cold["objects"].as_array().unwrap().len(),
            expected_objects as usize
        );
        assert!(!cold.as_object().unwrap().contains_key("ddp_frame_view"));
        for (ordinal, object) in cold["objects"].as_array().unwrap().iter().enumerate() {
            assert_eq!(object["object_ordinal"].as_u64(), Some(ordinal as u64));
            assert_eq!(object["event_cache"], Value::Null);
            assert_eq!(object["host_channel_index"], Value::Null);
            assert_eq!(object["sampler_previous_present"], false);
            assert_eq!(
                object["sampler_previous_xyz_u32"],
                serde_json::json!([0, 0, 0])
            );
        }

        let attempt = AttemptId::new(0);
        let owner = prepared.f01e_cold_owner(&provider, attempt).unwrap();
        let state = owner.committed();
        assert_eq!(owner.generation(), 0);
        assert_eq!(state.schema(), "F01E_ADM_INPUT_COLD_PROJECTION_V1");
        assert_eq!(state.programme_id(), prepared.programme_id());
        assert_eq!(state.attempt_id(), attempt);
        assert_eq!(state.callback_index().value(), 0);
        assert_eq!(state.normal_callbacks().value(), 0);
        assert_eq!(state.remaining_cached_drains(), 4);
        assert_eq!(state.objects().len(), expected_objects as usize);
        assert_eq!(state.normalized_config().channels(), u32::from(channels));
        assert_eq!(state.normalized_config().line_drc(), "film_standard");
        assert_eq!(state.normalized_config().rf_drc(), "music_light");
        assert!(state.dbmd_binding().is_none());
        assert!(state.physical_binding().is_none());
        let prepared_with_controls = prepared
            .bind_primary_control_rows(u64::from(channels) * 3)
            .unwrap();
        let normal_input = prepared_with_controls
            .normal_control_adm_input_snapshot(&owner)
            .unwrap();
        assert_eq!(normal_input.schema(), "F01E_ADM_INPUT_NORMAL_PREFLIGHT_V1");
        assert_eq!(
            normal_input.programme_id(),
            prepared_with_controls.programme_id()
        );
        assert_eq!(normal_input.attempt_id(), attempt);
        assert_eq!(normal_input.owner_generation(), 0);
        assert_eq!(normal_input.expected_normal_callback_count(), 1);
        assert_eq!(normal_input.input_physical_rows(), channels);
        assert_eq!(normal_input.bed_track_count(), 10);
        assert_eq!(normal_input.object_track_count(), expected_objects as u16);
        assert_eq!(normal_input.physical_tracks().len(), usize::from(channels));
        assert_eq!(
            normal_input.object_owner_bindings().len(),
            expected_objects as usize
        );
        assert_eq!(
            normal_input.object_events().len(),
            expected_objects as usize
        );
        assert_eq!(
            normal_input.lfe_host(),
            prepared_with_controls
                .graph()
                .graph()
                .bed_channel_join()
                .lfe_host()
        );
        assert_eq!(
            normal_input.regular_control_hosts(),
            prepared_with_controls
                .graph()
                .graph()
                .bed_channel_join()
                .regular_control_order()
        );
        assert_ne!(normal_input.identity_sha256(), &[0_u8; 32]);
        input_snapshot_hashes.push(*normal_input.identity_sha256());
        let normal_proposal = prepared_with_controls
            .propose_f01e_adm_normal_callback(&owner)
            .unwrap();
        assert_eq!(normal_proposal.transaction_id().callback_index().value(), 0);
        assert_eq!(normal_proposal.before_generation(), 0);
        assert_eq!(normal_proposal.actual_samples(), 1);
        assert_eq!(
            normal_proposal.invocation_hosts().len(),
            expected_objects as usize
        );
        assert_eq!(
            normal_proposal.sampler_before().len(),
            expected_objects as usize
        );
        let after = normal_proposal.after_state();
        assert_eq!(after.schema(), "F01E_ADM_INPUT_NORMAL_STATE_V1");
        assert_eq!(after.source_origin_generation().value(), 0);
        assert_eq!(after.next_callback_index().value(), 1);
        assert_eq!(after.owner_generation(), 1);
        assert_eq!(after.processed_normal_callbacks().value(), 1);
        assert_eq!(after.expected_normal_callback_count(), 1);
        assert_eq!(after.remaining_cached_drains(), 4);
        assert_eq!(after.input_physical_rows().value(), channels);
        assert_eq!(after.normal_control_rows().value(), channels);
        assert_eq!(
            after.regular_control_rows().value(),
            u16::try_from(
                prepared_with_controls
                    .graph()
                    .graph()
                    .bed_channel_join()
                    .regular_control_order()
                    .len()
            )
            .unwrap()
        );
        assert_eq!(after.physical_rows().len(), usize::from(channels));
        assert_eq!(after.control_words().len(), usize::from(channels) * 13);
        assert_eq!(
            after
                .physical_rows()
                .iter()
                .filter(|row| matches!(row, NormalControlRow::Bed6 { .. }))
                .count(),
            10
        );
        assert_eq!(
            after
                .physical_rows()
                .iter()
                .filter(|row| matches!(row, NormalControlRow::Object13 { .. }))
                .count(),
            expected_objects as usize
        );
        assert_eq!(
            after.regular_control_words().len(),
            after.regular_rows().len() * 13
        );
        assert_eq!(after.object_owner_count().value(), expected_objects as u16);
        assert_eq!(after.lfe_host(), normal_input.lfe_host());
        assert_eq!(
            after.regular_control_hosts(),
            normal_input.regular_control_hosts()
        );

        let direct_lfe_probe_rows = (0..channels)
            .map(|physical_row| {
                let mut row = vec![0_u32; 1_536];
                row[0] = u32::from(physical_row) + 1;
                row
            })
            .collect::<Vec<_>>();
        let direct_lfe_probe_frame = MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
            callback_index: MeterCallbackIndex::new(0),
            upstream_generation: MeterUpstreamGeneration::new(1),
            phase: MeterCallbackPhase::NormalSource,
            source_identity: MeterSourceIdentity::from_sha256(
                prepared_with_controls.resource_identity().input_sha256(),
            ),
            source_samples: MeterSourceSampleCount::new(1).unwrap(),
            normal_callbacks: MeterCallbackCount::new(1),
            source_cursor_before: MeterSourcePosition::new(0),
            source_cursor_after: MeterSourcePosition::new(1),
            source_read_index_before: MeterSourceReadIndex::new(0),
            source_read_index_after: MeterSourceReadIndex::new(1),
            host_input_count: MeterSampleCount::new(1),
            source_padding_count: MeterSampleCount::new(1_535),
            rows: &direct_lfe_probe_rows,
        });
        let direct_lfe_probe =
            MeterSourceCallbackView::new(&prepared_with_controls, direct_lfe_probe_frame).unwrap();
        let source_geometry = prepared_with_controls
            .propose_f01e_adm_normal_source_geometry(
                &owner,
                &normal_proposal,
                &direct_lfe_probe,
                &generate_selected_p11_geometry(0x111b, true).unwrap(),
            )
            .unwrap();
        assert_eq!(source_geometry.rows().len(), usize::from(channels) + 10);
        assert_eq!(source_geometry.source_origin_generation().value(), 0);
        let point_response_sources = source_geometry
            .point_response_snapshot(
                SourceControlSnapshotClock::for_normal_callback(
                    FrontendCallbackIndex::new(0),
                    NormalCallbackCount::new(1),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            point_response_sources.rows().len(),
            usize::from(channels) + 10
        );
        let point_response_gate = PointResponseGateInput::new(
            FrontendCallbackIndex::new(0),
            source_geometry.transaction_id().clone(),
            vec![0],
            vec![0],
        )
        .unwrap();
        let p15_geometry = generate_selected_p15_geometry(0x159b, true).unwrap();
        let point_response_batch =
            prepare_point_response_sources(point_response_sources, point_response_gate).unwrap();
        let point_response =
            produce_point_response_columns(&p15_geometry, point_response_batch).unwrap();
        assert_eq!(point_response.columns().len(), 1);
        assert_eq!(
            point_response.columns()[0].source_index(),
            CodecSourceIndex::new(0)
        );
        let mut active_flags = [0_u8; 15];
        active_flags[0] = 1;
        let target_state = TargetSlotState {
            xyz_words: std::array::from_fn(|slot| p15_geometry.points()[slot].xyz_f32_bits()),
            ids_u32: std::array::from_fn(|slot| 200 + u32::try_from(slot).unwrap()),
            attributes_u32: std::array::from_fn(|slot| u32::try_from(slot).unwrap() << 4),
            previous_xyz_words: std::array::from_fn(|slot| {
                p15_geometry.points()[slot].xyz_f32_bits()
            }),
            previous_ids_u32: std::array::from_fn(|slot| 200 + u32::try_from(slot).unwrap()),
            previous_attributes_u32: std::array::from_fn(|slot| u32::try_from(slot).unwrap() << 4),
            active_flags,
        };
        let installation = install_target_slots(
            &target_state,
            &SelectedTargetSlots {
                xyz_words: Vec::new(),
                ids_u32: Vec::new(),
                attributes_u32: Vec::new(),
            },
        )
        .unwrap();
        let target_context = build_target_context_projection(
            &p15_geometry,
            &installation,
            source_geometry.transaction_id().clone(),
            FrontendCallbackIndex::new(0),
        )
        .unwrap();
        let compact_outer =
            produce_compact_outer_matrix(&target_context, &source_geometry, &point_response)
                .unwrap();
        let active_sources = usize::from(channels) + 10;
        assert_eq!(compact_outer.active_source_count(), active_sources as u16);
        assert_eq!(
            compact_outer.c_rows_u32().len(),
            usize::from(target_context.compact_target_count())
        );
        assert!(
            compact_outer
                .c_rows_u32()
                .iter()
                .all(|row| row.len() == active_sources)
        );
        if channels == 36 {
            assert_eq!(
                source_geometry.row_for_source(CodecSourceIndex::new(46)),
                None
            );
            assert_eq!(
                source_geometry.row_for_source(CodecSourceIndex::new(47)),
                None
            );
        }
        let direct_lfe_binding = direct_lfe_probe
            .physical_row_map()
            .binding_for_physical_host(direct_lfe_probe.direct_lfe_host())
            .unwrap();
        assert_eq!(
            direct_lfe_binding.track_index().value() - 1,
            u16::from(direct_lfe_binding.pcm_row().value())
        );
        let direct_lfe_pcm_row = usize::from(direct_lfe_binding.pcm_row().value());
        assert_eq!(
            direct_lfe_probe.direct_lfe_words(),
            direct_lfe_probe_rows[direct_lfe_pcm_row].as_slice()
        );
        assert_eq!(
            direct_lfe_probe.direct_lfe_words()[0],
            u32::from(direct_lfe_binding.pcm_row().value()) + 1
        );

        let zero_rows = vec![vec![0_u32; 1_536]; usize::from(channels)];
        for delivery in 1_u32..=3 {
            let transaction = CallbackLineageIdentity::new(
                prepared_with_controls.programme_id().clone(),
                attempt,
                FrontendCallbackIndex::new(delivery),
            );
            let view = after.fresh_zero_view(transaction.clone()).unwrap();
            let local_receipt = view.validate_zero_physical_input_rows(&zero_rows).unwrap();
            assert_eq!(
                local_receipt
                    .delivery_transaction_id()
                    .callback_index()
                    .value(),
                delivery
            );
            assert_eq!(
                local_receipt
                    .source_origin_transaction_id()
                    .callback_index()
                    .value(),
                0
            );
            assert_eq!(local_receipt.input_physical_rows().value(), channels);
            assert_eq!(local_receipt.normal_control_rows().value(), channels);
            assert_eq!(local_receipt.zero_samples_per_physical_row(), 1_536);
            let source_identity = MeterSourceIdentity::from_sha256(
                prepared_with_controls.resource_identity().input_sha256(),
            );
            let frame = MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
                callback_index: MeterCallbackIndex::new(u64::from(delivery)),
                upstream_generation: MeterUpstreamGeneration::new(u64::from(delivery) + 1),
                phase: MeterCallbackPhase::ZeroInputDrain,
                source_identity,
                source_samples: MeterSourceSampleCount::new(1).unwrap(),
                normal_callbacks: MeterCallbackCount::new(1),
                source_cursor_before: MeterSourcePosition::new(1),
                source_cursor_after: MeterSourcePosition::new(1),
                source_read_index_before: MeterSourceReadIndex::new(1),
                source_read_index_after: MeterSourceReadIndex::new(1),
                host_input_count: MeterSampleCount::new(0),
                source_padding_count: MeterSampleCount::new(1_536),
                rows: &zero_rows,
            });
            let source_view = MeterSourceCallbackView::new(&prepared_with_controls, frame).unwrap();
            let bound_receipt = prepared_with_controls
                .bind_f01e_adm_fresh_zero_rows(&owner, &normal_proposal, &source_view)
                .unwrap();
            assert_eq!(bound_receipt.delivery_transaction_id(), &transaction);
            assert_eq!(bound_receipt.input_physical_rows().value(), channels);
            assert_eq!(bound_receipt.zero_samples_per_physical_row(), 1_536);
        }
        let invalid_zero = after
            .fresh_zero_view(CallbackLineageIdentity::new(
                prepared_with_controls.programme_id().clone(),
                attempt,
                FrontendCallbackIndex::new(1),
            ))
            .unwrap();
        assert!(matches!(
            after.fresh_zero_view(CallbackLineageIdentity::new(
                prepared_with_controls.programme_id().clone(),
                attempt,
                FrontendCallbackIndex::new(4),
            )),
            Err(NormalControlSnapshotError::FreshZeroWindowMismatch)
        ));
        let mut short_zero_rows = zero_rows.clone();
        short_zero_rows.pop();
        assert_eq!(
            invalid_zero.validate_zero_physical_input_rows(&short_zero_rows),
            Err(NormalControlSnapshotError::FreshZeroInputShape)
        );
        let mut nonzero_rows = zero_rows.clone();
        nonzero_rows[0][0] = 1;
        assert_eq!(
            invalid_zero.validate_zero_physical_input_rows(&nonzero_rows),
            Err(NormalControlSnapshotError::FreshZeroInputNotZero)
        );
        if channels == 36 {
            let (same_content_resource, same_content_spans) =
                selected_resource_with_channels(channels);
            let same_content_prepared = prepare_selected_job_input(
                test_programme_id(),
                case["input_xml"].as_str().unwrap().as_bytes(),
                SelectedWaveDescriptor::new(48_000, channels, 24),
                1,
                Arc::clone(&same_content_resource),
                metadata_payloads(&same_content_resource, &same_content_spans),
            )
            .unwrap()
            .bind_primary_control_rows(u64::from(channels) * 3)
            .unwrap();
            assert_eq!(
                same_content_prepared.normal_control_adm_input_snapshot(&owner),
                Err(PreparedJobInputError::PreparedIdentityMismatch)
            );
            let same_content_frame = MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
                callback_index: MeterCallbackIndex::new(0),
                upstream_generation: MeterUpstreamGeneration::new(1),
                phase: MeterCallbackPhase::NormalSource,
                source_identity: MeterSourceIdentity::from_sha256(
                    same_content_prepared.resource_identity().input_sha256(),
                ),
                source_samples: MeterSourceSampleCount::new(1).unwrap(),
                normal_callbacks: MeterCallbackCount::new(1),
                source_cursor_before: MeterSourcePosition::new(0),
                source_cursor_after: MeterSourcePosition::new(1),
                source_read_index_before: MeterSourceReadIndex::new(0),
                source_read_index_after: MeterSourceReadIndex::new(1),
                host_input_count: MeterSampleCount::new(1),
                source_padding_count: MeterSampleCount::new(1_535),
                rows: &direct_lfe_probe_rows,
            });
            let same_content_view =
                MeterSourceCallbackView::new(&same_content_prepared, same_content_frame).unwrap();
            assert_eq!(
                prepared_with_controls.propose_f01e_adm_normal_source_geometry(
                    &owner,
                    &normal_proposal,
                    &same_content_view,
                    &generate_selected_p11_geometry(0x111b, true).unwrap(),
                ),
                Err(PreparedJobInputError::NormalSourceGeometryMismatch)
            );
            assert_eq!(
                prepared_with_controls
                    .normal_control_prepared_snapshot(state)
                    .unwrap_err(),
                PreparedJobInputError::NormalControlSnapshot(
                    NormalControlSnapshotError::StateShapeMismatch
                )
            );
        } else {
            assert!(
                prepared_with_controls
                    .normal_control_prepared_snapshot(state)
                    .is_ok()
            );
        }
        assert_eq!(provider.canonical_bytes(), original_canonical);
    }

    assert_ne!(input_snapshot_hashes[0], input_snapshot_hashes[1]);
    assert_eq!(provider.object_count(), 28);
}

#[test]
fn explicit_dialogue_measurement_inputs_are_preserved_then_gated_before_meter_admission() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = &vectors["config_cases"][0];
    let base_xml = case["input_xml"].as_str().unwrap();
    let (resource, spans) = selected_resource_with_channels(38);

    for (dialogue_intelligence, speech_threshold) in [(true, 15), (true, 100), (false, 15)] {
        let xml = base_xml
            .replace(
                "<dialogue_intelligence>false</dialogue_intelligence>",
                &format!("<dialogue_intelligence>{dialogue_intelligence}</dialogue_intelligence>"),
            )
            .replace(
                "<speech_threshold>100</speech_threshold>",
                &format!("<speech_threshold>{speech_threshold}</speech_threshold>"),
            );
        let prepared = prepare_selected_job_input(
            test_programme_id(),
            xml.as_bytes(),
            SelectedWaveDescriptor::new(48_000, 38, 24),
            1,
            Arc::clone(&resource),
            metadata_payloads(&resource, &spans),
        )
        .unwrap()
        .bind_primary_control_rows(38 * 3)
        .unwrap();
        assert_eq!(
            prepared.configuration().loudness().dialogue_intelligence(),
            dialogue_intelligence
        );
        assert_eq!(
            prepared.configuration().loudness().speech_threshold(),
            speech_threshold
        );

        let error = admit_meter_source_owner(
            &prepared,
            AttemptId::new(0),
            MeterSourceSampleCount::new(1).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error,
            MeterSourceAdmissionError::UnsupportedDialogueMeasurementEffect {
                dialogue_intelligence,
                speech_threshold,
            }
        );
    }
}

#[test]
fn prepared_adm_input_preserves_each_washed_rate_through_profile_and_frame_size() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "accepted_full_adm")
        .unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();
    let (resource, spans) = selected_resource_with_channels(38);
    let provider = validated_f01e_provider();
    let rows = [
        (384, 0, 767, 1_536, CodecRateProfile::Kbps384),
        (448, 1, 895, 1_792, CodecRateProfile::Kbps448),
        (576, 2, 1_151, 2_304, CodecRateProfile::Kbps576),
        (640, 3, 1_279, 2_560, CodecRateProfile::Kbps640),
        (768, 4, 1_535, 3_072, CodecRateProfile::Kbps768),
        (1_024, 5, 2_047, 4_096, CodecRateProfile::Kbps1024),
    ];

    for (rate, selector, frame_size_code, frame_bytes, expected_profile) in rows {
        let xml = base_xml.replace(
            "<data_rate>1024</data_rate>",
            &format!("<data_rate>{rate}</data_rate>"),
        );
        let prepared = prepare_selected_job_input(
            test_programme_id(),
            xml.as_bytes(),
            SelectedWaveDescriptor::new(48_000, 38, 24),
            1,
            Arc::clone(&resource),
            metadata_payloads(&resource, &spans),
        )
        .unwrap();
        let configuration = prepared.configuration();
        assert_eq!(configuration.bit_rate_kbps(), rate);
        assert_eq!(configuration.bit_rate_enum(), selector);
        assert_eq!(
            configuration.selected_profile().codec_rate(),
            expected_profile
        );
        let frame_size = jocforge_domain::codec::frame_size::Eac3FrameSize::from_codec_rate(
            configuration.selected_profile().codec_rate(),
        )
        .unwrap();
        assert_eq!(frame_size.code().value(), frame_size_code);
        assert_eq!(frame_size.bytes(), frame_bytes);
        let meter_admission = admit_meter_source_owner(
            &prepared,
            AttemptId::new(0),
            MeterSourceSampleCount::new(1).unwrap(),
        )
        .unwrap();
        assert!(
            meter_admission
                .selected_profile_identity()
                .unwrap()
                .contains(expected_profile.identity_label())
        );
        prepared
            .f01e_cold_owner(&provider, AttemptId::new(0))
            .unwrap();
    }
}

#[test]
fn unsupported_authored_silence_effects_stop_before_meter_and_f15_admission() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let base_xml = vectors["config_cases"][0]["input_xml"].as_str().unwrap();
    let (resource, spans) = selected_resource_with_channels(38);
    let owner = OutputStagingOwnerId::new("silence-timeline-gate").unwrap();
    let cases = [
        base_xml.replace(
            "<prepend_silence_duration>0.0</prepend_silence_duration>",
            "<prepend_silence_duration>0.00390625</prepend_silence_duration>",
        ),
        base_xml.replace(
            "<append_silence_duration>0.0</append_silence_duration>",
            "<append_silence_duration>0.000001</append_silence_duration>",
        ),
        base_xml.replace("<append_silence_duration>0.0</append_silence_duration>", ""),
    ];

    for xml in cases {
        let prepared = prepare_selected_job_input(
            test_programme_id(),
            xml.as_bytes(),
            SelectedWaveDescriptor::new(48_000, 38, 24),
            1,
            Arc::clone(&resource),
            metadata_payloads(&resource, &spans),
        )
        .expect("typed configuration remains inspectable before selected product admission");

        assert_eq!(
            admit_meter_source_owner(
                &prepared,
                AttemptId::new(0),
                MeterSourceSampleCount::new(1).unwrap(),
            )
            .unwrap_err(),
            MeterSourceAdmissionError::UnsupportedSilenceTimelineEffect
        );
        assert!(matches!(
            admit_selected_f15_plan(
                &prepared,
                AttemptId::new(0),
                &owner,
                "C:\\temp\\silence-timeline-gate.ec3",
            ),
            Err(EncodeError::F15AdmissionRejected)
        ));
    }
}

fn selected_resource() -> (Arc<[u8]>, [Range<usize>; 3]) {
    selected_resource_with_channels(38)
}

fn selected_resource_with_channels(channels: u16) -> (Arc<[u8]>, [Range<usize>; 3]) {
    selected_resource_with_channel_samples(channels, 1)
}

fn selected_resource_with_channel_samples(
    channels: u16,
    source_samples: u64,
) -> (Arc<[u8]>, [Range<usize>; 3]) {
    let track_count = usize::from(channels);
    assert!((10..=38).contains(&track_count));
    let chna = if track_count == 38 {
        CHNA.to_vec()
    } else {
        let mut subset = CHNA[..4 + track_count * 40].to_vec();
        subset[..2].copy_from_slice(&channels.to_le_bytes());
        subset[2..4].copy_from_slice(&channels.to_le_bytes());
        subset
    };
    let mut bytes = b"RIFF\0\0\0\0WAVE".to_vec();
    append_chunk(&mut bytes, *b"fmt ", &selected_format_payload(channels));
    let data_bytes = usize::try_from(source_samples).unwrap() * track_count * 3;
    append_chunk(&mut bytes, *b"data", &vec![0_u8; data_bytes]);
    let axml = append_chunk(&mut bytes, *b"axml", AXML);
    let chna = append_chunk(&mut bytes, *b"chna", &chna);
    let dbmd_payload = synthetic_dbmd_payload_for_physical_rows(channels);
    let dbmd = append_chunk(&mut bytes, *b"dbmd", &dbmd_payload);
    let riff_size = u32::try_from(bytes.len() - 8).unwrap();
    bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
    (Arc::from(bytes.into_boxed_slice()), [axml, chna, dbmd])
}

fn synthetic_dbmd_payload_for_physical_rows(channels: u16) -> Vec<u8> {
    if channels == 38 {
        return DBMD.to_vec();
    }
    assert!((10..=37).contains(&channels));
    if channels == 36 {
        let washed_dimensions: Value = serde_json::from_slice(include_bytes!(
            "fixtures/authorized-adm-dbmd-row-count.json"
        ))
        .unwrap();
        assert_eq!(
            washed_dimensions["wave_channels"].as_u64(),
            Some(u64::from(channels))
        );
        assert_eq!(
            washed_dimensions["chna_track_count"].as_u64(),
            Some(u64::from(channels))
        );
        assert_eq!(
            washed_dimensions["dbmd_section10_track_count"].as_u64(),
            Some(u64::from(channels))
        );
        assert_eq!(washed_dimensions["pcm_bytes_read"], 0);
        assert_eq!(
            washed_dimensions["source_package_manifest_sha256"],
            "fd6b5b9e0075ebcf555f7c2b4688ecbc1f88c5af9c34006d1e8f97a245186374"
        );
    }

    let original = parse_selected_dbmd_sections(DBMD).unwrap();
    assert_eq!(original.track_fields().len(), 38);
    let mut output = DBMD[..DBMD_PREFIX_BYTES].to_vec();
    let mut offset = DBMD_PREFIX_BYTES;
    let mut found_section10 = false;
    while offset < DBMD.len() {
        let id = DBMD[offset];
        if id == 0 {
            output.extend_from_slice(&DBMD[offset..]);
            break;
        }
        let header_end = offset + DBMD_SECTION_HEADER_BYTES;
        let old_length = usize::from(u16::from_le_bytes([DBMD[offset + 1], DBMD[offset + 2]]));
        let body_end = header_end + old_length;
        let section_end = body_end + 1;
        let old_body = &DBMD[header_end..body_end];
        if id == DBMD_SECTION_10_ID {
            found_section10 = true;
            let old_track_count = usize::from(u16::from_le_bytes([old_body[4], old_body[5]]));
            assert_eq!(old_track_count, 38);
            let old_arrays_end = DBMD_SECTION_10_FIXED_BYTES + old_track_count * 2;
            assert!(old_body.len() >= old_arrays_end);
            let mut body = old_body[..DBMD_SECTION_10_FIXED_BYTES].to_vec();
            body[4..6].copy_from_slice(&channels.to_le_bytes());
            // Zero synthetic per-track values: this tests shape and physical index mapping only.
            body.resize(DBMD_SECTION_10_FIXED_BYTES + usize::from(channels) * 2, 0);
            body.extend_from_slice(&old_body[old_arrays_end..]);
            let length = u16::try_from(body.len()).unwrap();
            output.push(id);
            output.extend_from_slice(&length.to_le_bytes());
            output.extend_from_slice(&body);
            output.push(dbmd_section_checksum_for_test(length, &body));
        } else {
            output.extend_from_slice(&DBMD[offset..section_end]);
        }
        offset = section_end;
    }
    assert!(found_section10);
    let parsed = parse_selected_dbmd_sections(&output).unwrap();
    assert_eq!(parsed.track_fields().len(), usize::from(channels));
    for (index, fields) in parsed.track_fields().iter().enumerate() {
        assert_eq!(fields.track_index(), u16::try_from(index).unwrap());
        assert_eq!(fields.enum48(), 0);
        assert_eq!(fields.enum52(), 0);
    }
    output
}

fn dbmd_section_checksum_for_test(length: u16, body: &[u8]) -> u8 {
    body.iter()
        .copied()
        .fold(length as u8, u8::wrapping_add)
        .wrapping_neg()
}

#[test]
fn dynamic_f01e_normal_continuation_binds_36_and_38_row_callback_clocks() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "adm"
        })
        .unwrap();
    let provider = validated_f01e_provider();

    for channels in [36_u16, 38] {
        let (resource, spans) = selected_resource_with_channel_samples(channels, 1_537);
        let prepared = prepare_selected_job_input(
            test_programme_id(),
            case["input_xml"].as_str().unwrap().as_bytes(),
            SelectedWaveDescriptor::new(48_000, channels, 24),
            1_537,
            Arc::clone(&resource),
            metadata_payloads(&resource, &spans),
        )
        .unwrap()
        .bind_primary_control_rows(1_537 * u64::from(channels) * 3)
        .unwrap();
        let attempt = AttemptId::new(0);
        let owner = prepared.f01e_cold_owner(&provider, attempt).unwrap();
        let normal_input = prepared.normal_control_adm_input_snapshot(&owner).unwrap();
        assert_eq!(normal_input.expected_normal_callback_count(), 2);

        let first = prepared.propose_f01e_adm_normal_callback(&owner).unwrap();
        assert_eq!(first.transaction_id().callback_index().value(), 0);
        assert_eq!(first.query_sample(), 0);
        assert_eq!(first.actual_samples(), 1_536);
        assert!(
            first
                .sampler_before()
                .iter()
                .all(|sampler| !sampler.sampler_previous_present())
        );

        let rows = vec![vec![0_u32; 1_536]; usize::from(channels)];
        let first_frame = MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
            callback_index: MeterCallbackIndex::new(0),
            upstream_generation: MeterUpstreamGeneration::new(1),
            phase: MeterCallbackPhase::NormalSource,
            source_identity: MeterSourceIdentity::from_sha256(
                prepared.resource_identity().input_sha256(),
            ),
            source_samples: MeterSourceSampleCount::new(1_537).unwrap(),
            normal_callbacks: MeterCallbackCount::new(2),
            source_cursor_before: MeterSourcePosition::new(0),
            source_cursor_after: MeterSourcePosition::new(1_536),
            source_read_index_before: MeterSourceReadIndex::new(0),
            source_read_index_after: MeterSourceReadIndex::new(1),
            host_input_count: MeterSampleCount::new(1_536),
            source_padding_count: MeterSampleCount::new(0),
            rows: &rows,
        });
        let first_view = MeterSourceCallbackView::new(&prepared, first_frame).unwrap();
        let source_geometry = prepared
            .propose_f01e_adm_normal_source_geometry(
                &owner,
                &first,
                &first_view,
                &generate_selected_p11_geometry(0x111b, true).unwrap(),
            )
            .unwrap();
        assert_eq!(source_geometry.source_origin_generation().value(), 0);
        assert_eq!(source_geometry.rows().len(), usize::from(channels) + 10);

        let second_frame = MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
            callback_index: MeterCallbackIndex::new(1),
            upstream_generation: MeterUpstreamGeneration::new(2),
            phase: MeterCallbackPhase::NormalSource,
            source_identity: MeterSourceIdentity::from_sha256(
                prepared.resource_identity().input_sha256(),
            ),
            source_samples: MeterSourceSampleCount::new(1_537).unwrap(),
            normal_callbacks: MeterCallbackCount::new(2),
            source_cursor_before: MeterSourcePosition::new(1_536),
            source_cursor_after: MeterSourcePosition::new(1_537),
            source_read_index_before: MeterSourceReadIndex::new(1),
            source_read_index_after: MeterSourceReadIndex::new(2),
            host_input_count: MeterSampleCount::new(1),
            source_padding_count: MeterSampleCount::new(1_535),
            rows: &rows,
        });
        let second_view = MeterSourceCallbackView::new(&prepared, second_frame).unwrap();
        let second = prepared
            .propose_f01e_adm_normal_continuation(&normal_input, first.after_state(), &second_view)
            .unwrap();

        assert!(std::ptr::eq(second.before_state(), first.after_state()));
        assert_eq!(second.transaction_id().callback_index().value(), 1);
        assert_eq!(second.query_sample(), 1_536);
        assert_eq!(second.actual_samples(), 1);
        assert_eq!(second.sampler_before().len(), usize::from(channels) - 10);
        assert!(
            second
                .sampler_before()
                .iter()
                .all(|sampler| sampler.sampler_previous_present())
        );

        let after = second.after_state();
        assert_eq!(after.source_origin_generation().value(), 1);
        assert_eq!(after.next_callback_index().value(), 2);
        assert_eq!(after.owner_generation(), 2);
        assert_eq!(after.processed_normal_callbacks().value(), 2);
        assert_eq!(after.expected_normal_callback_count(), 2);
        assert_eq!(after.input_physical_rows().value(), channels);
        assert_eq!(after.normal_control_rows().value(), channels);
        assert_eq!(after.regular_control_rows().value(), channels - 1);
        assert_eq!(after.physical_rows().len(), usize::from(channels));
        assert_eq!(after.regular_rows().len(), usize::from(channels) - 1);

        let zero_rows = vec![vec![0_u32; 1_536]; usize::from(channels)];
        for delivery in 2_u32..=4 {
            let transaction = CallbackLineageIdentity::new(
                prepared.programme_id().clone(),
                attempt,
                FrontendCallbackIndex::new(delivery),
            );
            let receipt = after
                .fresh_zero_view(transaction)
                .unwrap()
                .validate_zero_physical_input_rows(&zero_rows)
                .unwrap();
            assert_eq!(
                receipt
                    .source_origin_transaction_id()
                    .callback_index()
                    .value(),
                1
            );
            assert_eq!(
                receipt.delivery_transaction_id().callback_index().value(),
                delivery
            );
        }
    }
}

#[test]
fn dynamic_f01e_owner_installs_36_and_38_rows_and_keeps_three_meter_drains() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "adm"
        })
        .unwrap();
    let provider = validated_f01e_provider();

    for channels in [36_u16, 38] {
        let (resource, spans) = selected_resource_with_channel_samples(channels, 1_537);
        let prepared = prepare_selected_job_input(
            test_programme_id(),
            case["input_xml"].as_str().unwrap().as_bytes(),
            SelectedWaveDescriptor::new(48_000, channels, 24),
            1_537,
            Arc::clone(&resource),
            metadata_payloads(&resource, &spans),
        )
        .unwrap()
        .bind_primary_control_rows(1_537 * u64::from(channels) * 3)
        .unwrap();
        let attempt = AttemptId::new(0);
        let owner = prepared.f01e_cold_owner(&provider, attempt).unwrap();
        let prepared_normal = prepared.normal_control_adm_input_snapshot(&owner).unwrap();
        let first = prepared.propose_f01e_adm_normal_callback(&owner).unwrap();
        let normal_output = first.output(Arc::clone(&prepared_normal));
        let profile_identity = resolve_selected_profile(&EncodingProfile::selected()).unwrap();
        let source_projection = project_normal_control_adm_input_output_source_controls(
            &normal_output,
            prepared.graph().graph(),
            profile_identity,
        )
        .unwrap();
        assert_eq!(
            source_projection.route().regular_hosts().len(),
            usize::from(channels - 1)
        );
        assert_eq!(
            source_projection.controls().rows().len(),
            usize::from(channels - 1)
        );
        let source_map = build_meter_installed48_source_map(
            prepared.primary_metadata_controls().unwrap(),
            &generate_selected_p11_geometry(0x111b, true).unwrap(),
        )
        .unwrap();
        let source_geometry = build_global_source_geometry_snapshot_from_adm_input_output(
            prepared.graph().graph(),
            &source_map,
            &normal_output,
        )
        .unwrap();
        assert_eq!(source_geometry.rows().len(), usize::from(channels) + 10);

        if channels == 38 {
            let fixed_provider = provider
                .for_admitted_adm_input(&prepared.configuration(), prepared.graph().graph())
                .unwrap();
            let fixed_machine = FrontendControlMachine::from_provider(
                fixed_provider,
                prepared.programme_id().clone(),
                attempt,
            )
            .unwrap();
            let fixed_snapshot = prepared
                .normal_control_prepared_snapshot(fixed_machine.committed())
                .unwrap();
            let fixed_before = NormalControlBeforeState::from_frontend_state(
                fixed_machine.committed(),
                fixed_machine.generation(),
            )
            .unwrap();
            let fixed = propose_normal_control_snapshot(
                &fixed_before,
                &fixed_snapshot,
                CallbackLineageIdentity::new(
                    prepared.programme_id().clone(),
                    attempt,
                    FrontendCallbackIndex::new(0),
                ),
            )
            .unwrap();
            assert_eq!(first.after_state().physical_rows(), fixed.control38_rows());
            assert_eq!(first.after_state().regular_rows(), fixed.regular37_rows());
            assert_eq!(
                first.after_state().control_words(),
                fixed.control38_carrier_words()
            );
            assert_eq!(
                first.after_state().regular_control_words(),
                fixed.regular37_carrier_words()
            );
        }

        let staged = owner
            .machine()
            .stage_normal_adm_input(Arc::clone(&prepared_normal), first)
            .unwrap();
        let after_first = staged.after_state();
        assert_eq!(after_first.schema(), "F01E_ADM_INPUT_NORMAL_STATE_V1");
        assert_eq!(after_first.callback_index().value(), 1);
        assert_eq!(after_first.normal_callbacks().value(), 1);
        assert_eq!(
            after_first
                .dynamic_after_state()
                .unwrap()
                .input_physical_rows()
                .value(),
            channels
        );
        assert_eq!(
            after_first
                .dynamic_after_state()
                .unwrap()
                .regular_control_rows()
                .value(),
            channels - 1
        );
        let canonical: Value = serde_json::from_slice(after_first.canonical_bytes()).unwrap();
        assert_eq!(
            canonical["input_physical_rows"].as_u64(),
            Some(u64::from(channels))
        );
        assert_eq!(
            canonical["normal_control_rows"].as_u64(),
            Some(u64::from(channels))
        );
        assert_eq!(
            canonical["regular_control_rows"].as_u64(),
            Some(u64::from(channels - 1))
        );
        assert_eq!(
            canonical["last_control_rows"].as_array().unwrap().len(),
            usize::from(channels)
        );
        assert_eq!(
            canonical["last_regular_rows"].as_array().unwrap().len(),
            usize::from(channels - 1)
        );

        let (machine_after_first, _) = owner
            .machine()
            .replacement_candidate(
                staged,
                CallbackTransactionId::new("f01e-dynamic-normal-g0").unwrap(),
            )
            .unwrap()
            .into_parts();
        assert_eq!(machine_after_first.generation(), 1);
        let installed_prepared = Arc::clone(
            machine_after_first
                .committed()
                .dynamic_prepared_identity()
                .unwrap(),
        );
        let installed_after = machine_after_first
            .committed()
            .dynamic_after_state()
            .unwrap();

        let rows = vec![vec![0_u32; 1_536]; usize::from(channels)];
        let second_frame = MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
            callback_index: MeterCallbackIndex::new(1),
            upstream_generation: MeterUpstreamGeneration::new(2),
            phase: MeterCallbackPhase::NormalSource,
            source_identity: MeterSourceIdentity::from_sha256(
                prepared.resource_identity().input_sha256(),
            ),
            source_samples: MeterSourceSampleCount::new(1_537).unwrap(),
            normal_callbacks: MeterCallbackCount::new(2),
            source_cursor_before: MeterSourcePosition::new(1_536),
            source_cursor_after: MeterSourcePosition::new(1_537),
            source_read_index_before: MeterSourceReadIndex::new(1),
            source_read_index_after: MeterSourceReadIndex::new(2),
            host_input_count: MeterSampleCount::new(1),
            source_padding_count: MeterSampleCount::new(1_535),
            rows: &rows,
        });
        let second_view = MeterSourceCallbackView::new(&prepared, second_frame).unwrap();
        let second = prepared
            .propose_f01e_adm_normal_continuation(
                &installed_prepared,
                installed_after,
                &second_view,
            )
            .unwrap();
        let staged_second = machine_after_first
            .stage_normal_adm_input_continuation(installed_prepared, second)
            .unwrap();
        let (machine_after_second, _) = machine_after_first
            .replacement_candidate(
                staged_second,
                CallbackTransactionId::new("f01e-dynamic-normal-g1").unwrap(),
            )
            .unwrap()
            .into_parts();
        assert_eq!(machine_after_second.generation(), 2);
        assert_eq!(machine_after_second.committed().callback_index().value(), 2);

        let zero_rows = vec![vec![0_u32; 1_536]; usize::from(channels)];
        let after_two_normals = machine_after_second
            .committed()
            .dynamic_after_state()
            .unwrap();
        for delivery in 2_u32..=4 {
            let transaction = CallbackLineageIdentity::new(
                prepared.programme_id().clone(),
                attempt,
                FrontendCallbackIndex::new(delivery),
            );
            let receipt = after_two_normals
                .fresh_zero_view(transaction)
                .unwrap()
                .validate_zero_physical_input_rows(&zero_rows)
                .unwrap();
            assert_eq!(receipt.input_physical_rows().value(), channels);
            assert_eq!(receipt.normal_control_rows().value(), channels);
            assert_eq!(
                receipt
                    .source_origin_transaction_id()
                    .callback_index()
                    .value(),
                1
            );
            assert_eq!(
                receipt.delivery_transaction_id().callback_index().value(),
                delivery
            );
        }
    }
}

#[test]
fn selected_meter_two_normal_and_three_drain_callbacks_complete_for_36_rows() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "adm"
        })
        .unwrap();
    let programme_id = test_programme_id();
    let attempt = AttemptId::new(0);
    let source_samples = 3_072_u64;
    let normal_callbacks = 2_u64;
    let (resource, spans) = selected_resource_with_channel_samples(36, source_samples);
    let prepared = prepare_selected_job_input(
        programme_id.clone(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(48_000, 36, 24),
        source_samples,
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap()
    .bind_primary_control_rows(source_samples * 36 * 3)
    .unwrap();
    let source_samples = MeterSourceSampleCount::new(source_samples).unwrap();
    let meter_before = selected_meter_root(&prepared, source_samples);
    let zero_rows = vec![vec![0_u32; 1_536]; 36];
    let g0_frame = MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
        callback_index: MeterCallbackIndex::new(0),
        upstream_generation: MeterUpstreamGeneration::new(1),
        phase: MeterCallbackPhase::NormalSource,
        source_identity: MeterSourceIdentity::from_sha256(
            prepared.resource_identity().input_sha256(),
        ),
        source_samples,
        normal_callbacks: MeterCallbackCount::new(normal_callbacks),
        source_cursor_before: MeterSourcePosition::new(0),
        source_cursor_after: MeterSourcePosition::new(1_536),
        source_read_index_before: MeterSourceReadIndex::new(0),
        source_read_index_after: MeterSourceReadIndex::new(1),
        host_input_count: MeterSampleCount::new(1_536),
        source_padding_count: MeterSampleCount::new(0),
        rows: &zero_rows,
    });
    let g0_view = MeterSourceCallbackView::new(&prepared, g0_frame).unwrap();
    let g0_binding = MeterCallbackGtcBinding::new(
        CallbackTransactionId::new("synthetic-36-meter-g0").unwrap(),
        programme_id.clone(),
        attempt,
        MeterCallbackIndex::new(0),
    );
    let providers = independent_meter_g0_providers();
    let source_map = build_selected_installed48_source_map(
        prepared.primary_metadata_controls().unwrap(),
        &providers.p11_geometry,
    )
    .unwrap();
    let cold_frontend = prepared.f01e_cold_owner(&providers.f01e, attempt).unwrap();
    let cold_spatial = SpatialControlMachine::new_cold_for_source_map(
        EncodingProfile::selected(),
        cold_frontend.provider(),
        &providers.f03_target,
        &source_map,
    )
    .unwrap();
    assert_eq!(source_map.regular_count(), 35);
    assert_eq!(
        cold_spatial.committed().regular_raw_history_bits().len(),
        35 * 1_536
    );
    let mut candidate = stage_independent_meter_g0_callback(
        &prepared,
        g0_view,
        &meter_before,
        g0_binding,
        MeterKFilterExponentState::new([0; 6]),
        &providers,
    )
    .unwrap();
    assert_eq!(
        candidate.frontend_candidate().receipt().after_generation(),
        1
    );

    let normal_frame = MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
        callback_index: MeterCallbackIndex::new(1),
        upstream_generation: MeterUpstreamGeneration::new(2),
        phase: MeterCallbackPhase::NormalSource,
        source_identity: MeterSourceIdentity::from_sha256(
            prepared.resource_identity().input_sha256(),
        ),
        source_samples,
        normal_callbacks: MeterCallbackCount::new(normal_callbacks),
        source_cursor_before: MeterSourcePosition::new(1_536),
        source_cursor_after: MeterSourcePosition::new(3_072),
        source_read_index_before: MeterSourceReadIndex::new(1),
        source_read_index_after: MeterSourceReadIndex::new(2),
        host_input_count: MeterSampleCount::new(1_536),
        source_padding_count: MeterSampleCount::new(0),
        rows: &zero_rows,
    });
    let normal_view = MeterSourceCallbackView::new(&prepared, normal_frame).unwrap();
    let normal_binding = MeterCallbackGtcBinding::new(
        CallbackTransactionId::new("synthetic-36-meter-normal-1").unwrap(),
        programme_id.clone(),
        attempt,
        MeterCallbackIndex::new(1),
    );
    let k_filter_exponents = MeterKFilterExponentState::new(
        *candidate
            .meter_proposal()
            .after_root()
            .k_filter_exponents_i32_by_channel(),
    );
    candidate = stage_independent_meter_normal_callback(
        &prepared,
        normal_view,
        candidate,
        normal_binding,
        k_filter_exponents,
        &providers,
    )
    .unwrap();
    assert_eq!(candidate.source_receipt().callback_index().value(), 1);
    assert_eq!(candidate.installed48_after().callback_index().value(), 2);
    assert_eq!(
        candidate.frontend_candidate().next_machine().generation(),
        2
    );

    for callback in 2_u32..=4 {
        let frame = MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
            callback_index: MeterCallbackIndex::new(u64::from(callback)),
            upstream_generation: MeterUpstreamGeneration::new(u64::from(callback) + 1),
            phase: MeterCallbackPhase::ZeroInputDrain,
            source_identity: MeterSourceIdentity::from_sha256(
                prepared.resource_identity().input_sha256(),
            ),
            source_samples,
            normal_callbacks: MeterCallbackCount::new(normal_callbacks),
            source_cursor_before: MeterSourcePosition::new(3_072),
            source_cursor_after: MeterSourcePosition::new(3_072),
            source_read_index_before: MeterSourceReadIndex::new(2),
            source_read_index_after: MeterSourceReadIndex::new(2),
            host_input_count: MeterSampleCount::new(0),
            source_padding_count: MeterSampleCount::new(1_536),
            rows: &zero_rows,
        });
        let view = MeterSourceCallbackView::new(&prepared, frame).unwrap();
        let binding = MeterCallbackGtcBinding::new(
            CallbackTransactionId::new(&format!("synthetic-36-meter-drain-{callback}")).unwrap(),
            programme_id.clone(),
            attempt,
            MeterCallbackIndex::new(u64::from(callback)),
        );
        let exponents = MeterKFilterExponentState::new(
            *candidate
                .meter_proposal()
                .after_root()
                .k_filter_exponents_i32_by_channel(),
        );
        candidate = stage_independent_meter_fresh_zero_drain_callback(
            &prepared, view, candidate, binding, exponents, &providers,
        )
        .unwrap();
        assert_eq!(
            candidate.source_receipt().callback_index().value(),
            u64::from(callback)
        );
        assert_eq!(
            candidate.installed48_after().callback_index().value(),
            u64::from(callback) + 1
        );
    }

    assert_eq!(
        candidate
            .meter_proposal()
            .gtc_receipt()
            .unwrap()
            .after_generation(),
        5
    );
    assert_eq!(
        candidate.frontend_candidate().next_machine().generation(),
        2
    );
}

#[test]
fn main_spatial_callback_stages_dynamic_36_row_controls() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "adm"
        })
        .unwrap();
    let source_samples = 3_072_u64;
    let (resource, spans) = selected_resource_with_channel_samples(36, source_samples);
    let prepared = prepare_selected_job_input(
        test_programme_id(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(48_000, 36, 24),
        source_samples,
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap()
    .bind_primary_control_rows(source_samples * 36 * 3)
    .unwrap();
    let providers = independent_meter_g0_providers();
    let source_map = build_selected_installed48_source_map(
        prepared.primary_metadata_controls().unwrap(),
        &providers.p11_geometry,
    )
    .unwrap();
    let owner = prepared
        .f01e_cold_owner(&providers.f01e, AttemptId::new(0))
        .unwrap();
    let spatial = SpatialControlMachine::new_cold_for_source_map(
        EncodingProfile::selected(),
        owner.provider(),
        &providers.f03_target,
        &source_map,
    )
    .unwrap();
    let prepared_normal = prepared.normal_control_adm_input_snapshot(&owner).unwrap();
    let normal = prepared.propose_f01e_adm_normal_callback(&owner).unwrap();
    let controls: NormalControlAdmInputOutput = normal.output(prepared_normal);
    let physical_pcm_rows = vec![vec![0_u32; 1_536]; 36];

    let proposal = stage_spatial_control_installed48(
        &spatial,
        SpatialControlInstalled48Input {
            callback: SpatialControlInstalled48Callback::NormalAdmInputOutput {
                controls: &controls,
                physical_pcm_rows: &physical_pcm_rows,
            },
            source_map: &source_map,
            diffusion_parameters: Some(&providers.diffusion),
        },
    )
    .unwrap();

    assert_eq!(source_map.physical_host_count(), 36);
    assert_eq!(source_map.regular_count(), 35);
    assert_eq!(source_map.source_count(), 46);
    assert_eq!(proposal.quiet_rows(), &[1; 35]);
    let (_, after) = proposal.into_parts();
    let after = after.into_parts();
    assert_eq!(after.source_pcm_rows.len(), 36);
    assert_eq!(after.regular_raw_history_bits.len(), 35 * 1_536);
    assert_eq!(after.current_rows_bits.len(), 46 * 1_536);

    let spatial_audio = SpatialAudioMachine::new_cold(EncodingProfile::selected()).unwrap();
    let spatial_callback = stage_main_spatial_callback(MainSpatialCallbackInput {
        prepared: &prepared,
        controls: MainNormalControlOutput::AdmInput(controls),
        physical_pcm_rows: &physical_pcm_rows,
        spatial_control: &spatial,
        spatial_audio: &spatial_audio,
        p11_geometry: &providers.p11_geometry,
        p15_geometry: &providers.p15_geometry,
        diffusion_parameters: &providers.diffusion,
        feature_processor: &providers.feature_processor,
        feature_fft: &providers.feature_fft,
        window: &providers.window,
        component_parameters: &providers.component_parameters,
        spatial_audio_providers: None,
    })
    .unwrap();
    assert_eq!(spatial_callback.callback_index().value(), 0);
    assert_eq!(spatial.generation(), 0);
}

#[test]
fn main_dynamic_row_shapes_share_f01e_spatial_and_reader_eof_owners() {
    for channels in [10_u16, 20, 35, 36, 37, 38] {
        main_dynamic_schedule_for_channels(channels, 1_536);
    }
    for channels in [36_u16, 38] {
        main_dynamic_schedule_for_channels(channels, 3_072);
    }
    assert_eq!(
        SelectedSourceRowDimensions::for_physical_rows(39),
        Err(SelectedSourceRowShapeError::SourceAxisCapacityExceeded {
            required: 49,
            capacity: 48,
        })
    );
}

fn main_dynamic_schedule_for_channels(channels: u16, source_samples: u64) {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "adm"
        })
        .unwrap();
    let (resource, spans) = selected_resource_with_channel_samples(channels, source_samples);
    let prepared = prepare_selected_job_input(
        test_programme_id(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(48_000, channels, 24),
        source_samples,
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap()
    .bind_primary_control_rows(source_samples * u64::from(channels) * 3)
    .unwrap();
    let providers = independent_meter_g0_providers();
    let dimensions = SelectedSourceRowDimensions::for_physical_rows(channels).unwrap();
    let physical_rows = usize::from(dimensions.physical_row_count());
    let regular_rows = usize::from(dimensions.regular_row_count());
    let active_sources = usize::from(dimensions.active_source_count());
    let roots = MainSpatialColdRoots::from_selected_providers(
        &prepared,
        AttemptId::new(0),
        &providers.f01e,
        &providers.f03_target,
        &providers.p11_geometry,
    )
    .unwrap();

    assert_eq!(
        roots.frontend_control().committed().schema(),
        "F01E_ADM_INPUT_COLD_PROJECTION_V1"
    );
    assert_eq!(roots.frontend_control().generation(), 0);
    assert_eq!(roots.spatial_control().generation(), 0);
    assert_eq!(
        roots.frontend_control().committed().objects().len(),
        physical_rows - 10
    );
    assert_eq!(
        roots.spatial_control().committed().physical_row_count(),
        physical_rows
    );
    assert_eq!(
        roots.spatial_control().committed().regular_row_count(),
        regular_rows
    );
    assert_eq!(
        roots
            .spatial_control()
            .committed()
            .active_source_ids()
            .len(),
        active_sources
    );
    assert_eq!(
        roots
            .spatial_control()
            .committed()
            .active_source_ids()
            .iter()
            .map(|source| source.value())
            .collect::<Vec<_>>(),
        (0..u64::try_from(active_sources).unwrap()).collect::<Vec<_>>()
    );
    assert_eq!(
        roots
            .spatial_control()
            .committed()
            .target_current_xyz_u32()
            .len(),
        45
    );
    assert_eq!(
        roots
            .spatial_control()
            .committed()
            .previous_pcm_bits()
            .len(),
        physical_rows * 1_536
    );
    assert_eq!(
        roots
            .spatial_control()
            .committed()
            .regular_raw_history_bits()
            .len(),
        regular_rows * 1_536
    );
    let prepared_normal = roots.normal_control_prepared();
    assert_eq!(
        prepared_normal.schema(),
        "F01E_ADM_INPUT_NORMAL_PREFLIGHT_V1"
    );
    assert_eq!(prepared_normal.physical_tracks().len(), physical_rows);
    assert_eq!(
        prepared_normal.regular_source_bindings().len(),
        regular_rows
    );
    assert_eq!(
        prepared_normal.object_owner_bindings().len(),
        physical_rows - 10
    );

    let attempt = AttemptId::new(0);
    let physical_pcm_rows = vec![vec![0_u32; 1_536]; physical_rows];
    let reader_eof = ReaderEofMachine::cold();
    let transaction_g0 = CallbackTransactionId::new("synthetic-main-adm-g0").unwrap();
    let frontend_g0 = stage_main_frontend_adm_input_snapshot(
        &prepared,
        roots.frontend_control(),
        prepared_normal,
        attempt,
        transaction_g0.clone(),
    )
    .unwrap();
    let (frontend_candidate_g0, controls_g0) = frontend_g0.into_parts();
    assert_eq!(controls_g0.physical_rows().len(), physical_rows);
    assert_eq!(controls_g0.regular_rows().len(), regular_rows);
    assert_eq!(controls_g0.normal_callbacks().value(), 1);
    assert_eq!(
        frontend_candidate_g0.next_machine().committed().schema(),
        "F01E_ADM_INPUT_NORMAL_STATE_V1"
    );
    let g0_clock = ReaderEofNormalCallbackClock::new(
        FrontendCallbackIndex::new(0),
        AdmSampleIndex::new(0),
        ReaderEofActualSampleCount::new(1_536).unwrap(),
        ReaderEofPaddingSampleCount::new(0).unwrap(),
        NormalCallbackCount::new(1),
    )
    .unwrap();
    let reader_g0 = stage_reader_eof_adm_input_normal_callback(
        &reader_eof,
        g0_clock,
        &physical_pcm_rows,
        &controls_g0,
        transaction_g0.clone(),
    )
    .unwrap();
    let (reader_after_g0, _, _) = reader_g0.into_parts();
    assert_eq!(
        reader_after_g0
            .committed()
            .last_normal_host_frame()
            .unwrap()
            .physical_rows(),
        u16::try_from(physical_rows).unwrap()
    );
    assert_eq!(
        reader_after_g0
            .committed()
            .last_control_snapshot()
            .unwrap()
            .words()
            .len(),
        physical_rows * 13
    );
    let spatial_audio = roots.spatial_audio();
    let spatial_g0 = stage_main_spatial_callback(MainSpatialCallbackInput {
        prepared: &prepared,
        controls: MainNormalControlOutput::AdmInput(controls_g0),
        physical_pcm_rows: &physical_pcm_rows,
        spatial_control: roots.spatial_control(),
        spatial_audio,
        p11_geometry: &providers.p11_geometry,
        p15_geometry: &providers.p15_geometry,
        diffusion_parameters: &providers.diffusion,
        feature_processor: &providers.feature_processor,
        feature_fft: &providers.feature_fft,
        window: &providers.window,
        component_parameters: &providers.component_parameters,
        spatial_audio_providers: None,
    })
    .unwrap();
    let parent_g0 = roots
        .prepare_spatial_after_callback(
            &prepared,
            frontend_candidate_g0,
            spatial_g0,
            transaction_g0,
        )
        .unwrap();
    let (mut main_after_normal, _, _, _) = parent_g0.into_parts();
    let mut reader_after_normal = reader_after_g0;
    let normal_callback_count = source_samples.div_ceil(1_536);
    assert_eq!(main_after_normal.frontend_control().generation(), 1);
    assert_eq!(main_after_normal.spatial_control().generation(), 1);
    assert_eq!(main_after_normal.spatial_audio().generation(), 1);
    assert!(Arc::ptr_eq(
        prepared_normal,
        main_after_normal.normal_control_prepared()
    ));
    assert_eq!(
        main_after_normal
            .spatial_control()
            .committed()
            .previous_current_rows_bits()
            .len(),
        active_sources * 1_536
    );

    if normal_callback_count > 1 {
        let transaction_g1 = CallbackTransactionId::new("synthetic-main-adm-g1").unwrap();
        let frontend_g1 = stage_main_frontend_adm_input_snapshot(
            &prepared,
            main_after_normal.frontend_control(),
            main_after_normal.normal_control_prepared(),
            attempt,
            transaction_g1.clone(),
        )
        .unwrap();
        let (frontend_candidate_g1, controls_g1) = frontend_g1.into_parts();
        assert_eq!(controls_g1.transaction_id().callback_index().value(), 1);
        assert_eq!(controls_g1.physical_rows().len(), physical_rows);
        assert_eq!(controls_g1.regular_rows().len(), regular_rows);
        assert_eq!(frontend_candidate_g1.next_machine().generation(), 2);
        let g1_clock = ReaderEofNormalCallbackClock::new(
            FrontendCallbackIndex::new(1),
            AdmSampleIndex::new(1_536),
            ReaderEofActualSampleCount::new(1_536).unwrap(),
            ReaderEofPaddingSampleCount::new(0).unwrap(),
            NormalCallbackCount::new(2),
        )
        .unwrap();
        let reader_g1 = stage_reader_eof_adm_input_normal_callback(
            &reader_after_normal,
            g1_clock,
            &physical_pcm_rows,
            &controls_g1,
            transaction_g1.clone(),
        )
        .unwrap();
        let (reader_after_g1, _, _) = reader_g1.into_parts();
        let spatial_g1 = stage_main_spatial_callback(MainSpatialCallbackInput {
            prepared: &prepared,
            controls: MainNormalControlOutput::AdmInput(controls_g1),
            physical_pcm_rows: &physical_pcm_rows,
            spatial_control: main_after_normal.spatial_control(),
            spatial_audio: main_after_normal.spatial_audio(),
            p11_geometry: &providers.p11_geometry,
            p15_geometry: &providers.p15_geometry,
            diffusion_parameters: &providers.diffusion,
            feature_processor: &providers.feature_processor,
            feature_fft: &providers.feature_fft,
            window: &providers.window,
            component_parameters: &providers.component_parameters,
            spatial_audio_providers: None,
        })
        .unwrap();
        let parent_g1 = main_after_normal
            .prepare_spatial_after_callback(
                &prepared,
                frontend_candidate_g1,
                spatial_g1,
                transaction_g1,
            )
            .unwrap();
        (main_after_normal, _, _, _) = parent_g1.into_parts();
        reader_after_normal = reader_after_g1;
        assert_eq!(main_after_normal.frontend_control().generation(), 2);
        assert_eq!(main_after_normal.spatial_control().generation(), 2);
        assert_eq!(main_after_normal.spatial_audio().generation(), 2);
    }
    assert_eq!(
        reader_after_normal.committed().normal_callbacks().value(),
        normal_callback_count
    );
    assert_eq!(
        main_after_normal.normal_control_prepared().schema(),
        "F01E_ADM_INPUT_NORMAL_PREFLIGHT_V1"
    );
    let spatial_audio_provider_bundle = selected_spatial_audio_provider_bundle();
    let mut source_producing_drain_count = 0_u32;
    for drain_offset in 0..4_u32 {
        let callback_value = u32::try_from(normal_callback_count).unwrap() + drain_offset;
        let callback_index = FrontendCallbackIndex::new(callback_value);
        if callback_value >= 2 {
            source_producing_drain_count += 1;
        }
        let transaction_id = CallbackTransactionId::new(&format!(
            "jocforge-main-callback-g{}",
            callback_index.value()
        ))
        .unwrap();
        let delivery =
            CallbackLineageIdentity::new(prepared.programme_id().clone(), attempt, callback_index);
        let held_controls = main_after_normal
            .frontend_control()
            .committed()
            .dynamic_after_state()
            .unwrap()
            .held_source_view_for_cached_callback(delivery)
            .unwrap();
        let spatial = stage_main_spatial_cached_callback(MainSpatialCachedCallbackInput {
            prepared: &prepared,
            controls: MainHeldControlOutput::AdmInput(held_controls),
            spatial_control: main_after_normal.spatial_control(),
            spatial_audio: main_after_normal.spatial_audio(),
            p11_geometry: &providers.p11_geometry,
            p15_geometry: &providers.p15_geometry,
            diffusion_parameters: &providers.diffusion,
            feature_processor: &providers.feature_processor,
            feature_fft: &providers.feature_fft,
            window: &providers.window,
            component_parameters: &providers.component_parameters,
            spatial_audio_providers: Some(&spatial_audio_provider_bundle),
        })
        .unwrap();
        let main_drain = main_after_normal
            .prepare_spatial_after_cached_callback(&prepared, spatial, transaction_id.clone())
            .unwrap();
        (main_after_normal, _, _, _) = main_drain.into_parts();
        let drain = stage_reader_eof_cached_drain(
            &reader_after_normal,
            callback_index,
            NormalCallbackCount::new(normal_callback_count),
            transaction_id,
        )
        .unwrap();
        (reader_after_normal, _, _) = drain.into_parts();
        assert_eq!(
            reader_after_normal
                .committed()
                .last_normal_host_frame()
                .unwrap()
                .physical_rows(),
            u16::try_from(physical_rows).unwrap()
        );
        assert_eq!(
            reader_after_normal
                .committed()
                .last_control_snapshot()
                .unwrap()
                .words()
                .len(),
            physical_rows * 13
        );
    }
    assert_eq!(
        source_producing_drain_count,
        if normal_callback_count == 1 { 3 } else { 4 }
    );
    assert_eq!(
        reader_after_normal.committed().source_cursor_samples(),
        source_samples
    );
    assert_eq!(reader_after_normal.committed().remaining_cached_drains(), 0);
    assert_eq!(
        main_after_normal.frontend_control().generation(),
        normal_callback_count
    );
    let terminal_callback_index = u32::try_from(normal_callback_count + 4).unwrap();
    assert_eq!(
        main_after_normal
            .spatial_control()
            .committed()
            .callback_index()
            .value(),
        terminal_callback_index
    );
    assert_eq!(
        main_after_normal
            .spatial_audio()
            .committed()
            .callback_index()
            .value(),
        terminal_callback_index
    );
    assert_eq!(
        main_after_normal
            .spatial_control()
            .committed()
            .source_cursor_samples()
            .value(),
        source_samples
    );
    assert_eq!(
        main_after_normal
            .spatial_control()
            .committed()
            .remaining_cached_drains(),
        0
    );
    assert_eq!(
        main_after_normal
            .spatial_control()
            .committed()
            .active_source_ids()
            .len(),
        active_sources
    );
}

fn selected_format_payload(channels: u16) -> Vec<u8> {
    let block_align = channels * 3;
    let mut format = Vec::with_capacity(16);
    format.extend_from_slice(&1_u16.to_le_bytes());
    format.extend_from_slice(&channels.to_le_bytes());
    format.extend_from_slice(&48_000_u32.to_le_bytes());
    format.extend_from_slice(&(48_000_u32 * u32::from(block_align)).to_le_bytes());
    format.extend_from_slice(&block_align.to_le_bytes());
    format.extend_from_slice(&24_u16.to_le_bytes());
    format
}

fn append_chunk(bytes: &mut Vec<u8>, chunk_id: [u8; 4], payload: &[u8]) -> Range<usize> {
    bytes.extend_from_slice(&chunk_id);
    bytes.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_le_bytes());
    let start = bytes.len();
    bytes.extend_from_slice(payload);
    let end = bytes.len();
    if payload.len() % 2 != 0 {
        bytes.push(0);
    }
    start..end
}

fn metadata_payloads<'a>(
    resource: &'a [u8],
    spans: &'a [Range<usize>; 3],
) -> SelectedAdmResourcePayloads<'a> {
    SelectedAdmResourcePayloads::new(
        &resource[spans[0].clone()],
        &resource[spans[1].clone()],
        &resource[spans[2].clone()],
    )
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
