// pattern: Functional Core
//! Stateless F03U projection and selector-status refinement admission primitives.

use super::candidate_selector::SelectorStatus;
use crate::numeric::{FiniteF32, r32_add, r32_mul};

const U04_EPSILON_WORD: u32 = 0x257f_f2d6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum U04ProjectionError {
    InvalidInput,
    NumericFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct U04ProjectionResult {
    normalized_m_u32: Vec<Vec<u32>>,
    projected_xyz_u32: Vec<[u32; 3]>,
}

impl U04ProjectionResult {
    pub fn normalized_m_u32(&self) -> &[Vec<u32>] {
        &self.normalized_m_u32
    }

    pub fn projected_xyz_u32(&self) -> &[[u32; 3]] {
        &self.projected_xyz_u32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum U04AdmissionError {
    SelectedCountOutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct U04AdmissionResult {
    refinement_called: bool,
    selected_count: u8,
    initial_positions_preserved: bool,
}

impl U04AdmissionResult {
    pub const fn refinement_called(self) -> bool {
        self.refinement_called
    }

    pub const fn selected_count(self) -> u8 {
        self.selected_count
    }

    pub const fn initial_positions_preserved(self) -> bool {
        self.initial_positions_preserved
    }
}

/// Admits refinement only for the selector's explicit CAPACITY_REACHED status.
pub fn u04_refinement_admission(
    selector_status: SelectorStatus,
    selected_count: u8,
) -> Result<U04AdmissionResult, U04AdmissionError> {
    if selected_count > 15 {
        return Err(U04AdmissionError::SelectedCountOutOfRange);
    }
    let refinement_called = selector_status == SelectorStatus::CapacityReached;
    Ok(U04AdmissionResult {
        refinement_called,
        selected_count,
        initial_positions_preserved: !refinement_called,
    })
}

/// Applies the two ordered reciprocal normalization layers and protected-anchor projection.
pub fn project_targets(
    source_xyz_u32: &[[u32; 3]],
    target_xyz_u32: &[[u32; 3]],
    target_attributes_u32: &[u32],
    source_weight_u32: &[u32],
    gain_u32: &[Vec<u32>],
) -> Result<U04ProjectionResult, U04ProjectionError> {
    let source_count = source_xyz_u32.len();
    let target_count = target_xyz_u32.len();
    if !(1..=48).contains(&source_count)
        || !(1..=15).contains(&target_count)
        || target_attributes_u32.len() != target_count
        || source_weight_u32.len() != source_count
        || gain_u32.len() != source_count
        || gain_u32.iter().any(|row| row.len() != target_count)
        || source_xyz_u32
            .iter()
            .flatten()
            .chain(target_xyz_u32.iter().flatten())
            .chain(source_weight_u32)
            .chain(gain_u32.iter().flatten())
            .any(|word| !f32::from_bits(*word).is_finite())
        || source_weight_u32
            .iter()
            .chain(gain_u32.iter().flatten())
            .any(|word| f32::from_bits(*word) < 0.0)
    {
        return Err(U04ProjectionError::InvalidInput);
    }

    let mut membership = vec![vec![0; source_count]; target_count];
    for source in 0..source_count {
        let mut gain_sum = 0;
        for gain in &gain_u32[source] {
            gain_sum = add(gain_sum, *gain)?;
        }
        let source_scale = reciprocal(add(gain_sum, U04_EPSILON_WORD)?)?;
        for target in 0..target_count {
            let normalized = multiply(source_scale, gain_u32[source][target])?;
            let weighted = multiply(normalized, source_weight_u32[source])?;
            membership[target][source] = multiply(weighted, weighted)?;
        }
    }

    for row in &mut membership {
        let mut row_sum = 0;
        for value in row.iter() {
            row_sum = add(row_sum, *value)?;
        }
        let row_scale = reciprocal(add(row_sum, U04_EPSILON_WORD)?)?;
        for value in row.iter_mut() {
            *value = multiply(row_scale, *value)?;
        }
    }

    let mut projected = vec![[0; 3]; target_count];
    for target in 0..target_count {
        for axis in 0..3 {
            let mut coordinate = 0;
            for source in 0..source_count {
                coordinate = add(
                    coordinate,
                    multiply(membership[target][source], source_xyz_u32[source][axis])?,
                )?;
            }
            projected[target][axis] = coordinate;
        }
        if target_attributes_u32[target] & 1 != 0 {
            projected[target] = target_xyz_u32[target];
        }
    }

    Ok(U04ProjectionResult {
        normalized_m_u32: membership,
        projected_xyz_u32: projected,
    })
}

fn reciprocal(value: u32) -> Result<u32, U04ProjectionError> {
    let numerator = FiniteF32::new(1.0).expect("one is finite");
    let denominator =
        FiniteF32::new(f32::from_bits(value)).map_err(|_| U04ProjectionError::NumericFailure)?;
    if denominator.value() == 0.0 {
        return Err(U04ProjectionError::NumericFailure);
    }
    FiniteF32::new(numerator.value() / denominator.value())
        .map(FiniteF32::to_bits)
        .map_err(|_| U04ProjectionError::NumericFailure)
}

fn add(left: u32, right: u32) -> Result<u32, U04ProjectionError> {
    let left =
        FiniteF32::new(f32::from_bits(left)).map_err(|_| U04ProjectionError::NumericFailure)?;
    let right =
        FiniteF32::new(f32::from_bits(right)).map_err(|_| U04ProjectionError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| U04ProjectionError::NumericFailure)
}

fn multiply(left: u32, right: u32) -> Result<u32, U04ProjectionError> {
    let left =
        FiniteF32::new(f32::from_bits(left)).map_err(|_| U04ProjectionError::NumericFailure)?;
    let right =
        FiniteF32::new(f32::from_bits(right)).map_err(|_| U04ProjectionError::NumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| U04ProjectionError::NumericFailure)
}
