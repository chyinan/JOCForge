// pattern: Functional Core
//! Pure 512-sample long MDCT for one 256-sample overlap window.

pub const LONG_MDCT_BLOCK_SAMPLES: usize = 256;
pub const LONG_MDCT_WINDOW_SAMPLES: usize = LONG_MDCT_BLOCK_SAMPLES * 2;
pub const LONG_MDCT_COEFFICIENT_COUNT: usize = LONG_MDCT_BLOCK_SAMPLES;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LongMdctError {
    NonFiniteInput,
    NumericFailure,
}

/// Computes one ETSI long-transform window from prior and new samples.
///
/// The supplied window contains the 256 coefficients used back-to-back to form the symmetric
/// 512-sample analysis window. The function owns no channel history and returns all 256 bins.
pub fn forward_long_mdct_512(
    previous_samples: &[f32; LONG_MDCT_BLOCK_SAMPLES],
    new_samples: &[f32; LONG_MDCT_BLOCK_SAMPLES],
    window_f32_bits: &[u32; LONG_MDCT_BLOCK_SAMPLES],
) -> Result<[f32; LONG_MDCT_COEFFICIENT_COUNT], LongMdctError> {
    if previous_samples
        .iter()
        .chain(new_samples.iter())
        .any(|sample| !sample.is_finite())
        || window_f32_bits
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(LongMdctError::NonFiniteInput);
    }

    let mut output = [0.0_f32; LONG_MDCT_COEFFICIENT_COUNT];
    for (coefficient_index, output_value) in output.iter_mut().enumerate() {
        let mut sum = 0.0_f64;
        for sample_index in 0..LONG_MDCT_WINDOW_SAMPLES {
            let sample = if sample_index < LONG_MDCT_BLOCK_SAMPLES {
                previous_samples[sample_index]
            } else {
                new_samples[sample_index - LONG_MDCT_BLOCK_SAMPLES]
            };
            let window_index = if sample_index < LONG_MDCT_BLOCK_SAMPLES {
                sample_index
            } else {
                LONG_MDCT_WINDOW_SAMPLES - 1 - sample_index
            };
            let window_value = f64::from(f32::from_bits(window_f32_bits[window_index]));
            let phase = (std::f64::consts::PI / 1_024.0)
                * (2 * sample_index + 1) as f64
                * (2 * coefficient_index + 1) as f64
                + (std::f64::consts::PI / 4.0) * (2 * coefficient_index + 1) as f64;
            sum += f64::from(sample) * window_value * phase.cos();
        }
        let transformed = (-2.0 / LONG_MDCT_WINDOW_SAMPLES as f64) * sum;
        #[allow(clippy::cast_possible_truncation)]
        let transformed = transformed as f32;
        if !transformed.is_finite() {
            return Err(LongMdctError::NumericFailure);
        }
        *output_value = transformed;
    }
    Ok(output)
}
