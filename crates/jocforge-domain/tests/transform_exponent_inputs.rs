use jocforge_domain::transform::exponent_inputs::{
    f08_full_band_exponent_inputs, f08_lfe_exponent_inputs,
};
use jocforge_domain::transform::fbw_transform::f08_fbw_transform_block;
use jocforge_domain::transform::lfe_transform::f08_lfe_transform_block;
use jocforge_domain::transform::math_generated_parameters::MathGeneratedTransformParameters;
use jocforge_domain::transform::window_fold::f08_window_fold_block;
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUT_VECTORS_SHA256: &str =
    "eb825796adc60933b1c717122f991a9a0ced95ab0cd89d29ef4f6e1e28b6866c";
const RESULT_SHA256: &str = "a77806e29ea7e0cb361353a2a6313826e4cdcc2b3f36cebce6fc0b434cdcb906";
const INPUT_VECTORS: &str = include_str!("fixtures/window_fold/INPUT-ONLY-VECTORS.json");
const RESULT: &str = include_str!("fixtures/window_fold/RESULT.json");

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sha256_words(words: &[u32]) -> String {
    let bytes = words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    sha256(&bytes)
}

fn sha256_exponents(words: &[i16]) -> String {
    let bytes = words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    sha256(&bytes)
}

fn block_from_recipe(recipe: &Value) -> [u32; 256] {
    assert_eq!(recipe["length"].as_u64(), Some(256));
    let multiplier = recipe["multiplier"].as_u64().unwrap();
    let offset = recipe["offset"].as_u64().unwrap();
    let modulus = recipe["modulus"].as_u64().unwrap();
    let centre = recipe["centre"].as_i64().unwrap();
    let denominator = recipe["power_of_two_denominator"].as_u64().unwrap() as f32;
    std::array::from_fn(|index| {
        let residue = (u64::try_from(index).unwrap() * multiplier + offset) % modulus;
        let numerator = i64::try_from(residue).unwrap() - centre;
        ((numerator as f32) / denominator).to_bits()
    })
}

fn assert_prefix(actual: &[i16], expected: &Value) {
    let expected = expected
        .as_array()
        .unwrap()
        .iter()
        .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(&actual[..expected.len()], expected.as_slice());
}

#[test]
fn math_generated_fbw_and_lfe_spectra_feed_selected_f09_power_and_primary_rows() {
    let input_bytes = INPUT_VECTORS.as_bytes();
    let result_bytes = RESULT.as_bytes();
    assert_eq!(sha256(input_bytes), INPUT_VECTORS_SHA256);
    assert_eq!(sha256(result_bytes), RESULT_SHA256);

    let inputs: Value = serde_json::from_slice(input_bytes).unwrap();
    let result: Value = serde_json::from_slice(result_bytes).unwrap();
    assert_eq!(inputs["input_only"], true);
    assert_eq!(result["production_equivalence_ready"], "NO");
    assert_eq!(result["whole_implementation_ready"], "NO");
    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();

    let fbw_blocks = inputs["fbw_continuous"]["blocks"].as_array().unwrap();
    let fbw_flags = inputs["fbw_continuous"]["short_flags_i1"]
        .as_array()
        .unwrap();
    let fbw_traces = result["fbw_continuous"]["traces"].as_array().unwrap();
    assert_eq!(fbw_blocks.len(), 3);
    assert_eq!(fbw_flags.len(), fbw_blocks.len());
    assert_eq!(fbw_traces.len(), fbw_blocks.len());

    let mut primary_history = [0_u32; 128];
    let mut companion_history = [0_u32; 128];
    for ((recipe, flag), trace) in fbw_blocks.iter().zip(fbw_flags).zip(fbw_traces) {
        let input = block_from_recipe(recipe);
        let fold = f08_window_fold_block(&input, &primary_history, &companion_history, &parameters)
            .unwrap();
        let selector = i32::try_from(flag.as_i64().unwrap()).unwrap();
        let spectra = f08_fbw_transform_block(&fold, selector, &parameters).unwrap();
        let exponents = f08_full_band_exponent_inputs(&spectra).unwrap();

        assert_eq!(
            sha256_words(exponents.power_f32_bits()),
            trace["power_sha256"].as_str().unwrap(),
            "{} POWER row",
            recipe["id"]
        );
        assert_eq!(
            sha256_exponents(exponents.power_raw_exponents_i16()),
            trace["power_raw_sha256"].as_str().unwrap(),
            "{} POWER raw exponent row",
            recipe["id"]
        );
        assert_eq!(
            sha256_exponents(exponents.primary_raw_exponents_i16()),
            trace["primary_raw_sha256"].as_str().unwrap(),
            "{} PRIMARY raw exponent row",
            recipe["id"]
        );
        assert_prefix(
            exponents.power_raw_exponents_i16(),
            &trace["power_raw_preview"],
        );
        assert_prefix(
            exponents.primary_raw_exponents_i16(),
            &trace["primary_raw_preview"],
        );
        primary_history = *fold.primary_history_after();
        companion_history = *fold.companion_history_after();
    }

    let lfe_blocks = inputs["lfe_continuous"]["blocks"].as_array().unwrap();
    let lfe_traces = result["lfe_continuous"]["traces"].as_array().unwrap();
    assert_eq!(lfe_blocks.len(), 3);
    assert_eq!(lfe_traces.len(), lfe_blocks.len());
    primary_history = [0_u32; 128];
    companion_history = [0_u32; 128];
    for (recipe, trace) in lfe_blocks.iter().zip(lfe_traces) {
        let input = block_from_recipe(recipe);
        let fold = f08_window_fold_block(&input, &primary_history, &companion_history, &parameters)
            .unwrap();
        let spectra = f08_lfe_transform_block(&fold, &parameters).unwrap();
        let exponents = f08_lfe_exponent_inputs(&spectra).unwrap();

        assert_eq!(
            sha256_words(exponents.power_f32_bits()),
            trace["power_sha256"].as_str().unwrap(),
            "{} LFE POWER row",
            recipe["id"]
        );
        assert_eq!(
            sha256_exponents(exponents.power_raw_exponents_i16()),
            trace["power_raw_sha256"].as_str().unwrap(),
            "{} LFE POWER raw exponent row",
            recipe["id"]
        );
        assert_eq!(
            sha256_exponents(exponents.primary_raw_exponents_i16()),
            trace["primary_raw_sha256"].as_str().unwrap(),
            "{} LFE PRIMARY raw exponent row",
            recipe["id"]
        );
        assert_prefix(
            exponents.power_raw_exponents_i16(),
            &trace["power_raw_preview"],
        );
        assert_prefix(
            exponents.primary_raw_exponents_i16(),
            &trace["primary_raw_preview"],
        );
        primary_history = *fold.primary_history_after();
        companion_history = *fold.companion_history_after();
    }
}
