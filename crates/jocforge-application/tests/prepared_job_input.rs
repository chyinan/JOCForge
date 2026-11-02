// pattern: Functional Core
use jocforge_application::{
    PreparedJobInputError, SelectedAdmResourcePayloads,
    meter_source_admission::{MeterSourceAdmissionError, admit_meter_source_owner},
    meter_source_bridge::{
        MeterCallbackCount, MeterCallbackIndex, MeterCallbackPhase, MeterSampleCount,
        MeterSourceCallbackFrame, MeterSourceCallbackFrameParts, MeterSourceCallbackView,
        MeterSourceIdentity, MeterSourcePosition, MeterSourceReadIndex, MeterSourceSampleCount,
        MeterUpstreamGeneration,
    },
    prepare_selected_job_input,
};
use jocforge_domain::frontend::adm_file_storage_map::AdmFileStorageTrackSlot;
use jocforge_domain::frontend::dbmd_sections::parse_selected_dbmd_sections;
use jocforge_domain::frontend::selected_adm_graph::SelectedTrackKind;
use jocforge_domain::frontend::{
    FrontendControlMachine, PackageManifestEvidence, ProviderFileEvidence,
    job_configuration::SelectedWaveDescriptor,
    normal_control_snapshot::{NormalControlRow, NormalControlSnapshotError},
    validate_f01e_package_manifest, validate_f01e_provider,
};
use jocforge_domain::timeline::FrontendCallbackIndex;
use jocforge_domain::transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId};
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
fn f01e_cold_provider_projects_36_and_38_adm_rows_from_existing_defaults() {
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

    for (channels, expected_objects) in [(36_u16, 26_u64), (38, 28)] {
        let (resource, spans) = selected_resource_with_channels(channels);
        let prepared = prepare_selected_job_input(
            test_programme_id(),
            case["input_xml"].as_str().unwrap().as_bytes(),
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
            .unwrap();
            assert_eq!(
                same_content_prepared.normal_control_adm_input_snapshot(&owner),
                Err(PreparedJobInputError::PreparedIdentityMismatch)
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

fn selected_resource() -> (Arc<[u8]>, [Range<usize>; 3]) {
    selected_resource_with_channels(38)
}

fn selected_resource_with_channels(channels: u16) -> (Arc<[u8]>, [Range<usize>; 3]) {
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
    append_chunk(&mut bytes, *b"data", &vec![0_u8; track_count * 3]);
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
    assert_eq!(channels, 36);
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
