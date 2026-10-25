// pattern: Functional Core
//! Value-only selected full-programme loudness to U5 dialnorm conversion.

use crate::numeric::{FiniteF32, FiniteF32Error, R32Error, r32_sub};

const DIALNORM_SENTINEL_LUFS: f32 = -120.0;
const DIALNORM_SENTINEL_TOLERANCE: f32 = 0.000001;
const DIALNORM_HALF: f32 = 0.5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedDialnorm(u8);

impl SelectedDialnorm {
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedDialnormError {
    FixedConstant(FiniteF32Error),
    Numeric(R32Error),
    OutputRange,
}

impl SelectedDialnormError {
    pub const fn outcome_code(self) -> &'static str {
        "NUMERIC_FAILURE"
    }
}

/// Computes the selected numeric dialnorm value from one finite selected_lufs value.
///
/// This leaf proves only the formula. It does not carry a programme identity or
/// a finalized-measurement receipt into F06ND or F12.
pub fn selected_dialnorm_from_lufs(
    selected_lufs: FiniteF32,
) -> Result<SelectedDialnorm, SelectedDialnormError> {
    let sentinel =
        FiniteF32::new(DIALNORM_SENTINEL_LUFS).map_err(SelectedDialnormError::FixedConstant)?;
    let sentinel_distance =
        r32_sub(selected_lufs, sentinel).map_err(SelectedDialnormError::Numeric)?;
    if sentinel_distance.value().abs() < DIALNORM_SENTINEL_TOLERANCE {
        return Ok(SelectedDialnorm(31));
    }

    let half = FiniteF32::new(DIALNORM_HALF).map_err(SelectedDialnormError::FixedConstant)?;
    let adjusted = r32_sub(selected_lufs, half).map_err(SelectedDialnormError::Numeric)?;
    let clamped = adjusted.value().trunc().clamp(-31.0, -1.0);
    let dialnorm =
        u8::try_from((-clamped) as i32).map_err(|_| SelectedDialnormError::OutputRange)?;
    Ok(SelectedDialnorm(dialnorm))
}
