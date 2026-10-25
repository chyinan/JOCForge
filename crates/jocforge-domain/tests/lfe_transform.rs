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
fn math_generated_lfe_transform_matches_three_continuous_nonzero_traces() {
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
    let blocks = inputs["lfe_continuous"]["blocks"].as_array().unwrap();
    let traces = result["lfe_continuous"]["traces"].as_array().unwrap();
    assert_eq!(blocks.len(), 3);
    assert_eq!(traces.len(), blocks.len());

    let mut primary_history = [0_u32; 128];
    let mut companion_history = [0_u32; 128];
    for (recipe, trace) in blocks.iter().zip(traces) {
        let input = block_from_recipe(recipe);
        let fold = f08_window_fold_block(&input, &primary_history, &companion_history, &parameters)
            .unwrap();
        let output = f08_lfe_transform_block(&fold, &parameters).unwrap();

        assert_eq!(trace["case_id"], recipe["id"]);
        assert_eq!(trace["branch"], "LFE_RECURSIVE");
        assert_eq!(
            sha256_words(output.primary_spectrum_bits()),
            trace["primary_spectrum_sha256"].as_str().unwrap(),
            "{} primary spectrum",
            recipe["id"]
        );
        assert_eq!(
            sha256_words(output.companion_spectrum_bits()),
            trace["companion_spectrum_sha256"].as_str().unwrap(),
            "{} companion spectrum",
            recipe["id"]
        );
        assert_eq!(
            output
                .primary_spectrum_bits()
                .iter()
                .filter(|word| **word != 0)
                .count(),
            trace["nonzero_primary_spectrum_words"].as_u64().unwrap() as usize
        );
        assert_eq!(
            output
                .companion_spectrum_bits()
                .iter()
                .filter(|word| **word != 0)
                .count(),
            trace["nonzero_companion_spectrum_words"].as_u64().unwrap() as usize
        );

        primary_history = *fold.primary_history_after();
        companion_history = *fold.companion_history_after();
    }
}
