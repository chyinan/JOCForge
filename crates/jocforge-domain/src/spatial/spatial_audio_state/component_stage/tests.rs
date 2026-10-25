// pattern: Imperative Shell
use super::*;
use crate::{
    frontend::{
        PackageManifestEvidence, ProviderFileEvidence, validate_f01e_package_manifest,
        validate_f01e_provider, validate_f03_package_manifest, validate_f03_target_cold_provider,
    },
    numeric::FiniteF32,
    profile::{EmdfCarriageProfile, EncodingProfile},
    spatial::source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
    spatial::{
        SpatialControlJoinMetadata, validate_component_parameter_provider,
        validate_f04_parameter_package_manifest, validate_window_q15_provider,
    },
    timeline::FrontendCallbackIndex,
    transaction::CallbackLineageIdentity,
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{fmt::Write as FmtWrite, sync::Arc};

const MANIFEST_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f01e_provider/MANIFEST.json");
const F01E_PROVIDER_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f01e_provider/f01e-selected-constructor-v1.json");
const F03_PROVIDER_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");
const WINDOW_Q15_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_parameters/window-q15.json");
const COMPONENT_PARAMETERS_BYTES: &[u8] = include_bytes!(
    "../../../../tests/fixtures/f04_parameters/component-parameter-mathematical-v1.json"
);
const UNIT_VECTOR_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_component/unit-vectors.json");
const PARAMETER_GENERATION_BYTES: &[u8] = include_bytes!(
    "../../../../tests/fixtures/f04_component/COMPONENT-PARAMETER-GENERATION-VERIFICATION.json"
);
const WHOLE_SPATIAL_BYTES: &[u8] = include_bytes!(
    "../../../../tests/fixtures/f04_component/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json"
);

struct TestContext {
    spatial_audio: SpatialAudioMachine,
    spatial_control: SpatialControlMachine,
    window: ValidatedWindowQ15Provider,
    parameters: ValidatedComponentParameterProvider,
    profile: EncodingProfile,
}

fn context() -> TestContext {
    let profile = EncodingProfile::selected();
    let manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
    let f01e_manifest = validate_f01e_package_manifest(&manifest_evidence).unwrap();
    let f01e = validate_f01e_provider(
        &ProviderFileEvidence::new(Arc::from(F01E_PROVIDER_BYTES), None),
        &f01e_manifest,
    )
    .unwrap();
    let f03_manifest = validate_f03_package_manifest(&manifest_evidence).unwrap();
    let f03 = validate_f03_target_cold_provider(
        &ProviderFileEvidence::new(Arc::from(F03_PROVIDER_BYTES), None),
        &f03_manifest,
    )
    .unwrap();
    let spatial_control = SpatialControlMachine::new_cold(profile, &f01e, &f03).unwrap();

    let f04_manifest = validate_f04_parameter_package_manifest(&manifest_evidence).unwrap();
    let window = validate_window_q15_provider(
        &ProviderFileEvidence::new(Arc::from(WINDOW_Q15_BYTES), None),
        &f04_manifest,
    )
    .unwrap();
    let parameters = validate_component_parameter_provider(
        &ProviderFileEvidence::new(Arc::from(COMPONENT_PARAMETERS_BYTES), None),
        &f04_manifest,
        &profile,
    )
    .unwrap();
    let spatial_audio = SpatialAudioMachine::new_cold(profile).unwrap();

    TestContext {
        spatial_audio,
        spatial_control,
        window,
        parameters,
        profile,
    }
}

fn provider_lineage(context: &TestContext) -> [F04ParameterProviderLineage; 2] {
    [
        *context.window.receipt().validated_manifest_lineage(),
        *context.parameters.receipt().validated_manifest_lineage(),
    ]
}

fn component_programme_id() -> SourceFlagsProgrammeId {
    crate::transaction::test_programme_id("component-test-programme")
}

fn component_transaction_id(callback_index: FrontendCallbackIndex) -> CallbackLineageIdentity {
    CallbackLineageIdentity::new(
        component_programme_id(),
        SourceFlagsAttempt::new(7),
        callback_index,
    )
}

pub(in crate::spatial::spatial_audio_state) fn type10_test_context() -> (
    SpatialAudioMachine,
    SpatialControlMachine,
    [F04ParameterProviderLineage; 2],
    EncodingProfile,
) {
    let context = context();
    let lineage = provider_lineage(&context);
    (
        context.spatial_audio,
        context.spatial_control,
        lineage,
        context.profile,
    )
}

fn callback_join(context: &TestContext, index: FrontendCallbackIndex) -> SpatialAudioJoinMetadata {
    SpatialAudioJoinMetadata::new(
        component_programme_id(),
        SourceFlagsAttempt::new(7),
        component_transaction_id(index),
        index,
        context.spatial_audio.generation(),
        context.spatial_audio.digest(),
        context.profile,
    )
}

fn control_join(context: &TestContext, index: FrontendCallbackIndex) -> SpatialControlJoinMetadata {
    control_join_with(
        context,
        component_transaction_id(index),
        index,
        context.spatial_control.generation(),
        None,
        context.profile,
    )
}

fn control_join_with(
    context: &TestContext,
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    before_generation: u64,
    before_digest: Option<[u8; 32]>,
    profile_identity: EncodingProfile,
) -> SpatialControlJoinMetadata {
    SpatialControlJoinMetadata::fixture(
        &context.spatial_control,
        transaction_id,
        callback_index,
        before_generation,
        before_digest,
        profile_identity,
    )
}

fn zero_packet(context: &TestContext) -> SpatialAudioInputPacket {
    SpatialAudioInputPacket::fixture(ComponentPacketFixture {
        programme_id: component_programme_id(),
        attempt: SourceFlagsAttempt::new(7),
        transaction_id: component_transaction_id(FrontendCallbackIndex::new(0)),
        callback_index: FrontendCallbackIndex::new(0),
        phase: ComponentCallbackPhase::Normal,
        profile_identity: context.profile,
        spatial_control: &context.spatial_control,
        provider_lineage: provider_lineage(context),
        installed48_f32_bits: vec![0; 48 * 3072],
        matrix_rows: 0,
        matrix_f32_bits: Vec::new(),
        destinations: Vec::new(),
    })
}

fn join_with(
    context: &TestContext,
    callback: FrontendCallbackIndex,
    before_generation: u64,
    profile: EncodingProfile,
) -> SpatialAudioJoinMetadata {
    SpatialAudioJoinMetadata::new(
        component_programme_id(),
        SourceFlagsAttempt::new(7),
        component_transaction_id(callback),
        callback,
        before_generation,
        context.spatial_audio.digest(),
        profile,
    )
}

fn assert_stage_error(
    context: &TestContext,
    join: &SpatialAudioJoinMetadata,
    phase: ComponentCallbackPhase,
    packet: SpatialAudioInputPacket,
    expected: ComponentStageError,
) {
    assert_stage_error_with_control_join(
        context,
        join,
        &control_join(context, join.callback_index()),
        phase,
        packet,
        expected,
    );
}

fn assert_stage_error_with_control_join(
    context: &TestContext,
    join: &SpatialAudioJoinMetadata,
    control_join: &SpatialControlJoinMetadata,
    phase: ComponentCallbackPhase,
    packet: SpatialAudioInputPacket,
    expected: ComponentStageError,
) {
    let before_generation = context.spatial_audio.generation();
    let before_digest = context.spatial_audio.digest();
    let Err(actual) = produce_component_stage(
        &context.spatial_audio,
        join,
        control_join,
        &context.spatial_control,
        phase,
        packet,
        &context.window,
        &context.parameters,
    ) else {
        panic!("invalid component packet unexpectedly produced a proposal");
    };
    assert_eq!(actual, expected);
    assert_eq!(actual.code(), expected.code());
    assert_eq!(context.spatial_audio.generation(), before_generation);
    assert_eq!(context.spatial_audio.digest(), before_digest);
}

fn unit_vectors() -> Value {
    serde_json::from_slice(UNIT_VECTOR_BYTES).expect("pinned unit vectors are valid JSON")
}

fn vector_case<'a>(document: &'a Value, id: &str) -> &'a Value {
    document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("missing vector case {id}"))
}

fn json_u32(value: &Value) -> u32 {
    u32::try_from(
        value
            .as_u64()
            .unwrap_or_else(|| panic!("expected nonnegative U32 vector word, got {value}")),
    )
    .unwrap()
}

fn finite_from_bits(bits: u32) -> FiniteF32 {
    FiniteF32::new(f32::from_bits(bits)).unwrap()
}

fn digest_words(values: &[FiniteF32]) -> String {
    let mut bytes = Vec::with_capacity(values.len() * F32_BYTES);
    for value in values {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    digest_bytes(&bytes)
}

fn digest_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("writing a hash byte to String cannot fail");
    }
    hex
}

fn row_hashes(values: &[FiniteF32], row_width: usize) -> Vec<String> {
    values.chunks_exact(row_width).map(digest_words).collect()
}

fn row_probes(values: &[FiniteF32], row_width: usize) -> Vec<Vec<u32>> {
    values
        .chunks_exact(row_width)
        .map(|row| {
            [
                row[0].to_bits(),
                row[row_width / 2].to_bits(),
                row[row_width - 1].to_bits(),
            ]
            .to_vec()
        })
        .collect()
}

fn assert_rows_equal(actual: &[FiniteF32], expected: &Value, row_width: usize) {
    assert_eq!(
        row_hashes(actual, row_width),
        expected["row_sha256"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    );
    let expected_probes = expected["probes_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row.as_array()
                .unwrap()
                .iter()
                .map(json_u32)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(row_probes(actual, row_width), expected_probes);
}

fn validated_payload(
    installed48_bits: Vec<u32>,
    matrix_rows: usize,
    matrix_bits: Vec<u32>,
    destinations: Vec<u8>,
) -> ValidatedComponentPayload {
    ValidatedComponentPayload {
        installed48: installed48_bits
            .into_iter()
            .map(finite_from_bits)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        matrix: matrix_bits
            .into_iter()
            .map(finite_from_bits)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        destinations: destinations.into_boxed_slice(),
        matrix_rows,
    }
}

#[test]
fn matrix_stage_replays_u_front_matrix_case() {
    let document = unit_vectors();
    let case = vector_case(&document, "U-FRONT-MATRIX");
    let mut installed48 = vec![0_u32; INSTALLED_WORDS];
    for source in case["input"]["constant_source_rows"].as_array().unwrap() {
        let row = usize::try_from(source["row"].as_u64().unwrap()).unwrap();
        let bits = json_u32(&source["value_u32"]);
        installed48[row * INSTALLED_SAMPLES..(row + 1) * INSTALLED_SAMPLES].fill(bits);
    }
    let mut matrix = vec![0_u32; MATRIX_WORDS];
    for coefficient in case["input"]["nonzero_coefficients"].as_array().unwrap() {
        let row = usize::try_from(coefficient["row"].as_u64().unwrap()).unwrap();
        let column = usize::try_from(coefficient["column"].as_u64().unwrap()).unwrap();
        matrix[row * MATRIX_COLUMN_COUNT + column] = json_u32(&coefficient["value_u32"]);
    }
    let destinations = case["input"]["destinations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();

    let output = mix_matrix(&validated_payload(installed48, 15, matrix, destinations)).unwrap();
    assert_eq!(output.len(), COMPONENT_ROW_COUNT * INSTALLED_SAMPLES);
    assert_eq!(
        row_hashes(&output, INSTALLED_SAMPLES),
        case["expected"]["row_sha256"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    );
    let probes = (0..COMPONENT_ROW_COUNT)
        .map(|row| {
            let offset = row * INSTALLED_SAMPLES;
            vec![
                output[offset].to_bits(),
                output[offset + 1].to_bits(),
                output[offset + INSTALLED_SAMPLES - 1].to_bits(),
            ]
        })
        .collect::<Vec<_>>();
    let expected = case["expected"]["probes_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row.as_array()
                .unwrap()
                .iter()
                .map(json_u32)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(probes, expected);
}

fn assert_raw_rows_equal(
    rows: &[RawBytes; COMPONENT_ROW_COUNT],
    expected: &Value,
    row_width: usize,
) {
    let actual = rows
        .iter()
        .flat_map(|row| (0..row_width).map(|sample| raw_finite_at(row, sample).unwrap()))
        .collect::<Vec<_>>();
    assert_rows_equal(&actual, expected, row_width);
}

#[test]
fn overlap_and_ring_stage_replay_u_front_overlap_case() {
    let document = unit_vectors();
    let case = vector_case(&document, "U-FRONT-OVERLAP");
    let mut component_state = ComponentAudioState::cold();
    let calls = case["input"]["calls"].as_array().unwrap();

    for (call_index, call) in calls.iter().enumerate() {
        let head_offset = call["head_offset"].as_f64().unwrap() as f32;
        let tail_offset = call["tail_offset"].as_f64().unwrap() as f32;
        let filtered_offset = call["filtered_offset"].as_f64().unwrap() as f32;
        let mut mixer =
            vec![finite_constant(0.0).unwrap(); COMPONENT_ROW_COUNT * INSTALLED_SAMPLES];
        let mut filtered = vec![finite_constant(0.0).unwrap(); COMPONENT_WORDS];
        for row in 0..COMPONENT_ROW_COUNT {
            for sample in 0..INSTALLED_SAMPLES {
                let offset = if sample < FRAME_SAMPLES {
                    head_offset
                } else {
                    tail_offset
                };
                mixer[row * INSTALLED_SAMPLES + sample] =
                    FiniteF32::new(row as f32 + offset).unwrap();
            }
            filtered[row * FRAME_SAMPLES..(row + 1) * FRAME_SAMPLES]
                .fill(FiniteF32::new(row as f32 + filtered_offset).unwrap());
        }

        let overlap = make_overlap(&component_state, &mixer).unwrap();
        assert_rows_equal(
            &overlap.samples,
            &case["expected"]["calls"][call_index]["overlap"],
            FRAME_SAMPLES,
        );
        let (type10_component, after) =
            apply_source_ring(&component_state, overlap.next_overlap, &filtered).unwrap();
        assert_rows_equal(
            &type10_component,
            &case["expected"]["calls"][call_index]["type10_component"],
            FRAME_SAMPLES,
        );
        component_state = after;
    }

    assert_raw_rows_equal(
        &component_state.overlap_history_rows,
        &case["expected"]["terminal_overlap"],
        FRAME_SAMPLES,
    );
    assert_raw_rows_equal(
        &component_state.source_ring_rows,
        &case["expected"]["terminal_ring"],
        RING_SAMPLES,
    );
}

#[test]
fn filter_stage_replays_u_front_filter64_case() {
    let document = unit_vectors();
    let case = vector_case(&document, "U-FRONT-FILTER64");
    let parameters = &case["input"]["parameters"];
    let kernel = parameters["kernel_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| finite_from_bits(json_u32(word)))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let mut history = vec![finite_constant(0.0).unwrap(); COMPONENT_ROW_COUNT * BLOCK_SAMPLES];
    let previous_gain = finite_from_bits(json_u32(&parameters["previous_gain_u32"]));
    let smoothed_gain = finite_from_bits(json_u32(&parameters["smoothed_gain_u32"]));
    let output_level = finite_from_bits(json_u32(&parameters["output_level_u32"]));
    let gain_scale = finite_from_bits(json_u32(&parameters["gain_scale_u32"]));
    let clamp_limit = finite_from_bits(json_u32(&parameters["clamp_limit_u32"]));

    for (block_index, amplitudes) in case["input"]["block_amplitudes_u32"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let mut source = vec![finite_constant(0.0).unwrap(); COMPONENT_ROW_COUNT * BLOCK_SAMPLES];
        for row in 0..COMPONENT_ROW_COUNT {
            let amplitude = finite_from_bits(json_u32(&amplitudes[row]));
            source[row * BLOCK_SAMPLES..(row + 1) * BLOCK_SAMPLES].fill(amplitude);
        }
        let actual = filter_component_block(
            &source,
            &mut history,
            &FilterBlockSettings {
                kernel: &kernel,
                output_level,
                previous_gain,
                smoothed_gain,
                gain_scale,
                clamp_limit,
            },
        )
        .unwrap();
        assert_rows_equal(
            &actual.preclip,
            &case["expected"]["blocks"][block_index]["preclip"],
            BLOCK_SAMPLES,
        );
        assert_rows_equal(
            &actual.samples,
            &case["expected"]["blocks"][block_index]["postclip"],
            BLOCK_SAMPLES,
        );
    }
    assert_eq!(
        row_hashes(&history, BLOCK_SAMPLES),
        case["expected"]["terminal_history"]["row_sha256"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    );
    let actual_probes = row_probes(&history, BLOCK_SAMPLES);
    let expected_probes = case["expected"]["terminal_history"]["probes_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row.as_array()
                .unwrap()
                .iter()
                .map(json_u32)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_probes, expected_probes);
}

fn synthetic_controller(parameters: &Value) -> ComponentControllerParameters {
    let coefficients = parameters["coefficients_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| finite_from_bits(json_u32(word)))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    ComponentControllerParameters {
        coefficients,
        reciprocal_pivot: finite_from_bits(json_u32(&parameters["reciprocal_pivot_u32"])),
        target_pivot: finite_from_bits(json_u32(&parameters["target_pivot_u32"])),
        envelope_weight: finite_from_bits(json_u32(&parameters["envelope_weight_u32"])),
        gain_scale: finite_from_bits(0x3b80_0000),
        clamp_limit: finite_from_bits(0x3f80_0000),
        hold_limit: 1,
    }
}

fn assert_parameter_state_matches(state: &ComponentFilterParameterState, expected: &Value) {
    assert_eq!(state.peak.to_bits(), json_u32(&expected["peak_u32"]));
    assert_eq!(
        state.previous_gain.to_bits(),
        json_u32(&expected["previous_gain_u32"])
    );
    assert_eq!(state.hold, expected["hold"].as_i64().unwrap());
    assert_eq!(
        state.envelope.to_bits(),
        json_u32(&expected["envelope_u32"])
    );
    assert_eq!(
        state.smoothed_gain.to_bits(),
        json_u32(&expected["smoothed_gain_u32"])
    );
    assert_eq!(
        state.target_gain.to_bits(),
        json_u32(&expected["target_gain_u32"])
    );
    assert_eq!(
        state.output_level.to_bits(),
        json_u32(&expected["output_level_u32"])
    );
    assert_eq!(
        state.blocks_processed,
        expected["blocks_processed"].as_i64().unwrap()
    );
}

fn unit_branch_name(branch: GainBranch) -> &'static str {
    match branch {
        GainBranch::HoldCounterIncrement => "hold",
        GainBranch::TargetRecurrenceAndSmoothedClamp => "recover",
        GainBranch::GainFallsToEnvelope => "fall",
    }
}

fn f32_value(value: &Value) -> f32 {
    value.as_f64().unwrap() as f32
}

fn f32_bits_value(value: &Value) -> u32 {
    f32_value(value).to_bits()
}

fn parameter_state_from_input(value: &Value) -> ComponentFilterParameterState {
    ComponentFilterParameterState {
        peak: f32_value(&value["peak"]),
        previous_gain: f32_value(&value["previous_gain"]),
        hold: value["hold"].as_i64().unwrap(),
        envelope: f32_value(&value["envelope"]),
        smoothed_gain: f32_value(&value["smoothed_gain"]),
        target_gain: f32_value(&value["target_gain"]),
        output_level: f32_value(&value["output_level"]),
        blocks_processed: value["blocks_processed"].as_i64().unwrap(),
    }
}

fn generation_block(input: &Value) -> Vec<FiniteF32> {
    let mut block = vec![finite_constant(0.0).unwrap(); COMPONENT_ROW_COUNT * BLOCK_SAMPLES];
    if let Some(magnitude) = input.get("max_magnitude") {
        let magnitude = f32_value(magnitude);
        if magnitude != 0.0 {
            let negative = input["negative_witness"].as_array().unwrap();
            let positive = input["positive_equal_witness"].as_array().unwrap();
            let negative_index = usize::try_from(negative[0].as_u64().unwrap()).unwrap()
                * BLOCK_SAMPLES
                + usize::try_from(negative[1].as_u64().unwrap()).unwrap();
            let positive_index = usize::try_from(positive[0].as_u64().unwrap()).unwrap()
                * BLOCK_SAMPLES
                + usize::try_from(positive[1].as_u64().unwrap()).unwrap();
            block[negative_index] = FiniteF32::new(-magnitude).unwrap();
            block[positive_index] = FiniteF32::new(magnitude).unwrap();
        }
    } else if let Some(peak_value) = input.get("peak_value") {
        let location = input["peak_row_sample"].as_array().unwrap();
        let index = usize::try_from(location[0].as_u64().unwrap()).unwrap() * BLOCK_SAMPLES
            + usize::try_from(location[1].as_u64().unwrap()).unwrap();
        block[index] = FiniteF32::new(f32_value(peak_value)).unwrap();
    }
    block
}

fn parameter_words_before(
    state: &ComponentFilterParameterState,
    parameters: &ComponentControllerParameters,
) -> Map<String, Value> {
    let mut fields = Map::new();
    fields.insert("active_u32".to_owned(), Value::from(1_u32));
    fields.insert(
        "clamp_limit_f32_u32".to_owned(),
        Value::from(parameters.clamp_limit.to_bits()),
    );
    fields.insert(
        "gain_scale_f32_u32".to_owned(),
        Value::from(parameters.gain_scale.to_bits()),
    );
    fields.insert("direct_route_flag_u32".to_owned(), Value::from(0_u32));
    fields.insert(
        "row_count_u32".to_owned(),
        Value::from(COMPONENT_ROW_COUNT as u32),
    );
    fields.insert(
        "block_sample_count_u32".to_owned(),
        Value::from(BLOCK_SAMPLES as u32),
    );
    fields.insert(
        "processing_format_u32".to_owned(),
        Value::from(BLOCK_SAMPLES as u32),
    );
    fields.insert(
        "peak_before_f32_u32".to_owned(),
        Value::from(state.peak.to_bits()),
    );
    fields.insert(
        "previous_gain_f32_u32".to_owned(),
        Value::from(state.previous_gain.to_bits()),
    );
    fields.insert(
        "gain_hold_counter_u32".to_owned(),
        Value::from(u32::try_from(state.hold).unwrap()),
    );
    fields.insert(
        "gain_hold_limit_u32".to_owned(),
        Value::from(u32::try_from(parameters.hold_limit).unwrap()),
    );
    fields.insert(
        "envelope_f32_u32".to_owned(),
        Value::from(state.envelope.to_bits()),
    );
    fields.insert(
        "smoothed_gain_f32_u32".to_owned(),
        Value::from(state.smoothed_gain.to_bits()),
    );
    fields.insert(
        "target_gain_f32_u32".to_owned(),
        Value::from(state.target_gain.to_bits()),
    );
    fields.insert("filter_iteration_count_u32".to_owned(), Value::from(1_u32));
    fields.insert(
        "output_level_f32_u32".to_owned(),
        Value::from(state.output_level.to_bits()),
    );
    fields
}

fn parameter_words_after(
    state: &ComponentFilterParameterState,
    parameters: &ComponentControllerParameters,
) -> Map<String, Value> {
    let mut fields = Map::new();
    fields.insert(
        "peak_before_f32_u32".to_owned(),
        Value::from(state.peak.to_bits()),
    );
    fields.insert(
        "previous_gain_f32_u32".to_owned(),
        Value::from(state.previous_gain.to_bits()),
    );
    fields.insert(
        "gain_hold_counter_u32".to_owned(),
        Value::from(u32::try_from(state.hold).unwrap()),
    );
    fields.insert(
        "gain_hold_limit_u32".to_owned(),
        Value::from(u32::try_from(parameters.hold_limit).unwrap()),
    );
    fields.insert(
        "envelope_f32_u32".to_owned(),
        Value::from(state.envelope.to_bits()),
    );
    fields.insert(
        "smoothed_gain_f32_u32".to_owned(),
        Value::from(state.smoothed_gain.to_bits()),
    );
    fields.insert(
        "target_gain_f32_u32".to_owned(),
        Value::from(state.target_gain.to_bits()),
    );
    fields.insert(
        "output_level_f32_u32".to_owned(),
        Value::from(state.output_level.to_bits()),
    );
    fields
}

fn assert_parameter_state_json(state: &ComponentFilterParameterState, expected: &Value) {
    for (field, actual_bits) in [
        ("peak", state.peak.to_bits()),
        ("previous_gain", state.previous_gain.to_bits()),
        ("envelope", state.envelope.to_bits()),
        ("smoothed_gain", state.smoothed_gain.to_bits()),
        ("target_gain", state.target_gain.to_bits()),
        ("output_level", state.output_level.to_bits()),
    ] {
        assert_eq!(
            actual_bits,
            f32_bits_value(&expected[field]),
            "field {field}"
        );
    }
    assert_eq!(state.hold, expected["hold"].as_i64().unwrap());
    assert_eq!(
        state.blocks_processed,
        expected["blocks_processed"].as_i64().unwrap()
    );
}

#[test]
fn parameter_stage_replays_u_front_parameter64_case() {
    let document = unit_vectors();
    let case = vector_case(&document, "U-FRONT-PARAMETER64");
    let controller = synthetic_controller(&case["input"]["parameters"]);
    let mut state = ComponentFilterParameterState::cold();

    for (index, amplitudes) in case["input"]["block_amplitudes_u32"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let mut block = vec![finite_constant(0.0).unwrap(); COMPONENT_ROW_COUNT * BLOCK_SAMPLES];
        for row in 0..COMPONENT_ROW_COUNT {
            block[row * BLOCK_SAMPLES] = finite_from_bits(json_u32(&amplitudes[row]));
        }
        let transition = update_parameters(&state, &block, &controller).unwrap();
        let expected = &case["expected"]["records"][index];
        assert_eq!(
            transition.current_peak.to_bits(),
            json_u32(&expected["peak_u32"])
        );
        assert_eq!(
            transition.scaled_peak.to_bits(),
            json_u32(&expected["scaled_peak_u32"])
        );
        assert_eq!(
            transition.attenuation.to_bits(),
            json_u32(&expected["attenuation_u32"])
        );
        assert_eq!(
            transition.after.previous_gain.to_bits(),
            json_u32(&expected["previous_gain_u32"])
        );
        assert_eq!(transition.after.hold, expected["hold"].as_i64().unwrap());
        assert_eq!(
            transition.after.envelope.to_bits(),
            json_u32(&expected["envelope_u32"])
        );
        assert_eq!(
            transition.after.smoothed_gain.to_bits(),
            json_u32(&expected["smoothed_gain_u32"])
        );
        assert_eq!(
            transition.after.target_gain.to_bits(),
            json_u32(&expected["target_gain_u32"])
        );
        assert_eq!(
            transition.after.output_level.to_bits(),
            json_u32(&expected["output_level_u32"])
        );
        assert_eq!(
            transition.after.blocks_processed,
            expected["blocks_processed"].as_i64().unwrap()
        );
        assert_eq!(
            unit_branch_name(transition.gain_branch),
            expected["branch"].as_str().unwrap()
        );
        assert_eq!(
            transition.scaled_peak.value() > 1.0,
            expected["attenuation_executed"].as_bool().unwrap()
        );
        state = transition.after;
    }
    assert_parameter_state_matches(&state, &case["expected"]["terminal"]);
}

#[test]
fn parameter_generation_replays_all_23_input_cases_in_five_sequences() {
    let context = context();
    let parameters = ComponentControllerParameters::from_provider(&context.parameters).unwrap();
    let document: Value = serde_json::from_slice(PARAMETER_GENERATION_BYTES).unwrap();
    let cases = document["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 23);
    let mut executed = 0;

    for sequence in document["execution_plan"].as_array().unwrap() {
        let sequence_id = sequence["sequence_id"].as_str().unwrap();
        let mut state = ComponentFilterParameterState::cold();
        let mut predecessor: Option<String> = None;
        for (block_index, case_id) in sequence["case_ids"].as_array().unwrap().iter().enumerate() {
            let case_id = case_id.as_str().unwrap();
            let case = cases
                .iter()
                .find(|candidate| candidate["id"] == case_id)
                .unwrap();
            let execution = &case["execution"];
            assert_eq!(execution["sequence_id"].as_str().unwrap(), sequence_id);
            assert_eq!(
                execution["block_index"].as_u64().unwrap(),
                block_index as u64
            );
            assert_eq!(execution["predecessor"].as_str(), predecessor.as_deref());
            assert_eq!(
                execution["reset_before"].as_bool().unwrap(),
                block_index == 0
            );

            if let Some(input_state) = case["input"].get("state") {
                assert_eq!(block_index, 0, "state override is first-block only");
                state = parameter_state_from_input(input_state);
            }

            assert_eq!(
                Value::Object(parameter_words_before(&state, &parameters)),
                case["expected_before_words"],
                "before words for {case_id}"
            );
            let transition =
                update_parameters(&state, &generation_block(&case["input"]), &parameters)
                    .unwrap_or_else(|error| panic!("{case_id}: {error:?}"));
            assert_eq!(
                Value::Object(parameter_words_after(&transition.after, &parameters)),
                case["expected_after_words"],
                "after words for {case_id}"
            );
            assert_parameter_state_json(&transition.after, &case["expected_state"]);
            if let Some(word) = case
                .get("expected_attenuation_word")
                .filter(|value| !value.is_null())
            {
                assert_eq!(
                    transition.attenuation.to_bits(),
                    json_u32(word),
                    "attenuation for {case_id}"
                );
            }
            if let Some(word) = case
                .get("expected_peak_word")
                .filter(|value| !value.is_null())
            {
                assert_eq!(
                    transition.current_peak.to_bits(),
                    json_u32(word),
                    "peak for {case_id}"
                );
            }
            if case["input"].get("max_magnitude").is_some() {
                match case.get("expected_peak_witness") {
                    Some(Value::Null) | None => assert_eq!(transition.peak_witness, None),
                    Some(value) => {
                        let witness = value.as_array().unwrap();
                        assert_eq!(
                            transition.peak_witness,
                            Some((
                                usize::try_from(witness[0].as_u64().unwrap()).unwrap(),
                                usize::try_from(witness[1].as_u64().unwrap()).unwrap(),
                            )),
                            "peak witness for {case_id}"
                        );
                    }
                }
            }
            if let Some(branch) = case.get("expected_gain_branch").and_then(Value::as_str) {
                assert_eq!(
                    transition.gain_branch.as_str(),
                    branch,
                    "branch for {case_id}"
                );
            }

            state = transition.after;
            predecessor = Some(case_id.to_owned());
            executed += 1;
        }
    }
    assert_eq!(executed, 23);
}

fn digest_u32_words(words: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(words.len() * F32_BYTES);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    digest_bytes(&bytes)
}

fn whole_chain_installed48(callback: usize) -> Vec<u32> {
    let mut words = Vec::with_capacity(INSTALLED_WORDS);
    for row in 0..SOURCE_ROW_COUNT {
        for sample in 0..INSTALLED_SAMPLES {
            let value = match callback {
                0 => {
                    let numerator =
                        i32::try_from((sample * (row + 1) + 7 * row) % 127).unwrap() - 63;
                    f32::from(i16::try_from(numerator).unwrap()) / 16_384.0_f32
                }
                1 => {
                    let numerator =
                        i32::try_from((sample * (row + 3) + 11 * row) % 257).unwrap() - 128;
                    f32::from(i16::try_from(numerator).unwrap()) / 32_768.0_f32
                }
                _ if (sample + row + callback) % 29 == 0 => 0.0,
                _ => {
                    let numerator = i32::try_from((sample * (row + 5) + 17 * callback + row) % 193)
                        .unwrap()
                        - 96;
                    f32::from(i16::try_from(numerator).unwrap()) / 65_536.0_f32
                }
            };
            words.push(value.to_bits());
        }
    }
    words
}

fn whole_chain_matrix(callback: usize) -> (Vec<u32>, Vec<u8>) {
    let mut matrix = vec![0_u32; MATRIX_WORDS];
    let mut destinations = Vec::with_capacity(COMPONENT_ROW_COUNT);
    for row in 0..COMPONENT_ROW_COUNT {
        let first = (3 * row + callback) % MATRIX_COLUMN_COUNT;
        let second = (first + 17) % MATRIX_COLUMN_COUNT;
        matrix[row * MATRIX_COLUMN_COUNT + first] = 0x3f00_0000;
        matrix[row * MATRIX_COLUMN_COUNT + second] = 0x3e80_0000;
        destinations.push(((row + callback) % COMPONENT_ROW_COUNT) as u8);
    }
    (matrix, destinations)
}

#[test]
fn whole_chain_component_projection_replays_two_normal_and_four_drain_callbacks() {
    let context = context();
    let parameters = ComponentControllerParameters::from_provider(&context.parameters).unwrap();
    let kernel = component_kernel_from_provider(&context.window).unwrap();
    let document: Value = serde_json::from_slice(WHOLE_SPATIAL_BYTES).unwrap();
    assert!(document["input_only"].as_bool().unwrap());
    let mut component_state = ComponentAudioState::cold();
    let mut parameter_state = ComponentFilterParameterState::cold();
    let mut filter_history = std::array::from_fn(|_| RawBytes::zeroed(FILTER_HISTORY_BYTES));

    for (callback, case) in document["cases"].as_array().unwrap().iter().enumerate() {
        let phase = if callback < 2 {
            "NORMAL"
        } else {
            "CACHED_DRAIN"
        };
        assert_eq!(case["input"]["phase"].as_str().unwrap(), phase);
        assert_eq!(
            case["input"]["callback_index"].as_u64().unwrap(),
            callback as u64
        );
        let installed48 = whole_chain_installed48(callback);
        assert_eq!(
            digest_u32_words(&installed48),
            case["input"]["installed48_f32le_sha256"].as_str().unwrap()
        );
        let (matrix_bits, destinations) = whole_chain_matrix(callback);
        assert_eq!(
            digest_u32_words(&matrix_bits),
            case["input"]["matrix_u32le_sha256"].as_str().unwrap()
        );
        let payload =
            validated_payload(installed48, COMPONENT_ROW_COUNT, matrix_bits, destinations);
        let mixer = mix_matrix(&payload).unwrap();
        assert_eq!(
            digest_words(&mixer),
            case["expected"]["matrix15x3072_f32le_sha256"]
                .as_str()
                .unwrap()
        );

        let (overlap_samples, filtered, component_output, after_component) = {
            let before = ComponentStateView {
                component_state: &component_state,
                parameters: &parameter_state,
                filter_history: &filter_history,
            };
            validate_component_state(&before).unwrap();
            let overlap = make_overlap(before.component_state, &mixer).unwrap();
            let overlap_samples = overlap.samples;
            let filtered = filter_component(
                &overlap_samples,
                before.parameters,
                before.filter_history,
                &kernel,
                &parameters,
            )
            .unwrap();
            let (component_output, after_component) = apply_source_ring(
                before.component_state,
                overlap.next_overlap,
                &filtered.samples,
            )
            .unwrap();
            (overlap_samples, filtered, component_output, after_component)
        };

        assert_eq!(
            digest_words(&overlap_samples),
            case["expected"]["overlap15x1536_f32le_sha256"]
                .as_str()
                .unwrap()
        );
        assert_eq!(
            digest_words(&component_output),
            case["expected"]["component15x1536_f32le_sha256"]
                .as_str()
                .unwrap()
        );
        assert_eq!(
            u64::try_from(filtered.parameters.blocks_processed).unwrap(),
            case["expected"]["component_parameter_blocks_after"]
                .as_u64()
                .unwrap()
        );
        component_state = after_component;
        parameter_state = filtered.parameters;
        filter_history = filtered.history;
    }
}

#[test]
fn cold_component_stage_produces_complete_proposal_without_local_commit() {
    let context = context();
    let before_generation = context.spatial_audio.generation();
    let before_digest = context.spatial_audio.digest();
    let join = callback_join(&context, FrontendCallbackIndex::new(0));

    let proposal = produce_component_stage(
        &context.spatial_audio,
        &join,
        &control_join(&context, FrontendCallbackIndex::new(0)),
        &context.spatial_control,
        ComponentCallbackPhase::Normal,
        zero_packet(&context),
        &context.window,
        &context.parameters,
    )
    .expect("the selected cold component consumer stage produces a proposal");

    assert_eq!(context.spatial_audio.generation(), before_generation);
    assert_eq!(context.spatial_audio.digest(), before_digest);
    assert_eq!(
        proposal.receipt.spatial_audio_before_generation,
        before_generation
    );
    assert_eq!(proposal.receipt.spatial_audio_before_digest, before_digest);
    assert_eq!(proposal.after.parameters.blocks_processed, 24);
    assert_eq!(proposal.after.parameters.hold, 1);
    assert_eq!(
        proposal.after.audio.overlap_history_rows.len(),
        COMPONENT_ROW_COUNT
    );
    assert_eq!(
        proposal.after.audio.source_ring_rows.len(),
        COMPONENT_ROW_COUNT
    );
    assert_eq!(proposal.after.filter_history.len(), COMPONENT_ROW_COUNT);
    assert!(proposal.after.audio.overlap_history_rows.iter().all(|row| {
        row.bytes.len() == FRAME_SAMPLES * F32_BYTES && row.bytes.iter().all(|byte| *byte == 0)
    }));
    assert!(proposal.after.audio.source_ring_rows.iter().all(|row| {
        row.bytes.len() == RING_SAMPLES * F32_BYTES && row.bytes.iter().all(|byte| *byte == 0)
    }));
    assert!(proposal.after.filter_history.iter().all(|row| {
        row.bytes.len() == FILTER_HISTORY_BYTES && row.bytes.iter().all(|byte| *byte == 0)
    }));
    assert_eq!(proposal.component_output.len(), 15 * 1536);
    assert!(
        proposal
            .component_output
            .iter()
            .all(|sample| sample.to_bits() == 0)
    );
    assert_eq!(
        proposal.receipt.callback_index,
        FrontendCallbackIndex::new(0)
    );
    assert_eq!(
        proposal.receipt.provider_lineage,
        provider_lineage(&context)
    );
    assert_eq!(
        proposal.receipt.spatial_control_before_generation,
        context.spatial_control.generation()
    );
    assert_eq!(
        proposal.receipt.spatial_control_before_digest,
        context.spatial_control.digest()
    );
    assert_eq!(
        proposal.receipt.spatial_control_provider_lineage,
        context.spatial_control.provider_lineage()
    );
    assert_eq!(proposal.receipt.phase, ComponentCallbackPhase::Normal);
    assert_eq!(proposal.receipt.profile_identity, context.profile);
    assert_eq!(proposal.receipt.programme_id, component_programme_id());
    assert_eq!(proposal.receipt.attempt, SourceFlagsAttempt::new(7));
    assert_eq!(
        proposal.receipt.transaction_id,
        component_transaction_id(FrontendCallbackIndex::new(0))
    );
}

#[test]
fn component_stage_rejects_lineage_profile_phase_and_input_mismatches_without_proposal() {
    let context = context();
    let callback = FrontendCallbackIndex::new(0);
    let join = callback_join(&context, callback);

    let mut packet = zero_packet(&context);
    packet.programme_id = crate::transaction::test_programme_id("other-programme");
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::Input,
    );

    let mut packet = zero_packet(&context);
    packet.attempt = SourceFlagsAttempt::new(8);
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::Input,
    );

    let mut packet = zero_packet(&context);
    packet.transaction_id = CallbackLineageIdentity::new(
        crate::transaction::test_programme_id("other-component-programme"),
        SourceFlagsAttempt::new(7),
        callback,
    );
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::Input,
    );

    let mut packet = zero_packet(&context);
    packet.callback_index = FrontendCallbackIndex::new(1);
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::CallbackOrder,
    );

    let packet = zero_packet(&context);
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::CachedDrain,
        packet,
        ComponentStageError::CallbackOrder,
    );

    let mut packet = zero_packet(&context);
    packet.profile_identity =
        EncodingProfile::selected_with_carriage(EmdfCarriageProfile::EtsiEmdf);
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::ProfileUnsupported,
    );

    let stale_join = join_with(&context, callback, 1, context.profile);
    let packet = zero_packet(&context);
    assert_stage_error(
        &context,
        &stale_join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::State,
    );

    let stale_digest_join = SpatialAudioJoinMetadata::new(
        component_programme_id(),
        SourceFlagsAttempt::new(7),
        component_transaction_id(callback),
        callback,
        context.spatial_audio.generation(),
        super::super::SpatialAudioDigest([0x55; 32]),
        context.profile,
    );
    let mut packet = zero_packet(&context);
    packet.transaction_id = CallbackLineageIdentity::new(
        crate::transaction::test_programme_id("wrong-component-programme"),
        SourceFlagsAttempt::new(7),
        callback,
    );
    assert_stage_error(
        &context,
        &stale_digest_join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::State,
    );

    let mut packet = zero_packet(&context);
    packet.spatial_control_before_generation += 1;
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::State,
    );

    let mut packet = zero_packet(&context);
    packet.provider_lineage.swap(0, 1);
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::Input,
    );

    let mut packet = zero_packet(&context);
    let mut malformed_installed = packet.installed48_f32_bits.to_vec();
    malformed_installed.pop();
    packet.installed48_f32_bits = malformed_installed.into_boxed_slice();
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::Input,
    );

    let mut packet = zero_packet(&context);
    packet.installed48_f32_bits[0] = 0x7f80_0000;
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::Input,
    );

    let mut packet = zero_packet(&context);
    packet.matrix_rows = 2;
    packet.matrix_f32_bits = vec![0; 2 * MATRIX_COLUMN_COUNT].into_boxed_slice();
    packet.destinations = vec![0, 0].into_boxed_slice();
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::Input,
    );
}

#[test]
fn component_stage_validates_independent_spatial_control_join_metadata() {
    let context = context();
    let callback = FrontendCallbackIndex::new(0);
    let audio_join = callback_join(&context, callback);

    let wrong_transaction = control_join_with(
        &context,
        CallbackLineageIdentity::new(
            crate::transaction::test_programme_id("other-component-programme"),
            SourceFlagsAttempt::new(7),
            callback,
        ),
        callback,
        context.spatial_control.generation(),
        None,
        context.profile,
    );
    assert_stage_error_with_control_join(
        &context,
        &audio_join,
        &wrong_transaction,
        ComponentCallbackPhase::Normal,
        zero_packet(&context),
        ComponentStageError::Input,
    );

    let wrong_callback = control_join(&context, FrontendCallbackIndex::new(1));
    assert_stage_error_with_control_join(
        &context,
        &audio_join,
        &wrong_callback,
        ComponentCallbackPhase::Normal,
        zero_packet(&context),
        ComponentStageError::CallbackOrder,
    );

    let wrong_profile = control_join_with(
        &context,
        component_transaction_id(callback),
        callback,
        context.spatial_control.generation(),
        None,
        EncodingProfile::selected_with_carriage(EmdfCarriageProfile::EtsiEmdf),
    );
    assert_stage_error_with_control_join(
        &context,
        &audio_join,
        &wrong_profile,
        ComponentCallbackPhase::Normal,
        zero_packet(&context),
        ComponentStageError::ProfileUnsupported,
    );

    let wrong_generation = control_join_with(
        &context,
        component_transaction_id(callback),
        callback,
        context.spatial_control.generation() + 1,
        None,
        context.profile,
    );
    assert_stage_error_with_control_join(
        &context,
        &audio_join,
        &wrong_generation,
        ComponentCallbackPhase::Normal,
        zero_packet(&context),
        ComponentStageError::State,
    );

    let wrong_digest = control_join_with(
        &context,
        component_transaction_id(callback),
        callback,
        context.spatial_control.generation(),
        Some([0x55; 32]),
        context.profile,
    );
    assert_stage_error_with_control_join(
        &context,
        &audio_join,
        &wrong_digest,
        ComponentCallbackPhase::Normal,
        zero_packet(&context),
        ComponentStageError::State,
    );
}

#[test]
fn component_stage_uses_fail_closed_error_precedence_and_rejects_numeric_overflow() {
    let mut context = context();
    let callback = FrontendCallbackIndex::new(0);
    let join = callback_join(&context, callback);

    context.spatial_audio.committed.root.retired = true;
    let mut packet = zero_packet(&context);
    packet.profile_identity =
        EncodingProfile::selected_with_carriage(EmdfCarriageProfile::EtsiEmdf);
    packet.callback_index = FrontendCallbackIndex::new(1);
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::PipelineRetired,
    );
    context.spatial_audio.committed.root.retired = false;

    let join = callback_join(&context, callback);
    let mut packet = zero_packet(&context);
    packet.matrix_rows = 1;
    packet.matrix_f32_bits = vec![0; MATRIX_COLUMN_COUNT].into_boxed_slice();
    packet.matrix_f32_bits[0] = f32::MAX.to_bits();
    packet.destinations = vec![0].into_boxed_slice();
    packet.installed48_f32_bits[0] = f32::MAX.to_bits();
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::NumericFailure,
    );

    context
        .spatial_audio
        .committed
        .root
        .component_state
        .overlap_history_rows[0] = RawBytes::zeroed(0);
    let join = callback_join(&context, callback);
    let packet = zero_packet(&context);
    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::State,
    );
}

#[test]
fn component_stage_rejects_late_block_numeric_failure_without_partial_proposal() {
    let context = context();
    let callback = FrontendCallbackIndex::new(0);
    let join = callback_join(&context, callback);
    let mut packet = zero_packet(&context);
    packet.matrix_rows = 1;
    packet.matrix_f32_bits = vec![0; MATRIX_COLUMN_COUNT].into_boxed_slice();
    packet.matrix_f32_bits[0] = 0x3f80_0000;
    packet.destinations = vec![0].into_boxed_slice();
    packet.installed48_f32_bits[23 * BLOCK_SAMPLES] = f32::MAX.to_bits();

    assert_stage_error(
        &context,
        &join,
        ComponentCallbackPhase::Normal,
        packet,
        ComponentStageError::NumericFailure,
    );
}

#[test]
fn component_stage_error_codes_are_explicit_local_translations() {
    assert_eq!(ComponentStageError::State.code(), "F04_STATE_ERROR");
    assert_eq!(
        ComponentStageError::PipelineRetired.code(),
        "F04_PIPELINE_RETIRED"
    );
    assert_eq!(
        ComponentStageError::CallbackOrder.code(),
        "F04_CALLBACK_ORDER"
    );
    assert_eq!(ComponentStageError::Input.code(), "F04_INPUT_ERROR");
    assert_eq!(
        ComponentStageError::ProfileUnsupported.code(),
        "F04_PROFILE_UNSUPPORTED"
    );
    assert_eq!(
        ComponentStageError::NumericFailure.code(),
        "F04_NUMERIC_FAILURE"
    );
}
