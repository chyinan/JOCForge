// pattern: Functional Core
//! F08.3/F08.4/F08.6 short-route pre-rotation, normalized FFT64, and post-rotation.

use super::math_generated_parameters::MathGeneratedTransformParameters;
use super::staged_fft::normalized_fft64;
use super::window_fold::F08WindowFoldOutput;
use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const SHORT_COMPLEX_COUNT: usize = 64;
const SHORT_FFT_WORDS: usize = SHORT_COMPLEX_COUNT * 2;
const SHORT_SPECTRUM_WORDS: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum F08ShortTransformError {
    NumericFailure,
}

impl F08ShortTransformError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::NumericFailure => "F08_NUMERIC_FAILURE",
        }
    }
}

/// Primary and companion spectra from the two interleaved short transforms.
#[derive(Debug, Eq, PartialEq)]
pub struct F08ShortTransformOutput {
    primary_spectrum_bits: [u32; SHORT_SPECTRUM_WORDS],
    companion_spectrum_bits: [u32; SHORT_SPECTRUM_WORDS],
}

impl F08ShortTransformOutput {
    pub const fn primary_spectrum_bits(&self) -> &[u32; SHORT_SPECTRUM_WORDS] {
        &self.primary_spectrum_bits
    }

    pub const fn companion_spectrum_bits(&self) -> &[u32; SHORT_SPECTRUM_WORDS] {
        &self.companion_spectrum_bits
    }
}

/// Functional Core: run only the short FBW branch on an already validated F08.2 fold.
///
/// The caller must select this branch only for a strict F08 short flag of one. This function
/// does not select the flag or update histories; those remain with the caller and fold result.
pub fn f08_short_transform_block(
    folded: &F08WindowFoldOutput,
    parameters: &MathGeneratedTransformParameters,
) -> Result<F08ShortTransformOutput, F08ShortTransformError> {
    let (mut primary_first, mut primary_second) =
        short_pre_rotation(folded.primary_fold_bits(), parameters)?;
    let (mut companion_first, mut companion_second) =
        short_pre_rotation(folded.companion_fold_bits(), parameters)?;

    normalized_fft64(&mut primary_first, parameters)
        .map_err(|_| F08ShortTransformError::NumericFailure)?;
    normalized_fft64(&mut primary_second, parameters)
        .map_err(|_| F08ShortTransformError::NumericFailure)?;
    normalized_fft64(&mut companion_first, parameters)
        .map_err(|_| F08ShortTransformError::NumericFailure)?;
    normalized_fft64(&mut companion_second, parameters)
        .map_err(|_| F08ShortTransformError::NumericFailure)?;

    let (primary_spectrum_bits, companion_spectrum_bits) = short_post_rotation(
        &primary_first,
        &primary_second,
        &companion_first,
        &companion_second,
        parameters,
    )?;
    Ok(F08ShortTransformOutput {
        primary_spectrum_bits,
        companion_spectrum_bits,
    })
}

fn short_pre_rotation(
    folded: &[u32; 256],
    parameters: &MathGeneratedTransformParameters,
) -> Result<([f32; SHORT_FFT_WORDS], [f32; SHORT_FFT_WORDS]), F08ShortTransformError> {
    let mut first = [0.0_f32; SHORT_FFT_WORDS];
    let mut second = [0.0_f32; SHORT_FFT_WORDS];
    for index in 0..SHORT_COMPLEX_COUNT {
        let cosine = f32::from_bits(parameters.short_cosine_f32_bits()[index]);
        let sine = f32::from_bits(parameters.short_sine_f32_bits()[index]);
        let first_real = f32::from_bits(folded[2 * index]);
        let first_imaginary = f32::from_bits(folded[129 + 2 * index]);
        let second_real = f32::from_bits(folded[128 + 2 * index]);
        let second_imaginary = f32::from_bits(folded[1 + 2 * index]);

        first[2 * index] = add(
            negate(multiply(cosine, first_real)?),
            multiply(sine, first_imaginary)?,
        )?;
        first[2 * index + 1] = add(
            multiply(cosine, first_imaginary)?,
            multiply(sine, first_real)?,
        )?;
        second[2 * index] = add(
            negate(multiply(cosine, second_real)?),
            multiply(sine, second_imaginary)?,
        )?;
        second[2 * index + 1] = add(
            multiply(cosine, second_imaginary)?,
            multiply(sine, second_real)?,
        )?;
    }
    Ok((first, second))
}

fn short_post_rotation(
    primary_first: &[f32; SHORT_FFT_WORDS],
    primary_second: &[f32; SHORT_FFT_WORDS],
    companion_first: &[f32; SHORT_FFT_WORDS],
    companion_second: &[f32; SHORT_FFT_WORDS],
    parameters: &MathGeneratedTransformParameters,
) -> Result<([u32; SHORT_SPECTRUM_WORDS], [u32; SHORT_SPECTRUM_WORDS]), F08ShortTransformError> {
    let banks = [
        (primary_first, primary_second),
        (companion_first, companion_second),
    ];
    let mut spectra = [[0_u32; SHORT_SPECTRUM_WORDS]; 2];
    for (bank_index, (first, second)) in banks.into_iter().enumerate() {
        for index in 0..SHORT_COMPLEX_COUNT {
            let source = usize::from(parameters.spectrum_permutation()[2 * index]);
            let cosine = f32::from_bits(parameters.short_cosine_f32_bits()[index]);
            let sine = f32::from_bits(parameters.short_sine_f32_bits()[index]);
            let first_pair = [first[2 * source], first[2 * source + 1]];
            let second_pair = [second[2 * source], second[2 * source + 1]];

            let (first_low, second_low) = if bank_index == 0 {
                (
                    subtract(
                        multiply(cosine, first_pair[1])?,
                        multiply(sine, first_pair[0])?,
                    )?,
                    subtract(
                        multiply(cosine, second_pair[1])?,
                        multiply(sine, second_pair[0])?,
                    )?,
                )
            } else {
                (
                    subtract(
                        multiply(sine, first_pair[0])?,
                        multiply(cosine, first_pair[1])?,
                    )?,
                    subtract(
                        multiply(sine, second_pair[0])?,
                        multiply(cosine, second_pair[1])?,
                    )?,
                )
            };
            let first_high = add(
                multiply(cosine, first_pair[0])?,
                multiply(sine, first_pair[1])?,
            )?;
            let second_high = add(
                multiply(cosine, second_pair[0])?,
                multiply(sine, second_pair[1])?,
            )?;

            spectra[bank_index][4 * index] = first_low.to_bits();
            spectra[bank_index][4 * index + 1] = second_low.to_bits();
            spectra[bank_index][254 - 4 * index] = first_high.to_bits();
            spectra[bank_index][255 - 4 * index] = second_high.to_bits();
        }
    }
    Ok((spectra[0], spectra[1]))
}

fn multiply(left: f32, right: f32) -> Result<f32, F08ShortTransformError> {
    let left = FiniteF32::new(left).map_err(|_| F08ShortTransformError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08ShortTransformError::NumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08ShortTransformError::NumericFailure)
}

fn add(left: f32, right: f32) -> Result<f32, F08ShortTransformError> {
    let left = FiniteF32::new(left).map_err(|_| F08ShortTransformError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08ShortTransformError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08ShortTransformError::NumericFailure)
}

fn subtract(left: f32, right: f32) -> Result<f32, F08ShortTransformError> {
    let left = FiniteF32::new(left).map_err(|_| F08ShortTransformError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08ShortTransformError::NumericFailure)?;
    r32_sub(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08ShortTransformError::NumericFailure)
}

fn negate(value: f32) -> f32 {
    f32::from_bits(value.to_bits() ^ 0x8000_0000)
}
