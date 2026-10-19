// pattern: Functional Core
//! Exact-rational enclosure for the selected mathematical R64-to-R32 panner contract.

use super::ReaderTargetError;
use num_bigint::{BigInt, BigUint, Sign};
use std::cmp::Ordering;

const SIGN_MASK: u32 = 0x8000_0000;
const EXPONENT_MASK: u32 = 0x7f80_0000;
const MAXIMUM_ANGLE_BITS: u32 = 0x3f49_0fdb;

/// Rounded panner words and the actual number of enclosure steps for each series.
#[allow(dead_code)] // Series step counts are consumed by the KAT tests, not runtime callers.
pub(super) struct MathematicalR64R32 {
    pub(super) cosine: f32,
    pub(super) sine: f32,
    pub(super) cosine_series_steps: u64,
    pub(super) sine_series_steps: u64,
}

/// Applies the approved exact-rational alternating enclosure, followed by R64 then R32 rounding.
pub(super) fn mathematical_r64_r32(angle: f32) -> Result<MathematicalR64R32, ReaderTargetError> {
    let bits = angle.to_bits();
    let magnitude = bits & !SIGN_MASK;
    if magnitude == 0 {
        return Ok(MathematicalR64R32 {
            cosine: 1.0,
            sine: f32::from_bits(bits & SIGN_MASK),
            cosine_series_steps: 0,
            sine_series_steps: 0,
        });
    }
    if bits & EXPONENT_MASK == EXPONENT_MASK {
        return Err(ReaderTargetError::NumericFailure);
    }
    if bits & SIGN_MASK != 0 || magnitude > MAXIMUM_ANGLE_BITS {
        return Err(ReaderTargetError::ProfileUnsupported);
    }

    let exact_angle = ExactRational::from_positive_binary32_bits(bits)?;
    let (cosine, cosine_series_steps) =
        rounded_alternating_series(&exact_angle, SeriesKind::Cosine)?;
    let (sine, sine_series_steps) = rounded_alternating_series(&exact_angle, SeriesKind::Sine)?;
    Ok(MathematicalR64R32 {
        cosine,
        sine,
        cosine_series_steps,
        sine_series_steps,
    })
}

#[derive(Clone, Copy)]
enum SeriesKind {
    Cosine,
    Sine,
}

#[derive(Clone)]
struct ExactRational {
    numerator: BigInt,
    denominator: BigUint,
}

impl ExactRational {
    fn one() -> Self {
        Self {
            numerator: BigInt::from(1_u8),
            denominator: BigUint::from(1_u8),
        }
    }

    fn from_positive_binary32_bits(bits: u32) -> Result<Self, ReaderTargetError> {
        let exponent = (bits >> 23) & 0xff;
        let fraction = bits & 0x007f_ffff;
        let (significand, binary_exponent) = if exponent == 0 {
            (fraction, -149_i32)
        } else {
            (
                (1_u32 << 23) | fraction,
                i32::try_from(exponent).unwrap() - 150,
            )
        };
        if significand == 0 {
            return Ok(Self {
                numerator: BigInt::from(0_u8),
                denominator: BigUint::from(1_u8),
            });
        }

        let mut numerator = BigInt::from(significand);
        let denominator = if binary_exponent >= 0 {
            numerator <<=
                usize::try_from(binary_exponent).map_err(|_| ReaderTargetError::NumericFailure)?;
            BigUint::from(1_u8)
        } else {
            BigUint::from(1_u8)
                << usize::try_from(-binary_exponent)
                    .map_err(|_| ReaderTargetError::NumericFailure)?
        };
        Ok(Self {
            numerator,
            denominator,
        })
    }

    fn multiply(&self, other: &Self) -> Self {
        Self {
            numerator: &self.numerator * &other.numerator,
            denominator: &self.denominator * &other.denominator,
        }
    }

    fn divide_by_positive_integer(&self, divisor: u64) -> Self {
        Self {
            numerator: self.numerator.clone(),
            denominator: &self.denominator * BigUint::from(divisor),
        }
    }

    fn add(&self, other: &Self) -> Self {
        let left_scale = BigInt::from(other.denominator.clone());
        let right_scale = BigInt::from(self.denominator.clone());
        Self {
            numerator: &self.numerator * left_scale + &other.numerator * right_scale,
            denominator: &self.denominator * &other.denominator,
        }
    }

    fn negated(&self) -> Self {
        Self {
            numerator: -&self.numerator,
            denominator: self.denominator.clone(),
        }
    }
}

fn rounded_alternating_series(
    x: &ExactRational,
    kind: SeriesKind,
) -> Result<(f32, u64), ReaderTargetError> {
    let x_squared = x.multiply(x);
    let mut term = match kind {
        SeriesKind::Cosine => ExactRational::one(),
        SeriesKind::Sine => x.clone(),
    };
    let mut total = term.clone();
    let mut index = 0_u64;
    let mut steps = 0_u64;

    loop {
        let (first_factor, second_factor) = match kind {
            SeriesKind::Sine => (
                index
                    .checked_mul(2)
                    .and_then(|value| value.checked_add(2))
                    .ok_or(ReaderTargetError::NumericFailure)?,
                index
                    .checked_mul(2)
                    .and_then(|value| value.checked_add(3))
                    .ok_or(ReaderTargetError::NumericFailure)?,
            ),
            SeriesKind::Cosine => (
                index
                    .checked_mul(2)
                    .and_then(|value| value.checked_add(1))
                    .ok_or(ReaderTargetError::NumericFailure)?,
                index
                    .checked_mul(2)
                    .and_then(|value| value.checked_add(2))
                    .ok_or(ReaderTargetError::NumericFailure)?,
            ),
        };
        let divisor = first_factor
            .checked_mul(second_factor)
            .ok_or(ReaderTargetError::NumericFailure)?;
        let next_term = term
            .multiply(&x_squared)
            .divide_by_positive_integer(divisor)
            .negated();
        let next_total = total.add(&next_term);
        steps = steps
            .checked_add(1)
            .ok_or(ReaderTargetError::NumericFailure)?;

        let total_r64 = round_positive_rational_to_f64_bits(&total)?;
        let next_r64 = round_positive_rational_to_f64_bits(&next_total)?;
        if total_r64 == next_r64 {
            return Ok((f64::from_bits(total_r64) as f32, steps));
        }

        term = next_term;
        total = next_total;
        index = index
            .checked_add(1)
            .ok_or(ReaderTargetError::NumericFailure)?;
    }
}

fn round_positive_rational_to_f64_bits(value: &ExactRational) -> Result<u64, ReaderTargetError> {
    let numerator = match value.numerator.sign() {
        Sign::NoSign => return Ok(0),
        Sign::Plus => value.numerator.magnitude(),
        Sign::Minus => return Err(ReaderTargetError::NumericFailure),
    };
    let denominator = &value.denominator;
    let mut exponent = floor_binary_exponent(numerator, denominator)?;

    if exponent < -1022 {
        let significand = round_quotient_to_even(numerator, denominator, 1074)?;
        let significand = small_unsigned(&significand)?;
        if significand > (1_u64 << 52) {
            return Err(ReaderTargetError::NumericFailure);
        }
        return Ok(significand);
    }
    if exponent > 1023 {
        return Err(ReaderTargetError::NumericFailure);
    }

    let significand = round_quotient_to_even(numerator, denominator, 52 - exponent)?;
    let mut significand = small_unsigned(&significand)?;
    if significand == (1_u64 << 53) {
        exponent += 1;
        significand = 1_u64 << 52;
    }
    if exponent > 1023 || significand < (1_u64 << 52) {
        return Err(ReaderTargetError::NumericFailure);
    }

    let exponent_field =
        u64::try_from(exponent + 1023).map_err(|_| ReaderTargetError::NumericFailure)?;
    Ok((exponent_field << 52) | (significand - (1_u64 << 52)))
}

fn floor_binary_exponent(
    numerator: &BigUint,
    denominator: &BigUint,
) -> Result<i64, ReaderTargetError> {
    let mut exponent = i64::try_from(numerator.bits())
        .and_then(|numerator_bits| {
            i64::try_from(denominator.bits())
                .map(|denominator_bits| numerator_bits - denominator_bits)
        })
        .map_err(|_| ReaderTargetError::NumericFailure)?;
    let ratio_is_below_candidate = if exponent >= 0 {
        numerator < &(denominator << usize::try_from(exponent).unwrap())
    } else {
        &(numerator << usize::try_from(-exponent).unwrap()) < denominator
    };
    if ratio_is_below_candidate {
        exponent -= 1;
    }
    Ok(exponent)
}

fn round_quotient_to_even(
    numerator: &BigUint,
    denominator: &BigUint,
    binary_shift: i64,
) -> Result<BigUint, ReaderTargetError> {
    let (scaled_numerator, scaled_denominator) = if binary_shift >= 0 {
        (
            numerator
                << usize::try_from(binary_shift).map_err(|_| ReaderTargetError::NumericFailure)?,
            denominator.clone(),
        )
    } else {
        (
            numerator.clone(),
            denominator
                << usize::try_from(-binary_shift).map_err(|_| ReaderTargetError::NumericFailure)?,
        )
    };
    let quotient = &scaled_numerator / &scaled_denominator;
    let remainder = &scaled_numerator % &scaled_denominator;
    match (&remainder << 1_usize).cmp(&scaled_denominator) {
        Ordering::Greater => Ok(quotient + BigUint::from(1_u8)),
        Ordering::Equal
            if quotient
                .to_u64_digits()
                .first()
                .is_some_and(|word| word & 1 == 1) =>
        {
            Ok(quotient + BigUint::from(1_u8))
        }
        Ordering::Less | Ordering::Equal => Ok(quotient),
    }
}

fn small_unsigned(value: &BigUint) -> Result<u64, ReaderTargetError> {
    match value.to_u64_digits().as_slice() {
        [] => Ok(0),
        [word] => Ok(*word),
        _ => Err(ReaderTargetError::NumericFailure),
    }
}

#[cfg(test)]
mod tests {
    use super::mathematical_r64_r32;
    use serde_json::Value;

    const TRIGONOMETRY_KAT: &str =
        include_str!("../../../../tests/fixtures/reader_target/panner-trigonometry-kat.json");

    #[test]
    fn direct_angles_match_the_rational_enclosure_kat() {
        let vectors: Value = serde_json::from_str(TRIGONOMETRY_KAT).unwrap();
        for case in vectors["direct_angles"].as_array().unwrap() {
            let angle_bits =
                u32::from_str_radix(case["angle_f32_hex"].as_str().unwrap(), 16).unwrap();
            let result = mathematical_r64_r32(f32::from_bits(angle_bits)).unwrap();
            let expected_cosine =
                u32::from_str_radix(case["cosine_f32_hex"].as_str().unwrap(), 16).unwrap();
            let expected_sine =
                u32::from_str_radix(case["sine_f32_hex"].as_str().unwrap(), 16).unwrap();
            assert_eq!(result.cosine.to_bits(), expected_cosine, "{case}");
            assert_eq!(result.sine.to_bits(), expected_sine, "{case}");
        }
    }
}
