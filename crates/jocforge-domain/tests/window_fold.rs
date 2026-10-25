use jocforge_domain::transform::math_generated_parameters::{
    MathGeneratedTransformParameterError, MathGeneratedTransformParameters,
};
use jocforge_domain::transform::window_fold::{F08WindowFoldError, f08_window_fold_block};
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUT_VECTORS_SHA256: &str =
    "eb825796adc60933b1c717122f991a9a0ced95ab0cd89d29ef4f6e1e28b6866c";
const RESULT_SHA256: &str = "a77806e29ea7e0cb361353a2a6313826e4cdcc2b3f36cebce6fc0b434cdcb906";
const PARAMETER_SHA256: &str = "f1fb1a16b3caedab6ddc583201b9454755fd8871ecdf7881a6a3ffbad10fddf1";
const INPUT_VECTORS: &str = include_str!("fixtures/window_fold/INPUT-ONLY-VECTORS.json");
const RESULT: &str = include_str!("fixtures/window_fold/RESULT.json");
const PARAMETERS: &[u8] =
    include_bytes!("../src/transform/ordinary_transform_math_parameters.json");

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
fn selected_math_generated_provider_is_identity_gated() {
    assert_eq!(sha256(PARAMETERS), PARAMETER_SHA256);
    assert_eq!(
        MathGeneratedTransformParameters::selected_math_generated_v1()
            .unwrap()
            .window_f32_bits()
            .len(),
        256
    );

    let mut missing_field: Value = serde_json::from_slice(PARAMETERS).unwrap();
    missing_field["fields"]
        .as_object_mut()
        .unwrap()
        .remove("butterfly");
    let missing_bytes = serde_json::to_vec(&missing_field).unwrap();
    assert_eq!(
        MathGeneratedTransformParameters::from_provider_bytes(&missing_bytes),
        Err(MathGeneratedTransformParameterError::ParameterUnavailable)
    );

    let mut invalid_permutation: Value = serde_json::from_slice(PARAMETERS).unwrap();
    invalid_permutation["fields"]["spectrum_permutation"][2] =
        invalid_permutation["fields"]["spectrum_permutation"][0].clone();
    let invalid_bytes = serde_json::to_vec(&invalid_permutation).unwrap();
    assert_eq!(
        MathGeneratedTransformParameters::from_provider_bytes(&invalid_bytes),
        Err(MathGeneratedTransformParameterError::ParameterIdentityError)
    );

    let mut changed_identity = PARAMETERS.to_vec();
    let marker = b"math-generated-v1";
    let offset = changed_identity
        .windows(marker.len())
        .position(|window| window == marker)
        .unwrap();
    changed_identity[offset + marker.len() - 1] = b'2';
    assert_eq!(
        MathGeneratedTransformParameters::from_provider_bytes(&changed_identity),
        Err(MathGeneratedTransformParameterError::ParameterIdentityError)
    );
}

#[test]
fn f08_window_fold_matches_fbw_and_lfe_nonzero_trace_hashes() {
    let input_bytes = INPUT_VECTORS.as_bytes();
    let result_bytes = RESULT.as_bytes();
    assert_eq!(sha256(input_bytes), INPUT_VECTORS_SHA256);
    assert_eq!(sha256(result_bytes), RESULT_SHA256);
    let inputs: Value = serde_json::from_slice(input_bytes).unwrap();
    let result: Value = serde_json::from_slice(result_bytes).unwrap();
    assert_eq!(inputs["input_only"], true);
    assert_eq!(inputs["forbidden_inputs"].as_array().unwrap().len(), 5);
    assert_eq!(
        result["local_composition_ready"],
        "YES_FOR_MATH_GENERATED_V1"
    );
    assert_eq!(result["production_equivalence_ready"], "NO");
    assert_eq!(result["whole_implementation_ready"], "NO");

    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let fbw_inputs = &inputs["fbw_continuous"]["blocks"];
    let fbw_flags = &inputs["fbw_continuous"]["short_flags_i1"];
    let fbw_traces = &result["fbw_continuous"]["traces"];
    assert_eq!(fbw_inputs.as_array().unwrap().len(), 3);
    assert_eq!(fbw_traces.as_array().unwrap().len(), 3);
    assert_eq!(fbw_flags, &serde_json::json!([0, 1, 0]));

    let mut primary_history = [0_u32; 128];
    let mut companion_history = [0_u32; 128];
    for (input, trace) in fbw_inputs
        .as_array()
        .unwrap()
        .iter()
        .zip(fbw_traces.as_array().unwrap())
    {
        let block = block_from_recipe(input);
        let output =
            f08_window_fold_block(&block, &primary_history, &companion_history, &parameters)
                .unwrap();
        assert_eq!(
            sha256_words(output.primary_fold_bits()),
            trace["primary_fold_sha256"].as_str().unwrap(),
            "{} primary fold",
            input["id"]
        );
        assert_eq!(
            sha256_words(output.companion_fold_bits()),
            trace["companion_fold_sha256"].as_str().unwrap(),
            "{} companion fold",
            input["id"]
        );
        assert_eq!(
            sha256_words(output.primary_history_after()),
            trace["primary_next_history_sha256"].as_str().unwrap(),
            "{} primary history",
            input["id"]
        );
        assert_eq!(
            sha256_words(output.companion_history_after()),
            trace["companion_next_history_sha256"].as_str().unwrap(),
            "{} companion history",
            input["id"]
        );
        primary_history = *output.primary_history_after();
        companion_history = *output.companion_history_after();
    }

    let lfe_inputs = &inputs["lfe_continuous"]["blocks"];
    let lfe_traces = &result["lfe_continuous"]["traces"];
    assert_eq!(lfe_inputs.as_array().unwrap().len(), 3);
    assert_eq!(lfe_traces.as_array().unwrap().len(), 3);
    let mut lfe_primary_history = [0_u32; 128];
    let mut lfe_companion_history = [0_u32; 128];
    for (input, trace) in lfe_inputs
        .as_array()
        .unwrap()
        .iter()
        .zip(lfe_traces.as_array().unwrap())
    {
        let block = block_from_recipe(input);
        let output = f08_window_fold_block(
            &block,
            &lfe_primary_history,
            &lfe_companion_history,
            &parameters,
        )
        .unwrap();
        assert_eq!(
            sha256_words(output.primary_fold_bits()),
            trace["primary_fold_sha256"].as_str().unwrap(),
            "{} LFE primary fold",
            input["id"]
        );
        assert_eq!(
            sha256_words(output.companion_fold_bits()),
            trace["companion_fold_sha256"].as_str().unwrap(),
            "{} LFE companion fold",
            input["id"]
        );
        assert_eq!(
            sha256_words(output.primary_history_after()),
            trace["primary_next_history_sha256"].as_str().unwrap(),
            "{} LFE primary history",
            input["id"]
        );
        assert_eq!(
            sha256_words(output.companion_history_after()),
            trace["companion_next_history_sha256"].as_str().unwrap(),
            "{} LFE companion history",
            input["id"]
        );
        lfe_primary_history = *output.primary_history_after();
        lfe_companion_history = *output.companion_history_after();
    }
}

#[test]
fn f08_window_fold_rejects_bad_inputs_and_numeric_overflow_without_state_changes() {
    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let input = [0_u32; 256];
    let primary = [0_u32; 128];
    let companion = [0_u32; 128];
    let before_primary = primary;
    let before_companion = companion;

    assert_eq!(
        f08_window_fold_block(&input[..255], &primary, &companion, &parameters),
        Err(F08WindowFoldError::InputError)
    );
    assert_eq!(
        f08_window_fold_block(&input, &primary[..127], &companion, &parameters),
        Err(F08WindowFoldError::InputError)
    );

    let mut nonfinite_input = input;
    nonfinite_input[0] = f32::INFINITY.to_bits();
    assert_eq!(
        f08_window_fold_block(&nonfinite_input, &primary, &companion, &parameters),
        Err(F08WindowFoldError::InputError)
    );
    let mut nonfinite_history = primary;
    nonfinite_history[0] = f32::NAN.to_bits();
    assert_eq!(
        f08_window_fold_block(&input, &nonfinite_history, &companion, &parameters),
        Err(F08WindowFoldError::InputError)
    );

    let overflowing_input = [f32::MAX.to_bits(); 256];
    assert_eq!(
        f08_window_fold_block(&overflowing_input, &primary, &companion, &parameters),
        Err(F08WindowFoldError::NumericFailure)
    );
    assert_eq!(primary, before_primary);
    assert_eq!(companion, before_companion);
}
