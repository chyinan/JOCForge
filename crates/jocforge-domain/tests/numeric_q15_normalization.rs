// pattern: Functional Core
use jocforge_domain::numeric::{normalize_q15_coordinate, quantize_q15_coordinate};

#[test]
fn zero_sources_normalize_to_positive_zero() {
    for input in [0.0_f32, -0.0_f32] {
        let q = quantize_q15_coordinate(input).unwrap();
        assert_eq!(normalize_q15_coordinate(q).to_bits(), 0x0000_0000);
    }
}

#[test]
fn selected_positive_and_negative_boundaries_have_exact_bits() {
    let cases = [
        (1.0_f32, 0x3f7f_fe00),
        (-1.0_f32, 0xbf80_0000),
        (-1.25_f32, 0xbfa0_0000),
        (-65_536.0_f32, 0xc780_0000),
    ];
    for (input, expected_bits) in cases {
        let q = quantize_q15_coordinate(input).unwrap();
        assert_eq!(normalize_q15_coordinate(q).to_bits(), expected_bits);
    }
}

#[test]
fn reachable_large_integer_grid_round_trips_without_early_narrowing() {
    let q = quantize_q15_coordinate(-512.00006_f32).unwrap();
    let normalized = normalize_q15_coordinate(q);
    assert_eq!(normalized.to_bits(), (-512.00006_f32).to_bits());
}
