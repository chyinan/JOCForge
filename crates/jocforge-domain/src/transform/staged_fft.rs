// pattern: Functional Core
//! Stage-normalized F08 FFT64/FFT128 core over interleaved F32 complex words.

use super::math_generated_parameters::MathGeneratedTransformParameters;

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
            let begin = 4 * half * group;
            let (a, b) = values[begin..begin + 4 * half].split_at_mut(2 * half);
            let (cosine, sine) = if group >= 2 {
                let table = parameters.butterfly_f32_bits();
                let sine = if group % 2 == 1 {
                    f32::from_bits(table[group - 2])
                } else {
                    -f32::from_bits(table[group])
                };
                (f32::from_bits(table[group - 1]), sine)
            } else {
                (0.0, 0.0)
            };
            for (a, b) in a.chunks_exact_mut(2).zip(b.chunks_exact_mut(2)) {
                let (ar, ai, br, bi) = (a[0], a[1], b[0], b[1]);
                let (real, imaginary) = match group {
                    0 => ((ar + br) * 0.5, (ai + bi) * 0.5),
                    1 => ((ar + bi) * 0.5, (ai - br) * 0.5),
                    _ => (
                        ((ar - cosine * br) - sine * bi) * 0.5,
                        ((ai - cosine * bi) + sine * br) * 0.5,
                    ),
                };
                a.copy_from_slice(&[real, imaginary]);
                b.copy_from_slice(&[ar - real, ai - imaginary]);
            }
        }
    }
    // Every intermediate feeds an output through ordinary IEEE additions and products.
    // Infinity or NaN therefore persists to the final stage.
    if values.iter().any(|v| !v.is_finite()) {
        return Err(F08FftNumericFailure);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflowing_butterfly_is_rejected_even_with_finite_inputs() {
        let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
        let mut input = [0.0; 256];
        input[0] = f32::MAX;
        input[128] = f32::MAX;
        assert_eq!(
            normalized_fft128(&mut input, &parameters),
            Err(F08FftNumericFailure)
        );
    }
}
