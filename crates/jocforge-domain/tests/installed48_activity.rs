// pattern: Imperative Shell
use jocforge_domain::frontend::dbmd_sections::parse_selected_dbmd_sections;
use jocforge_domain::frontend::job_configuration::bind_selected_primary_control_projection;
use jocforge_domain::frontend::primary_metadata_control::produce_primary_metadata_control_rows;
use jocforge_domain::frontend::selected_adm_graph::parse_selected_adm_graph;
use jocforge_domain::spatial::installed48_activity::{
    PrimaryActivityInput, PrimaryInstalled48ActivityProducer,
};
use jocforge_domain::spatial::installed48_source_map::build_primary_installed48_source_map;
use jocforge_domain::spatial::selected_p11_geometry::generate_selected_p11_geometry;
use jocforge_domain::timeline::AdmSampleIndex;

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
fn activity_uses_history_updates_and_excludes_the_direct_lfe() {
    let source_map = primary_source_map();
    let producer = PrimaryInstalled48ActivityProducer::new(&source_map).unwrap();
    let mut state = producer.cold_state().unwrap();

    let cold = producer
        .produce_next(&source_map, &state, normal_read(0, zero_frame()))
        .unwrap();
    let (cold_output, next_state, _) = cold.into_parts();
    assert_eq!(cold_output.quiet37(), &[1; 37]);
    assert_eq!(cold_output.callback_index().value(), 0);
    assert_eq!(cold_output.actual_source_samples(), 1_536);
    state = next_state;

    let mut prior = zero_frame();
    prior[3][0] = 1.0_f32.to_bits();
    let lfe_only = producer
        .produce_next(&source_map, &state, normal_read(1_536, prior))
        .unwrap();
    let (lfe_output, next_state, _) = lfe_only.into_parts();
    assert_eq!(lfe_output.quiet37(), &[1; 37]);
    state = next_state;

    let mut regular_impulse = zero_frame();
    regular_impulse[0][1_535] = 0.5_f32.to_bits();
    let impulse = producer
        .produce_next(&source_map, &state, normal_read(3_072, regular_impulse))
        .unwrap();
    let (impulse_output, next_state, _) = impulse.into_parts();
    assert_eq!(impulse_output.quiet37()[0], 1);
    state = next_state;

    let following = producer
        .produce_next(&source_map, &state, normal_read(4_608, zero_frame()))
        .unwrap();
    let (following_output, _, _) = following.into_parts();
    assert_eq!(following_output.quiet37()[0], 0);
    assert_eq!(following_output.quiet37()[1..], [1; 36]);
}

fn primary_source_map() -> jocforge_domain::spatial::installed48_source_map::Installed48SourceMap {
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
    build_primary_installed48_source_map(&controls, &geometry).unwrap()
}

fn zero_frame() -> Vec<Vec<u32>> {
    vec![vec![0_u32; 1_536]; 38]
}

fn normal_read(first_sample: u64, rows: Vec<Vec<u32>>) -> PrimaryActivityInput {
    PrimaryActivityInput::NormalRead {
        reader_relative_sample: AdmSampleIndex::new(first_sample),
        pcm_word_rows: rows,
    }
}
