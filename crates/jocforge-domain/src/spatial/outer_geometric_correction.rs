// pattern: Functional Core
//! Stateless F03S.3 F32 geometry system and lower-Cholesky correction.

use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const FLAG_PENALTY_WORD: u32 = 0x3dcc_cccd;
const DISTANCE_PENALTY_WORD: u32 = 0x3c23_d70a;
const ONE_WORD: u32 = 0x3f80_0000;
const TWO_WORD: u32 = 0x4000_0000;
const NEGATIVE_TWO_WORD: u32 = 0xc000_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OuterGeometricCorrectionError {
    InvalidInput,
    NumericFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OuterGeometricCorrectionResult {
    left: Vec<Vec<u32>>,
    system: Vec<Vec<u32>>,
    correction: Vec<u32>,
}

impl OuterGeometricCorrectionResult {
    pub fn left_u32(&self) -> &[Vec<u32>] {
        &self.left
    }

    pub fn system_u32(&self) -> &[Vec<u32>] {
        &self.system
    }

    pub fn correction_u32(&self) -> &[u32] {
        &self.correction
    }
}

/// Builds the selected geometry system and solves its F32 lower-Cholesky projection.
pub fn geometric_correction(
    target_xyz_u32: &[Vec<u32>],
    source_xyz_u32: &[u32],
    scalar_flag: u32,
    index_state: &[u32],
) -> Result<OuterGeometricCorrectionResult, OuterGeometricCorrectionError> {
    let count = target_xyz_u32.len();
    if !(1..=15).contains(&count)
        || source_xyz_u32.len() != 3
        || index_state.len() != count
        || target_xyz_u32.iter().any(|target| {
            target.len() != 3 || target.iter().any(|word| !f32::from_bits(*word).is_finite())
        })
        || source_xyz_u32
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(OuterGeometricCorrectionError::InvalidInput);
    }

    let q = energy32(source_xyz_u32)?;
    let mut projection = vec![0; count];
    let mut distances = vec![0; count];
    for index in 0..count {
        projection[index] = dot32(&target_xyz_u32[index], source_xyz_u32)?;
        let difference = (0..3)
            .map(|axis| subtract(target_xyz_u32[index][axis], source_xyz_u32[axis]))
            .collect::<Result<Vec<_>, _>>()?;
        distances[index] = energy32(&difference)?;
    }

    let mut gram = vec![vec![0; count]; count];
    for row in 0..count {
        for column in 0..count {
            gram[row][column] = dot32(&target_xyz_u32[row], &target_xyz_u32[column])?;
        }
    }

    let mut left = vec![vec![0; count]; count];
    for row in 0..count {
        for column in 0..count {
            let q_plus_one = add(q, ONE_WORD)?;
            let first = add(q_plus_one, gram[column][row])?;
            let doubled_projection = multiply(projection[row], TWO_WORD)?;
            left[row][column] = subtract(first, doubled_projection)?;
        }
    }

    for index in 0..count {
        let mismatch = u32::from(scalar_flag != index_state[index]);
        let mismatch_word = if mismatch == 0 { 0 } else { ONE_WORD };
        let flag_term = multiply(mismatch_word, FLAG_PENALTY_WORD)?;
        let distance_term = multiply(distances[index], DISTANCE_PENALTY_WORD)?;
        let delta = add(flag_term, distance_term)?;
        left[index][index] = add(left[index][index], delta)?;
    }

    let mut system = vec![vec![0; count]; count];
    for row in 0..count {
        for column in 0..count {
            system[row][column] = add(left[row][column], left[column][row])?;
        }
    }

    let lower = cholesky_lower(&system)?;
    let forward = forward_substitute(&lower)?;
    let solution = backward_substitute(&lower, &forward)?;
    let correction_u32 = solution
        .iter()
        .map(|value| {
            let correction = -*value;
            if correction > 0.0 {
                finite_word(correction)
            } else {
                Ok(0)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(OuterGeometricCorrectionResult {
        left,
        system,
        correction: correction_u32,
    })
}

fn dot32(left: &[u32], right: &[u32]) -> Result<u32, OuterGeometricCorrectionError> {
    if left.len() != right.len() {
        return Err(OuterGeometricCorrectionError::InvalidInput);
    }
    let mut total = 0;
    for (left, right) in left.iter().zip(right) {
        total = add(total, multiply(*left, *right)?)?;
    }
    Ok(total)
}

fn energy32(values: &[u32]) -> Result<u32, OuterGeometricCorrectionError> {
    dot32(values, values)
}

fn cholesky_lower(system: &[Vec<u32>]) -> Result<Vec<Vec<u32>>, OuterGeometricCorrectionError> {
    let count = system.len();
    let mut lower = vec![vec![0; count]; count];
    for row in 0..count {
        for column in 0..row {
            let mut sum = 0;
            for (left_value, right_value) in lower[row]
                .iter()
                .take(column)
                .zip(lower[column].iter().take(column))
            {
                sum = add(sum, multiply(*left_value, *right_value)?)?;
            }
            let numerator = subtract(system[row][column], sum)?;
            if lower[column][column] == 0 {
                return Err(OuterGeometricCorrectionError::NumericFailure);
            }
            lower[row][column] = divide(numerator, lower[column][column])?;
        }
        let mut row_square_sum = 0;
        for value in lower[row].iter().take(row) {
            row_square_sum = add(row_square_sum, multiply(*value, *value)?)?;
        }
        let remainder = subtract(system[row][row], row_square_sum)?;
        lower[row][row] = if f32::from_bits(remainder) > 0.0 {
            sqrt32(remainder)?
        } else {
            0
        };
    }
    Ok(lower)
}

fn forward_substitute(lower: &[Vec<u32>]) -> Result<Vec<u32>, OuterGeometricCorrectionError> {
    let count = lower.len();
    let mut forward = vec![0; count];
    for row in 0..count {
        let mut value = NEGATIVE_TWO_WORD;
        for column in 0..row {
            value = subtract(value, multiply(lower[row][column], forward[column])?)?;
        }
        if lower[row][row] == 0 {
            return Err(OuterGeometricCorrectionError::NumericFailure);
        }
        forward[row] = divide(value, lower[row][row])?;
    }
    Ok(forward)
}

fn backward_substitute(
    lower: &[Vec<u32>],
    forward: &[u32],
) -> Result<Vec<f32>, OuterGeometricCorrectionError> {
    let count = lower.len();
    let mut solution: Vec<f32> = vec![0.0; count];
    for row in (0..count).rev() {
        let mut value = f32::from_bits(forward[row]);
        for column in row + 1..count {
            let product = multiply(lower[column][row], solution[column].to_bits())?;
            value = f32::from_bits(subtract(value.to_bits(), product)?);
        }
        if lower[row][row] == 0 {
            return Err(OuterGeometricCorrectionError::NumericFailure);
        }
        solution[row] = f32::from_bits(divide(value.to_bits(), lower[row][row])?);
    }
    Ok(solution)
}

fn finite_word(value: f32) -> Result<u32, OuterGeometricCorrectionError> {
    FiniteF32::new(value)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterGeometricCorrectionError::NumericFailure)
}

fn multiply(left: u32, right: u32) -> Result<u32, OuterGeometricCorrectionError> {
    let left = FiniteF32::new(f32::from_bits(left))
        .map_err(|_| OuterGeometricCorrectionError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right))
        .map_err(|_| OuterGeometricCorrectionError::NumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterGeometricCorrectionError::NumericFailure)
}

fn add(left: u32, right: u32) -> Result<u32, OuterGeometricCorrectionError> {
    let left = FiniteF32::new(f32::from_bits(left))
        .map_err(|_| OuterGeometricCorrectionError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right))
        .map_err(|_| OuterGeometricCorrectionError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterGeometricCorrectionError::NumericFailure)
}

fn subtract(left: u32, right: u32) -> Result<u32, OuterGeometricCorrectionError> {
    let left = FiniteF32::new(f32::from_bits(left))
        .map_err(|_| OuterGeometricCorrectionError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right))
        .map_err(|_| OuterGeometricCorrectionError::NumericFailure)?;
    r32_sub(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| OuterGeometricCorrectionError::NumericFailure)
}

fn divide(numerator: u32, denominator: u32) -> Result<u32, OuterGeometricCorrectionError> {
    let numerator = f32::from_bits(numerator);
    let denominator = f32::from_bits(denominator);
    if !numerator.is_finite() || !denominator.is_finite() || denominator == 0.0 {
        return Err(OuterGeometricCorrectionError::NumericFailure);
    }
    finite_word(numerator / denominator)
}

fn sqrt32(value: u32) -> Result<u32, OuterGeometricCorrectionError> {
    let value = f32::from_bits(value);
    if !value.is_finite() || value < 0.0 {
        return Err(OuterGeometricCorrectionError::NumericFailure);
    }
    finite_word(value.sqrt())
}
