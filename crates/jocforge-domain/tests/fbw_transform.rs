use jocforge_domain::transform::fbw_transform::{
    F08FbwBranch, F08FbwTransformError, f08_fbw_transform_block,
};
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
fn math_generated_fbw_long_short_long_path_matches_nonzero_trace_hashes() {
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
    let blocks = inputs["fbw_continuous"]["blocks"].as_array().unwrap();
    let flags = inputs["fbw_continuous"]["short_flags_i1"]
        .as_array()
        .unwrap();
    let traces = result["fbw_continuous"]["traces"].as_array().unwrap();
    assert_eq!(blocks.len(), 3);
    assert_eq!(flags.len(), blocks.len());
    assert_eq!(traces.len(), blocks.len());

    let mut primary_history = [0_u32; 128];
    let mut companion_history = [0_u32; 128];
    for ((recipe, flag), trace) in blocks.iter().zip(flags).zip(traces) {
        let input = block_from_recipe(recipe);
        let fold = f08_window_fold_block(&input, &primary_history, &companion_history, &parameters)
            .unwrap();
        let short_flag = i32::try_from(flag.as_i64().unwrap()).unwrap();
        let output = f08_fbw_transform_block(&fold, short_flag, &parameters).unwrap();
        let expected_branch = if short_flag == 0 {
            "LONG"
        } else {
            "SHORT_INTERLEAVED"
        };
        assert_eq!(
            output.branch(),
            if short_flag == 0 {
                F08FbwBranch::Long
            } else {
                F08FbwBranch::Short
            }
        );
        assert_eq!(trace["branch"], expected_branch, "{}", recipe["id"]);
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
        primary_history = *fold.primary_history_after();
        companion_history = *fold.companion_history_after();
    }
}

#[test]
fn fbw_transform_rejects_non_i1_short_flag_before_selecting_a_route() {
    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let input = [0_u32; 256];
    let history = [0_u32; 128];
    let fold = f08_window_fold_block(&input, &history, &history, &parameters).unwrap();

    assert_eq!(
        f08_fbw_transform_block(&fold, -1, &parameters),
        Err(F08FbwTransformError::InputError)
    );
    assert_eq!(
        f08_fbw_transform_block(&fold, 2, &parameters),
        Err(F08FbwTransformError::InputError)
    );
    assert_eq!(fold.primary_history_after(), &history);
    assert_eq!(fold.companion_history_after(), &history);
}
