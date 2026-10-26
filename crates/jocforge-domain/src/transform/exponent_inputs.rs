// pattern: Functional Core
//! Selected F08 spectrum projection into F09 POWER and PRIMARY exponent inputs.

use super::fbw_transform::F08FbwTransformOutput;
use super::lfe_transform::F08LfeTransformOutput;
use crate::codec::mantissa::{
    ClippedMantissaRow, MantissaBoundaryError, clip_lfe_main_coefficients,
    clip_primary_coefficients,
};
use crate::codec::raw_exponent_from_power::raw_exponent_from_power_bits;
use crate::numeric::{FiniteF32, r32_add, r32_mul};

pub const FULL_BAND_EXPONENT_INPUT_BINS: usize = 217;
pub const LFE_EXPONENT_INPUT_BINS: usize = 7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransformExponentInputError {
    ExponentInput,
    NumericFailure,
}

impl TransformExponentInputError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ExponentInput => "EXPONENT_INPUT_ERROR",
            Self::NumericFailure => "NUMERIC_FAILURE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformExponentInputs<const BINS: usize> {
    power_f32_bits: [u32; BINS],
    power_raw_exponents_i16: [i16; BINS],
    primary_clipped_f32_bits: [u32; BINS],
    primary_raw_exponents_i16: [i16; BINS],
}

impl<const BINS: usize> TransformExponentInputs<BINS> {
    pub const fn power_f32_bits(&self) -> &[u32; BINS] {
        &self.power_f32_bits
    }

    pub const fn power_raw_exponents_i16(&self) -> &[i16; BINS] {
        &self.power_raw_exponents_i16
    }

    pub const fn primary_clipped_f32_bits(&self) -> &[u32; BINS] {
        &self.primary_clipped_f32_bits
    }

    pub const fn primary_raw_exponents_i16(&self) -> &[i16; BINS] {
        &self.primary_raw_exponents_i16
    }
}

pub type FullBandTransformExponentInputs = TransformExponentInputs<FULL_BAND_EXPONENT_INPUT_BINS>;
pub type LfeTransformExponentInputs = TransformExponentInputs<LFE_EXPONENT_INPUT_BINS>;

/// Produces the selected first 217 F09 bins from the corresponding F08 FBW banks.
pub fn f08_full_band_exponent_inputs(
    spectra: &F08FbwTransformOutput,
) -> Result<FullBandTransformExponentInputs, TransformExponentInputError> {
    build_exponent_inputs(
        &spectra.primary_spectrum_bits()[..FULL_BAND_EXPONENT_INPUT_BINS],
        &spectra.companion_spectrum_bits()[..FULL_BAND_EXPONENT_INPUT_BINS],
        clip_primary_coefficients,
    )
}

/// Produces all seven selected F09 LFE bins from the corresponding F08 LFE banks.
pub fn f08_lfe_exponent_inputs(
    spectra: &F08LfeTransformOutput,
) -> Result<LfeTransformExponentInputs, TransformExponentInputError> {
    build_exponent_inputs(
        spectra.primary_spectrum_bits(),
        spectra.companion_spectrum_bits(),
        clip_lfe_main_coefficients,
    )
}

fn build_exponent_inputs<const BINS: usize>(
    primary_words: &[u32],
    companion_words: &[u32],
    clip_primary: fn(&[u32]) -> Result<ClippedMantissaRow, MantissaBoundaryError>,
) -> Result<TransformExponentInputs<BINS>, TransformExponentInputError> {
    if primary_words.len() != BINS
        || companion_words.len() != BINS
        || primary_words
            .iter()
            .chain(companion_words)
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(TransformExponentInputError::ExponentInput);
    }

    let clipped =
        clip_primary(primary_words).map_err(|_| TransformExponentInputError::ExponentInput)?;
    let primary_clipped_f32_bits = to_word_array::<BINS>(clipped.clipped_words())?;
    let primary_raw_exponents_i16 = clipped
        .raw_exponents()
        .iter()
        .copied()
        .map(i16::from)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| TransformExponentInputError::ExponentInput)?;

    let mut power_f32_bits = [0_u32; BINS];
    let mut power_raw_exponents_i16 = [0_i16; BINS];
    for index in 0..BINS {
        let primary = finite(primary_words[index])?;
        let companion = finite(companion_words[index])?;
        let primary_square =
            r32_mul(primary, primary).map_err(|_| TransformExponentInputError::NumericFailure)?;
        let companion_square = r32_mul(companion, companion)
            .map_err(|_| TransformExponentInputError::NumericFailure)?;
        let sum = r32_add(primary_square, companion_square)
            .map_err(|_| TransformExponentInputError::NumericFailure)?;
        let capped_power = sum.value().min(1.0);
        if !capped_power.is_finite() || capped_power < 0.0 {
            return Err(TransformExponentInputError::NumericFailure);
        }
        let power_word = capped_power.to_bits();
        power_f32_bits[index] = power_word;
        power_raw_exponents_i16[index] = raw_exponent_from_power_bits(power_word, 0, 48);
    }

    Ok(TransformExponentInputs {
        power_f32_bits,
        power_raw_exponents_i16,
        primary_clipped_f32_bits,
        primary_raw_exponents_i16,
    })
}

fn finite(word: u32) -> Result<FiniteF32, TransformExponentInputError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| TransformExponentInputError::ExponentInput)
}

fn to_word_array<const BINS: usize>(
    words: &[u32],
) -> Result<[u32; BINS], TransformExponentInputError> {
    words
        .try_into()
        .map_err(|_| TransformExponentInputError::ExponentInput)
}
