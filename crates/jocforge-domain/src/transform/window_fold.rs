// pattern: Functional Core
//! Selected F08.2 window-fold and two-bank history transition.

use super::math_generated_parameters::MathGeneratedTransformParameters;
use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

pub const WINDOW_FOLD_INPUT_WORDS: usize = 256;
pub const WINDOW_FOLD_HISTORY_WORDS: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum F08WindowFoldError {
    InputError,
    NumericFailure,
}

impl F08WindowFoldError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputError => "F08_INPUT_ERROR",
            Self::NumericFailure => "F08_NUMERIC_FAILURE",
        }
    }
}

/// Folded values and next histories for one primary/companion F08.2 block.
#[derive(Debug, Eq, PartialEq)]
pub struct F08WindowFoldOutput {
    primary_fold_bits: [u32; WINDOW_FOLD_INPUT_WORDS],
    companion_fold_bits: [u32; WINDOW_FOLD_INPUT_WORDS],
    primary_history_after: [u32; WINDOW_FOLD_HISTORY_WORDS],
    companion_history_after: [u32; WINDOW_FOLD_HISTORY_WORDS],
}

impl F08WindowFoldOutput {
    pub const fn primary_fold_bits(&self) -> &[u32; WINDOW_FOLD_INPUT_WORDS] {
        &self.primary_fold_bits
    }

    pub const fn companion_fold_bits(&self) -> &[u32; WINDOW_FOLD_INPUT_WORDS] {
        &self.companion_fold_bits
    }

    pub const fn primary_history_after(&self) -> &[u32; WINDOW_FOLD_HISTORY_WORDS] {
        &self.primary_history_after
    }

    pub const fn companion_history_after(&self) -> &[u32; WINDOW_FOLD_HISTORY_WORDS] {
        &self.companion_history_after
    }
}

/// Functional Core: applies the shared F08.2 window fold and computes both bank states.
///
/// This stage does not select a long/short route, consume a source clock, or install either
/// returned history into a parent owner.
pub fn f08_window_fold_block(
    input_bits: &[u32],
    primary_history_bits: &[u32],
    companion_history_bits: &[u32],
    parameters: &MathGeneratedTransformParameters,
) -> Result<F08WindowFoldOutput, F08WindowFoldError> {
    if input_bits.len() != WINDOW_FOLD_INPUT_WORDS
        || primary_history_bits.len() != WINDOW_FOLD_HISTORY_WORDS
        || companion_history_bits.len() != WINDOW_FOLD_HISTORY_WORDS
        || input_bits
            .iter()
            .chain(primary_history_bits)
            .chain(companion_history_bits)
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(F08WindowFoldError::InputError);
    }

    let mut primary_fold_bits = [0_u32; WINDOW_FOLD_INPUT_WORDS];
    let mut companion_fold_bits = [0_u32; WINDOW_FOLD_INPUT_WORDS];
    let mut primary_history_after = [0_u32; WINDOW_FOLD_HISTORY_WORDS];
    let mut companion_history_after = [0_u32; WINDOW_FOLD_HISTORY_WORDS];
    let window = parameters.window_f32_bits();

    for index in 0..WINDOW_FOLD_HISTORY_WORDS {
        let tail = 255 - index;
        let (row, side, history_slot) = if index % 2 == 0 {
            (128 + index, 129 + index, 64 + index / 2)
        } else {
            (128 - index, 127 - index, 63 - index / 2)
        };

        let sample_head = f32::from_bits(input_bits[index]);
        let sample_tail = f32::from_bits(input_bits[tail]);
        let window_head = f32::from_bits(window[index]);
        let window_tail = f32::from_bits(window[tail]);
        let a = multiply(sample_head, window_tail)?;
        let b = multiply(sample_tail, window_head)?;
        let c = multiply(sample_tail, window_tail)?;
        let d = multiply(window_head, sample_head)?;

        primary_fold_bits[row] = add(a, b)?.to_bits();
        primary_fold_bits[side] = primary_history_bits[history_slot];
        primary_history_after[history_slot] = subtract(c, d)?.to_bits();

        companion_fold_bits[row] = companion_history_bits[history_slot];
        companion_fold_bits[side] = subtract(a, b)?.to_bits();
        companion_history_after[history_slot] = add(c, d)?.to_bits();
    }

    Ok(F08WindowFoldOutput {
        primary_fold_bits,
        companion_fold_bits,
        primary_history_after,
        companion_history_after,
    })
}

fn multiply(left: f32, right: f32) -> Result<f32, F08WindowFoldError> {
    let left = FiniteF32::new(left).map_err(|_| F08WindowFoldError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08WindowFoldError::NumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08WindowFoldError::NumericFailure)
}

fn add(left: f32, right: f32) -> Result<f32, F08WindowFoldError> {
    let left = FiniteF32::new(left).map_err(|_| F08WindowFoldError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08WindowFoldError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08WindowFoldError::NumericFailure)
}

fn subtract(left: f32, right: f32) -> Result<f32, F08WindowFoldError> {
    let left = FiniteF32::new(left).map_err(|_| F08WindowFoldError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08WindowFoldError::NumericFailure)?;
    r32_sub(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08WindowFoldError::NumericFailure)
}
