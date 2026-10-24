use std::{fmt::Write as _, sync::Arc};

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{
    DiffusionFilterParameters, DiffusionFilterState, PackageManifestEvidence, ProviderFileEvidence,
    preview_filter_block, validate_diffusion_filter_package_manifest,
    validate_diffusion_filter_parameters,
};

const PACKAGE_MANIFEST: &[u8] =
    include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
const DIFFUSION_PARAMETERS: &[u8] =
    include_bytes!("../../../tests/fixtures/diffusion_filter/diffusion-mathematical-v1.json");
const CONSTRUCTOR_PARAMETERS: &[u8] = include_bytes!(
    "../../../tests/fixtures/diffusion_filter/diffusion-constructor-mathematical-v1.json"
);
const LOCAL_VECTORS: &str =
    include_str!("../../../tests/fixtures/diffusion_filter/DIFFUSION-LOCAL-VERIFICATION.json");
const MATH_VECTORS: &str = include_str!(
    "../../../tests/fixtures/diffusion_filter/DIFFUSION-MATH-GENERATION-VERIFICATION.json"
);
const CONSTRUCTOR_VECTORS: &str = include_str!(
    "../../../tests/fixtures/diffusion_filter/DIFFUSION-CONSTRUCTOR-GENERATION-VERIFICATION.json"
);

fn selected_parameters() -> DiffusionFilterParameters {
    let package_manifest = PackageManifestEvidence::new(Arc::from(PACKAGE_MANIFEST), None);
    let manifest = validate_diffusion_filter_package_manifest(&package_manifest).unwrap();
    let coefficient_file = ProviderFileEvidence::new(Arc::from(DIFFUSION_PARAMETERS), None);
    let constructor_file = ProviderFileEvidence::new(Arc::from(CONSTRUCTOR_PARAMETERS), None);
    validate_diffusion_filter_parameters(&coefficient_file, &constructor_file, &manifest).unwrap()
}

#[test]
fn selected_parameters_create_the_complete_washed_cold_state() {
    let parameters = selected_parameters();
    let state = parameters.cold_state().unwrap();
    let expected: Value = serde_json::from_str(CONSTRUCTOR_VECTORS).unwrap();

    assert_eq!(state.sample_rate_hz(), 48_000);
    assert_eq!(state.bank_lengths(), [152, 200, 263, 346]);
    assert_eq!(state.ring_length(), 96);
    assert_eq!(
        state.sha256_hex(),
        expected["expected_cold_state_sha256"].as_str().unwrap()
    );
    assert_eq!(
        expected["actual_cold_state_sha256"],
        expected["expected_cold_state_sha256"]
    );
}

#[test]
fn selected_continuous_block_recipes_match_lane_and_complete_state_digests() {
    let parameters = selected_parameters();
    let math: Value = serde_json::from_str(MATH_VECTORS).unwrap();
    let constructor: Value = serde_json::from_str(CONSTRUCTOR_VECTORS).unwrap();
    let mut state = parameters.cold_state().unwrap();

    for (math_case, constructor_case) in math["continuous_cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(constructor["continuous_cases"].as_array().unwrap())
    {
        assert_eq!(math_case["callback"], constructor_case["callback"]);
        let callback = math_case["callback"].as_u64().unwrap();
        let sample_count = math_case["samples"].as_u64().unwrap() as usize;
        let mut input = Vec::with_capacity(sample_count);
        for sample_index in 0..sample_count {
            let callback_zero_index = callback - 1;
            let residue =
                u16::try_from((sample_index as u64 * 37 + callback_zero_index * 13) % 257).unwrap();
            let numerator = i16::try_from(i32::from(residue) - 128).unwrap();
            input.push(f32::from(numerator) / 512.0);
        }

        let proposal = preview_filter_block(&parameters, &state, &input).unwrap();
        let (lanes, next_state) = proposal.into_parts();
        let math_hashes = math_case["output_sha256"].as_array().unwrap();
        let constructor_hashes = constructor_case["lane_sha256"].as_array().unwrap();
        for lane in 0..4 {
            let actual_hash = lane_sha256(&lanes[lane]);
            assert_eq!(actual_hash, math_hashes[lane].as_str().unwrap());
            assert_eq!(actual_hash, constructor_hashes[lane].as_str().unwrap());
        }
        assert_eq!(
            next_state.sha256_hex(),
            math_case["expected_state_sha256"].as_str().unwrap()
        );
        assert_eq!(
            next_state.sha256_hex(),
            constructor_case["expected_state_sha256"].as_str().unwrap()
        );
        state = next_state;
    }
}

#[test]
fn local_synthetic_delay_fade_empty_and_warm_split_vectors_match() {
    let local: Value = serde_json::from_str(LOCAL_VECTORS).unwrap();
    let parameters = synthetic_parameters(
        local["synthetic_envelope_epsilon"].as_f64().unwrap() as f32,
        local["synthetic_envelope_limit"].as_f64().unwrap() as f32,
    );
    let seed = synthetic_state(&local["synthetic_state"]);

    let delay_case = find_case(&local, "zero_coefficient_delay");
    let delay_input = f32_values(&delay_case["input"]);
    let (delay_lanes, delay_after) = preview_filter_block(&parameters, &seed, &delay_input)
        .unwrap()
        .into_parts();
    for lane in 0..4 {
        assert_eq!(
            lane_words(&delay_lanes[lane]),
            u32_values(&delay_case["actual_lane_words"][lane])
        );
    }
    assert_state_matches_json(&delay_after, &delay_case["actual_state"]);

    let fade_case = find_case(&local, "cold_fade_length_6");
    let mut fade_seed = seed_for_test(&seed);
    fade_seed.fade_pending = true;
    let fade_input = f32_values(&fade_case["input"]);
    let (fade_lanes, _) = preview_filter_block(&parameters, &fade_seed, &fade_input)
        .unwrap()
        .into_parts();
    let fade_expected = u32_values(&fade_case["expected_each_lane_words"]);
    for lane in &fade_lanes {
        assert_eq!(lane_words(lane), fade_expected);
    }

    let empty_case = find_case(&local, "empty_clears_fade");
    let mut empty_seed = seed_for_test(&seed);
    empty_seed.fade_pending = true;
    let before = empty_seed.canonical_bytes();
    let (_, empty_after) = preview_filter_block(&parameters, &empty_seed, &[])
        .unwrap()
        .into_parts();
    assert!(!empty_after.fade_pending);
    assert_eq!(empty_after.canonical_bytes().len(), before.len());
    assert_eq!(
        empty_case["expected_only_state_change"],
        "fade_pending=false"
    );

    let warm_case = find_case(&local, "warm_split_5_7");
    let warm_input = f32_values(&warm_case["input"]);
    let (whole_lanes, whole_state) = preview_filter_block(&parameters, &seed, &warm_input)
        .unwrap()
        .into_parts();
    let (first_lanes, middle_state) = preview_filter_block(&parameters, &seed, &warm_input[..5])
        .unwrap()
        .into_parts();
    let (second_lanes, split_state) =
        preview_filter_block(&parameters, &middle_state, &warm_input[5..])
            .unwrap()
            .into_parts();
    for lane in 0..4 {
        let mut split = first_lanes[lane].clone();
        split.extend_from_slice(&second_lanes[lane]);
        assert_eq!(split, whole_lanes[lane]);
    }
    assert_eq!(split_state.canonical_bytes(), whole_state.canonical_bytes());
    assert_eq!(warm_case["output_bitwise_equal_to_unsplit"], true);
    assert_eq!(warm_case["entire_state_equal_to_unsplit"], true);
}

#[test]
fn malformed_inputs_and_numeric_overflow_fail_without_mutating_the_before_state() {
    let parameters = selected_parameters();
    let state = parameters.cold_state().unwrap();
    let before = state.canonical_bytes();

    assert_eq!(
        preview_filter_block(&parameters, &state, &[f32::NAN])
            .unwrap_err()
            .code(),
        "F04_INPUT_ERROR"
    );
    assert_eq!(
        preview_filter_block(&parameters, &state, &vec![0.0; 4097])
            .unwrap_err()
            .code(),
        "F04_INPUT_ERROR"
    );
    assert_eq!(
        preview_filter_block(&parameters, &state, &[f32::MAX; 3072])
            .unwrap_err()
            .code(),
        "F04_NUMERIC_FAILURE"
    );
    let mut malformed_state = seed_for_test(&state);
    malformed_state.ring_read_index = malformed_state.ring.len();
    assert_eq!(
        preview_filter_block(&parameters, &malformed_state, &[0.0])
            .unwrap_err()
            .code(),
        "F04_STATE_ERROR"
    );
    assert_eq!(state.canonical_bytes(), before);
}

#[test]
fn provider_identity_is_rejected_before_parsing_changed_parameter_bytes() {
    let manifest = PackageManifestEvidence::new(Arc::from(PACKAGE_MANIFEST), None);
    let validated_manifest = validate_diffusion_filter_package_manifest(&manifest).unwrap();
    let mut altered = DIFFUSION_PARAMETERS.to_vec();
    altered[0] ^= 1;
    let coefficient_file = ProviderFileEvidence::new(Arc::from(altered), None);
    let constructor_file = ProviderFileEvidence::new(Arc::from(CONSTRUCTOR_PARAMETERS), None);

    assert_eq!(
        validate_diffusion_filter_parameters(
            &coefficient_file,
            &constructor_file,
            &validated_manifest
        )
        .unwrap_err()
        .code(),
        "DIFFUSION_PROVIDER_IDENTITY"
    );
}

fn synthetic_parameters(epsilon: f32, limit: f32) -> DiffusionFilterParameters {
    let mut configuration_words = [0_u32; 10];
    configuration_words[8] = epsilon.to_bits();
    configuration_words[9] = limit.to_bits();
    DiffusionFilterParameters {
        coefficients: [[0.0; 4]; 4],
        modes: [0; 11],
        filter_lanes: [-1; 11],
        configuration_words,
        smoothing_words: [0, 0, 0, 0.5_f32.to_bits()],
        bank_lengths: [1; 4],
        ring_length: 2,
    }
}

fn synthetic_state(value: &Value) -> DiffusionFilterState {
    let coefficients: [[f32; 4]; 4] = value["coefficients_f32"]
        .as_array()
        .unwrap()
        .iter()
        .map(f32_values)
        .map(|row| row.try_into().unwrap())
        .collect::<Vec<[f32; 4]>>()
        .try_into()
        .unwrap();
    let banks: [Vec<[f32; 4]>; 4] = value["filter_banks_f32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|bank| {
            bank.as_array()
                .unwrap()
                .iter()
                .map(|entry| f32_values(entry).try_into().unwrap())
                .collect()
        })
        .collect::<Vec<Vec<[f32; 4]>>>()
        .try_into()
        .unwrap();
    let sub = &value["sub240290"];
    let slow = f32_values(&sub["envelope_slow"]);
    let fast = f32_values(&sub["envelope_fast"]);
    let input = f32_values(&sub["input_smoother"]);
    let ring = &value["delay_ring"];
    DiffusionFilterState {
        sample_rate_hz: value["sample_rate_hz"].as_u64().unwrap() as u32,
        fade_pending: value["fade_in_next_block"].as_bool().unwrap(),
        coefficients,
        banks,
        bank_indices: u32_values(&value["bank_indexes"])
            .into_iter()
            .map(|word| word as usize)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
        ring: f32_values(&ring["samples_f32"]),
        ring_read_index: ring["read_index"].as_u64().unwrap() as usize,
        ring_write_index: ring["write_index"].as_u64().unwrap() as usize,
        gain0: sub["gain0"].as_f64().unwrap() as f32,
        gain1: sub["gain1"].as_f64().unwrap() as f32,
        gain_smoothing0: sub["gain_smoothing0"].as_f64().unwrap() as f32,
        gain_smoothing1: sub["gain_smoothing1"].as_f64().unwrap() as f32,
        slow_coefficient: slow[0],
        slow_complement: slow[1],
        slow_accumulator: slow[2],
        fast_coefficient: fast[0],
        fast_complement: fast[1],
        fast_accumulator: fast[2],
        input_coefficient: input[0],
        input_accumulator: input[1],
    }
}

fn seed_for_test(state: &DiffusionFilterState) -> DiffusionFilterState {
    DiffusionFilterState {
        sample_rate_hz: state.sample_rate_hz,
        fade_pending: state.fade_pending,
        coefficients: state.coefficients,
        banks: state.banks.clone(),
        bank_indices: state.bank_indices,
        ring: state.ring.clone(),
        ring_read_index: state.ring_read_index,
        ring_write_index: state.ring_write_index,
        gain0: state.gain0,
        gain1: state.gain1,
        gain_smoothing0: state.gain_smoothing0,
        gain_smoothing1: state.gain_smoothing1,
        slow_coefficient: state.slow_coefficient,
        slow_complement: state.slow_complement,
        slow_accumulator: state.slow_accumulator,
        fast_coefficient: state.fast_coefficient,
        fast_complement: state.fast_complement,
        fast_accumulator: state.fast_accumulator,
        input_coefficient: state.input_coefficient,
        input_accumulator: state.input_accumulator,
    }
}

fn assert_state_matches_json(state: &DiffusionFilterState, expected: &Value) {
    assert_eq!(
        state.sample_rate_hz,
        expected["sample_rate_hz"].as_u64().unwrap() as u32
    );
    assert_eq!(
        state.fade_pending,
        expected["fade_in_next_block"].as_bool().unwrap()
    );
    assert_eq!(
        state.bank_indices.map(|index| index as u32).to_vec(),
        u32_values(&expected["bank_indexes"])
    );
    for stage in 0..4 {
        let expected_bank: Vec<[u32; 4]> = expected["filter_banks_f32"][stage]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                f32_values(entry)
                    .into_iter()
                    .map(f32::to_bits)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            })
            .collect();
        let actual_bank: Vec<[u32; 4]> = state.banks[stage]
            .iter()
            .map(|entry| entry.map(f32::to_bits))
            .collect();
        assert_eq!(actual_bank, expected_bank);
    }
    assert_eq!(
        state
            .ring
            .iter()
            .copied()
            .map(f32::to_bits)
            .collect::<Vec<_>>(),
        f32_values(&expected["delay_ring"]["samples_f32"])
            .into_iter()
            .map(f32::to_bits)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        state.ring_read_index,
        expected["delay_ring"]["read_index"].as_u64().unwrap() as usize
    );
    assert_eq!(
        state.ring_write_index,
        expected["delay_ring"]["write_index"].as_u64().unwrap() as usize
    );
    let sub = &expected["sub240290"];
    assert_eq!(
        state.gain0.to_bits(),
        (sub["gain0"].as_f64().unwrap() as f32).to_bits()
    );
    assert_eq!(
        state.gain1.to_bits(),
        (sub["gain1"].as_f64().unwrap() as f32).to_bits()
    );
    assert_eq!(
        state.gain_smoothing0.to_bits(),
        (sub["gain_smoothing0"].as_f64().unwrap() as f32).to_bits()
    );
    assert_eq!(
        state.gain_smoothing1.to_bits(),
        (sub["gain_smoothing1"].as_f64().unwrap() as f32).to_bits()
    );
    for (actual, expected) in [
        (
            state.slow_coefficient,
            sub["envelope_slow"][0].as_f64().unwrap() as f32,
        ),
        (
            state.slow_complement,
            sub["envelope_slow"][1].as_f64().unwrap() as f32,
        ),
        (
            state.slow_accumulator,
            sub["envelope_slow"][2].as_f64().unwrap() as f32,
        ),
        (
            state.fast_coefficient,
            sub["envelope_fast"][0].as_f64().unwrap() as f32,
        ),
        (
            state.fast_complement,
            sub["envelope_fast"][1].as_f64().unwrap() as f32,
        ),
        (
            state.fast_accumulator,
            sub["envelope_fast"][2].as_f64().unwrap() as f32,
        ),
        (
            state.input_coefficient,
            sub["input_smoother"][0].as_f64().unwrap() as f32,
        ),
        (
            state.input_accumulator,
            sub["input_smoother"][1].as_f64().unwrap() as f32,
        ),
    ] {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
}

fn lane_words(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn lane_sha256(values: &[f32]) -> String {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    hex(&Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut output, byte| {
            write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
            output
        },
    )
}

fn find_case<'a>(document: &'a Value, id: &str) -> &'a Value {
    document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()
}

fn f32_values(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|word| word.as_f64().unwrap() as f32)
        .collect()
}

fn u32_values(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|word| word.as_u64().unwrap() as u32)
        .collect()
}
