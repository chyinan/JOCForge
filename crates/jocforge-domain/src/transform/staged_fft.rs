// pattern: Functional Core
//! Stage-normalized F08 FFT64/FFT128 core over interleaved F32 complex words.

use super::math_generated_parameters::MathGeneratedTransformParameters;
use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct F08FftNumericFailure;

pub(super) fn normalized_fft64(
    values: &mut [f32; 128],
    parameters: &MathGeneratedTransformParameters,
) -> Result<(), F08FftNumericFailure> {
    normalized_fft(values, 64, 6, parameters)
}

pub(super) fn normalized_fft128(
    values: &mut [f32; 256],
    parameters: &MathGeneratedTransformParameters,
) -> Result<(), F08FftNumericFailure> {
    normalized_fft(values, 128, 7, parameters)
}

fn normalized_fft(
    values: &mut [f32],
    complex_count: usize,
    stage_count: usize,
    parameters: &MathGeneratedTransformParameters,
) -> Result<(), F08FftNumericFailure> {
    if values.len() != complex_count * 2 {
        return Err(F08FftNumericFailure);
    }
    for stage in 0..stage_count {
        let half = complex_count >> (stage + 1);
        let group_count = 1_usize << stage;
        for group in 0..group_count {
            let rotation = if group >= 2 {
                let cosine = f32::from_bits(parameters.butterfly_f32_bits()[group - 1]);
                let sine = if group % 2 == 1 {
                    f32::from_bits(parameters.butterfly_f32_bits()[group - 2])
                } else {
                    negate(f32::from_bits(parameters.butterfly_f32_bits()[group]))
                };
                Some((cosine, sine))
            } else {
                None
            };

            for pair in 0..half {
                let a_complex = 2 * half * group + pair;
                let b_complex = a_complex + half;
                let a = 2 * a_complex;
                let b = 2 * b_complex;
                let ar = values[a];
                let ai = values[a + 1];
                let br = values[b];
                let bi = values[b + 1];

                let (real, imaginary) = match group {
                    0 => (multiply(add(ar, br)?, 0.5)?, multiply(add(ai, bi)?, 0.5)?),
                    1 => (
                        multiply(add(ar, bi)?, 0.5)?,
                        multiply(subtract(ai, br)?, 0.5)?,
                    ),
                    _ => {
                        let (cosine, sine) = rotation.ok_or(F08FftNumericFailure)?;
                        let cosine_real = multiply(cosine, br)?;
                        let sine_imaginary = multiply(sine, bi)?;
                        let real_before_scale =
                            subtract(subtract(ar, cosine_real)?, sine_imaginary)?;
                        let cosine_imaginary = multiply(cosine, bi)?;
                        let sine_real = multiply(sine, br)?;
                        let imaginary_before_scale =
                            subtract(subtract(ai, cosine_imaginary)?, negate(sine_real))?;
                        (
                            multiply(real_before_scale, 0.5)?,
                            multiply(imaginary_before_scale, 0.5)?,
                        )
                    }
                };

                let b_real = subtract(ar, real)?;
                let b_imaginary = subtract(ai, imaginary)?;
                values[a] = real;
                values[a + 1] = imaginary;
                values[b] = b_real;
                values[b + 1] = b_imaginary;
            }
        }
    }
    Ok(())
}

fn multiply(left: f32, right: f32) -> Result<f32, F08FftNumericFailure> {
    let left = FiniteF32::new(left).map_err(|_| F08FftNumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08FftNumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08FftNumericFailure)
}

fn add(left: f32, right: f32) -> Result<f32, F08FftNumericFailure> {
    let left = FiniteF32::new(left).map_err(|_| F08FftNumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08FftNumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08FftNumericFailure)
}

fn subtract(left: f32, right: f32) -> Result<f32, F08FftNumericFailure> {
    let left = FiniteF32::new(left).map_err(|_| F08FftNumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08FftNumericFailure)?;
    r32_sub(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08FftNumericFailure)
}

fn negate(value: f32) -> f32 {
    f32::from_bits(value.to_bits() ^ 0x8000_0000)
}
