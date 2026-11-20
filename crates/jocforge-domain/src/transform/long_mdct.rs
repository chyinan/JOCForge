// pattern: Functional Core
//! Pure 512-sample long MDCT for one 256-sample overlap window.

use std::sync::LazyLock;

pub const LONG_MDCT_BLOCK_SAMPLES: usize = 256;
pub const LONG_MDCT_WINDOW_SAMPLES: usize = LONG_MDCT_BLOCK_SAMPLES * 2;
pub const LONG_MDCT_COEFFICIENT_COUNT: usize = LONG_MDCT_BLOCK_SAMPLES;

// Rows follow the original scalar transform's phase expression. Reducing the
// angle modulo a period would change rounding before cos() for some bins.
const BIN_LANES: usize = 8;
static COSINE_TILES: LazyLock<Box<[[f64; BIN_LANES]]>> = LazyLock::new(|| {
    (0..LONG_MDCT_COEFFICIENT_COUNT / BIN_LANES)
        .flat_map(|tile| {
            (0..LONG_MDCT_WINDOW_SAMPLES).map(move |sample_index| {
                std::array::from_fn(|lane| {
                    let coefficient_index = tile * BIN_LANES + lane;
                    let phase = (std::f64::consts::PI / 1_024.0)
                        * (2 * sample_index + 1) as f64
                        * (2 * coefficient_index + 1) as f64
                        + (std::f64::consts::PI / 4.0) * (2 * coefficient_index + 1) as f64;
                    phase.cos()
                })
            })
        })
        .collect()
});

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

    let mut windowed = [0.0_f64; LONG_MDCT_WINDOW_SAMPLES];
    let (previous, current) = windowed.split_at_mut(LONG_MDCT_BLOCK_SAMPLES);
    for index in 0..LONG_MDCT_BLOCK_SAMPLES {
        previous[index] =
            f64::from(previous_samples[index]) * f64::from(f32::from_bits(window_f32_bits[index]));
        current[index] = f64::from(new_samples[index])
            * f64::from(f32::from_bits(
                window_f32_bits[LONG_MDCT_BLOCK_SAMPLES - 1 - index],
            ));
    }

    // A zero dot product starts at +0. The final negative scale produces -0
    // for every bin, including windows containing both signs of zero.
    if windowed.iter().all(|&sample| sample == 0.0) {
        return Ok([-0.0; LONG_MDCT_COEFFICIENT_COUNT]);
    }

    let mut output = [0.0_f32; LONG_MDCT_COEFFICIENT_COUNT];
    for (bins, tile) in output
        .chunks_exact_mut(BIN_LANES)
        .zip(COSINE_TILES.chunks_exact(LONG_MDCT_WINDOW_SAMPLES))
    {
        // Each lane is an independent coefficient with the original sample order.
        let mut sums = [0.0; BIN_LANES];
        for (&sample, cosines) in windowed.iter().zip(tile) {
            for lane in 0..BIN_LANES {
                sums[lane] += sample * cosines[lane];
            }
        }
        for (out, sum) in bins.iter_mut().zip(sums) {
            #[allow(clippy::cast_possible_truncation)]
            let transformed = ((-2.0 / LONG_MDCT_WINDOW_SAMPLES as f64) * sum) as f32;
            if !transformed.is_finite() {
                return Err(LongMdctError::NumericFailure);
            }
            *out = transformed;
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_windows_preserve_negative_zero_and_validate_nonfinite_input() {
        let previous = [0.0; LONG_MDCT_BLOCK_SAMPLES];
        let current = [-0.0; LONG_MDCT_BLOCK_SAMPLES];
        let mut window = [1.0_f32.to_bits(); LONG_MDCT_BLOCK_SAMPLES];
        assert_eq!(
            forward_long_mdct_512(&previous, &current, &window)
                .unwrap()
                .map(f32::to_bits),
            [(-0.0_f32).to_bits(); LONG_MDCT_COEFFICIENT_COUNT]
        );
        window[0] = f32::NAN.to_bits();
        assert_eq!(
            forward_long_mdct_512(&previous, &current, &window),
            Err(LongMdctError::NonFiniteInput)
        );
    }

    #[test]
    fn long_mdct_matches_scalar_bits_for_every_bin() {
        let previous = std::array::from_fn(|i| ((i * 37 % 251) as f32 - 125.0) / 128.0);
        let current = std::array::from_fn(|i| ((i * 83 % 239) as f32 - 119.0) / 128.0);
        let window = std::array::from_fn(|i| ((i + 1) as f32 / 257.0).to_bits());
        let actual = forward_long_mdct_512(&previous, &current, &window).unwrap();

        for (bin, value) in actual.into_iter().enumerate() {
            let mut sum = 0.0_f64;
            let samples = previous.iter().chain(&current);
            let weights = window.iter().chain(window.iter().rev());
            for (i, (&sample, &weight)) in samples.zip(weights).enumerate() {
                let phase =
                    (std::f64::consts::PI / 1_024.0) * (2 * i + 1) as f64 * (2 * bin + 1) as f64
                        + (std::f64::consts::PI / 4.0) * (2 * bin + 1) as f64;
                sum += f64::from(sample) * f64::from(f32::from_bits(weight)) * phase.cos();
            }
            let expected = ((-2.0 / 512.0) * sum) as f32;
            assert_eq!(value.to_bits(), expected.to_bits(), "bin {bin}");
        }
    }
}
