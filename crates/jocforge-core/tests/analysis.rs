// pattern: Functional Core

use jocforge_core::{factorize_rank_controlled, quantize_coefficient};

#[test]
fn rank_factorization_recovers_known_linear_combinations() {
    let objects = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![1.0, 1.0, 0.0, 0.0],
    ];

    let result = factorize_rank_controlled(&objects, 2).expect("rank factorization");

    assert!((result.scale_factor - 820.0 / 4096.0).abs() < 1.0e-12);
    assert!((result.scaled_matrix[0][0] - result.scale_factor).abs() < 1.0e-12);
    assert!((result.scaled_matrix[2][0] - result.scale_factor).abs() < 1.0e-12);
    assert!(result.scaled_matrix[2][1].abs() > 0.19);
}

#[test]
fn coefficient_quantization_reports_ideal_encoded_decoded_and_error() {
    let coefficient = quantize_coefficient(820.0 / 4096.0).expect("coefficient in MVP domain");

    assert_eq!(coefficient.encoded_symbol, 1);
    assert!((coefficient.decoded - 820.0 / 4096.0).abs() < 1.0e-12);
    assert!(coefficient.error.abs() < 1.0e-12);
}
