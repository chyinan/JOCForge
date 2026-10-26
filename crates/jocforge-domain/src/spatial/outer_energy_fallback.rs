// pattern: Functional Core
//! Stateless F03S.2 energy-branch diagnostics and fallback-only coefficient path.

use super::outer_postprocess::{OuterPostprocessError, postprocess_coefficients};
use crate::numeric::{FiniteF32, r32_add, r32_mul};

const ENERGY_RATIO_U32: u32 = 0x3f21_866c;
const EPSILON_U32: u32 = 0x3400_0000;
pub(crate) const FLOOR_U32: u32 = 0x3586_37bd;
pub(crate) const DOMINANCE_RATIO_U32: u32 = 0x3f7f_fe09;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OuterEnergyFallbackError {
    InvalidInput,
    NumericFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OuterEnergyFallbackResult {
    response_energy_u32: u32,
    context_energy_u32: u32,
    threshold_u32: u32,
    scale_u32: u32,
    active: bool,
    final_u32: Vec<u32>,
}

impl OuterEnergyFallbackResult {
    pub const fn response_energy_u32(&self) -> u32 {
        self.response_energy_u32
    }

    pub const fn context_energy_u32(&self) -> u32 {
        self.context_energy_u32
    }

    pub const fn threshold_u32(&self) -> u32 {
        self.threshold_u32
    }

    pub const fn scale_u32(&self) -> u32 {
        self.scale_u32
    }

    pub const fn active(&self) -> bool {
        self.active
    }

    pub fn final_u32(&self) -> &[u32] {
        &self.final_u32
    }
}

/// Computes the F03S.2 branch/fallback quantities for one explicit cold local input.
///
/// An active result returns the seed unchanged as a diagnostic boundary only. The
/// active correction route is not implemented by this function.
pub fn evaluate_outer_energy_fallback(
    context_u32: &[Vec<u32>],
    response_u32: &[u32],
    seed_u32: &[u32],
) -> Result<OuterEnergyFallbackResult, OuterEnergyFallbackError> {
    let variable_count = seed_u32.len();
    if !(1..=15).contains(&variable_count)
        || context_u32.len() != 15
        || context_u32.iter().any(|row| row.len() != variable_count)
        || response_u32.len() != 15
        || context_u32
            .iter()
            .flatten()
            .chain(response_u32)
            .chain(seed_u32)
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(OuterEnergyFallbackError::InvalidInput);
    }

    let mut projected = Vec::with_capacity(15);
    for row in context_u32 {
        let mut total = finite(0.0)?;
        for (coefficient, seed) in row.iter().zip(seed_u32) {
            let product = multiply(*coefficient, *seed)?;
            total = add(total, product)?;
        }
        projected.push(total);
    }
    let context_energy_u32 = energy32(&projected)?;
    let response_energy_u32 = energy32(response_u32)?;
    let threshold_u32 = multiply(response_energy_u32, ENERGY_RATIO_U32)?;
    let active = f32::from_bits(context_energy_u32) <= f32::from_bits(threshold_u32);

    let numerator = add(response_energy_u32, EPSILON_U32)?;
    let denominator = add(context_energy_u32, EPSILON_U32)?;
    let ratio = divide(numerator, denominator)?;
    if f32::from_bits(ratio) < 0.0 {
        return Err(OuterEnergyFallbackError::NumericFailure);
    }
    let scale_value = f32::from_bits(ratio).sqrt();
    if !scale_value.is_finite() {
        return Err(OuterEnergyFallbackError::NumericFailure);
    }
    let scale_u32 = scale_value.to_bits();

    let final_u32 = if active {
        seed_u32.to_vec()
    } else {
        let scaled_seed = seed_u32
            .iter()
            .map(|word| multiply(*word, scale_u32))
            .collect::<Result<Vec<_>, _>>()?;
        postprocess_coefficients(&scaled_seed, FLOOR_U32, DOMINANCE_RATIO_U32)
            .map_err(map_postprocess_error)?
            .final_u32()
            .to_vec()
    };

    Ok(OuterEnergyFallbackResult {
        response_energy_u32,
        context_energy_u32,
        threshold_u32,
        scale_u32,
        active,
        final_u32,
    })
}

fn energy32(values: &[u32]) -> Result<u32, OuterEnergyFallbackError> {
    let mut total = finite(0.0)?;
    for value in values {
        total = add(total, multiply(*value, *value)?)?;
    }
    Ok(total)
}

fn finite(value: f32) -> Result<u32, OuterEnergyFallbackError> {
    FiniteF32::new(value)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterEnergyFallbackError::NumericFailure)
}

fn multiply(left: u32, right: u32) -> Result<u32, OuterEnergyFallbackError> {
    let left = FiniteF32::new(f32::from_bits(left))
        .map_err(|_| OuterEnergyFallbackError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right))
        .map_err(|_| OuterEnergyFallbackError::NumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterEnergyFallbackError::NumericFailure)
}

fn add(left: u32, right: u32) -> Result<u32, OuterEnergyFallbackError> {
    let left = FiniteF32::new(f32::from_bits(left))
        .map_err(|_| OuterEnergyFallbackError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right))
        .map_err(|_| OuterEnergyFallbackError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterEnergyFallbackError::NumericFailure)
}

fn divide(numerator: u32, denominator: u32) -> Result<u32, OuterEnergyFallbackError> {
    let numerator = f32::from_bits(numerator);
    let denominator = f32::from_bits(denominator);
    if !numerator.is_finite() || !denominator.is_finite() || denominator == 0.0 {
        return Err(OuterEnergyFallbackError::NumericFailure);
    }
    finite(numerator / denominator)
}

fn map_postprocess_error(error: OuterPostprocessError) -> OuterEnergyFallbackError {
    match error {
        OuterPostprocessError::InvalidInput => OuterEnergyFallbackError::InvalidInput,
        OuterPostprocessError::NumericFailure => OuterEnergyFallbackError::NumericFailure,
    }
}
