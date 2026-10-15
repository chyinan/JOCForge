// pattern: Functional Core
use jocforge_domain::numeric::{Q15CoordinateError, quantize_q15_coordinate};

#[test]
fn q15_coordinate_matches_scoped_rounding_and_saturation_rules() {
    let cases = [
        (0.0_f32, 0),
        (-0.0_f32, 0),
        (0.5_f32, 16_384),
        (-0.5_f32, -16_384),
        (1.0_f32, 32_767),
        (2.0_f32, 32_767),
        (-1.0_f32, -32_768),
    ];
    for (input, expected) in cases {
        assert_eq!(quantize_q15_coordinate(input).unwrap().value(), expected);
    }
}

#[test]
fn non_finite_input_and_intermediate_are_rejected() {
    for input in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            quantize_q15_coordinate(input),
            Err(Q15CoordinateError::NonFiniteInput)
        );
    }
    assert_eq!(
        quantize_q15_coordinate(f32::MAX),
        Err(Q15CoordinateError::NonFiniteIntermediate)
    );
    assert_eq!(
        quantize_q15_coordinate(-f32::MAX),
        Err(Q15CoordinateError::NonFiniteIntermediate)
    );
}

#[test]
fn positive_saturation_never_wraps_and_negative_values_are_not_clamped_to_zero() {
    assert_eq!(quantize_q15_coordinate(65_536.0).unwrap().value(), 32_767);
    assert!(quantize_q15_coordinate(-1.25).unwrap().value() < -32_768);
}
