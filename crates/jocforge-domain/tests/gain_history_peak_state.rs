// pattern: Functional Core
use jocforge_domain::analysis::gain_history::{
    FULLBAND_GAIN_CHANNELS, GAIN_BLOCK_SAMPLES, GAIN_BLOCKS_PER_FRAME, GAIN_CHANNELS,
    GainFormulaTrace, GainHistoryState, GainReuseOutput, PeakReductionProfile, PeakReductionTrace,
    apply_reuse_gain_minimum, process_gain_block,
};
use jocforge_domain::numeric::{FiniteF32, r32_mul};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/gain_history_peak_state";
const PACKAGE_MANIFEST_SHA256: &str =
    "32a5d555edbb83123e17a51843ce2d3d905944815eee3d70924854a23ddbb419";
const INDEPENDENT_CHECK_SHA256: &str =
    "dc49214b761ead381a230f9a10a7ec44ffe95691391230105fe1b520c44333c2";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 10] = [
    (
        "CONTRACT.md",
        5314,
        "e10cc8ba709ff953f0e7983d4f5d964f6fdb11e4a4341cb7155cf9623e4677f0",
    ),
    (
        "CONTRACT.json",
        7098,
        "1af85465ab33ae6885b306baf628393b6b3bb4fa6e99e70bdfadfcaa23fc4dd8",
    ),
    (
        "INPUTS.json",
        807125,
        "8987d512edbea6a977f9a9327556b5d3f54cc84a27f08b20870559382d31a744",
    ),
    (
        "EXPECTED.json",
        485161,
        "9ce8d4a339642bc3694f041d2884ec80dce846c3a42c338d5a9e94baa62547c2",
    ),
    (
        "PARAMETERS.json",
        1223,
        "af1eb3e483f984fb78df870119c34ad211f531e875750a76557c4ad0d2ceb027",
    ),
    (
        "DERIVED-TRACES.json",
        2409151,
        "bf54157d2c8bfafd3ae070f3f2eff73f519342ca9c85f8b1c89852717c8f2fd2",
    ),
    (
        "AUXILIARY-EFFECTS.json",
        18935,
        "fe12921d58650305794d9b42ac3e12889504bea0a41e89c6c2216b899f38d504",
    ),
    (
        "NATIVE-CONTEXT.json",
        739,
        "010e9d80ce3809acac6a691852216bb30f5b3cc56933ade886e89636dbe3a25e",
    ),
    (
        "RUNTIME-RECEIPTS.json",
        1615,
        "27ac964e88eaa2de90f65c94c42fbac862711362e25a74d0b9f9f7311ac2301f",
    ),
    (
        "PROVENANCE.json",
        2749,
        "24b17009522c57f6ae09521fc30ffd883c9d9bd0db7240ae4e20a7340437d2d0",
    ),
];
const TEST_GAIN_TAIL_CANARY: i16 = 12345;
const TEST_REUSE_TAIL_CANARY: u32 = 305419896;
const CENTER_F32_BITS: u32 = 1_060_437_492;
const SIDE_F32_BITS: u32 = 1_056_959_542;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(crate_root().join(FIXTURE_DIRECTORY).join(name)).unwrap()
}

fn fixture_json(name: &str) -> Value {
    serde_json::from_slice(&fixture_bytes(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn u32_array<const N: usize>(value: &Value) -> [u32; N] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| u32::try_from(item.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn i16_array<const N: usize>(value: &Value) -> [i16; N] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| i16::try_from(item.as_i64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn input_samples(
    kind: &str,
    frame: usize,
    block: usize,
    channel: usize,
) -> [f32; GAIN_BLOCK_SAMPLES] {
    const CARRY_CHANNEL_SCALE: [f32; GAIN_CHANNELS] = [1.0, 0.5, 0.25, 0.125, 0.0625, 0.75];
    const CARRY_BLOCK_CYCLE: [f32; GAIN_BLOCKS_PER_FRAME] = [0.125, 0.25, 0.0625, 0.5, 0.0, 0.25];
    const WEIGHTED_CHANNEL_SCALE: [f32; GAIN_CHANNELS] =
        [0.015625, 0.03125, 0.0625, 0.125, 0.25, 0.75];
    const WEIGHTED_BLOCK_CYCLE: [f32; GAIN_BLOCKS_PER_FRAME] = [1.0, 0.5, 2.0, 0.0, 1.5, 0.25];
    const UNIT: [f32; 5] = [0.0, 1.0, -1.0, 0.5, -0.5];
    const LFE_CYCLE: [f32; GAIN_BLOCKS_PER_FRAME] = [0.0, 0.125, 0.5, 0.75, 0.25, 0.0];
    const ZERO_PERMUTATION: [usize; GAIN_BLOCKS_PER_FRAME] = [251, 255, 254, 255, 0, 255];

    let mut samples = [0.0; GAIN_BLOCK_SAMPLES];
    match kind {
        "zero" => {}
        "carry" | "carry_unaligned" => {
            let amplitude = r32_mul(
                FiniteF32::new(CARRY_BLOCK_CYCLE[block]).unwrap(),
                FiniteF32::new(CARRY_CHANNEL_SCALE[channel]).unwrap(),
            )
            .unwrap()
            .value();
            let impulse = (13 * block + 7 * channel + frame) % GAIN_BLOCK_SAMPLES;
            let negative = (block + channel) % 2 == 1;
            samples[impulse] = if negative { -amplitude } else { amplitude };
        }
        "lfe_only" => {
            if channel == 5 {
                samples.fill(LFE_CYCLE[block]);
            }
        }
        "weighted" => {
            let amplitude = r32_mul(
                FiniteF32::new(WEIGHTED_CHANNEL_SCALE[channel]).unwrap(),
                FiniteF32::new(WEIGHTED_BLOCK_CYCLE[block]).unwrap(),
            )
            .unwrap()
            .value();
            for (index, sample) in samples.iter_mut().enumerate() {
                *sample = r32_mul(
                    FiniteF32::new(amplitude).unwrap(),
                    FiniteF32::new(UNIT[(index + channel + block) % UNIT.len()]).unwrap(),
                )
                .unwrap()
                .value();
            }
        }
        "ties" => {
            let numerator = f32::from(u8::try_from(channel + 1).unwrap());
            let amplitude = numerator / 32.0;
            for index in [0, 4, 128, 252, 255] {
                samples[index] = if index == 4 || index == 255 {
                    -amplitude
                } else {
                    amplitude
                };
            }
        }
        "signed_zero" => {
            for (index, sample) in samples.iter_mut().enumerate() {
                let negative_zero = channel == 1
                    || (channel == 2 && index % 2 == 1)
                    || (channel == 3 && index == 255)
                    || (channel == 4 && index == 0)
                    || (channel == 5 && index != 255);
                *sample = if negative_zero { -0.0 } else { 0.0 };
            }
        }
        "q15_ties" => {
            let numerator = f32::from(u8::try_from(2 * channel + block).unwrap()) + 0.5;
            let amplitude = numerator / 65_536.0;
            let impulse = (13 * block + 7 * channel + frame) % GAIN_BLOCK_SAMPLES;
            samples[impulse] = if (block + channel) % 2 == 1 {
                -amplitude
            } else {
                amplitude
            };
        }
        "zero_permutations" => samples[ZERO_PERMUTATION[block]] = -0.0,
        other => panic!("unsupported gain-history input recipe {other}"),
    }
    samples
}

fn words_sha256(samples: &[f32; GAIN_BLOCK_SAMPLES]) -> String {
    let bytes = samples
        .iter()
        .flat_map(|sample| sample.to_bits().to_le_bytes())
        .collect::<Vec<_>>();
    sha256(&bytes)
}

#[test]
fn package_payloads_and_parameter_identities_are_hash_bound() {
    let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");
    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 10);
    assert_eq!(check["payload_hashes_match"], true);
    assert_eq!(check["upstream_manifest_check_and_payloads_match"], true);
    assert_eq!(check["history_cases"], 17);
    assert_eq!(check["reuse_cases"], 3);
    assert_eq!(check["input_words_replayed"], 165888);
    assert_eq!(check["peak_transitions_replayed"], 648);
    assert_eq!(check["gain_values_replayed"], 540);
    assert_eq!(check["reuse_values_replayed"], 90);
    assert_eq!(check["independent_formula_replay"], "PASS all values");
    assert_eq!(check["native_lane_register_capture"], "NOT_RUN");
    assert_eq!(
        check["filter_state_numerics"],
        "NOT_VALIDATED_IN_THIS_SLICE"
    );
    assert_eq!(check["reset_eof"], "NOT_RUN");
    assert_eq!(check["whole_f09_closed"], false);
    assert_eq!(check["whole_implementation_ready"], "NO");
    assert_eq!(
        provenance["source_manifest_sha256"],
        "7acbd2cba5ce0f8c67738a9915d7f98484854aedd66007930526339cb8b5bd68"
    );
    assert_eq!(
        provenance["source_independent_check_sha256"],
        "dcff19c3a9996ef8e7cd60faf3b5815711b9e6d514693f8098fc1b6ecb3ea37a"
    );

    for (name, expected_bytes, expected_hash) in PACKAGE_PAYLOADS {
        let bytes = fixture_bytes(name);
        assert_eq!(bytes.len(), expected_bytes, "{name} size");
        assert_eq!(sha256(&bytes), expected_hash, "{name} SHA-256");
    }

    let parameters = fixture_json("PARAMETERS.json");
    assert_eq!(
        u32::try_from(parameters["center_f32_bits"].as_u64().unwrap()).unwrap(),
        CENTER_F32_BITS
    );
    assert_eq!(
        u32::try_from(parameters["side_mono_f32_bits"].as_u64().unwrap()).unwrap(),
        SIDE_F32_BITS
    );
    for (name, value) in [
        ("center_f32_bits", CENTER_F32_BITS),
        ("side_mono_f32_bits", SIDE_F32_BITS),
    ] {
        let identity = parameters["identities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["name"] == name)
            .unwrap();
        let bytes = value.to_le_bytes();
        assert_eq!(identity["bytes"], 4);
        assert_eq!(identity["sha256"], sha256(&bytes));
    }
}

#[test]
fn seventeen_synthetic_cases_replay_peak_gain_and_state_transitions() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    let derived = fixture_json("DERIVED-TRACES.json");
    let cases = inputs["history_cases"].as_array().unwrap();
    let expected_cases = expected["history_cases"].as_array().unwrap();
    let derived_cases = derived["traces"].as_array().unwrap();
    assert_eq!(inputs["schema"], "gain-history-input-recipes.v1");
    assert_eq!(cases.len(), 17);
    assert_eq!(expected_cases.len(), cases.len());
    assert_eq!(derived_cases.len(), cases.len());

    let mut input_words_checked = 0;
    let mut peak_transitions_checked = 0;
    let mut gain_values_checked = 0;

    for (case_index, ((recipe, expected_case), derived_case)) in cases
        .iter()
        .zip(expected_cases)
        .zip(derived_cases)
        .enumerate()
    {
        assert_eq!(recipe["pass_index"], 0);
        assert_eq!(recipe["summary_gate"], 1);
        assert_eq!(recipe["format"], 7);
        assert_eq!(recipe["fullband_channels"], FULLBAND_GAIN_CHANNELS);
        assert_eq!(recipe["channels"], GAIN_CHANNELS);
        assert_eq!(recipe["samples_per_block"], GAIN_BLOCK_SAMPLES);
        assert_eq!(recipe["blocks_per_frame"], GAIN_BLOCKS_PER_FRAME);

        let previous_peak_bits = u32_array::<GAIN_CHANNELS>(&recipe["initial_previous_peak_bits"]);
        let q15_max = i16_array::<GAIN_CHANNELS>(&recipe["initial_q15_max"]);
        let summary_bits =
            u32_matrix::<GAIN_BLOCKS_PER_FRAME, GAIN_CHANNELS>(&recipe["initial_summary_bits"]);
        let gain_output = i16_matrix::<GAIN_BLOCKS_PER_FRAME, FULLBAND_GAIN_CHANNELS>(
            &recipe["initial_gain_rows"],
        );
        let mut state =
            GainHistoryState::from_parts(previous_peak_bits, summary_bits, q15_max, gain_output);
        let profile = match recipe["peak_reduction_profile"].as_str().unwrap() {
            "ALIGNED_FOUR_LANE" => PeakReductionProfile::AlignedFourLane,
            "UNALIGNED_SCALAR" => PeakReductionProfile::UnalignedScalar,
            other => panic!("unexpected explicit peak reduction profile {other}"),
        };
        let mask = u16::try_from(recipe["mask"].as_u64().unwrap()).unwrap();
        let frame_count = usize::try_from(recipe["frames"].as_u64().unwrap()).unwrap();
        let expected_blocks = expected_case["blocks"].as_array().unwrap();
        let derived_blocks = derived_case["blocks"].as_array().unwrap();
        assert_eq!(expected_blocks.len(), frame_count * GAIN_BLOCKS_PER_FRAME);
        assert_eq!(derived_blocks.len(), expected_blocks.len());

        for (block_ordinal, (expected_block, derived_block)) in
            expected_blocks.iter().zip(derived_blocks).enumerate()
        {
            let frame = block_ordinal / GAIN_BLOCKS_PER_FRAME;
            let block = block_ordinal % GAIN_BLOCKS_PER_FRAME;
            assert_eq!(
                usize::try_from(expected_block["frame"].as_u64().unwrap()).unwrap(),
                frame
            );
            assert_eq!(
                usize::try_from(expected_block["block"].as_u64().unwrap()).unwrap(),
                block
            );

            let mut samples = [[0.0_f32; GAIN_BLOCK_SAMPLES]; GAIN_CHANNELS];
            for channel in 0..GAIN_CHANNELS {
                samples[channel] =
                    input_samples(recipe["kind"].as_str().unwrap(), frame, block, channel);
                let input_sha = words_sha256(&samples[channel]);
                assert_eq!(
                    input_sha,
                    expected_block["channels"][channel]["input_sha256"]
                        .as_str()
                        .unwrap(),
                    "case {case_index}, frame {frame}, block {block}, channel {channel} input"
                );
            }

            let previous_peak_before = *state.previous_peak_bits();
            let q15_max_before = *state.q15_max();
            let mut gain_output_buffer = [TEST_GAIN_TAIL_CANARY; FULLBAND_GAIN_CHANNELS + 1];
            gain_output_buffer[..FULLBAND_GAIN_CHANNELS]
                .copy_from_slice(&state.gain_output()[block]);
            let outcome = process_gain_block(
                &mut state,
                u8::try_from(block).unwrap(),
                mask,
                profile,
                &samples,
                &mut gain_output_buffer,
            );

            let expected_summary = u32_array::<GAIN_CHANNELS>(&expected_block["summary_six_bits"]);
            assert_eq!(outcome.summary_six_bits(), expected_summary);
            assert_eq!(
                outcome.gains_before().as_slice(),
                i16_array::<FULLBAND_GAIN_CHANNELS>(&expected_block["gains_before"]).as_slice()
            );
            assert_eq!(
                outcome.gains_after().as_slice(),
                i16_array::<FULLBAND_GAIN_CHANNELS>(&expected_block["gains_after"]).as_slice()
            );
            assert_eq!(
                &gain_output_buffer[..FULLBAND_GAIN_CHANNELS],
                outcome.gains_after().as_slice()
            );
            assert_eq!(
                gain_output_buffer[FULLBAND_GAIN_CHANNELS],
                TEST_GAIN_TAIL_CANARY
            );
            assert_eq!(
                expected_block["gain_tail_canary"],
                i64::from(TEST_GAIN_TAIL_CANARY)
            );
            assert_eq!(expected_block["lfe_gain_calls"], 0);

            let expected_channels = expected_block["channels"].as_array().unwrap();
            let derived_peak_traces = derived_block["peak_traces"].as_array().unwrap();
            let derived_gain_traces = derived_block["gain_traces"].as_array().unwrap();
            assert_eq!(expected_channels.len(), GAIN_CHANNELS);
            assert_eq!(derived_peak_traces.len(), GAIN_CHANNELS);
            assert_eq!(derived_gain_traces.len(), FULLBAND_GAIN_CHANNELS);
            for channel in 0..GAIN_CHANNELS {
                let expected_channel = &expected_channels[channel];
                let before = &expected_channel["before"];
                let after = &expected_channel["after"];
                assert_eq!(
                    previous_peak_before[channel],
                    u32::try_from(before["previous_peak_bits"].as_u64().unwrap()).unwrap()
                );
                assert_eq!(
                    q15_max_before[channel],
                    i16::try_from(before["peak_q15_max"].as_i64().unwrap()).unwrap()
                );
                assert_eq!(
                    outcome.channels()[channel].current_peak_bits(),
                    u32::try_from(after["current_peak_bits"].as_u64().unwrap()).unwrap()
                );
                assert_eq!(
                    outcome.channels()[channel].summary_bits(),
                    u32::try_from(after["summary_bits"].as_u64().unwrap()).unwrap()
                );
                assert_eq!(
                    outcome.channels()[channel].q15_max(),
                    i16::try_from(after["peak_q15_max"].as_i64().unwrap()).unwrap()
                );
                assert_eq!(expected_channel["input_unchanged"], true);
                assert_eq!(expected_channel["input_tail_canary"], 305419896_u64);

                compare_peak_trace(
                    outcome.channels()[channel].peak(),
                    &derived_peak_traces[channel],
                    profile,
                );
                peak_transitions_checked += 1;
                input_words_checked += GAIN_BLOCK_SAMPLES;
            }

            for (channel, gain_trace) in derived_gain_traces.iter().enumerate() {
                compare_gain_trace(outcome.gain_traces()[channel], gain_trace);
                gain_values_checked += 1;
            }
        }

        assert_eq!(
            state.previous_peak_bits().as_slice(),
            u32_array::<GAIN_CHANNELS>(&expected_case["final_previous_peak_bits"]).as_slice()
        );
        assert_eq!(
            state.q15_max().as_slice(),
            i16_array::<GAIN_CHANNELS>(&expected_case["final_q15_max"]).as_slice()
        );
    }

    assert_eq!(input_words_checked, 165888);
    assert_eq!(peak_transitions_checked, 648);
    assert_eq!(gain_values_checked, 540);
}

#[test]
fn three_reuse_minimum_recipes_match_and_preserve_the_output_tail_canary() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    let recipes = inputs["reuse_cases"].as_array().unwrap();
    let expected_cases = expected["reuse_cases"].as_array().unwrap();
    assert_eq!(recipes.len(), 3);
    assert_eq!(expected_cases.len(), recipes.len());
    let mut values_checked = 0;

    for (recipe, expected_case) in recipes.iter().zip(expected_cases) {
        assert_eq!(recipe["blocks"], GAIN_BLOCKS_PER_FRAME);
        assert_eq!(recipe["fullband_channels"], FULLBAND_GAIN_CHANNELS);
        assert_eq!(recipe["lfe_enabled"], 0);
        assert_eq!(recipe["exponent_end_count"], 1);
        assert_eq!(recipe["exponent_scratch_zero"], true);
        assert_eq!(recipe["normalization_option"], 0);
        assert_eq!(recipe["coupling_flags_zero"], true);
        assert_eq!(expected_case["recipe"]["name"], recipe["name"]);
        let gains = i16_matrix::<GAIN_BLOCKS_PER_FRAME, FULLBAND_GAIN_CHANNELS>(
            &recipe["input_gain_matrix"],
        );
        let strategy = i16_array::<GAIN_BLOCKS_PER_FRAME>(&recipe["pattern"]);
        let mut output = GainReuseOutput::new(gains, TEST_REUSE_TAIL_CANARY);
        apply_reuse_gain_minimum(&gains, &strategy, &mut output);

        let expected_rows = expected_case["after"].as_array().unwrap();
        assert_eq!(expected_rows.len(), GAIN_BLOCKS_PER_FRAME);
        for (block, expected_row) in expected_rows.iter().enumerate() {
            let expected_gain = i16_array::<FULLBAND_GAIN_CHANNELS>(expected_row);
            assert_eq!(output.rows()[block], expected_gain);
            values_checked += FULLBAND_GAIN_CHANNELS;
        }
        assert_eq!(output.tail_canary(), TEST_REUSE_TAIL_CANARY);
        assert_eq!(
            expected_case["tail_canary"],
            u64::from(TEST_REUSE_TAIL_CANARY)
        );
    }

    assert_eq!(values_checked, 90);
}

fn i16_matrix<const ROWS: usize, const COLS: usize>(value: &Value) -> [[i16; COLS]; ROWS] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(i16_array::<COLS>)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn u32_matrix<const ROWS: usize, const COLS: usize>(value: &Value) -> [[u32; COLS]; ROWS] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(u32_array::<COLS>)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn compare_peak_trace(actual: PeakReductionTrace, expected: &Value, profile: PeakReductionProfile) {
    let expected_profile = match profile {
        PeakReductionProfile::AlignedFourLane => "aligned_four_lane",
        PeakReductionProfile::UnalignedScalar => "unaligned_scalar",
    };
    assert_eq!(expected["path"], expected_profile);
    assert_eq!(
        expected["grade"],
        "COMPUTED_FROM_INPUT_AND_CONFIRMED_SSE_OPERAND_ORDER"
    );
    assert_eq!(
        actual.minimum_bits(),
        u32::try_from(expected["minimum_bits"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        actual.maximum_bits(),
        u32::try_from(expected["maximum_bits"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        actual.negated_minimum_bits(),
        u32::try_from(expected["negated_minimum_bits"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        actual.peak_bits(),
        u32::try_from(expected["peak_bits"].as_u64().unwrap()).unwrap()
    );
    match (
        actual.horizontal_minimum_pair_bits(),
        actual.horizontal_maximum_pair_bits(),
        expected.get("horizontal_min_pairs_bits"),
        expected.get("horizontal_max_pairs_bits"),
    ) {
        (Some(actual_min), Some(actual_max), Some(expected_min), Some(expected_max)) => {
            assert_eq!(actual_min, u32_array::<2>(expected_min));
            assert_eq!(actual_max, u32_array::<2>(expected_max));
        }
        (None, None, None, None) => {}
        other => panic!("unexpected aligned/unaligned trace shape: {other:?}"),
    }
}

fn compare_gain_trace(actual: GainFormulaTrace, expected: &Value) {
    assert_eq!(
        actual.weighted_peak_bits(),
        u32::try_from(expected["weighted_peak_bits"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        actual.clipped_peak_bits(),
        u32::try_from(expected["clipped_peak_bits"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        actual.doubled_word(),
        u32::try_from(expected["doubled_word"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        actual.binade(),
        i32::try_from(expected["binade"].as_i64().unwrap()).unwrap()
    );
    assert_eq!(
        actual.headroom(),
        i32::try_from(expected["headroom"].as_i64().unwrap()).unwrap()
    );
    assert_eq!(
        actual.gain(),
        i16::try_from(expected["gain"].as_i64().unwrap()).unwrap()
    );
    assert_eq!(
        expected["grade"],
        "COMPUTED_FROM_INPUT_SUMMARY_NOT_NATIVE_REGISTER_CAPTURE"
    );
}
