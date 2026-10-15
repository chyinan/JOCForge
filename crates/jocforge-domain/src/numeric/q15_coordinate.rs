// pattern: Functional Core
//! F03P.2-only Q15 coordinate quantization.

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

pub fn quantize_q15_coordinate(input: f32) -> Result<Q15Coordinate, Q15CoordinateError> {
    if !input.is_finite() {
        return Err(Q15CoordinateError::NonFiniteInput);
    }

    // Separate statements preserve the two specified binary32 rounding points.
    let scaled = input * 32_768.0_f32;
    if !scaled.is_finite() {
        return Err(Q15CoordinateError::NonFiniteIntermediate);
    }
    let half = if scaled.is_sign_negative() {
        -0.5_f32
    } else {
        0.5_f32
    };
    let biased = scaled + half;
    if !biased.is_finite() {
        return Err(Q15CoordinateError::NonFiniteIntermediate);
    }
    let truncated = biased.trunc();

    if truncated > 32_767.0_f32 {
        return Ok(Q15Coordinate(32_767));
    }
    if f64::from(truncated) < f64::from(i32::MIN) {
        return Err(Q15CoordinateError::IntegerRange);
    }
    Ok(Q15Coordinate(truncated as i32))
}
