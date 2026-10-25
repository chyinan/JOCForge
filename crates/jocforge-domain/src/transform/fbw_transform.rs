// pattern: Functional Core
//! Selected math-generated F08 long/short FBW spectral transform branches.

use super::math_generated_parameters::MathGeneratedTransformParameters;
use super::short_transform::{F08ShortTransformError, f08_short_transform_block};
use super::staged_fft::{F08FftNumericFailure, normalized_fft128};
use super::window_fold::F08WindowFoldOutput;
use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const SPECTRUM_WORDS: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum F08FbwBranch {
    Long,
    Short,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum F08FbwTransformError {
    InputError,
    NumericFailure,
}

impl F08FbwTransformError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputError => "F08_INPUT_ERROR",
            Self::NumericFailure => "F08_NUMERIC_FAILURE",
        }
    }
}

/// Primary/companion FBW spectra from one selected F08 long or short block.
#[derive(Debug, Eq, PartialEq)]
pub struct F08FbwTransformOutput {
    branch: F08FbwBranch,
    primary_spectrum_bits: [u32; SPECTRUM_WORDS],
    companion_spectrum_bits: [u32; SPECTRUM_WORDS],
}

impl F08FbwTransformOutput {
    pub const fn branch(&self) -> F08FbwBranch {
        self.branch
    }

    pub const fn primary_spectrum_bits(&self) -> &[u32; SPECTRUM_WORDS] {
        &self.primary_spectrum_bits
    }

    pub const fn companion_spectrum_bits(&self) -> &[u32; SPECTRUM_WORDS] {
        &self.companion_spectrum_bits
    }
}

/// Functional Core: select the long or short FBW route from a strict integer I1 flag.
///
/// The caller supplies an already validated F08.2 fold and provider. This stage assigns no
/// source clock and does not install either bank history into a parent owner.
pub fn f08_fbw_transform_block(
    folded: &F08WindowFoldOutput,
    short_flag_i1: i32,
    parameters: &MathGeneratedTransformParameters,
) -> Result<F08FbwTransformOutput, F08FbwTransformError> {
    let branch = match short_flag_i1 {
        0 => F08FbwBranch::Long,
        1 => F08FbwBranch::Short,
        _ => return Err(F08FbwTransformError::InputError),
    };

    let (primary_spectrum_bits, companion_spectrum_bits) = match branch {
        F08FbwBranch::Long => (
            long_spectrum(
                folded.primary_fold_bits(),
                MathGeneratedBank::Primary,
                parameters,
            )?,
            long_spectrum(
                folded.companion_fold_bits(),
                MathGeneratedBank::Companion,
                parameters,
            )?,
        ),
        F08FbwBranch::Short => {
            let output =
                f08_short_transform_block(folded, parameters).map_err(map_short_transform_error)?;
            (
                *output.primary_spectrum_bits(),
                *output.companion_spectrum_bits(),
            )
        }
    };

    Ok(F08FbwTransformOutput {
        branch,
        primary_spectrum_bits,
        companion_spectrum_bits,
    })
}

#[derive(Clone, Copy)]
enum MathGeneratedBank {
    Primary,
    Companion,
}

pub(super) fn long_pre_rotation(
    folded: &[u32; SPECTRUM_WORDS],
    parameters: &MathGeneratedTransformParameters,
) -> Result<[f32; SPECTRUM_WORDS], F08FbwTransformError> {
    let mut pre_rotated = [0.0_f32; SPECTRUM_WORDS];
    for index in 0..128 {
        let x = f32::from_bits(folded[2 * index]);
        let y = f32::from_bits(folded[2 * index + 1]);
        let cosine = f32::from_bits(parameters.long_cosine_f32_bits()[index]);
        let sine = f32::from_bits(parameters.long_sine_f32_bits()[index]);
        let cosine_x = multiply(cosine, x)?;
        let sine_y = multiply(sine, y)?;
        let cosine_y = multiply(cosine, y)?;
        let sine_x = multiply(sine, x)?;
        pre_rotated[2 * index] = add(negate(cosine_x), sine_y)?;
        pre_rotated[2 * index + 1] = add(cosine_y, sine_x)?;
    }
    Ok(pre_rotated)
}

fn long_spectrum(
    folded: &[u32; SPECTRUM_WORDS],
    bank: MathGeneratedBank,
    parameters: &MathGeneratedTransformParameters,
) -> Result<[u32; SPECTRUM_WORDS], F08FbwTransformError> {
    let mut fft = long_pre_rotation(folded, parameters)?;
    normalized_fft128(&mut fft, parameters).map_err(map_fft_error)?;

    let mut spectrum = [0_u32; SPECTRUM_WORDS];
    for index in 0..128 {
        let source = usize::from(parameters.spectrum_permutation()[index]);
        let real = fft[2 * source];
        let imaginary = fft[2 * source + 1];
        let cosine = f32::from_bits(parameters.long_cosine_f32_bits()[index]);
        let sine = f32::from_bits(parameters.long_sine_f32_bits()[index]);
        let cosine_imaginary = multiply(cosine, imaginary)?;
        let sine_real = multiply(sine, real)?;
        let low = match bank {
            MathGeneratedBank::Primary => subtract(cosine_imaginary, sine_real)?,
            MathGeneratedBank::Companion => subtract(sine_real, cosine_imaginary)?,
        };
        spectrum[2 * index] = low.to_bits();
        spectrum[255 - 2 * index] =
            add(multiply(cosine, real)?, multiply(sine, imaginary)?)?.to_bits();
    }
    Ok(spectrum)
}

fn map_fft_error(_: F08FftNumericFailure) -> F08FbwTransformError {
    F08FbwTransformError::NumericFailure
}

fn map_short_transform_error(error: F08ShortTransformError) -> F08FbwTransformError {
    match error {
        F08ShortTransformError::NumericFailure => F08FbwTransformError::NumericFailure,
    }
}

fn multiply(left: f32, right: f32) -> Result<f32, F08FbwTransformError> {
    let left = FiniteF32::new(left).map_err(|_| F08FbwTransformError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08FbwTransformError::NumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08FbwTransformError::NumericFailure)
}

fn add(left: f32, right: f32) -> Result<f32, F08FbwTransformError> {
    let left = FiniteF32::new(left).map_err(|_| F08FbwTransformError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08FbwTransformError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08FbwTransformError::NumericFailure)
}

fn subtract(left: f32, right: f32) -> Result<f32, F08FbwTransformError> {
    let left = FiniteF32::new(left).map_err(|_| F08FbwTransformError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08FbwTransformError::NumericFailure)?;
    r32_sub(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08FbwTransformError::NumericFailure)
}

fn negate(value: f32) -> f32 {
    f32::from_bits(value.to_bits() ^ 0x8000_0000)
}
