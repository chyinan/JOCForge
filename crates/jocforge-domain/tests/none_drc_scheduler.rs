use jocforge_domain::drc::none_drc_dialnorm_offset::none_drc_dialnorm_offset_word;
use jocforge_domain::drc::none_drc_scheduler::{
    NoneDrcSchedulerFrameInput, NoneDrcSchedulerFrameOutput, NoneDrcSchedulerState,
    transition_none_drc_scheduler_frame,
};
use jocforge_domain::numeric::FiniteF32;
use sha2::{Digest, Sha256};

const AUTHORIZED_PARAMETERS: &str = include_str!("fixtures/none_drc/authorized_parameters.json");
const AUTHORIZED_MATH: &str = include_str!("fixtures/none_drc/authorized_math_parameters.json");
const AUTHORIZED_VERIFICATION: &str =
    include_str!("fixtures/none_drc/authorized_verification.json");

const BLOCK_COUNT: usize = 6;

#[test]
fn selected_scheduler_provider_and_vector_hashes_match_wash_sources() {
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
        json_words(&parameters["groups"]["DRC_NORMALIZE3"]["f32_words_u32"]),
        [0x3CAA_5BD7, 0x3C62_2BCA, 0x3BE6_35DE]
    );
    assert_eq!(
        json_words(&parameters["groups"]["DRC_QUANTIZE3"]["f32_words_u32"]),
        [0x3E2F_7469, 0x3F2A_DD8F, 0x3F7F_7A01]
    );

    let mathematical: serde_json::Value = serde_json::from_str(AUTHORIZED_MATH).unwrap();
    let dialnorm_words = json_words(&mathematical["dialnorm_words"]);
    assert_eq!(dialnorm_words.len(), 31);
    assert_eq!(
        none_drc_dialnorm_offset_word(24).unwrap(),
        dialnorm_words[23]
    );
}

#[test]
fn all_186_scheduler_recipes_recompute_then_advance_only_actual_state() {
    let verification: serde_json::Value = serde_json::from_str(AUTHORIZED_VERIFICATION).unwrap();
    assert_eq!(
        verification["expected_outputs_are_never_inputs"],
        serde_json::Value::Bool(true)
    );
    let sequence = &verification["scheduler_sequence"];
    assert_eq!(sequence["programme_dialnorm"].as_i64(), Some(24));
    assert_input_recipe(sequence);

    let cases = sequence["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 186);
    let mut state = NoneDrcSchedulerState::cold();
    assert_state_matches(&sequence["cold_state"], &state);

    for (index, case) in cases.iter().enumerate() {
        assert_eq!(case["sequence_step"].as_u64(), Some(index as u64));
        assert_eq!(case["input_recipe_index"].as_u64(), Some(index as u64));

        let rms_levels = reconstruct_rms_levels(index);
        let peak_inputs = reconstruct_peak_inputs(index);
        let input = NoneDrcSchedulerFrameInput {
            dialnorm: 24,
            rms_levels_six: finite_levels(rms_levels),
            peak_inputs_six: finite_levels(peak_inputs),
        };

        let transition = transition_none_drc_scheduler_frame(&state, &input).unwrap();
        let actual_output = transition.output;
        state = transition.next_state;

        assert_output_matches(&case["expected_output"], &actual_output, index);
        assert_state_matches(&case["expected_next_state_words"], &state);
    }
}

fn assert_input_recipe(sequence: &serde_json::Value) {
    let recipe = &sequence["input_recipe"];
    assert_eq!(recipe["index_domain"], serde_json::json!([0, 185]));
    assert_eq!(
        recipe["dialnorm"].as_str(),
        Some("fixed24 for the entire sequence")
    );
    assert_eq!(
        recipe["rms_levels_six"].as_str(),
        Some("R32(((index*7+j*13)%257-128)/128), j=0..5")
    );
    assert_eq!(
        recipe["peak_pattern"].as_str(),
        Some("select index%6 from patterns; each has6 entries j=0..5")
    );
    assert_eq!(
        recipe["patterns"],
        serde_json::json!([
            "all positive zero",
            "all R32(10^-12)",
            "R32((j+1)/16)",
            "R32((6-j)/2)",
            "all128",
            "j odd:1/8, j even:8"
        ])
    );
    assert_eq!(
        recipe["R32_10_minus12"].as_str(),
        Some("R32 of exact decimal 0.000000000001")
    );
}

fn reconstruct_rms_levels(index: usize) -> [f32; BLOCK_COUNT] {
    std::array::from_fn(|block| {
        let numerator = i32::try_from((index * 7 + block * 13) % 257).unwrap() - 128;
        numerator as f32 / 128.0_f32
    })
}

fn reconstruct_peak_inputs(index: usize) -> [f32; BLOCK_COUNT] {
    let pattern = index % BLOCK_COUNT;
    std::array::from_fn(|block| match pattern {
        0 => 0.0,
        1 => 1e-12_f32,
        2 => (block + 1) as f32 / 16.0_f32,
        3 => (6 - block) as f32 / 2.0_f32,
        4 => 128.0_f32,
        5 if block % 2 == 1 => 1.0_f32 / 8.0_f32,
        5 => 8.0_f32,
        _ => unreachable!("pattern is the remainder modulo six"),
    })
}

fn finite_levels(values: [f32; BLOCK_COUNT]) -> [FiniteF32; BLOCK_COUNT] {
    values.map(|value| FiniteF32::new(value).unwrap())
}

fn assert_output_matches(
    expected: &serde_json::Value,
    actual: &NoneDrcSchedulerFrameOutput,
    step: usize,
) {
    assert_eq!(
        u64::from(actual.rf_packed_word_u16),
        expected["RF_packed_word_u16"].as_u64().unwrap(),
        "RF packed word at step {step}"
    );
    assert_eq!(
        u64::from(actual.rf_flag_u16),
        expected["RF_flag_u16"].as_u64().unwrap(),
        "RF flag at step {step}"
    );
    assert_eq!(
        actual
            .line_packed_words_u16
            .into_iter()
            .map(u64::from)
            .collect::<Vec<_>>(),
        json_u64s(&expected["Line_packed_words_u16"]),
        "line packed words at step {step}"
    );
    assert_eq!(
        actual
            .line_flags_u16
            .into_iter()
            .map(u64::from)
            .collect::<Vec<_>>(),
        json_u64s(&expected["Line_flags_u16"]),
        "line flags at step {step}"
    );
    assert_eq!(
        u64::from(actual.rf_aux_packed_word_u16),
        expected["RF_aux_packed_word_u16"].as_u64().unwrap(),
        "RF auxiliary word at step {step}"
    );
    assert_eq!(
        u64::from(actual.rf_limiter_packed_word_u16),
        expected["RF_limiter_packed_word_u16"].as_u64().unwrap(),
        "RF limiter word at step {step}"
    );
    assert_eq!(
        actual
            .line_aux_packed_words_u16
            .into_iter()
            .map(u64::from)
            .collect::<Vec<_>>(),
        json_u64s(&expected["Line_aux_packed_words_u16"]),
        "line auxiliary words at step {step}"
    );
    assert_eq!(
        actual
            .line_limiter_packed_words_u16
            .into_iter()
            .map(u64::from)
            .collect::<Vec<_>>(),
        json_u64s(&expected["Line_limiter_packed_words_u16"]),
        "line limiter words at step {step}"
    );
    assert_eq!(
        i64::from(actual.line_minimum_signed_word),
        expected["Line_minimum_signed_word"].as_i64().unwrap(),
        "minimum signed line word at step {step}"
    );
    assert_eq!(
        actual.peak_logs_six_words.to_vec(),
        json_words(&expected["peak_logs_six_words"]),
        "peak logs at step {step}"
    );
    assert_eq!(
        actual.line_gain_words.to_vec(),
        json_words(&expected["line_gain_words"]),
        "line gains at step {step}"
    );
}

fn assert_state_matches(expected: &serde_json::Value, actual: &NoneDrcSchedulerState) {
    assert_eq!(actual.rf_dynamics_words(), dynamics_words(&expected["rf"]));
    assert_eq!(
        actual.line_dynamics_words(),
        dynamics_words(&expected["line"])
    );
    assert_eq!(
        actual.rf_limiter_words(),
        limiter_words(&expected["rf_limiter"])
    );
    assert_eq!(
        actual.line_limiter_words(),
        limiter_words(&expected["line_limiter"])
    );
    assert_eq!(
        actual.peak_six_words().to_vec(),
        json_words(&expected["peak_six"])
    );
    assert_eq!(
        actual.generation(),
        expected["generation"].as_u64().unwrap()
    );
}

fn dynamics_words(value: &serde_json::Value) -> (u32, u16, u32) {
    let values = value.as_array().unwrap();
    (
        u32::try_from(values[0].as_u64().unwrap()).unwrap(),
        u16::try_from(values[1].as_u64().unwrap()).unwrap(),
        u32::try_from(values[2].as_u64().unwrap()).unwrap(),
    )
}

fn limiter_words(value: &serde_json::Value) -> (u32, u16) {
    let values = value.as_array().unwrap();
    (
        u32::try_from(values[0].as_u64().unwrap()).unwrap(),
        u16::try_from(values[1].as_u64().unwrap()).unwrap(),
    )
}

fn json_words(value: &serde_json::Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
        .collect()
}

fn json_u64s(value: &serde_json::Value) -> Vec<u64> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|word| word.as_u64().unwrap())
        .collect()
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
