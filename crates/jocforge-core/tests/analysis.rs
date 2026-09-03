// pattern: Functional Core

use jocforge_core::{
    SOURCE_DERIVED_CARRIER_PEAK_LIMIT, factorize_rank_controlled, factorize_source_derived,
    limit_signal_peak, maximum_absolute_difference, quantize_coefficient, reconstruction_error,
    source_derived_carrier_causality,
};

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

#[test]
fn source_derived_factorization_supports_fewer_objects_than_profile_carriers() {
    let objects = vec![
        vec![0.1, 0.2, 0.3, 0.4],
        vec![0.4, 0.3, 0.2, 0.1],
        vec![0.2, 0.3, 0.4, 0.5],
    ];

    let result = factorize_source_derived(&objects, 7).expect("seven source-derived carriers");

    assert_eq!(result.carriers.len(), 7);
    assert_eq!(result.ideal_matrix.len(), 3);
    assert!(result.ideal_matrix.iter().all(|row| row.len() == 7));
    assert!(result.carriers.iter().all(|carrier| carrier.len() == 4));
    assert!(
        result
            .carriers
            .iter()
            .flatten()
            .all(|value| value.is_finite())
    );
}

#[test]
fn source_derived_factorization_regularizes_near_singular_windows() {
    let base = (0..1536)
        .map(|sample| (f64::from(sample) * 0.013).sin() * 0.2)
        .collect::<Vec<_>>();
    let objects = (0..15)
        .map(|object| {
            base.iter()
                .enumerate()
                .map(|(sample, value)| *value + (f64::from(object) + 1.0) * sample as f64 * 1.0e-12)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let result = factorize_source_derived(&objects, 7).expect("regularized factorization");

    assert!(
        result
            .ideal_matrix
            .iter()
            .flatten()
            .all(|value| value.is_finite())
    );
    assert!(
        result
            .scaled_matrix
            .iter()
            .flatten()
            .all(|value| value.is_finite())
    );
}

#[test]
fn factorization_reconstruction_error_is_an_independent_residual_measure() {
    let objects = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![1.0, 1.0, 0.0, 0.0],
    ];
    let factorization = factorize_rank_controlled(&objects, 2).expect("rank factorization");

    let error = reconstruction_error(&objects, &factorization).expect("reconstruction error");

    assert!(error.is_finite());
    assert!(error < 1.0e-12);
}

#[test]
fn coefficient_jump_measure_is_bounded_by_the_largest_actual_delta() {
    let previous = vec![vec![0.1, -0.2], vec![0.3, 0.4]];
    let current = vec![vec![0.2, -0.1], vec![0.0, 0.9]];

    let jump = maximum_absolute_difference(&previous, &current).expect("coefficient jump");

    assert!((jump - 0.5).abs() < 1.0e-12);
}

#[test]
fn shared_carrier_peak_policy_scales_lfe_sums_without_non_finite_values() {
    let limited = limit_signal_peak(&[1.2, -0.6, 0.3], SOURCE_DERIVED_CARRIER_PEAK_LIMIT)
        .expect("peak policy");

    assert!((limited[0] - SOURCE_DERIVED_CARRIER_PEAK_LIMIT).abs() < 1.0e-12);
    assert!(limited.iter().all(|value| value.is_finite()));
    assert!(
        limited.iter().map(|value| value.abs()).fold(0.0, f64::max)
            <= SOURCE_DERIVED_CARRIER_PEAK_LIMIT
    );
}

#[test]
fn source_derived_carrier_causality_covers_each_intended_profile_input() {
    for carrier_count in [5, 7] {
        let measurements = source_derived_carrier_causality(carrier_count)
            .expect("controlled carrier causality fixture");
        assert_eq!(measurements.len(), carrier_count);
        assert!(measurements.iter().all(|measurement| {
            measurement.carrier_index == measurement.source_index
                && measurement.carrier_changed
                && measurement.maximum_change > 1.0e-12
        }));
    }
}
