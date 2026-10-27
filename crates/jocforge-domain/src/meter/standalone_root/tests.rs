use super::*;
use crate::meter::{
    k_matrix::{KMatrixMode, KMatrixProfile, generate_k_matrix},
    lfe_delay_ring::MeterFrontendCallbackInput,
    object_renderer::ObjectRenderCallbackGeneration,
    sample_block_queue::MeterCallbackPhase,
    shared_gain_delay::SharedGainDelayParameters,
};
use serde_json::Value;

const CALLBACK_SAMPLES: usize = 1_536;
const SELECTED_KERNEL_Q15: [u16; 32] = [
    20, 177, 491, 958, 1_573, 2_331, 3_224, 4_244, 5_381, 6_624, 7_961, 9_379, 10_864, 12_403,
    13_980, 15_580, 17_188, 18_788, 20_365, 21_904, 23_389, 24_807, 26_144, 27_387, 28_524, 29_544,
    30_437, 31_195, 31_810, 32_277, 32_591, 32_748,
];
const ZERO_PROGRAMME: &[u8] =
    include_bytes!("../../../tests/fixtures/post_render_meter/zero_programme.json");

fn identity(attempt: u64) -> StandaloneMeterIdentity {
    StandaloneMeterIdentity::new(
        ProgrammeId::new("standalone-meter-root-test").unwrap(),
        AttemptId::new(attempt),
        [0x11; 32],
        [0x22; 32],
    )
}

fn providers() -> (SharedGainDelayParameters, KMatrixProvider) {
    let k_matrix = generate_k_matrix(KMatrixProfile {
        sample_rate_hz: 48_000,
        mode: KMatrixMode::Modern,
        stage_count: 2,
    })
    .unwrap();
    let gain_parameters = SharedGainDelayParameters::selected(&SELECTED_KERNEL_Q15).unwrap();
    (gain_parameters, k_matrix)
}

fn selected_source_samples() -> MeterSourceSampleCount {
    MeterSourceSampleCount::new(SELECTED_SOURCE_SAMPLES).unwrap()
}

fn controls(fixture: &Value) -> Vec<Vec<u32>> {
    let source_count =
        usize::try_from(fixture["oar_packet"]["source_count"].as_u64().unwrap()).unwrap();
    let direct = fixture["oar_packet"]["direct_control_words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    let object = fixture["oar_packet"]["object_control_words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    let mut rows = vec![object; source_count];
    rows[0] = direct;
    rows
}

fn callback_input(callback: u64, phase: MeterCallbackPhase) -> MeterFrontendCallbackInput {
    let fixture: Value = serde_json::from_slice(ZERO_PROGRAMME).unwrap();
    let regular_controls = controls(&fixture).into_iter().skip(1).collect();
    MeterFrontendCallbackInput::new(
        MeterCallbackIndex::new(callback),
        ObjectRenderCallbackGeneration::try_new(callback + 1).unwrap(),
        phase,
        vec![0; CALLBACK_SAMPLES],
        vec![vec![0; CALLBACK_SAMPLES]; 15],
        regular_controls,
    )
}

fn cold_root(attempt: u64) -> StandaloneMeterRoot {
    let (gain_parameters, k_matrix) = providers();
    StandaloneMeterRoot::cold_selected(
        identity(attempt),
        selected_source_samples(),
        gain_parameters,
        k_matrix,
    )
    .unwrap()
}

#[test]
fn standalone_meter_cold_uses_the_fixed_gtc_schedule_and_fresh_leaves() {
    let root = cold_root(0);
    let schedule = StandaloneMeterSchedule::selected();

    assert_eq!(root.identity().attempt(), AttemptId::new(0));
    assert_eq!(
        root.source_samples(),
        MeterSourceSampleCount::new(6_522_906).unwrap()
    );
    assert_eq!(root.meter_callback_index(), MeterCallbackIndex::new(0));
    assert_eq!(root.normal_callback_count(), MeterCallbackCount::new(4_247));
    assert_eq!(schedule.zero_drains(), MeterCallbackCount::new(3));
    assert_eq!(schedule.meter_callbacks(), MeterCallbackCount::new(4_250));
    assert_eq!(
        schedule.published_samples(),
        MeterSampleCount::new(6_523_392)
    );
    assert_eq!(schedule.complete_blocks(), MeterBlockCount::new(6_370));
    assert_eq!(
        schedule.complete_prefix_samples(),
        MeterSampleCount::new(6_522_880)
    );
    assert_eq!(
        schedule.unused_block_tail_samples(),
        MeterSampleCount::new(512)
    );
    assert_eq!(schedule.energy_windows(), 6_794);
    assert_eq!(schedule.energy_samples(), MeterSampleCount::new(6_522_240));
    assert_eq!(
        schedule.energy_window_remainder(),
        MeterSampleCount::new(640)
    );
    assert_eq!(
        schedule.source_padding_samples(),
        MeterSampleCount::new(486)
    );
    assert_eq!(
        root.prepass().next_callback_index(),
        MeterCallbackIndex::new(0)
    );
    assert!(!root.prepass().ready_to_finalize());
    assert!(matches!(
        root.prepare_finalization_values(),
        Err(StandaloneMeterRootError::FinalizationNotReady)
    ));
}

#[test]
fn standalone_meter_rejects_a_different_source_extent() {
    let (gain_parameters, k_matrix) = providers();
    let result = StandaloneMeterRoot::cold_selected(
        identity(0),
        MeterSourceSampleCount::new(1).unwrap(),
        gain_parameters,
        k_matrix,
    );
    assert!(matches!(
        result,
        Err(StandaloneMeterRootError::UnsupportedSourceExtent)
    ));
}

#[test]
fn callback_proposal_keeps_the_before_root_and_returns_a_complete_after_root() {
    let root = cold_root(0);
    let before_lfe = root.prepass().lfe_state().clone();
    let before_post_render = root.prepass().post_render_state().clone();
    let proposal = root
        .prepare_callback(callback_input(0, MeterCallbackPhase::NormalSource))
        .unwrap();

    assert_eq!(root.meter_callback_index(), MeterCallbackIndex::new(0));
    assert_eq!(root.prepass().lfe_state(), &before_lfe);
    assert_eq!(root.prepass().post_render_state(), &before_post_render);
    assert_eq!(
        proposal.receipt().callback_index(),
        MeterCallbackIndex::new(0)
    );
    assert_eq!(
        proposal.receipt().next_callback_index(),
        MeterCallbackIndex::new(1)
    );
    assert_eq!(
        proposal.after().meter_callback_index(),
        MeterCallbackIndex::new(1)
    );
    assert_ne!(proposal.after().prepass().lfe_state(), &before_lfe);
    assert_ne!(
        proposal.after().prepass().post_render_state(),
        &before_post_render
    );
}

#[test]
fn full_reset_requires_the_next_attempt_and_reconstructs_meter_cold() {
    let root = cold_root(4);
    let (gain_parameters, k_matrix) = providers();
    let reset = root
        .full_reset(
            identity(5),
            selected_source_samples(),
            gain_parameters,
            k_matrix,
        )
        .unwrap();
    assert_eq!(reset.identity().attempt(), AttemptId::new(5));
    assert_eq!(reset.meter_callback_index(), MeterCallbackIndex::new(0));
    assert_eq!(
        reset.normal_callback_count(),
        MeterCallbackCount::new(4_247)
    );

    let root = cold_root(4);
    let (gain_parameters, k_matrix) = providers();
    assert!(matches!(
        root.full_reset(
            identity(6),
            selected_source_samples(),
            gain_parameters,
            k_matrix
        ),
        Err(StandaloneMeterRootError::AttemptSequence)
    ));
}
