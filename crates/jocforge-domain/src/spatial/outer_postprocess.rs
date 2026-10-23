// pattern: Functional Core
//! Stateless F03S.5 floor and dominance postprocess for one coefficient vector.

use crate::numeric::{FiniteF32, r32_add, r32_mul};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OuterPostprocessError {
    InvalidInput,
    NumericFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OuterPostprocessResult {
    after_floor_u32: Vec<u32>,
    final_u32: Vec<u32>,
}

impl OuterPostprocessResult {
    pub fn after_floor_u32(&self) -> &[u32] {
        &self.after_floor_u32
    }

    pub fn final_u32(&self) -> &[u32] {
        &self.final_u32
    }
}

/// Applies the strict floor, sequential F32 total, and dominance one-hot rule.
pub fn postprocess_coefficients(
    coefficient_u32: &[u32],
    floor_u32: u32,
    dominance_ratio_u32: u32,
) -> Result<OuterPostprocessResult, OuterPostprocessError> {
    if coefficient_u32.is_empty()
        || coefficient_u32
            .iter()
            .chain([&floor_u32, &dominance_ratio_u32])
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(OuterPostprocessError::InvalidInput);
    }

    let floor = f32::from_bits(floor_u32);
    let after_floor_u32 = coefficient_u32
        .iter()
        .map(|word| {
            if floor > f32::from_bits(*word) {
                0
            } else {
                *word
            }
        })
        .collect::<Vec<_>>();

    let mut total = FiniteF32::new(0.0).map_err(|_| OuterPostprocessError::NumericFailure)?;
    for word in &after_floor_u32 {
        let value = FiniteF32::new(f32::from_bits(*word))
            .map_err(|_| OuterPostprocessError::InvalidInput)?;
        total = r32_add(total, value).map_err(|_| OuterPostprocessError::NumericFailure)?;
    }

    let mut maximum_index = 0;
    let mut maximum = f32::from_bits(after_floor_u32[0]);
    for (index, word) in after_floor_u32.iter().copied().enumerate().skip(1) {
        let candidate = f32::from_bits(word);
        if candidate > maximum {
            maximum = candidate;
            maximum_index = index;
        }
    }

    let ratio = FiniteF32::new(f32::from_bits(dominance_ratio_u32))
        .map_err(|_| OuterPostprocessError::InvalidInput)?;
    let threshold = r32_mul(total, ratio).map_err(|_| OuterPostprocessError::NumericFailure)?;
    let final_u32 = if maximum < threshold.value() {
        after_floor_u32.clone()
    } else {
        let mut one_hot = vec![0; after_floor_u32.len()];
        let capped_total = if total.value() > 1.0 {
            1.0_f32
        } else {
            total.value()
        };
        one_hot[maximum_index] = capped_total.to_bits();
        one_hot
    };

    Ok(OuterPostprocessResult {
        after_floor_u32,
        final_u32,
    })
}
