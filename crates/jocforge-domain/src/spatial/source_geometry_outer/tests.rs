use super::*;
use crate::{
    frontend::object_control::ObjectControl52,
    frontend::{
        FrontendControlMachine, PackageManifestEvidence, ProviderFileEvidence,
        dbmd_sections::parse_selected_dbmd_sections,
        job_configuration::{
            PrimaryMetadataChunkHashes, SelectedPrimaryRuntimeIdentity, SelectedWaveDescriptor,
            bind_admitted_selected_input, parse_selected_job_configuration,
        },
        normal_control_snapshot::{
            NormalControlBeforeState, NormalControlPreparedSnapshot, NormalControlSnapshotError,
            propose_normal_control_snapshot,
        },
        primary_metadata_control::{
            PrimaryMetadataControlRows, produce_primary_metadata_control_rows,
        },
        selected_adm_graph::parse_selected_adm_graph,
        validate_f01e_package_manifest, validate_f01e_provider,
    },
    spatial::{
        installed48_source_map::build_meter_installed48_source_map,
        point_response::{
            PointResponseGateInput, PointResponseSourceInput, PointResponseSourceSnapshot,
            prepare_point_response_sources, produce_point_response_columns,
        },
        selected_p11_geometry::generate_selected_p11_geometry,
        selected_p15_geometry::{SelectedP15Geometry, generate_selected_p15_geometry},
        source_flags::{
            SourceControlSnapshotClock, project_held_normal_control_snapshot_source_controls,
            project_normal_control_snapshot_source_controls,
        },
        target_context::build_target_context_projection,
        target_slot_install::{SelectedTargetSlots, TargetSlotState, install_target_slots},
    },
    timeline::{AdmSampleCount, CodecSourceIndex, FrontendCallbackIndex, NormalCallbackCount},
    transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const AXML: &[u8] =
    include_bytes!("../../../tests/fixtures/selected_adm_graph/source_metadata.xml");
const CHNA: &[u8] =
    include_bytes!("../../../tests/fixtures/selected_adm_graph/source_track_associations.bin");
const DBMD: &[u8] =
    include_bytes!("../../../tests/fixtures/selected_adm_graph/source_metadata.dbmd");
const JOB_CONFIGURATION_VECTORS: &[u8] =
    include_bytes!("../../../tests/fixtures/job_configuration/job-configuration-vectors.json");
const F01E_MANIFEST: &[u8] = include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
const F01E_COLD_PROVIDER: &[u8] =
    include_bytes!("../../../tests/fixtures/f01e_provider/f01e-selected-constructor-v1.json");
const PRIMARY_INPUT_IDENTITY: &[u8] =
    include_bytes!("../../../tests/fixtures/primary_installed48/PRIMARY-INPUT-IDENTITY.json");

pub(in crate::spatial) struct F01bF01eFixture {
    pub(in crate::spatial) graph: SelectedAdmGraph,
    pub(in crate::spatial) source_map: Installed48SourceMap,
    pub(in crate::spatial) primary_controls: PrimaryMetadataControlRows,
    pub(in crate::spatial) normal: NormalControlSnapshotProposal,
}

pub(in crate::spatial) fn f01b_f01e_fixture() -> F01bF01eFixture {
    let vectors: Value = serde_json::from_slice(JOB_CONFIGURATION_VECTORS).unwrap();
    let case = &vectors["config_cases"][0];
    let descriptor = &case["pcm_descriptor"];
    let xml = case["input_xml"].as_str().unwrap().as_bytes();
    let source_samples = descriptor["source_samples"].as_u64().unwrap();
    let wave = SelectedWaveDescriptor::new(
        u32::try_from(descriptor["sample_rate"].as_u64().unwrap()).unwrap(),
        u16::try_from(descriptor["channels"].as_u64().unwrap()).unwrap(),
        u16::try_from(descriptor["pcm_bits"].as_u64().unwrap()).unwrap(),
    );
    let configuration = parse_selected_job_configuration(xml, wave, source_samples).unwrap();
    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let dbmd = parse_selected_dbmd_sections(DBMD).unwrap();
    let metadata_hashes = PrimaryMetadataChunkHashes::new(
        *graph.receipt().axml_sha256(),
        *graph.receipt().chna_sha256(),
        *dbmd.payload_sha256(),
    );
    let primary_identity: Value = serde_json::from_slice(PRIMARY_INPUT_IDENTITY).unwrap();
    let primary_format = &primary_identity["primary_source_format"];
    let mut metadata_identity = b"target-context-test-metadata-identity-v1".to_vec();
    metadata_identity.extend_from_slice(AXML);
    metadata_identity.extend_from_slice(CHNA);
    metadata_identity.extend_from_slice(DBMD);
    let job_xml_sha256: [u8; 32] = Sha256::digest(xml).into();
    let job_xml_sha256_hex = format!("{:x}", Sha256::digest(xml));
    let config_identity = json!({
        "schema": "selected-job-config-identity.v1",
        "job_xml_sha256": job_xml_sha256_hex,
        "selected_input_label": configuration.input_kind().selected_label(),
        "normalized_f01e": configuration.normalized_identity_value()
    });
    let config_sha256: [u8; 32] =
        Sha256::digest(serde_json::to_vec(&config_identity).unwrap()).into();
    let identity = SelectedPrimaryRuntimeIdentity {
        source_file_bytes: primary_format["source_file_bytes"].as_u64().unwrap(),
        data_bytes: primary_format["data_bytes"].as_u64().unwrap(),
        metadata_hashes,
        // This test-only identity names the metadata fixture; no PCM bytes are read or rehashed.
        resource_sha256: Sha256::digest(metadata_identity).into(),
        config_sha256,
        job_xml_sha256,
        job_xml_bytes: u64::try_from(xml.len()).unwrap(),
    };
    let binding = bind_admitted_selected_input(&configuration, identity).unwrap();
    let primary_controls = produce_primary_metadata_control_rows(&binding, &graph, &dbmd).unwrap();
    let p11 = generate_selected_p11_geometry(0x111b, true).unwrap();
    let source_map = build_meter_installed48_source_map(&primary_controls, &p11).unwrap();
    let admitted = graph
        .admit_object_events(AdmSampleCount::new(source_samples).unwrap())
        .unwrap();

    let manifest = PackageManifestEvidence::new(Arc::<[u8]>::from(F01E_MANIFEST), None);
    let validated_manifest = validate_f01e_package_manifest(&manifest).unwrap();
    let provider = ProviderFileEvidence::new(Arc::<[u8]>::from(F01E_COLD_PROVIDER), None);
    let validated_provider = validate_f01e_provider(&provider, &validated_manifest).unwrap();
    let programme_id = ProgrammeId::new("source-geometry-join-test").unwrap();
    let attempt_id = AttemptId::new(0);
    let machine =
        FrontendControlMachine::from_provider(validated_provider, programme_id.clone(), attempt_id)
            .unwrap();
    let prepared = NormalControlPreparedSnapshot::from_validated(
        machine.committed(),
        &binding,
        &graph,
        &admitted,
        &dbmd,
    )
    .unwrap();
    let before = NormalControlBeforeState::from_frontend_state(
        machine.committed(),
        machine.generation(),
        None,
    )
    .unwrap();
    let transaction_id =
        CallbackLineageIdentity::new(programme_id, attempt_id, FrontendCallbackIndex::new(0));
    let normal = propose_normal_control_snapshot(&before, &prepared, transaction_id).unwrap();

    F01bF01eFixture {
        graph,
        source_map,
        primary_controls,
        normal,
    }
}

fn target_context(
    geometry: &SelectedP15Geometry,
    active_flags: [u8; 15],
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
) -> TargetContextProjection {
    let state = TargetSlotState {
        xyz_words: std::array::from_fn(|slot| geometry.points()[slot].xyz_f32_bits()),
        ids_u32: std::array::from_fn(|slot| 200 + u32::try_from(slot).unwrap()),
        attributes_u32: std::array::from_fn(|slot| u32::try_from(slot).unwrap() << 4),
        previous_xyz_words: std::array::from_fn(|slot| geometry.points()[slot].xyz_f32_bits()),
        previous_ids_u32: std::array::from_fn(|slot| 200 + u32::try_from(slot).unwrap()),
        previous_attributes_u32: std::array::from_fn(|slot| u32::try_from(slot).unwrap() << 4),
        active_flags,
    };
    let installation = install_target_slots(
        &state,
        &SelectedTargetSlots {
            xyz_words: Vec::new(),
            ids_u32: Vec::new(),
            attributes_u32: Vec::new(),
        },
    )
    .unwrap();
    build_target_context_projection(geometry, &installation, transaction_id, callback_index)
        .unwrap()
}

fn point_response_for_sources(
    geometry: &SelectedP15Geometry,
    source_geometry: &GlobalSourceGeometrySnapshot,
    normal: &NormalControlSnapshotProposal,
    candidate_ids: Vec<u32>,
    processing_ids: Vec<u32>,
) -> PointResponseProduction {
    let normal_callbacks = NormalCallbackCount::new(
        normal
            .after_state()
            .prepared_identity()
            .source_samples()
            .div_ceil(1_536),
    );
    let source_control_clock = SourceControlSnapshotClock::for_normal_callback(
        normal.value_generation(),
        normal_callbacks,
    )
    .unwrap();
    let source_rows = source_geometry
        .rows()
        .iter()
        .map(|row| {
            PointResponseSourceInput::new(
                row.source_index(),
                row.xyz_f32_bits(),
                0.0_f32.to_bits(),
                0,
                0,
                0,
                1,
            )
        })
        .collect::<Vec<_>>();
    let source_snapshot = PointResponseSourceSnapshot::new(
        normal.value_generation(),
        normal.transaction_id().clone(),
        source_control_clock,
        source_rows,
    )
    .unwrap();
    let gate = PointResponseGateInput::new(
        normal.value_generation(),
        normal.transaction_id().clone(),
        candidate_ids,
        processing_ids,
    )
    .unwrap();
    let source_batch = prepare_point_response_sources(source_snapshot, gate).unwrap();
    produce_point_response_columns(geometry, source_batch).unwrap()
}

#[test]
fn global_source_geometry_joins_p11_beds_objects_and_lfe_exclusion_by_identity() {
    let fixture = f01b_f01e_fixture();
    let source_geometry =
        build_global_source_geometry_snapshot(&fixture.graph, &fixture.source_map, &fixture.normal)
            .unwrap();

    assert_eq!(source_geometry.rows().len(), 48);
    assert_eq!(
        source_geometry.transaction_id(),
        fixture.normal.transaction_id()
    );
    assert_eq!(
        source_geometry.value_generation(),
        fixture.normal.value_generation()
    );
    assert_eq!(
        source_geometry.source_origin_generation(),
        fixture.normal.normal_origin_generation()
    );
    assert_eq!(
        source_geometry.source_receipt_sha256(),
        fixture.source_map.source_binding().source_receipt_sha256()
    );

    for source in 0..VIRTUAL_PREFIX_SOURCE_COUNT {
        let row = &source_geometry.rows()[source];
        let point = fixture.source_map.p11_geometry().points()[source];
        assert_eq!(row.source_index().value(), u64::try_from(source).unwrap());
        assert_eq!(row.xyz_f32_bits(), point.xyz_words());
        assert!(matches!(
            row.owner(),
            GlobalSourceGeometryOwner::VirtualPrefix {
                prefix_ordinal,
                source_layout_index
            } if prefix_ordinal.value() == point.ordinal().value()
                && *source_layout_index == point.source_layout_index()
        ));
    }

    let direct_lfe_host = fixture.graph.receipt().direct_lfe_host();
    assert!(
        !fixture
            .graph
            .regular_source_bindings()
            .iter()
            .any(|binding| binding.physical_host() == direct_lfe_host)
    );

    for binding in fixture.graph.regular_source_bindings() {
        let source = source_geometry
            .row_for_source(binding.global_source())
            .unwrap();
        let host = usize::from(binding.physical_host().value());
        assert_eq!(source.source_index(), binding.global_source());
        match &fixture.normal.control38_rows()[host] {
            NormalControlRow::Bed6 { words, .. } => {
                let mapped = fixture
                    .source_map
                    .row_for_source(binding.global_source())
                    .unwrap();
                let xyz = &mapped.metadata_words_8_to_55()[4..7];
                assert_eq!(source.xyz_f32_bits(), xyz);
                assert_eq!(
                    words[5],
                    match source.owner() {
                        GlobalSourceGeometryOwner::Bed { bed_id, .. } => u32::from(*bed_id),
                        _ => panic!("Bed6 row must map to bed geometry"),
                    }
                );
            }
            NormalControlRow::Object13 {
                control, track_uid, ..
            } => {
                let words = control.words();
                assert_eq!(source.xyz_f32_bits(), [words[5], words[6], words[7]]);
                assert!(matches!(
                    source.owner(),
                    GlobalSourceGeometryOwner::Object { track_uid: source_uid, physical_host, .. }
                        if source_uid == track_uid
                            && *physical_host == binding.physical_host()
                ));
            }
        }
    }
}

#[test]
fn point_response_source_snapshot_preserves_all48_routed_fields_and_clock() {
    let fixture = f01b_f01e_fixture();
    let geometry =
        build_global_source_geometry_snapshot(&fixture.graph, &fixture.source_map, &fixture.normal)
            .unwrap();
    let source_samples = fixture
        .normal
        .after_state()
        .prepared_identity()
        .source_samples();
    let normal_callbacks = NormalCallbackCount::new(source_samples.div_ceil(1_536));
    let clock = SourceControlSnapshotClock::for_normal_callback(
        geometry.value_generation(),
        normal_callbacks,
    )
    .unwrap();
    let snapshot = geometry.point_response_snapshot(clock).unwrap();

    assert_eq!(snapshot.callback_index(), geometry.value_generation());
    assert_eq!(snapshot.transaction_id(), geometry.transaction_id());
    assert_eq!(snapshot.rows().len(), 48);
    for (source_id, input) in snapshot.rows().iter().copied().enumerate() {
        let codec_source = CodecSourceIndex::new(u64::try_from(source_id).unwrap());
        let geometry_row = geometry.row_for_source(codec_source).unwrap();
        assert_eq!(input.source_index(), codec_source);
        assert_eq!(input.xyz_f32_bits(), geometry_row.xyz_f32_bits());
        assert_eq!(
            input.original_audio_width_f32_bits(),
            geometry_row.original_audio_width_f32_bits()
        );
        assert_eq!(input.feature_mode(), geometry_row.feature_mode());
        assert_eq!(input.source_skip(), geometry_row.source_skip());
        assert_eq!(input.spatial_enum(), geometry_row.spatial_enum());
        assert_eq!(
            input.coordinate_parameter(),
            geometry_row.coordinate_parameter()
        );

        match geometry_row.owner() {
            GlobalSourceGeometryOwner::VirtualPrefix { .. }
            | GlobalSourceGeometryOwner::Bed { .. } => {
                let installed = fixture.source_map.row_for_source(codec_source).unwrap();
                let metadata = installed.metadata_words_8_to_55();
                let installed_fields = installed.point_response_source_fields();
                assert_eq!(input.feature_mode(), installed_fields[0]);
                assert_eq!(input.source_skip(), installed_fields[1]);
                assert_eq!(input.spatial_enum(), installed_fields[2]);
                assert_eq!(input.coordinate_parameter(), installed_fields[3]);
                assert_eq!(installed_fields[1], 0);
                assert_eq!(installed_fields[0], metadata[7]);
                assert_eq!(installed_fields[2], metadata[10]);
                assert_eq!(installed_fields[3], metadata[11]);
            }
            GlobalSourceGeometryOwner::Object { physical_host, .. } => {
                let row = &fixture.normal.control38_rows()[usize::from(physical_host.value())];
                let NormalControlRow::Object13 { control, .. } = row else {
                    panic!("object geometry must pair with its physical Object13 row");
                };
                let words = control.words();
                assert_eq!(input.feature_mode(), words[8]);
                assert_eq!(input.source_skip(), words[0]);
                assert_eq!(input.spatial_enum(), words[11]);
                assert_eq!(input.coordinate_parameter(), words[12]);
                assert_eq!(input.original_audio_width_f32_bits(), words[9]);
            }
        }
    }

    let wrong_origin = SourceControlSnapshotClock::fixture_with_source_origin(
        geometry.value_generation(),
        normal_callbacks,
        FrontendCallbackIndex::new(geometry.value_generation().value() + 1),
        false,
    );
    assert_eq!(
        geometry.point_response_snapshot(wrong_origin).err(),
        Some(GlobalSourceGeometryError::CallbackIdentityMismatch)
    );
}

#[test]
fn callback_point_response_consumes_the_typed_source_snapshot_and_gate_union() {
    let fixture = f01b_f01e_fixture();
    let source_geometry =
        build_global_source_geometry_snapshot(&fixture.graph, &fixture.source_map, &fixture.normal)
            .unwrap();
    let normal_callbacks = NormalCallbackCount::new(
        fixture
            .normal
            .after_state()
            .prepared_identity()
            .source_samples()
            .div_ceil(1_536),
    );
    let clock = SourceControlSnapshotClock::for_normal_callback(
        source_geometry.value_generation(),
        normal_callbacks,
    )
    .unwrap();
    let gate = PointResponseGateInput::new(
        source_geometry.value_generation(),
        source_geometry.transaction_id().clone(),
        vec![11],
        Vec::new(),
    )
    .unwrap();
    let p15 = generate_selected_p15_geometry(0x159b, true).unwrap();

    let response = produce_callback_point_response(&source_geometry, clock, &p15, gate).unwrap();

    assert_eq!(
        response.callback_index(),
        source_geometry.value_generation()
    );
    assert_eq!(response.transaction_id(), source_geometry.transaction_id());
    assert_eq!(response.candidate_ids(), [11]);
    assert!(response.processing_ids().is_empty());
    assert_eq!(response.response_source_ids(), [CodecSourceIndex::new(11)]);
    assert_eq!(response.columns().len(), 1);
    assert_eq!(response.candidate_feature_width_f32_bits(), [0]);
}

#[test]
fn main_cached_hold_at_q_keeps_source_origin_q_minus_one_and_delivery_g() {
    let fixture = f01b_f01e_fixture();
    // The counter fixture exercises the Q boundary while preserving the stored source bytes;
    // it does not claim a sampler replay at Q-1.
    let last_normal = fixture
        .normal
        .after_state()
        .fixture_as_last_normal_clock()
        .unwrap();
    let normal_callbacks = last_normal.next_normal_callbacks();
    let q = normal_callbacks.value();
    let delivery_index = FrontendCallbackIndex::new(u32::try_from(q).unwrap());
    let delivery_transaction = CallbackLineageIdentity::new(
        last_normal.programme_id().clone(),
        last_normal.attempt_id(),
        delivery_index,
    );
    let held = last_normal
        .held_source_view_for_cached_callback(delivery_transaction.clone(), normal_callbacks)
        .unwrap();
    let source_origin = FrontendCallbackIndex::new(u32::try_from(q - 1).unwrap());

    assert_eq!(held.delivery_transaction_id(), &delivery_transaction);
    assert_eq!(held.delivery_generation(), delivery_index);
    assert_eq!(held.source_origin_generation(), source_origin);
    assert_eq!(
        held.source_origin_transaction_id().callback_index(),
        source_origin
    );
    assert_eq!(held.control38_rows(), fixture.normal.control38_rows());
    assert_eq!(held.regular37_rows(), fixture.normal.regular37_rows());
    let selected_profile = fixture
        .normal
        .after_state()
        .prepared_identity()
        .profile_identity();
    let normal_projection = project_normal_control_snapshot_source_controls(
        &fixture.normal,
        &fixture.graph,
        selected_profile,
    )
    .unwrap();
    let held_projection = project_held_normal_control_snapshot_source_controls(
        &held,
        &fixture.graph,
        selected_profile,
    )
    .unwrap();
    assert_eq!(held_projection.clock().callback_index(), delivery_index);
    assert!(held_projection.clock().is_cached_drain());
    assert_eq!(held_projection.clock().source_origin(), source_origin);
    assert_eq!(
        held_projection.controls().rows(),
        normal_projection.controls().rows()
    );

    let held_geometry =
        build_held_global_source_geometry_snapshot(&fixture.graph, &fixture.source_map, &held)
            .unwrap();
    let last_normal_geometry =
        build_global_source_geometry_snapshot(&fixture.graph, &fixture.source_map, &fixture.normal)
            .unwrap();
    assert_eq!(held_geometry.value_generation(), delivery_index);
    assert_eq!(held_geometry.source_origin_generation(), source_origin);
    assert_eq!(held_geometry.transaction_id(), &delivery_transaction);
    assert_eq!(held_geometry.rows(), last_normal_geometry.rows());
    let cached_clock =
        SourceControlSnapshotClock::for_cached_drain(delivery_index, normal_callbacks).unwrap();
    let point_sources = held_geometry.point_response_snapshot(cached_clock).unwrap();
    assert_eq!(point_sources.callback_index(), delivery_index);
    assert_eq!(
        point_sources.source_control_clock().source_origin(),
        source_origin
    );
    assert_eq!(point_sources.rows().len(), 48);

    let final_main_drain = FrontendCallbackIndex::new(delivery_index.value() + 3);
    let final_drain_transaction = CallbackLineageIdentity::new(
        last_normal.programme_id().clone(),
        last_normal.attempt_id(),
        final_main_drain,
    );
    let final_held = last_normal
        .held_source_view_for_cached_callback(final_drain_transaction, normal_callbacks)
        .unwrap();
    assert_eq!(final_held.delivery_generation(), final_main_drain);
    assert_eq!(final_held.source_origin_generation(), source_origin);

    let after_drain_window =
        FrontendCallbackIndex::new(delivery_index.value().checked_add(4).unwrap());
    let invalid_transaction = CallbackLineageIdentity::new(
        last_normal.programme_id().clone(),
        last_normal.attempt_id(),
        after_drain_window,
    );
    assert_eq!(
        last_normal
            .held_source_view_for_cached_callback(invalid_transaction, normal_callbacks)
            .err(),
        Some(NormalControlSnapshotError::CallbackClockMismatch)
    );
}

#[test]
fn object_control_point_field_projection_preserves_zero_and_one_parameter_values() {
    for coordinate_parameter in [0, 1] {
        let mut words = [0_u32; 13];
        words[0] = 0;
        words[8] = 0;
        words[11] = 0;
        words[12] = coordinate_parameter;
        let control = ObjectControl52::fixture_from_words(words);
        assert_eq!(
            control.point_response_source_fields(),
            [words[8], words[0], words[11], coordinate_parameter]
        );
    }
}

#[test]
fn processing_sources_run_cold_outer_stages_into_compact_global_source_columns() {
    let fixture = f01b_f01e_fixture();
    let source_geometry =
        build_global_source_geometry_snapshot(&fixture.graph, &fixture.source_map, &fixture.normal)
            .unwrap();
    let p15 = generate_selected_p15_geometry(0x159b, true).unwrap();
    let all_targets = target_context(
        &p15,
        [1; 15],
        fixture.normal.transaction_id().clone(),
        fixture.normal.value_generation(),
    );
    let response =
        point_response_for_sources(&p15, &source_geometry, &fixture.normal, vec![0], vec![0, 1]);

    let matrix = produce_compact_outer_matrix(&all_targets, &source_geometry, &response).unwrap();
    assert_eq!(matrix.transaction_id(), fixture.normal.transaction_id());
    assert_eq!(matrix.value_generation(), fixture.normal.value_generation());
    assert_eq!(
        matrix.source_origin_generation(),
        fixture.normal.normal_origin_generation()
    );
    assert_eq!(matrix.destinations().len(), 15);
    assert_eq!(
        matrix.processing_source_order(),
        [CodecSourceIndex::new(0), CodecSourceIndex::new(1)]
    );
    assert_eq!(matrix.c_rows_u32().len(), 15);
    assert!(matrix.c_rows_u32().iter().all(|row| row.len() == 48));
    assert_eq!(matrix.source_results().len(), 2);
    assert!(!matrix.source_results()[0].active_branch());
    assert!(!matrix.source_results()[1].active_branch());
    assert_eq!(
        matrix.coefficient(
            matrix.destinations()[0].compact_ordinal(),
            CodecSourceIndex::new(0)
        ),
        Some(0x3f80_0000)
    );
    assert_eq!(
        matrix.coefficient(
            matrix.destinations()[1].compact_ordinal(),
            CodecSourceIndex::new(1)
        ),
        Some(0x3f80_0000)
    );
    assert_eq!(
        matrix.coefficient(
            matrix.destinations()[0].compact_ordinal(),
            CodecSourceIndex::new(1)
        ),
        Some(0)
    );
    assert_eq!(
        matrix.coefficient(
            matrix.destinations()[1].compact_ordinal(),
            CodecSourceIndex::new(0)
        ),
        Some(0)
    );
}

#[test]
fn active_outer_route_uses_raw_same_callback_source_xyz_and_skips_candidate_only_sources() {
    let fixture = f01b_f01e_fixture();
    let source_geometry =
        build_global_source_geometry_snapshot(&fixture.graph, &fixture.source_map, &fixture.normal)
            .unwrap();
    let p15 = generate_selected_p15_geometry(0x159b, true).unwrap();
    let mut active_flags = [0; 15];
    active_flags[0] = 1;
    let one_target = target_context(
        &p15,
        active_flags,
        fixture.normal.transaction_id().clone(),
        fixture.normal.value_generation(),
    );
    let response =
        point_response_for_sources(&p15, &source_geometry, &fixture.normal, vec![0], vec![1]);

    let matrix = produce_compact_outer_matrix(&one_target, &source_geometry, &response).unwrap();
    assert_eq!(matrix.processing_source_order(), [CodecSourceIndex::new(1)]);
    assert_eq!(matrix.source_results().len(), 1);
    assert_eq!(
        matrix.source_results()[0].source_index(),
        CodecSourceIndex::new(1)
    );
    assert!(matrix.source_results()[0].active_branch());
    assert_eq!(matrix.source_results()[0].nnls_status(), NnlsStatus::Solved);
    assert_eq!(matrix.source_results()[0].seed_u32(), [0]);
    assert_eq!(matrix.c_rows_u32().len(), 1);
    assert_eq!(
        matrix.coefficient(
            one_target.destinations()[0].compact_ordinal(),
            CodecSourceIndex::new(0)
        ),
        Some(0)
    );
    let active_source_coefficient = matrix
        .coefficient(
            one_target.destinations()[0].compact_ordinal(),
            CodecSourceIndex::new(1),
        )
        .unwrap();
    assert!(f32::from_bits(active_source_coefficient).is_finite());
    assert!(f32::from_bits(active_source_coefficient) >= 0.0);
}
