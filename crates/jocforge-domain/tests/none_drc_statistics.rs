use jocforge_domain::drc::none_drc_dialnorm_offset::none_drc_dialnorm_offset_word;
use jocforge_domain::drc::none_drc_statistics::{
    FiniteStatisticsPlanes, NoneDrcStatisticsFrameInput, NoneDrcStatisticsState,
    transition_none_drc_statistics_frame,
};
use jocforge_domain::numeric::FiniteF32;
use sha2::{Digest, Sha256};

const AUTHORIZED_PARAMETERS: &str = include_str!("fixtures/none_drc/authorized_parameters.json");
const AUTHORIZED_MATH: &str = include_str!("fixtures/none_drc/authorized_math_parameters.json");
const AUTHORIZED_VERIFICATION: &str =
    include_str!("fixtures/none_drc/authorized_verification.json");

const CHANNEL_COUNT: usize = 6;
const SAMPLE_COUNT: usize = 1_536;
const BLOCK_COUNT: usize = 6;

#[test]
fn selected_parameter_and_math_provider_identities_match_wash() {
    assert_eq!(
        sha256_hex(AUTHORIZED_PARAMETERS.as_bytes()),
        "35b28cc27a0bd903cdf58df431c1b2f17769d534425765eb1588573e1fba101f"
    );
    assert_eq!(
        sha256_hex(AUTHORIZED_MATH.as_bytes()),
        "f1a552443315414ad499118d905ef9fe6612f178dfe28c16aa5d9fa6b2002166"
    );
    assert_eq!(
        sha256_hex(AUTHORIZED_VERIFICATION.as_bytes()),
        "f4cd579712b2d77c485d354dac3f2132cba6106174441e6eef2914dbdf7c136d"
    );

    let parameters: serde_json::Value = serde_json::from_str(AUTHORIZED_PARAMETERS).unwrap();
    assert_eq!(
        json_words(&parameters["groups"]["DRC_FILTER5"]["f32_words_u32"]),
        [
            0x3F79_999A,
            0x0000_0000,
            0x3F13_33DB,
            0xBDEB_862B,
            0xBEEB_862B
        ]
    );
    assert_eq!(
        json_words(&parameters["groups"]["DRC_NORMALIZE3"]["f32_words_u32"]),
        [0x3CAA_5BD7, 0x3C62_2BCA, 0x3BE6_35DE]
    );
    assert_eq!(
        json_words(&parameters["groups"]["DRC_AUX_ROTATION2"]["f32_words_u32"]),
        [0x3F5E_F778, 0x3EFA_C44A]
    );

    let mathematical: serde_json::Value = serde_json::from_str(AUTHORIZED_MATH).unwrap();
    assert_eq!(
        json_words(&mathematical["mix_words"]),
        [0xBEB5_04F3, 0x3F80_0000, 0xBF00_0000]
    );
    let dialnorm_words = json_words(&mathematical["dialnorm_words"]);
    assert_eq!(dialnorm_words.len(), 31);
    assert_eq!(
        none_drc_dialnorm_offset_word(24).unwrap(),
        dialnorm_words[23]
    );
}

#[test]
fn eight_input_only_frames_recompute_hashes_outputs_and_actual_state_chain() {
    let verification: serde_json::Value = serde_json::from_str(AUTHORIZED_VERIFICATION).unwrap();
    assert_eq!(
        verification["expected_outputs_are_never_inputs"],
        serde_json::Value::Bool(true)
    );
    let sequence = &verification["statistics_sequence"];
    assert_eq!(sequence["frame_shape"], serde_json::json!([6, 1536]));
    assert_eq!(
        sequence["plane_order"],
        serde_json::json!(["L", "R", "C", "LFE", "Ls", "Rs"])
    );
    assert_recipe_definitions(sequence);

    let cases = sequence["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    let mut state = NoneDrcStatisticsState::cold();
    assert_state_matches(&sequence["cold_state"], &state);

    for (expected_step, case) in cases.iter().enumerate() {
        assert_eq!(case["sequence_step"].as_u64(), Some(expected_step as u64));
        let input_recipe = &case["input"];
        let recipe_id = input_recipe["recipe_id"].as_u64().unwrap();
        let dialnorm = i32::try_from(input_recipe["dialnorm"].as_i64().unwrap()).unwrap();
        let primary = reconstruct_planes(recipe_id, false);
        let secondary = reconstruct_planes(recipe_id, true);

        assert_eq!(
            sha256_hex(&plane_bytes(&primary)),
            case["input_primary_f32le_sha256"].as_str().unwrap(),
            "primary input recipe hash at step {expected_step}"
        );
        assert_eq!(
            sha256_hex(&plane_bytes(&secondary)),
            case["input_secondary_f32le_sha256"].as_str().unwrap(),
            "secondary input recipe hash at step {expected_step}"
        );

        let primary_finite = finite_planes(&primary);
        let secondary_finite = finite_planes(&secondary);
        let input = NoneDrcStatisticsFrameInput {
            primary_samples: &primary_finite,
            secondary_samples: &secondary_finite,
            dialnorm,
        };
        let transition = transition_none_drc_statistics_frame(&state, &input).unwrap();
        let actual_output = transition.output;
        state = transition.next_state;

        assert_output_matches(
            &case["expected_output_words"],
            &actual_output,
            expected_step,
        );
        assert_state_matches(&case["expected_next_state_words"], &state);
    }
}

fn assert_recipe_definitions(sequence: &serde_json::Value) {
    let recipes = &sequence["recipes"];
    assert_eq!(
        recipes["channel_index"].as_str(),
        Some("ch=0..5 in plane_order; sample i=0..1535")
    );
    assert_eq!(recipes["0"].as_str(), Some("positive zero"));
    assert_eq!(
        recipes["1"].as_str(),
        Some("(ch+1)/16 at i in {0,255,256,1535}, else positive zero")
    );
    assert_eq!(
        recipes["2_primary"].as_str(),
        Some("(((i*3+ch*7)%97)-48)/32")
    );
    assert_eq!(
        recipes["2_secondary"].as_str(),
        Some("(((i*5+ch*7)%97)-48)/32")
    );
    assert_eq!(recipes["3"].as_str(), Some("positive zero"));
    assert_eq!(recipes["4"].as_str(), Some("(i odd ? -1 : +1)*(ch+1)/4"));
    assert_eq!(recipes["5"].as_str(), Some("(ch+1)/64"));
    assert_eq!(recipes["6"].as_str(), Some("negative zero"));
    assert_eq!(recipes["7"].as_str(), Some("positive zero"));
    assert_eq!(
        recipes["final_conversion"].as_str(),
        Some(
            "For primary R32(v); for secondary R32(v*3/4), including negative-zero preservation. All formulas except explicit zero are exact dyadic rationals."
        )
    );
}

fn reconstruct_planes(recipe_id: u64, secondary: bool) -> [[f32; SAMPLE_COUNT]; CHANNEL_COUNT] {
    std::array::from_fn(|channel| {
        std::array::from_fn(|sample| {
            let value = recipe_value(recipe_id, channel, sample, secondary);
            if secondary {
                (value * 3.0_f32) / 4.0_f32
            } else {
                value
            }
        })
    })
}

fn recipe_value(recipe_id: u64, channel: usize, sample: usize, secondary: bool) -> f32 {
    let channel_scale = (channel + 1) as f32;
    match recipe_id {
        1 if [0, 255, 256, 1_535].contains(&sample) => channel_scale / 16.0_f32,
        0 | 1 | 3 | 7 => 0.0,
        2 => {
            let stride = if secondary { 5 } else { 3 };
            let numerator = i32::try_from((sample * stride + channel * 7) % 97).unwrap() - 48;
            numerator as f32 / 32.0_f32
        }
        4 => {
            let sign = if sample % 2 == 1 { -1.0_f32 } else { 1.0_f32 };
            (sign * channel_scale) / 4.0_f32
        }
        5 => channel_scale / 64.0_f32,
        6 => f32::from_bits(0x8000_0000),
        _ => panic!("unexpected statistics recipe id {recipe_id}"),
    }
}

fn finite_planes(planes: &[[f32; SAMPLE_COUNT]; CHANNEL_COUNT]) -> FiniteStatisticsPlanes {
    std::array::from_fn(|channel| {
        std::array::from_fn(|sample| FiniteF32::new(planes[channel][sample]).unwrap())
    })
}

fn plane_bytes(planes: &[[f32; SAMPLE_COUNT]; CHANNEL_COUNT]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(CHANNEL_COUNT * SAMPLE_COUNT * 4);
    for plane in planes {
        for sample in plane {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
    }
    bytes
}

fn assert_output_matches(
    expected: &serde_json::Value,
    actual: &jocforge_domain::drc::none_drc_statistics::NoneDrcStatisticsFrameOutput,
    step: usize,
) {
    assert_eq!(
        float_words(&actual.rms_levels_six),
        json_words(&expected["rms_levels_six"]),
        "rms_levels_six at step {step}"
    );
    assert_eq!(
        float_words(&actual.peak_inputs_six),
        json_words(&expected["peak_inputs_six"]),
        "peak_inputs_six at step {step}"
    );
    assert_eq!(
        float_words(&actual.auxiliary_raw_peaks_six),
        json_words(&expected["auxiliary_raw_peaks_six"]),
        "auxiliary_raw_peaks_six at step {step}"
    );

    let expected_blocks = expected["energy_blocks"].as_array().unwrap();
    assert_eq!(expected_blocks.len(), BLOCK_COUNT);
    for (block, expected_block) in expected_blocks.iter().enumerate() {
        assert_eq!(
            float_words(&actual.energy_blocks[block].mean_square_five),
            json_words(&expected_block["mean_square_five"]),
            "mean_square_five at step {step}, block {block}"
        );
        assert_eq!(
            actual.energy_blocks[block].summed_power.to_bits(),
            json_word(&expected_block["summed_power"]),
            "summed_power at step {step}, block {block}"
        );
        assert_eq!(
            actual.energy_blocks[block].normalized_level.to_bits(),
            json_word(&expected_block["normalized_level"]),
            "normalized_level at step {step}, block {block}"
        );
    }
}

fn assert_state_matches(expected: &serde_json::Value, actual: &NoneDrcStatisticsState) {
    let expected_filter = expected["filter"]
        .as_array()
        .unwrap()
        .iter()
        .map(json_words)
        .collect::<Vec<_>>();
    let actual_filter = actual
        .filter_pairs()
        .iter()
        .map(|pair| pair.iter().map(|value| value.to_bits()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert_eq!(actual_filter, expected_filter);
    assert_eq!(
        float_words(actual.previous_primary_peak()),
        json_words(&expected["primary_peak"])
    );
    assert_eq!(
        float_words(actual.previous_secondary_peak()),
        json_words(&expected["secondary_peak"])
    );
    assert_eq!(
        actual.previous_downmix_peak().to_bits(),
        json_word(&expected["downmix_peak"])
    );
}

fn json_words(value: &serde_json::Value) -> Vec<u32> {
    value.as_array().unwrap().iter().map(json_word).collect()
}

fn json_word(value: &serde_json::Value) -> u32 {
    u32::try_from(value.as_u64().unwrap()).unwrap()
}

fn float_words(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").unwrap();
    }
    hex
}
