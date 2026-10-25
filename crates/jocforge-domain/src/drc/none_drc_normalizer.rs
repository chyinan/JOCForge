// pattern: Functional Core
//! Selected F06ND scalar normalization polynomial.

use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const NORMALIZE_A: f32 = f32::from_bits(0x3CAA_5BD7);
const NORMALIZE_B: f32 = f32::from_bits(0x3C62_2BCA);
const NORMALIZE_C: f32 = f32::from_bits(0x3BE6_35DE);
const FOUR: f32 = 4.0;
const ONE_OVER_48: f32 = f32::from_bits(0x3CAA_AAAB);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoneDrcNormalizationError {
    NonFiniteInput,
    NegativeInput,
    NumericNonFinite,
}

impl NoneDrcNormalizationError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::NonFiniteInput => "NONE_DRC_NORMALIZER_INPUT_NONFINITE",
            Self::NegativeInput => "NONE_DRC_NORMALIZER_INPUT_NEGATIVE",
            Self::NumericNonFinite => "NONE_DRC_NORMALIZER_NUMERIC_NONFINITE",
        }
    }
}

impl std::fmt::Display for NoneDrcNormalizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for NoneDrcNormalizationError {}

/// Normalizes one nonnegative finite F32 using the selected F06ND polynomial.
///
/// The exponent-derived scaling and polynomial operations follow F06ND.3 in
/// binary32 order. The selected DRC_NORMALIZE3 words are A=0x3CAA5BD7,
/// B=0x3C622BCA, and C=0x3BE635DE.
pub fn normalize_none_drc_metric(value: f32) -> Result<f32, NoneDrcNormalizationError> {
    FiniteF32::new(value).map_err(|_| NoneDrcNormalizationError::NonFiniteInput)?;
    if value < 0.0 {
        return Err(NoneDrcNormalizationError::NegativeInput);
    }

    let doubled_word = value.to_bits().wrapping_mul(2);
    let exponent = if doubled_word == 0 {
        -127
    } else {
        i32::from((doubled_word >> 24).to_le_bytes()[0]) - 126
    };
    let scale_shift = 60_i32.min(0.max(-exponent));
    if value == 0.0 || scale_shift >= 48 {
        return Ok(1.0);
    }

    let scale_exponent =
        u8::try_from(scale_shift + 127).map_err(|_| NoneDrcNormalizationError::NumericNonFinite)?;
    let scale = f32::from_bits(u32::from(scale_exponent) << 23);
    let scaled_value = checked_r32_mul(value, scale)?;
    let linear_product = checked_r32_mul(scaled_value, NORMALIZE_A)?;
    let linear_term = checked_r32_sub(NORMALIZE_B, linear_product)?;
    let square = checked_r32_mul(scaled_value, scaled_value)?;
    let quadratic_term = checked_r32_mul(square, NORMALIZE_C)?;
    let polynomial = checked_r32_add(linear_term, quadratic_term)?;
    let scaled_polynomial = checked_r32_mul(polynomial, FOUR)?;
    let shift_value =
        u8::try_from(scale_shift).map_err(|_| NoneDrcNormalizationError::NumericNonFinite)?;
    let exponent_correction = checked_r32_mul(f32::from(shift_value), ONE_OVER_48)?;
    checked_r32_add(scaled_polynomial, exponent_correction)
}

fn checked_r32_mul(left: f32, right: f32) -> Result<f32, NoneDrcNormalizationError> {
    let left = FiniteF32::new(left).map_err(|_| NoneDrcNormalizationError::NumericNonFinite)?;
    let right = FiniteF32::new(right).map_err(|_| NoneDrcNormalizationError::NumericNonFinite)?;
    r32_mul(left, right)
        .map(FiniteF32::value)
        .map_err(|_| NoneDrcNormalizationError::NumericNonFinite)
}

fn checked_r32_add(left: f32, right: f32) -> Result<f32, NoneDrcNormalizationError> {
    let left = FiniteF32::new(left).map_err(|_| NoneDrcNormalizationError::NumericNonFinite)?;
    let right = FiniteF32::new(right).map_err(|_| NoneDrcNormalizationError::NumericNonFinite)?;
    r32_add(left, right)
        .map(FiniteF32::value)
        .map_err(|_| NoneDrcNormalizationError::NumericNonFinite)
}

fn checked_r32_sub(left: f32, right: f32) -> Result<f32, NoneDrcNormalizationError> {
    let left = FiniteF32::new(left).map_err(|_| NoneDrcNormalizationError::NumericNonFinite)?;
    let right = FiniteF32::new(right).map_err(|_| NoneDrcNormalizationError::NumericNonFinite)?;
    r32_sub(left, right)
        .map(FiniteF32::value)
        .map_err(|_| NoneDrcNormalizationError::NumericNonFinite)
}
