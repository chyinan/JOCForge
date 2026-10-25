use jocforge_domain::transform::math_generated_parameters::MathGeneratedTransformParameters;
use jocforge_domain::transform::short_transform::f08_short_transform_block;
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

#[test]
fn math_generated_short_branch_matches_nonzero_fbw_trace() {
    let input_bytes = INPUT_VECTORS.as_bytes();
    let result_bytes = RESULT.as_bytes();
    assert_eq!(sha256(input_bytes), INPUT_VECTORS_SHA256);
    assert_eq!(sha256(result_bytes), RESULT_SHA256);
    let inputs: Value = serde_json::from_slice(input_bytes).unwrap();
    let result: Value = serde_json::from_slice(result_bytes).unwrap();
    assert_eq!(result["production_equivalence_ready"], "NO");
    assert_eq!(result["whole_implementation_ready"], "NO");

    let blocks = inputs["fbw_continuous"]["blocks"].as_array().unwrap();
    let flags = inputs["fbw_continuous"]["short_flags_i1"]
        .as_array()
        .unwrap();
    let traces = result["fbw_continuous"]["traces"].as_array().unwrap();
    assert_eq!(flags.len(), 3);
    assert_eq!(flags[0].as_u64(), Some(0));
    assert_eq!(flags[1].as_u64(), Some(1));
    assert_eq!(traces[1]["branch"], "SHORT_INTERLEAVED");

    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let zero_history = [0_u32; 128];
    let long_input = block_from_recipe(&blocks[0]);
    let long_fold =
        f08_window_fold_block(&long_input, &zero_history, &zero_history, &parameters).unwrap();
    assert_eq!(
        sha256_words(long_fold.primary_history_after()),
        traces[0]["primary_next_history_sha256"].as_str().unwrap()
    );
    assert_eq!(
        sha256_words(long_fold.companion_history_after()),
        traces[0]["companion_next_history_sha256"].as_str().unwrap()
    );

    let short_input = block_from_recipe(&blocks[1]);
    let short_fold = f08_window_fold_block(
        &short_input,
        long_fold.primary_history_after(),
        long_fold.companion_history_after(),
        &parameters,
    )
    .unwrap();
    let short_output = f08_short_transform_block(&short_fold, &parameters).unwrap();

    assert_eq!(
        sha256_words(short_output.primary_spectrum_bits()),
        traces[1]["primary_spectrum_sha256"].as_str().unwrap()
    );
    assert_eq!(
        sha256_words(short_output.companion_spectrum_bits()),
        traces[1]["companion_spectrum_sha256"].as_str().unwrap()
    );
    assert_eq!(
        short_output
            .primary_spectrum_bits()
            .iter()
            .filter(|word| **word != 0)
            .count(),
        traces[1]["nonzero_primary_spectrum_words"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        short_output
            .companion_spectrum_bits()
            .iter()
            .filter(|word| **word != 0)
            .count(),
        traces[1]["nonzero_companion_spectrum_words"]
            .as_u64()
            .unwrap() as usize
    );
}
