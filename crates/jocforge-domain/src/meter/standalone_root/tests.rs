use super::*;
use crate::meter::{
    k_matrix::{KMatrixMode, KMatrixProfile, generate_k_matrix},
    lfe_delay_ring::MeterFrontendCallbackInput,
    object_renderer::ObjectRenderCallbackGeneration,
    sample_block_queue::{MeterCallbackPhase, MeterSourceSampleCount},
    shared_gain_delay::SharedGainDelayParameters,
    source_frame::MeterSourcePosition,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const CALLBACK_SAMPLES: usize = 1_536;
const CALLBACK_SAMPLES_U64: u64 = 1_536;
const SELECTED_KERNEL_Q15: [u16; 32] = [
    20, 177, 491, 958, 1_573, 2_331, 3_224, 4_244, 5_381, 6_624, 7_961, 9_379, 10_864, 12_403,
    13_980, 15_580, 17_188, 18_788, 20_365, 21_904, 23_389, 24_807, 26_144, 27_387, 28_524, 29_544,
    30_437, 31_195, 31_810, 32_277, 32_591, 32_748,
];
const ZERO_PROGRAMME: &[u8] =
    include_bytes!("../../../tests/fixtures/post_render_meter/zero_programme.json");
const V4_COLD_ROOT: &[u8] = include_bytes!(
    "../../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-ROOT-CANONICAL-STATE-REFERENCE-V4/input-only-cold-roots/N6522906_pre_callback0_cold_root.json"
);
const V4_ROOT_RECIPES: &[u8] = include_bytes!(
    "../../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-ROOT-CANONICAL-STATE-REFERENCE-V4/INPUT-ONLY-ROOT-RECIPES.json"
);

fn v4_cold_value() -> Value {
    serde_json::from_slice(V4_COLD_ROOT).unwrap()
}

fn v4_sample_queue_case(id: &str) -> Value {
    let recipes: Value = serde_json::from_slice(V4_ROOT_RECIPES).unwrap();
    recipes["sample_queue_state_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()
        .clone()
}

fn sha256_from_hex(value: &str) -> [u8; 32] {
    assert_eq!(value.len(), 64, "invalid SHA-256 text: {value}");
    let mut output = [0; 32];
    for (index, byte) in output.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap();
    }
    output
}

fn provider_set_from_v4() -> MeterProviderSet {
    let value = v4_cold_value();
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

fn vector_identity() -> StandaloneMeterIdentity {
    let binding = v4_cold_value()["binding"].clone();
    StandaloneMeterIdentity::new(
        ProgrammeId::new(binding["programme_id"].as_str().unwrap()).unwrap(),
        AttemptId::new(binding["attempt"].as_u64().unwrap()),
        sha256_from_hex(binding["input_sha256"].as_str().unwrap()),
        sha256_from_hex(binding["config_sha256"].as_str().unwrap()),
    )
}

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
        MeterSourceOwnerId::new(format!("test-parent-meter-source-attempt-{attempt}")).unwrap(),
        provider_set_from_v4(),
        MeterKFilterExponentState::new([0; METER_CHANNEL_COUNT]),
        gain_parameters,
        k_matrix,
    )
    .unwrap()
}

fn source_progress(
    root: &StandaloneMeterRoot,
    phase: MeterCallbackPhase,
) -> MeterRootCallbackProgress {
    let callback_index = root.meter_callback_index();
    let source_cursor_before = root.source_cursor();
    let read_index_before = root.source_read_index();
    let (source_cursor_after, read_index_after, host_input_count) = match phase {
        MeterCallbackPhase::NormalSource => {
            let remaining = root.source_samples().value() - source_cursor_before.value();
            let host_input_count = remaining.min(CALLBACK_SAMPLES_U64);
            (
                MeterSourcePosition::new(source_cursor_before.value() + host_input_count),
                MeterSourceReadIndex::new(read_index_before.value() + 1),
                MeterSampleCount::new(host_input_count),
            )
        }
        MeterCallbackPhase::ZeroInputDrain => (
            MeterSourcePosition::new(root.source_samples().value()),
            read_index_before,
            MeterSampleCount::new(0),
        ),
    };
    MeterRootCallbackProgress::new(
        callback_index,
        source_cursor_before,
        source_cursor_after,
        read_index_before,
        read_index_after,
        host_input_count,
        MeterKFilterExponentState::new(*root.k_filter_exponents_i32_by_channel()),
    )
}

#[test]
fn parent_must_supply_a_nonempty_opaque_source_owner_and_a_sorted_provider_set() {
    assert!(matches!(
        MeterSourceOwnerId::new(String::new()),
        Err(StandaloneMeterRootError::SourceOwnerId)
    ));
    assert!(matches!(
        MeterProviderSet::new([0; 32], vec![]),
        Err(StandaloneMeterRootError::ProviderSet)
    ));

    let first = MeterProviderIdentity::new("z-provider", [1; 32]).unwrap();
    let second = MeterProviderIdentity::new("a-provider", [2; 32]).unwrap();
    assert!(matches!(
        MeterProviderSet::new([3; 32], vec![first, second]),
        Err(StandaloneMeterRootError::ProviderSet)
    ));
}

#[test]
fn standalone_meter_cold_root_has_the_complete_v4_shape_and_separate_exponents() {
    let root = cold_root(0);
    let schedule = StandaloneMeterSchedule::selected();

    assert_eq!(root.identity().attempt(), AttemptId::new(0));
    assert_eq!(root.source_samples(), selected_source_samples());
    assert_eq!(root.source_cursor(), MeterSourcePosition::new(0));
    assert_eq!(root.source_read_index(), MeterSourceReadIndex::new(0));
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
    assert!(!root.prepass().ready_to_finalize());
    assert!(matches!(
        root.prepare_measurement_receipt_proposal(),
        Err(StandaloneMeterRootError::FinalizationNotReady)
    ));

    let bytes = root.canonical_preimage().unwrap();
    assert!(!bytes.ends_with(b"\n"));
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["schema"], ROOT_SCHEMA);
    assert_eq!(
        value
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        [
            "binding",
            "callback_clock",
            "input_cursor",
            "leaves",
            "schema"
        ]
    );
    assert_eq!(
        value["input_cursor"]["model_owner_key"],
        "test-parent-meter-source-attempt-0"
    );
    assert_eq!(value["input_cursor"]["source_cursor"], 0);
    assert_eq!(value["input_cursor"]["read_index"], 0);
    assert_eq!(value["callback_clock"]["committed_meter_callbacks"], 0);
    assert_eq!(value["callback_clock"]["next_meter_callback_index"], 0);
    assert_eq!(value["binding"]["data_length_bytes"], 743_611_284);
    assert_eq!(value["binding"]["channels"], 38);
    assert_eq!(value["binding"]["pcm_bits"], 24);
    assert_eq!(value["binding"]["sample_rate_hz"], 48_000);
    assert_eq!(value["binding"]["callback_samples"], 1_536);
    assert_eq!(value["leaves"]["frontend_oar"]["layout"], OAR_STATE_LAYOUT);
    assert_eq!(
        value["leaves"]["shared_dv32"]["layout"],
        SHARED_DV32_STATE_LAYOUT
    );
    assert_eq!(
        value["leaves"]["integrated"]["layout"],
        INTEGRATED_STATE_LAYOUT
    );
    assert_eq!(value["leaves"]["ungated"]["layout"], UNGATED_STATE_LAYOUT);
    let window = &value["leaves"]["window_k_energy"];
    assert_eq!(
        window["k_filter"]["exponents_i32_by_channel"],
        serde_json::json!([0, 0, 0, 0, 0, 0])
    );
    assert_eq!(
        window["energy_window_accumulator"]["exponent_i32_by_channel"],
        serde_json::json!([48, 48, 48, 48, 48, 48])
    );
    assert_eq!(
        window["sample_block_queue"]["pending_sample_words"],
        serde_json::json!([[], [], [], [], [], []])
    );
    assert_eq!(
        value["leaves"]["meter_lfe_delay_ring"]["frames"],
        serde_json::json!([null, null, null])
    );
    assert_eq!(
        root.canonical_sha256().unwrap(),
        root.cold_canonical_sha256().unwrap()
    );
}

#[test]
fn selected_v4_cold_vector_matches_the_complete_local_root_preimage() {
    let value = v4_cold_value();
    let (gain_parameters, k_matrix) = providers();
    let root = StandaloneMeterRoot::cold_selected(
        vector_identity(),
        selected_source_samples(),
        MeterSourceOwnerId::new(value["input_cursor"]["model_owner_key"].as_str().unwrap())
            .unwrap(),
        provider_set_from_v4(),
        MeterKFilterExponentState::new(
            value["leaves"]["window_k_energy"]["k_filter"]["exponents_i32_by_channel"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| i32::try_from(item.as_i64().unwrap()).unwrap())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        ),
        gain_parameters,
        k_matrix,
    )
    .unwrap();
    let actual = root.canonical_preimage().unwrap();
    assert_eq!(actual, V4_COLD_ROOT);
    assert_eq!(
        Sha256::digest(&actual).as_slice(),
        &sha256_from_hex("a983c9344a57357a3bd094b1002b2effe9e78490bc82132259b4ae6f39991c4b")
    );
}

#[test]
fn callback_proposal_keeps_the_before_root_and_advances_typed_cursor_progress() {
    let root = cold_root(0);
    let before_lfe = root.prepass().lfe_state().clone();
    let before_post_render = root.prepass().post_render_state().clone();
    let proposal = root
        .prepare_callback(
            callback_input(0, MeterCallbackPhase::NormalSource),
            source_progress(&root, MeterCallbackPhase::NormalSource),
        )
        .unwrap();

    assert_eq!(root.meter_callback_index(), MeterCallbackIndex::new(0));
    assert_eq!(root.source_cursor(), MeterSourcePosition::new(0));
    assert_eq!(root.source_read_index(), MeterSourceReadIndex::new(0));
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
    assert_eq!(
        proposal.after().source_cursor(),
        MeterSourcePosition::new(1_536)
    );
    assert_eq!(
        proposal.after().source_read_index(),
        MeterSourceReadIndex::new(1)
    );
    assert_ne!(proposal.after().prepass().lfe_state(), &before_lfe);
    assert_ne!(
        proposal.after().prepass().post_render_state(),
        &before_post_render
    );
}

#[test]
fn invalid_parent_cursor_progress_produces_no_after_root() {
    let root = cold_root(0);
    let progress = MeterRootCallbackProgress::new(
        MeterCallbackIndex::new(0),
        MeterSourcePosition::new(0),
        MeterSourcePosition::new(1_535),
        MeterSourceReadIndex::new(0),
        MeterSourceReadIndex::new(1),
        MeterSampleCount::new(1_535),
        MeterKFilterExponentState::new([0; METER_CHANNEL_COUNT]),
    );
    assert!(matches!(
        root.prepare_callback(
            callback_input(0, MeterCallbackPhase::NormalSource),
            progress
        ),
        Err(StandaloneMeterRootError::SourceCursorProgress)
    ));
    assert_eq!(root.meter_callback_index(), MeterCallbackIndex::new(0));
    assert_eq!(root.source_cursor(), MeterSourcePosition::new(0));
    assert_eq!(root.source_read_index(), MeterSourceReadIndex::new(0));
}

#[test]
fn approved_midstate_vector_clocks_are_preserved_without_discarding_fifo_words() {
    let expected = v4_sample_queue_case("after_g2_first_publish");
    let mut root = cold_root(0);
    for _ in 0..3 {
        let phase = MeterCallbackPhase::NormalSource;
        let proposal = root
            .prepare_callback(
                callback_input(root.meter_callback_index().value(), phase),
                source_progress(&root, phase),
            )
            .unwrap();
        root = proposal.into_parts().0;
    }
    let value: Value = serde_json::from_slice(&root.canonical_preimage().unwrap()).unwrap();
    let queue = &value["leaves"]["window_k_energy"]["sample_block_queue"];
    assert_eq!(
        root.meter_callback_index(),
        MeterCallbackIndex::new(expected["callback_index"].as_u64().unwrap())
    );
    assert_eq!(root.source_cursor(), MeterSourcePosition::new(4_608));
    assert_eq!(root.source_read_index(), MeterSourceReadIndex::new(3));
    assert!(matches!(
        root.prepare_measurement_receipt_proposal(),
        Err(StandaloneMeterRootError::FinalizationNotReady)
    ));
    assert_eq!(queue["published_samples"], expected["published_samples"]);
    assert_eq!(
        queue["pcm_enqueued_samples"],
        expected["pcm_enqueued_samples"]
    );
    assert_eq!(
        queue["pcm_consumed_samples"],
        expected["pcm_consumed_samples"]
    );
    assert_eq!(
        queue["complete_block_count"],
        expected["complete_block_count"]
    );
    assert_eq!(
        queue["pcm_block_fill_cursor"],
        expected["pcm_block_fill_cursor"]
    );
    assert!(
        queue["pending_sample_words"]
            .as_array()
            .unwrap()
            .iter()
            .all(|lane| {
                lane.as_array().unwrap().len()
                    == usize::try_from(expected["pending_lane_sample_count"].as_u64().unwrap())
                        .unwrap()
            })
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
            MeterSourceOwnerId::new("test-parent-meter-source-attempt-5").unwrap(),
            provider_set_from_v4(),
            MeterKFilterExponentState::new([0; METER_CHANNEL_COUNT]),
            gain_parameters,
            k_matrix,
        )
        .unwrap();
    assert_eq!(reset.identity().attempt(), AttemptId::new(5));
    assert_eq!(reset.meter_callback_index(), MeterCallbackIndex::new(0));
    assert_eq!(reset.source_cursor(), MeterSourcePosition::new(0));
    assert_eq!(reset.source_read_index(), MeterSourceReadIndex::new(0));
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
            MeterSourceOwnerId::new("test-parent-meter-source-attempt-6").unwrap(),
            provider_set_from_v4(),
            MeterKFilterExponentState::new([0; METER_CHANNEL_COUNT]),
            gain_parameters,
            k_matrix
        ),
        Err(StandaloneMeterRootError::AttemptSequence)
    ));
}

#[test]
#[ignore = "Runs the full selected-profile synthetic meter schedule; opt in for terminal receipt validation."]
fn terminal_receipt_proposal_requires_q_normal_reads_and_exactly_three_drains() {
    let expected = v4_sample_queue_case("terminal_after_three_drains");
    let mut root = cold_root(0);
    let terminal_callbacks = expected["callback_index"].as_u64().unwrap();
    let normal_callbacks = expected["Q"].as_u64().unwrap();
    for _ in 0..terminal_callbacks {
        let phase = if root.meter_callback_index().value() < normal_callbacks {
            MeterCallbackPhase::NormalSource
        } else {
            MeterCallbackPhase::ZeroInputDrain
        };
        let proposal = root
            .prepare_callback(
                callback_input(root.meter_callback_index().value(), phase),
                source_progress(&root, phase),
            )
            .unwrap();
        root = proposal.into_parts().0;
    }

    assert!(root.prepass().ready_to_finalize());
    assert_eq!(
        root.source_cursor(),
        MeterSourcePosition::new(SELECTED_SOURCE_SAMPLES)
    );
    assert_eq!(
        root.source_read_index(),
        MeterSourceReadIndex::new(SELECTED_NORMAL_CALLBACKS)
    );
    assert_eq!(
        root.meter_callback_index(),
        MeterCallbackIndex::new(terminal_callbacks)
    );

    let value: Value = serde_json::from_slice(&root.canonical_preimage().unwrap()).unwrap();
    let queue = &value["leaves"]["window_k_energy"]["sample_block_queue"];
    let energy = &value["leaves"]["window_k_energy"]["energy_window_accumulator"];
    assert_eq!(queue["published_samples"], expected["published_samples"]);
    assert_eq!(
        queue["pcm_enqueued_samples"],
        expected["pcm_enqueued_samples"]
    );
    assert_eq!(
        queue["pcm_consumed_samples"],
        expected["pcm_consumed_samples"]
    );
    assert_eq!(
        queue["complete_block_count"],
        expected["complete_block_count"]
    );
    assert_eq!(
        queue["pcm_block_fill_cursor"],
        expected["pcm_block_fill_cursor"]
    );
    assert_eq!(
        energy["completed_windows"],
        expected["energy_complete_windows"]
    );
    assert_eq!(
        energy["samples_in_window"],
        expected["energy_window_fill_cursor"]
    );

    let root_hash_before = root.canonical_sha256().unwrap();
    let proposal = root.prepare_measurement_receipt_proposal().unwrap();
    assert_eq!(
        proposal.schedule().normal_callbacks(),
        MeterCallbackCount::new(normal_callbacks)
    );
    assert_eq!(
        proposal.schedule().zero_drains(),
        MeterCallbackCount::new(terminal_callbacks - normal_callbacks)
    );
    assert_eq!(
        proposal.schedule().energy_windows(),
        expected["energy_complete_windows"].as_u64().unwrap()
    );
    assert_eq!(*proposal.terminal_meter_sha256(), root_hash_before);
    assert_eq!(
        proposal.receipt().programme_id(),
        root.identity().programme_id()
    );
    assert_eq!(proposal.receipt().attempt(), root.identity().attempt());
    assert_eq!(
        proposal.receipt().input_sha256(),
        root.identity().input_sha256()
    );
    assert_eq!(
        proposal.receipt().config_sha256(),
        root.identity().config_sha256()
    );
    assert_eq!(proposal.receipt().normal_callbacks(), normal_callbacks);
    assert_eq!(
        proposal.receipt().zero_drains(),
        terminal_callbacks - normal_callbacks
    );
    assert_eq!(
        proposal.receipt().energy_windows(),
        expected["energy_complete_windows"].as_u64().unwrap()
    );
    assert_eq!(root.canonical_sha256().unwrap(), root_hash_before);
}
