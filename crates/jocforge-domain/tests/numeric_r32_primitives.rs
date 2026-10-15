// pattern: Functional Core
use jocforge_domain::numeric::{FiniteF32, FiniteF32Error, R32Error, r32_add, r32_mul};

fn finite(value: f32) -> FiniteF32 {
    FiniteF32::new(value).unwrap()
}

#[test]
fn finite_wrapper_preserves_signed_zero_and_subnormal_bits() {
    for bits in [0x0000_0000, 0x8000_0000, 0x0000_0001, 0x8000_0001] {
        assert_eq!(
            FiniteF32::new(f32::from_bits(bits)).unwrap().to_bits(),
            bits
        );
    }
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(FiniteF32::new(value), Err(FiniteF32Error::NonFinite));
    }
}

#[test]
fn r32_add_preserves_specified_zero_branches() {
    assert_eq!(
        r32_add(finite(-0.0), finite(-0.0)).unwrap().to_bits(),
        0x8000_0000
    );
    assert_eq!(
        r32_add(finite(0.0), finite(-0.0)).unwrap().to_bits(),
        0x0000_0000
    );
    assert_eq!(
        r32_add(finite(1.0), finite(-1.0)).unwrap().to_bits(),
        0x0000_0000
    );
    assert_eq!(
        r32_add(finite(f32::from_bits(1)), finite(0.0))
            .unwrap()
            .to_bits(),
        1
    );
}

#[test]
fn r32_mul_uses_xor_zero_sign_and_preserves_subnormal() {
    assert_eq!(
        r32_mul(finite(-0.0), finite(2.0)).unwrap().to_bits(),
        0x8000_0000
    );
    assert_eq!(
        r32_mul(finite(-0.0), finite(-2.0)).unwrap().to_bits(),
        0x0000_0000
    );
    assert_eq!(
        r32_mul(finite(f32::from_bits(1)), finite(1.0))
            .unwrap()
            .to_bits(),
        1
    );
}

#[test]
fn multiply_then_add_has_two_rounding_points_and_is_not_fused() {
    let a = finite(f32::from_bits(0x3f80_0001));
    let b = finite(f32::from_bits(0x3f7f_fffe));
    let product = r32_mul(a, b).unwrap();
    let result = r32_add(product, finite(-1.0)).unwrap();
    assert_eq!(product.to_bits(), 1.0_f32.to_bits());
    assert_eq!(result.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn non_finite_results_are_rejected() {
    assert_eq!(
        r32_mul(finite(f32::MAX), finite(2.0)),
        Err(R32Error::NonFiniteResult)
    );
    assert_eq!(
        r32_add(finite(f32::MAX), finite(f32::MAX)),
        Err(R32Error::NonFiniteResult)
    );
}
