use super::*;
use crate::spatial::production_callback::{
    MeterSpatialProductionCallbackInputs, NormalControlCallbackSource,
    continue_meter_spatial_production_callback,
};

#[test]
fn meter_component_callback_returns_f03w_f04a_f05r56_without_receiver_advancement() {
    let fixture = crate::spatial::source_geometry_outer::tests::f01b_f01e_fixture();
    let owners = callback_owners();
    let before_control = owners.spatial_control.digest();
    let before_audio = owners.spatial_audio.digest();
    let before_audio_generation = owners.spatial_audio.generation();
    let before_receiver_count = owners.spatial_audio.committed().receiver_callback_count();

    let (activity, installed48) = stage_activity_and_installed48(&fixture);
    let resolved_profile = resolve_selected_profile(&owners.spatial_control.profile()).unwrap();
    let source_projection = project_normal_control_snapshot_source_controls(
        &fixture.normal,
        &fixture.graph,
        resolved_profile,
    )
    .unwrap();
    let source_control_clock = source_projection.clock();
    let before_history =
        BeforeRegularHistory::from_owner(&owners.spatial_control, source_projection.route())
            .unwrap();
    let prefix_writers =
        PrefixWriterSet::new(activity.output().prefix_writer_indices().to_vec()).unwrap();
    let transaction_id = fixture.normal.transaction_id().clone();
    let parent =
        SourceFlagsParentLineage::new(transaction_id.clone(), &owners.spatial_control).unwrap();
    let source_flags_input = SourceFlagsInput::from_primary_regular_source_controls(
        CurrentHostPcm::new(vec![vec![0.0_f32; 1_536]; 38]).unwrap(),
        Some(before_history),
        source_projection,
        prefix_writers,
        parent,
    )
    .unwrap();
    let frontend = stage_frontend_features_from_installed48(
        &owners.spatial_control,
        transaction_id.clone(),
        activity,
        installed48,
        source_flags_input,
        CallbackPhase::Normal,
        &owners.processor,
        &owners.fft,
    )
    .unwrap();
    let normal_source_geometry =
        stage_normal_source_geometry(&fixture.graph, &fixture.source_map, fixture.normal).unwrap();
    let (normal_control, source_geometry) = normal_source_geometry.into_parts();
    let (_, normal_output) = normal_control.into_frontend_parent_parts();
    let p15_geometry = generate_selected_p15_geometry(0x159b, true).unwrap();

    let proposal =
        continue_meter_spatial_production_callback(MeterSpatialProductionCallbackInputs {
            frontend,
            source_snapshot: NormalControlCallbackSource::NormalOutput(Box::new(normal_output)),
            source_geometry,
            source_control_clock,
            source_map: &fixture.source_map,
            p15_geometry: &p15_geometry,
            spatial_control: &owners.spatial_control,
            spatial_audio: &owners.spatial_audio,
            window: &owners.window,
            parameters: &owners.parameters,
        })
        .unwrap();

    assert_eq!(proposal.callback_index(), FrontendCallbackIndex::new(0));
    assert_eq!(proposal.transaction_id(), &transaction_id);
    assert_eq!(
        proposal.component_rows().callback_index(),
        proposal.callback_index()
    );
    assert_eq!(proposal.component_rows().transaction_id(), &transaction_id);
    assert_eq!(
        proposal.component_rows().canonical_rows_f32_bits().len(),
        15 * 1_536
    );
    assert_eq!(proposal.component_blocks_processed(), 24);
    assert_eq!(
        proposal.installed48_after().callback_index(),
        MeterCallbackIndex::new(1)
    );
    assert_eq!(
        proposal.installed48_after().source_cursor(),
        AdmSampleIndex::new(1_536)
    );
    assert_eq!(
        proposal.reader_target_controls().callback_index(),
        proposal.callback_index()
    );
    assert_eq!(proposal.reader_target_controls().rows().len(), 15);

    let control_after = proposal.spatial_control_after_proposal().after_state();
    let audio_after = proposal.spatial_audio_after_proposal().after_state();
    assert_eq!(
        control_after.callback_index(),
        FrontendCallbackIndex::new(1)
    );
    assert_eq!(audio_after.callback_index(), FrontendCallbackIndex::new(1));
    assert_ne!(audio_after.digest(), before_audio);
    assert_eq!(audio_after.receiver_callback_count(), before_receiver_count);

    assert_eq!(owners.spatial_control.digest(), before_control);
    assert_eq!(owners.spatial_audio.generation(), before_audio_generation);
    assert_eq!(owners.spatial_audio.digest(), before_audio);
}
