// pattern: Functional Core
//! F03P.2-only Q15 coordinate quantization.

use super::{FiniteF32, r32_add, r32_mul, trunc_f32_to_i32};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Q15CoordinateError {
    NonFiniteInput,
    NonFiniteIntermediate,
    IntegerRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Q15Coordinate(i32);

impl Q15Coordinate {
    pub const fn value(self) -> i32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalizedQ15Coordinate(f32);

impl NormalizedQ15Coordinate {
    pub const fn value(self) -> f32 {
        self.0
    }

    pub const fn to_bits(self) -> u32 {
        self.0.to_bits()
    }
}

/// Converts the typed Q15 integer to its normalized binary32 coordinate.
///
/// ```compile_fail
/// use jocforge_domain::numeric::normalize_q15_coordinate;
/// normalize_q15_coordinate(0_i32);
/// ```
pub fn normalize_q15_coordinate(coordinate: Q15Coordinate) -> NormalizedQ15Coordinate {
    let converted = coordinate.0 as f32;
    let normalized = converted / 32_768.0_f32;
    NormalizedQ15Coordinate(normalized)
}

pub fn quantize_q15_coordinate(input: f32) -> Result<Q15Coordinate, Q15CoordinateError> {
    let input = FiniteF32::new(input).map_err(|_| Q15CoordinateError::NonFiniteInput)?;

    // Separate statements preserve the two specified binary32 rounding points.
    let scale =
        FiniteF32::new(32_768.0_f32).map_err(|_| Q15CoordinateError::NonFiniteIntermediate)?;
    let scaled = r32_mul(input, scale).map_err(|_| Q15CoordinateError::NonFiniteIntermediate)?;
    let half = if scaled.value().is_sign_negative() {
        -0.5_f32
    } else {
        0.5_f32
    };
    let half = FiniteF32::new(half).map_err(|_| Q15CoordinateError::NonFiniteIntermediate)?;
    let biased = r32_add(scaled, half).map_err(|_| Q15CoordinateError::NonFiniteIntermediate)?;
    if biased.value().trunc() > 32_767.0_f32 {
        return Ok(Q15Coordinate(32_767));
    }
    let truncated = trunc_f32_to_i32(biased).map_err(|_| Q15CoordinateError::IntegerRange)?;
    Ok(Q15Coordinate(truncated.value()))
}
