// pattern: Functional Core
//! Stateless F03S.4 quadratic-energy update with caller-supplied correction.

use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const INITIAL_SCALE_WORD: u32 = 0x3fa1_2478;
const EPSILON_WORD: u32 = 0x3400_0000;
const FOUR_WORD: u32 = 0x4080_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OuterActiveUpdateError {
    InvalidInput,
    NegativeDiscriminant,
    NumericFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OuterActiveUpdateResult {
    discriminant: u32,
    step: u32,
    scaled_seed: Vec<u32>,
    output: Vec<u32>,
}

impl OuterActiveUpdateResult {
    pub const fn discriminant_u32(&self) -> u32 {
        self.discriminant
    }

    pub const fn step_u32(&self) -> u32 {
        self.step
    }

    pub fn scaled_seed_u32(&self) -> &[u32] {
        &self.scaled_seed
    }

    pub fn output_u32(&self) -> &[u32] {
        &self.output
    }
}

/// Computes one F03S.4 active update from explicit context, response, seed and correction arrays.
pub fn active_energy_update(
    context_u32: &[Vec<u32>],
    response_u32: &[u32],
    seed_u32: &[u32],
    correction_u32: &[u32],
) -> Result<OuterActiveUpdateResult, OuterActiveUpdateError> {
    let variable_count = seed_u32.len();
    if !(1..=15).contains(&variable_count)
        || context_u32.len() != 15
        || context_u32.iter().any(|row| {
            row.len() != variable_count || row.iter().any(|word| !f32::from_bits(*word).is_finite())
        })
        || response_u32.len() != 15
        || correction_u32.len() != variable_count
        || response_u32
            .iter()
            .chain(seed_u32)
            .chain(correction_u32)
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(OuterActiveUpdateError::InvalidInput);
    }

    let y = matvec32(context_u32, seed_u32)?;
    let scaled_seed_u32 = seed_u32
        .iter()
        .map(|word| multiply(*word, INITIAL_SCALE_WORD))
        .collect::<Result<Vec<_>, _>>()?;
    let scaled_y = y
        .iter()
        .map(|word| multiply(*word, INITIAL_SCALE_WORD))
        .collect::<Result<Vec<_>, _>>()?;
    let v = matvec32(context_u32, correction_u32)?;

    let cross = dot32(&v, &scaled_y)?;
    let c = add(cross, cross)?;
    let energy_v = dot32(&v, &v)?;
    let energy_scaled_y = dot32(&scaled_y, &scaled_y)?;
    let energy_response = dot32(response_u32, response_u32)?;

    let c_squared = multiply(c, c)?;
    let energy_difference = subtract(energy_scaled_y, energy_response)?;
    let four_energy_v = multiply(energy_v, FOUR_WORD)?;
    let penalty = multiply(energy_difference, four_energy_v)?;
    let discriminant_u32 = subtract(c_squared, penalty)?;
    if f32::from_bits(discriminant_u32) < 0.0 {
        return Err(OuterActiveUpdateError::NegativeDiscriminant);
    }

    let root = sqrt32(discriminant_u32)?;
    let numerator = subtract(root, c)?;
    let twice_energy_v = add(energy_v, energy_v)?;
    let denominator = add(twice_energy_v, EPSILON_WORD)?;
    let step_u32 = divide(numerator, denominator)?;
    let output_u32 = correction_u32
        .iter()
        .zip(&scaled_seed_u32)
        .map(|(correction, seed)| add(multiply(step_u32, *correction)?, *seed))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(OuterActiveUpdateResult {
        discriminant: discriminant_u32,
        step: step_u32,
        scaled_seed: scaled_seed_u32,
        output: output_u32,
    })
}

fn matvec32(matrix: &[Vec<u32>], vector: &[u32]) -> Result<Vec<u32>, OuterActiveUpdateError> {
    matrix.iter().map(|row| dot32(row, vector)).collect()
}

fn dot32(left: &[u32], right: &[u32]) -> Result<u32, OuterActiveUpdateError> {
    if left.len() != right.len() {
        return Err(OuterActiveUpdateError::InvalidInput);
    }
    let mut total = 0;
    for (left, right) in left.iter().zip(right) {
        total = add(total, multiply(*left, *right)?)?;
    }
    Ok(total)
}

fn finite(value: f32) -> Result<u32, OuterActiveUpdateError> {
    FiniteF32::new(value)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterActiveUpdateError::NumericFailure)
}

fn multiply(left: u32, right: u32) -> Result<u32, OuterActiveUpdateError> {
    let left =
        FiniteF32::new(f32::from_bits(left)).map_err(|_| OuterActiveUpdateError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right))
        .map_err(|_| OuterActiveUpdateError::NumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterActiveUpdateError::NumericFailure)
}

fn add(left: u32, right: u32) -> Result<u32, OuterActiveUpdateError> {
    let left =
        FiniteF32::new(f32::from_bits(left)).map_err(|_| OuterActiveUpdateError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right))
        .map_err(|_| OuterActiveUpdateError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterActiveUpdateError::NumericFailure)
}

fn subtract(left: u32, right: u32) -> Result<u32, OuterActiveUpdateError> {
    let left =
        FiniteF32::new(f32::from_bits(left)).map_err(|_| OuterActiveUpdateError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right))
        .map_err(|_| OuterActiveUpdateError::NumericFailure)?;
    r32_sub(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterActiveUpdateError::NumericFailure)
}

fn divide(numerator: u32, denominator: u32) -> Result<u32, OuterActiveUpdateError> {
    let numerator = f32::from_bits(numerator);
    let denominator = f32::from_bits(denominator);
    if !numerator.is_finite() || !denominator.is_finite() || denominator == 0.0 {
        return Err(OuterActiveUpdateError::NumericFailure);
    }
    finite(numerator / denominator)
}

fn sqrt32(value: u32) -> Result<u32, OuterActiveUpdateError> {
    let value = f32::from_bits(value);
    if !value.is_finite() || value < 0.0 {
        return Err(OuterActiveUpdateError::NumericFailure);
    }
    finite(value.sqrt())
}
