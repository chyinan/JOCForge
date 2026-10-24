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
    spatial::{
        installed48_activity::{PrimaryActivityInput, PrimaryInstalled48ActivityProducer},
        installed48_source_map::build_primary_installed48_source_map,
        selected_p11_geometry::generate_selected_p11_geometry,
    },
    timeline::AdmSampleIndex,
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
