use jocforge_domain::codec::policy_parameters::{PolicyProfile, load_policy_parameters};
use jocforge_domain::transform::exponent_inputs::f08_full_band_exponent_inputs;
use jocforge_domain::transform::fbw_transform::f08_fbw_transform_block;
use jocforge_domain::transform::fullband_exponent_pipeline::{
    FullBandExponentStageControls, FullBandExponentStageError, stage_fullband_f09_f10,
};
use jocforge_domain::transform::math_generated_parameters::MathGeneratedTransformParameters;
use jocforge_domain::transform::window_fold::f08_window_fold_block;
use serde_json::Value;
use std::path::PathBuf;

const INPUT_VECTORS: &str = include_str!("fixtures/window_fold/INPUT-ONLY-VECTORS.json");
const POLICY_PARAMETERS: &[u8] = include_bytes!("fixtures/policy_authorized/PARAMETERS.json");

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

fn first_fullband_exponent_inputs()
-> jocforge_domain::transform::exponent_inputs::FullBandTransformExponentInputs {
    let inputs: Value = serde_json::from_str(INPUT_VECTORS).unwrap();
    let recipe = &inputs["fbw_continuous"]["blocks"][0];
    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let input = block_from_recipe(recipe);
    let cold = [0_u32; 128];
    let fold = f08_window_fold_block(&input, &cold, &cold, &parameters).unwrap();
    let spectra = f08_fbw_transform_block(&fold, 0, &parameters).unwrap();
    f08_full_band_exponent_inputs(&spectra).unwrap()
}

fn selected_policy_parameters() -> jocforge_domain::codec::policy_parameters::PolicyParameters {
    load_policy_parameters(PolicyProfile::selected(), Some(POLICY_PARAMETERS)).unwrap()
}

#[test]
fn selected_f09_refresh_stages_into_f10_without_mutating_before_state() {
    let exponent_inputs = first_fullband_exponent_inputs();
    let work_before = [0_i16; 217];
    let marker_before = 1_i16;
    let controls = FullBandExponentStageControls {
        refresh_mode_i16: 1,
        refresh_special_i16: 0,
        normalization_span_u8: 1,
        normalization_option_u8: 1,
        normalization_gate_i16: 0,
    };
    let policy = selected_policy_parameters();

    let staged = stage_fullband_f09_f10(
        exponent_inputs.clone(),
        &work_before,
        marker_before,
        controls,
        Some(&policy),
    )
    .unwrap();

    assert_eq!(work_before, [0_i16; 217]);
    assert_eq!(marker_before, 1);
    assert_eq!(
        staged.current_after_i16(),
        exponent_inputs.power_raw_exponents_i16()
    );
    assert_eq!(staged.refresh_marker_after_i16(), 1);

    let current = exponent_inputs.power_raw_exponents_i16();
    let work_after = staged.work_after_i16();
    assert_eq!(work_after[0], current[0]);
    for index in 1..128 {
        assert_eq!(
            work_after[index],
            current[index].min(work_after[index - 1] + 2),
            "structural refresh bin {index}"
        );
    }
    assert_eq!(&work_after[128..], &[0_i16; 89]);

    assert_eq!(staged.normalized().grouped_sample_count_i16(), 216);
    assert!(staged.normalized().initial_sum_i32().is_some());
    assert!(staged.normalized().first_pass_square_error_i32().is_some());
    assert!(staged.normalized().error_shift_i32().is_some());
    assert!(staged.normalized().scaled_count_f32_bits().is_some());
    assert!(staged.normalized().lrint_threshold_i32().is_some());
    assert!(
        staged
            .normalized()
            .expanded_exponents()
            .iter()
            .all(|value| (0..=24).contains(value))
    );
}

#[test]
fn missing_f10_policy_and_invalid_refresh_before_state_return_without_proposals() {
    let exponent_inputs = first_fullband_exponent_inputs();
    let work_before = [0_i16; 217];
    let controls = FullBandExponentStageControls {
        refresh_mode_i16: 1,
        refresh_special_i16: 0,
        normalization_span_u8: 1,
        normalization_option_u8: 1,
        normalization_gate_i16: 0,
    };

    assert_eq!(
        stage_fullband_f09_f10(
            exponent_inputs.clone(),
            &work_before,
            1,
            controls,
            None,
        ),
        Err(FullBandExponentStageError::Normalization(
            jocforge_domain::codec::exponent_normalization::ExponentNormalizationError::
                ParameterUnavailable
        ))
    );

    let mut invalid_work = [0_i16; 217];
    invalid_work[7] = 32;
    let before_snapshot = invalid_work;
    assert_eq!(
        stage_fullband_f09_f10(
            exponent_inputs,
            &invalid_work,
            1,
            controls,
            Some(&selected_policy_parameters()),
        ),
        Err(FullBandExponentStageError::ExponentInputError)
    );
    assert_eq!(invalid_work, before_snapshot);
}

#[test]
fn selected_policy_fixture_is_the_approved_repository_asset() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = root.join("tests/fixtures/policy_authorized/PARAMETERS.json");
    assert_eq!(std::fs::read(fixture).unwrap(), POLICY_PARAMETERS);
}
