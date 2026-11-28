// pattern: Functional Core
//! Spectral flux and spectral slope distributions for a continuous 16 kHz channel.

use std::f64::consts::TAU;
use std::sync::{Arc, LazyLock};

const PEAK_FLOOR: f64 = 1.0 / 32768.0;
const FLUX_BLOCK: usize = 1024;
const SLOPE_BLOCK: usize = 512;
static FLUX_FFT: LazyLock<SpectrumPlan<FLUX_BLOCK>> = LazyLock::new(SpectrumPlan::new);
static SLOPE_FFT: LazyLock<SpectrumPlan<SLOPE_BLOCK>> = LazyLock::new(SpectrumPlan::new);

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SpeechSpectrum {
    previous_magnitudes: Arc<[f64; FLUX_BLOCK / 2]>,
    flux: [f64; 32],
    slopes: [f64; 64],
    accepted_peaks: [f64; 16],
}

impl Default for SpeechSpectrum {
    fn default() -> Self {
        Self {
            previous_magnitudes: Arc::new([0.0; FLUX_BLOCK / 2]),
            flux: [0.0; 32],
            slopes: [0.0; 64],
            accepted_peaks: [0.0; 16],
        }
    }
}

impl SpeechSpectrum {
    pub(super) fn push_block(&mut self, samples: &[f64; FLUX_BLOCK]) {
        let peak = samples.iter().map(|x| x.abs()).fold(PEAK_FLOOR, f64::max);
        let mut difference = 0.0;
        let mut energy = 0.0;
        let previous = Arc::make_mut(&mut self.previous_magnitudes);
        for ((bin, magnitude), last) in FLUX_FFT.magnitudes(samples, 0.0).zip(previous) {
            let normalized = magnitude / peak;
            if (7..=499).contains(&bin) {
                difference += (normalized - *last).powi(2);
            }
            energy += normalized * normalized + *last * *last;
            *last = normalized;
        }
        self.flux.rotate_left(1);
        self.flux[31] = if energy > 0.0 {
            difference / energy
        } else {
            0.0
        };
        for half in samples.chunks_exact(SLOPE_BLOCK) {
            let slope = self.block_slope(half);
            self.slopes.rotate_left(1);
            self.slopes[63] = slope;
        }
    }

    fn block_slope(&mut self, samples: &[f64]) -> f64 {
        let mean_absolute = samples.iter().map(|x| x.abs()).sum::<f64>() / SLOPE_BLOCK as f64;
        if mean_absolute < 5.0 * PEAK_FLOOR {
            return 0.0;
        }
        let mean = samples.iter().sum::<f64>() / SLOPE_BLOCK as f64;
        let peak = samples.iter().map(|x| x.abs()).fold(0.0, f64::max);
        self.accepted_peaks.rotate_left(1);
        self.accepted_peaks[15] = peak - mean;
        let scale = self.accepted_peaks.iter().copied().fold(1e-10, f64::max);
        let bins = (SLOPE_BLOCK / 2) as f64;
        let center_bin = (bins - 1.0) / 2.0;
        // Sum of squared deviations of the consecutive regression coordinates.
        let sum_squared_offsets = bins * (bins * bins - 1.0) / 12.0;
        let covariance: f64 = SLOPE_FFT
            .magnitudes(samples, mean)
            .map(|(bin, magnitude)| {
                let decibels = 20.0 * (magnitude / scale).max(1e-12).log10();
                (bin as f64 - center_bin) * decibels
            })
            .sum();
        covariance / sum_squared_offsets
    }

    pub(super) fn features(&self) -> [f64; 2] {
        let flux = self.flux.iter().sum::<f64>() * (FLUX_BLOCK as f64 / self.flux.len() as f64);
        let mean_slope = self.slopes.iter().sum::<f64>() / self.slopes.len() as f64;
        let third_moment = self
            .slopes
            .iter()
            .map(|s| (s - mean_slope).powi(3))
            .sum::<f64>()
            / self.slopes.len() as f64;
        [flux, third_moment.abs()].map(|v| if v > 0.0 { v.ln() / 32.0 } else { 0.0 })
    }
}

struct SpectrumPlan<const N: usize> {
    window: [f64; N],
    reversed: [usize; N],
    roots: Box<[(f64, f64)]>,
}

impl<const N: usize> SpectrumPlan<N> {
    fn new() -> Self {
        Self {
            window: std::array::from_fn(|i| {
                0.54 - 0.46 * (TAU * (i as f64 + 0.5) / N as f64).cos()
            }),
            reversed: std::array::from_fn(|i| i.reverse_bits() >> (usize::BITS - N.ilog2())),
            roots: (0..N / 2)
                .map(|i| (-TAU * i as f64 / N as f64).sin_cos())
                .collect(),
        }
    }

    fn magnitudes(&self, samples: &[f64], mean: f64) -> impl Iterator<Item = (usize, f64)> {
        let mut spectrum = [(0.0, 0.0); N];
        for (i, sample) in samples.iter().enumerate() {
            spectrum[self.reversed[i]].0 = (sample - mean) * self.window[i];
        }
        let mut length = 2;
        while length <= N {
            let half = length / 2;
            let root_step = N / length;
            for block in spectrum.chunks_exact_mut(length) {
                let (lower, upper) = block.split_at_mut(half);
                for (bin, (a, b)) in lower.iter_mut().zip(upper).enumerate() {
                    let (sin, cos) = self.roots[bin * root_step];
                    let real = b.0 * cos - b.1 * sin;
                    let imag = b.0 * sin + b.1 * cos;
                    *b = (a.0 - real, a.1 - imag);
                    *a = (a.0 + real, a.1 + imag);
                }
            }
            length *= 2;
        }
        spectrum
            .into_iter()
            .take(N / 2)
            .map(|(real, imag)| (real * real + imag * imag).sqrt())
            .enumerate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_full_windows_match_independent_fourier_reference() {
        let mut spectrum = SpeechSpectrum::default();
        for block in 0..40 {
            let amplitude = [0.0, 0.0001, 0.125, 1.0, 0.4, 0.7][block % 6];
            let samples = std::array::from_fn(|i| {
                let ramp = ((i * (block % 13 + 1) + 19 * block) % 97) as f64 - 48.0;
                let noise = ((i * i + 13 * i + 11 * block) % 257) as f64 - 128.0;
                amplitude * (ramp / 256.0 + noise / 1024.0)
            });
            spectrum.push_block(&samples);
            // Independent NumPy rfft, linear regression, and centered moments.
            let expected = match block {
                31 => [0.20441834389889768, -0.44047730711193733],
                39 => [0.20461167680156522, -0.4382655962232261],
                _ => continue,
            };
            for (value, reference) in spectrum.features().into_iter().zip(expected) {
                assert!(
                    (value - reference).abs() < 1e-10,
                    "block {block}: {value} versus {reference}"
                );
            }
        }
    }
}
