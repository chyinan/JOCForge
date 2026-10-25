// pattern: Functional Core
use jocforge_domain::{
    frontend::{
        dbmd_sections::parse_selected_dbmd_sections,
        job_configuration::bind_selected_primary_control_projection,
        primary_metadata_control::produce_primary_metadata_control_rows,
        selected_adm_graph::parse_selected_adm_graph,
    },
    meter::{
        installed48_renderer::{
            MeterInstalled48Input, MeterInstalled48State, stage_meter_installed48,
        },
        sample_block_queue::{MeterCallbackIndex, MeterSourceSampleCount},
    },
    profile::{EncodingProfile, resolve_selected_profile},
    spatial::{
        installed48_activity::{PrimaryActivityInput, PrimaryInstalled48ActivityProducer},
        installed48_source_map::build_primary_installed48_source_map,
        selected_p11_geometry::generate_selected_p11_geometry,
        source_flags::{
            HostRole, HostRouteRecord, RegularControls, SelectedFeatureFlagsControl,
            SourceFlagsProducer, ValidatedSemanticRoute,
        },
    },
    timeline::{AdmSampleIndex, FrontendCallbackIndex},
};

const AXML: &[u8] = include_bytes!("fixtures/primary_metadata_control/source_metadata.xml");
const CHNA: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/source_track_associations.bin");
const DBMD: &[u8] = include_bytes!("fixtures/primary_metadata_control/source_metadata.dbmd");
const JOB_PROJECTION: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/JOB-FIELD-PROJECTION.json");
const JOB_PROJECTION_RECEIPT: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/JOB-PROJECTION-RECEIPT.json");
const FROZEN_JOB_BINDING: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/FROZEN-JOB-BINDING.json");
const PRIMARY_SOURCE_RECEIPT: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/PRIMARY-SOURCE-RECEIPT.json");

#[test]
fn cold_zero_input_produces_one_noncommitting_postwindow_frame() {
    let binding = bind_selected_primary_control_projection(
        JOB_PROJECTION,
        JOB_PROJECTION_RECEIPT,
        FROZEN_JOB_BINDING,
        PRIMARY_SOURCE_RECEIPT,
    )
    .unwrap();
    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let dbmd = parse_selected_dbmd_sections(DBMD).unwrap();
    let controls = produce_primary_metadata_control_rows(&binding, &graph, &dbmd).unwrap();
    let geometry = generate_selected_p11_geometry(0x111b, true).unwrap();
    let source_map = build_primary_installed48_source_map(&controls, &geometry).unwrap();
    let activity = PrimaryInstalled48ActivityProducer::new(&source_map).unwrap();
    let activity_state = activity.cold_state().unwrap();
    let zero_host_rows = vec![vec![0_u32; 1_536]; 38];
    let activity_proposal = activity
        .produce_next(
            &source_map,
            &activity_state,
            PrimaryActivityInput::NormalRead {
                reader_relative_sample: AdmSampleIndex::new(0),
                pcm_word_rows: zero_host_rows.clone(),
            },
        )
        .unwrap();
    let (activity_output, next_activity_state, membership_route) = activity_proposal.into_parts();
    assert_eq!(activity_output.quiet37(), &[1; 37]);
    assert!(membership_route.contributions().is_empty());

    let source_sha256 = *source_map.source_binding().source_receipt_sha256();
    let source_samples = MeterSourceSampleCount::new(binding.data_bytes() / 114).unwrap();
    let before = MeterInstalled48State::cold(source_sha256, source_samples);
    let proposal = stage_meter_installed48(
        &before,
        MeterInstalled48Input {
            source_sha256,
            callback_index: MeterCallbackIndex::new(0),
            source_cursor_before: AdmSampleIndex::new(0),
            host_input_samples: 1_536,
            source_padding_samples: 0,
            raw_pcm_rows: &zero_host_rows,
            activity: &activity_output,
            membership_route: &membership_route,
            source_map: &source_map,
            controls: &controls,
            diffusion_parameters: None,
        },
    )
    .unwrap();
    let (after, frame) = proposal.into_parts();

    assert_eq!(frame.callback_index(), MeterCallbackIndex::new(0));
    assert_eq!(frame.upstream_generation(), 1);
    assert_eq!(frame.rows().len(), 48);
    assert!(frame.rows().iter().all(|row| row.len() == 3_072));
    assert!(frame.rows().iter().flatten().all(|word| *word == 0));
    assert_eq!(before.callback_index(), MeterCallbackIndex::new(0));
    assert_eq!(before.source_cursor(), AdmSampleIndex::new(0));
    assert_eq!(after.callback_index(), MeterCallbackIndex::new(1));
    assert_eq!(after.source_cursor(), AdmSampleIndex::new(1_536));
    assert_eq!(activity_state.next_callback().value(), 0);
    assert_eq!(next_activity_state.next_callback().value(), 1);
}

#[test]
fn authored_audio_diffuse_and_selected_feature_diffuse_reach_distinct_consumers() {
    let binding = bind_selected_primary_control_projection(
        JOB_PROJECTION,
        JOB_PROJECTION_RECEIPT,
        FROZEN_JOB_BINDING,
        PRIMARY_SOURCE_RECEIPT,
    )
    .unwrap();
    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let dbmd = parse_selected_dbmd_sections(DBMD).unwrap();
    let controls = produce_primary_metadata_control_rows(&binding, &graph, &dbmd).unwrap();
    let geometry = generate_selected_p11_geometry(0x111b, true).unwrap();
    let source_map = build_primary_installed48_source_map(&controls, &geometry).unwrap();
    let selected_profile = resolve_selected_profile(&EncodingProfile::selected()).unwrap();
    let callback_index = FrontendCallbackIndex::new(0);

    let mut route_records = Vec::with_capacity(38);
    let mut regular_controls = Vec::with_capacity(37);
    let mut seen_hosts = [false; 38];
    for binding in source_map.physical_to_global_source() {
        let host = binding.physical_host().value();
        let source_row = source_map.row_for_source(binding.source_index()).unwrap();
        seen_hosts[usize::from(host)] = true;
        route_records.push(
            HostRouteRecord::new(
                host,
                HostRole::Regular,
                true,
                Some(u8::try_from(binding.source_index().value()).unwrap()),
            )
            .unwrap(),
        );
        let feature_control = SelectedFeatureFlagsControl::from_admitted_profile(
            selected_profile,
            callback_index,
            host,
        )
        .unwrap();
        assert_eq!(feature_control.component_enum(), 0);
        assert_eq!(feature_control.diffuse(), 0);
        assert_eq!(feature_control.callback_index(), callback_index);
        let event_control = source_row.control52();
        if host == 10 {
            assert_eq!(event_control.unwrap().words()[10], 1);
        }
        let mute = source_row.input_mute().unwrap();
        let width = f32::from_bits(source_row.metadata_words_8_to_55()[8]);
        regular_controls.push(feature_control.into_regular_control(mute, width).unwrap());
    }
    let lfe_host = u8::try_from(seen_hosts.iter().position(|seen| !seen).unwrap()).unwrap();
    route_records.push(HostRouteRecord::new(lfe_host, HostRole::Lfe, true, None).unwrap());
    let route = ValidatedSemanticRoute::new(route_records).unwrap();
    let regular_controls = RegularControls::new(regular_controls).unwrap();

    let mut pcm_rows = vec![vec![0_u32; 1_536]; 38];
    pcm_rows[10][768] = 0.5_f32.to_bits();
    let previous_regular_rows = vec![vec![0_u32; 1_536]; 37];
    let values = SourceFlagsProducer::produce_meter_values(
        &pcm_rows,
        &previous_regular_rows,
        &regular_controls,
        &route,
        &[],
    )
    .unwrap();
    let host10_ordinal = route
        .regular_hosts()
        .iter()
        .position(|host| host.value() == 10)
        .unwrap();
    assert_eq!(values.raw_history_rows().count(), 37);
    assert_eq!(
        values.raw_history_rows().nth(host10_ordinal).unwrap()[768],
        0.5_f32.to_bits()
    );
    let host10_destination = 11 + host10_ordinal;
    assert_eq!(values.payload()[host10_destination], [0, 0]);

    let activity = PrimaryInstalled48ActivityProducer::new(&source_map).unwrap();
    let activity_state = activity.cold_state().unwrap();
    let activity_proposal = activity
        .produce_next(
            &source_map,
            &activity_state,
            PrimaryActivityInput::NormalRead {
                reader_relative_sample: AdmSampleIndex::new(0),
                pcm_word_rows: pcm_rows.clone(),
            },
        )
        .unwrap();
    let (activity_output, _, membership_route) = activity_proposal.into_parts();
    assert_eq!(activity_output.callback_index(), callback_index);
    assert_eq!(activity_output.quiet37()[host10_ordinal], 0);

    let source_sha256 = *source_map.source_binding().source_receipt_sha256();
    let source_samples = MeterSourceSampleCount::new(binding.data_bytes() / 114).unwrap();
    let renderer_state = MeterInstalled48State::cold(source_sha256, source_samples);
    let f04_result = stage_meter_installed48(
        &renderer_state,
        MeterInstalled48Input {
            source_sha256,
            callback_index: MeterCallbackIndex::new(0),
            source_cursor_before: AdmSampleIndex::new(0),
            host_input_samples: 1_536,
            source_padding_samples: 0,
            raw_pcm_rows: &pcm_rows,
            activity: &activity_output,
            membership_route: &membership_route,
            source_map: &source_map,
            controls: &controls,
            diffusion_parameters: None,
        },
    );
    assert!(matches!(
        f04_result,
        Err(
            jocforge_domain::meter::installed48_renderer::MeterInstalled48Error::
                DiffusionParametersUnavailable
        )
    ));
    assert_eq!(
        source_map
            .physical_to_global_source()
            .iter()
            .find(|binding| binding.physical_host().value() == 10)
            .and_then(|binding| source_map.row_for_source(binding.source_index()))
            .unwrap()
            .control52()
            .unwrap()
            .words()[10],
        1
    );
}
