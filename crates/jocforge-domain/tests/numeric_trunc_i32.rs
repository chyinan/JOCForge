// pattern: Functional Core
use jocforge_domain::numeric::{FiniteF32, TruncI32Error, trunc_f32_to_i32};

fn finite(value: f32) -> FiniteF32 {
    FiniteF32::new(value).unwrap()
}

#[test]
fn truncates_fractional_values_toward_zero() {
    assert_eq!(trunc_f32_to_i32(finite(1.9)).unwrap().value(), 1);
    assert_eq!(trunc_f32_to_i32(finite(-1.9)).unwrap().value(), -1);
    assert_eq!(
        trunc_f32_to_i32(finite(32_767.999)).unwrap().value(),
        32_767
    );
}

#[test]
fn signed_zero_and_subnormal_values_become_integer_zero() {
    for bits in [0x0000_0000, 0x8000_0000, 0x0000_0001, 0x8000_0001] {
        assert_eq!(
            trunc_f32_to_i32(finite(f32::from_bits(bits)))
                .unwrap()
                .value(),
            0
        );
    }
}

#[test]
fn exact_i32_boundary_is_accepted_and_adjacent_out_of_range_values_reject() {
    let max_in_range = f32::from_bits(0x4eff_ffff);
    assert_eq!(
        trunc_f32_to_i32(finite(max_in_range)).unwrap().value(),
        2_147_483_520
    );
    assert_eq!(
        trunc_f32_to_i32(finite(-2_147_483_648.0)).unwrap().value(),
        i32::MIN
    );
    assert_eq!(
        trunc_f32_to_i32(finite(2_147_483_648.0)),
        Err(TruncI32Error::OutOfRange)
    );
    assert_eq!(
        trunc_f32_to_i32(finite(f32::from_bits(0xcf00_0001))),
        Err(TruncI32Error::OutOfRange)
    );
}
