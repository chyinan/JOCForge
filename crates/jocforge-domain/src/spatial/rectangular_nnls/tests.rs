// pattern: Functional Core
use super::{NnlsError, f64_to_f32_word, solve_rectangular_nnls};

#[test]
fn binary64_to_binary32_conversion_uses_nearest_ties_even() {
    assert_eq!(f64_to_f32_word(1.0 + 2_f64.powi(-24)).unwrap(), 0x3f80_0000);
    assert_eq!(
        f64_to_f32_word(1.0 + 3.0 * 2_f64.powi(-24)).unwrap(),
        0x3f80_0002
    );
    assert_eq!(
        f64_to_f32_word(-1.0 - 2_f64.powi(-24)).unwrap(),
        0xbf80_0000
    );
    assert_eq!(f64_to_f32_word(2_f64.powi(-149)).unwrap(), 0x0000_0001);
    assert_eq!(f64_to_f32_word(-2_f64.powi(-150)).unwrap(), 0x8000_0000);
}

#[test]
fn malformed_rectangular_shapes_fail_before_solver_state_exists() {
    assert_eq!(
        solve_rectangular_nnls(&[vec![0.0_f32.to_bits()]], &[0; 15]),
        Err(NnlsError::InvalidInput)
    );
    assert_eq!(
        solve_rectangular_nnls(&vec![vec![0; 1]; 15], &[0; 14]),
        Err(NnlsError::InvalidInput)
    );
}
