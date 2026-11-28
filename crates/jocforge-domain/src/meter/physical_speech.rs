// pattern: Functional Core
//! Causal speech-feature extraction on one physical 48 kHz front channel.
use super::speech_spectrum::SpeechSpectrum;
use std::{f64::consts::PI, sync::LazyLock};

const FILTER_HALF: usize = 96;
const BLOCK_SIZES: [usize; 3] = [1023, 1023, 1026];
const PCM_GATE: f64 = 1.0 / 32768.0;

fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    for k in 1..=32 {
        term *= (x / (2.0 * f64::from(k))).powi(2);
        sum += term;
        if term < sum * f64::EPSILON {
            break;
        }
    }
    sum
}

// Symmetric FIR stored centre-first. The Kaiser denominator cancels on normalization.
static FILTER: LazyLock<[f64; FILTER_HALF + 1]> = LazyLock::new(|| {
    let cutoff = 7447.9 / 48_000.0;
    let mut half = std::array::from_fn(|i| {
        let x = i as f64;
        let sinc = if i == 0 {
            2.0 * cutoff
        } else {
            (2.0 * PI * cutoff * x).sin() / (PI * x)
        };
        sinc * bessel_i0(8.6 * (1.0 - (x / FILTER_HALF as f64).powi(2)).sqrt())
    });
    let scale = 0.494_998 / (half[0] + 2.0 * half[1..].iter().sum::<f64>());
    for tap in &mut half {
        *tap *= scale;
    }
    half
});

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PhysicalSpeech {
    fir_input: Vec<f64>,
    gate_phase: usize,
    block: Box<[f64; 1024]>,
    filled: usize,
    spectrum: SpeechSpectrum,
    rhythm: SpeechRhythm,
}

impl Default for PhysicalSpeech {
    fn default() -> Self {
        let mut fir_input = Vec::with_capacity(2 * FILTER_HALF + 1026);
        fir_input.resize(2 * FILTER_HALF, 0.0);
        Self {
            fir_input,
            gate_phase: 0,
            block: Box::new([0.0; 1024]),
            filled: 0,
            spectrum: SpeechSpectrum::default(),
            rhythm: SpeechRhythm::default(),
        }
    }
}

impl PhysicalSpeech {
    pub(super) fn push(&mut self, mut words: &[u32], mut publish: impl FnMut([f64; 7])) {
        while !words.is_empty() {
            let full = 2 * FILTER_HALF + BLOCK_SIZES[self.gate_phase];
            let count = words.len().min(full - self.fir_input.len());
            self.fir_input.extend(
                words[..count]
                    .iter()
                    .map(|&word| f64::from(f32::from_bits(word))),
            );
            words = &words[count..];
            if self.fir_input.len() < full {
                continue;
            }
            let fresh = &mut self.fir_input[2 * FILTER_HALF..];
            if fresh.iter().all(|x| x.abs() < PCM_GATE) {
                fresh.fill(0.0);
            }
            let silent = self.fir_input.iter().all(|&x| x == 0.0);
            for i in (0..BLOCK_SIZES[self.gate_phase]).step_by(3) {
                let signal = &self.fir_input[i..=i + 2 * FILTER_HALF];
                self.block[self.filled] = if silent { 0.0 } else { filter_sample(signal) };
                self.filled += 1;
                if self.filled == self.block.len() {
                    self.spectrum.push_block(&self.block);
                    for part in self.block.chunks_exact(256) {
                        self.rhythm.push_block(part);
                    }
                    self.filled = 0;
                    if self.rhythm.blocks >= 128 && self.rhythm.blocks % 32 == 0 {
                        let [flux, slope] = self.spectrum.features();
                        let [crossings, skew, median, short, long] = self.rhythm.features();
                        publish([flux, slope, crossings, skew, median, short, long]);
                    }
                }
            }
            self.fir_input.copy_within(full - 2 * FILTER_HALF.., 0);
            self.fir_input.truncate(2 * FILTER_HALF);
            self.gate_phase = (self.gate_phase + 1) % BLOCK_SIZES.len();
        }
    }
}

fn filter_sample(signal: &[f64]) -> f64 {
    let (before, from_centre) = signal.split_at(FILTER_HALF);
    before
        .iter()
        .rev()
        .zip(&from_centre[1..])
        .zip(&FILTER[1..])
        .fold(from_centre[0] * FILTER[0], |sum, ((a, b), tap)| {
            sum + (a + b) * tap
        })
}

#[derive(Clone, Debug, PartialEq)]
struct SpeechRhythm {
    blocks: usize,
    previous_negative: bool,
    short_power: [f64; 128],
    long_power: [f64; 256],
    crossings: [f64; 128],
    variance: [f64; 128],
    prior_correlations: [[f64; 128]; 2],
}

impl Default for SpeechRhythm {
    fn default() -> Self {
        Self {
            blocks: 0,
            previous_negative: false,
            short_power: [0.0; 128],
            long_power: [0.0; 256],
            crossings: [0.0; 128],
            variance: [0.0; 128],
            prior_correlations: [[0.0; 128]; 2],
        }
    }
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn autocorrelation<const N: usize>(values: &[f64; N]) -> [f64; N] {
    std::array::from_fn(|lag| {
        values[..N - lag]
            .iter()
            .zip(&values[lag..])
            .map(|(a, b)| a * b)
            .sum()
    })
}

fn periodicity(correlation: &[f64], first_lag: usize) -> f64 {
    if correlation[0] > 0.0 {
        correlation[first_lag..].iter().copied().fold(0.0, f64::max) / correlation[0]
    } else {
        0.0
    }
}

impl SpeechRhythm {
    fn push_block(&mut self, samples: &[f64]) {
        let mut count = 0;
        let mut energy = 0.0;
        let mut variance = 0.0;
        let average = mean(samples);
        for &sample in samples {
            let negative = sample < 0.0;
            count += usize::from(self.previous_negative != negative);
            self.previous_negative = negative;
            energy += sample * sample;
            variance += (sample - average).powi(2);
        }
        let i = self.blocks % 128;
        self.crossings[i] = count as f64 / 256.0;
        self.variance[i] = variance / 256.0;
        self.short_power[i] = energy / 256.0;
        self.long_power[self.blocks % 256] = energy / 256.0;
        self.blocks += 1;
    }

    fn features(&mut self) -> [f64; 5] {
        let gate = mean(&self.short_power) / 4.0;
        let down = self
            .short_power
            .windows(2)
            .filter(|pair| pair[0] > gate && pair[1] < gate)
            .count();
        let avg = mean(&self.crossings);
        let variance = self
            .crossings
            .iter()
            .map(|z| (z - avg).powi(2))
            .sum::<f64>()
            / 128.0;
        let third = self
            .crossings
            .iter()
            .map(|z| (z - avg).powi(3))
            .sum::<f64>()
            / 128.0;
        let skew = if variance > 0.0 {
            third / (variance * variance.sqrt()) / 16.0
        } else {
            0.0
        };
        let median = if avg > 0.0 {
            let mut sorted = self.crossings;
            sorted.sort_unstable_by(f64::total_cmp);
            (sorted[63] + sorted[64]) / (4.0 * avg)
        } else {
            0.0
        };

        let mut short = self.variance;
        short.rotate_left(self.blocks % 128);
        let avg = mean(&short);
        if avg > 0.0 {
            for x in &mut short {
                *x /= avg;
            }
            let normalized_avg = mean(&short);
            for x in &mut short {
                *x -= normalized_avg;
            }
        }
        let current = autocorrelation(&short);
        let combined: [f64; 128] = std::array::from_fn(|i| {
            current[i] + 0.5 * self.prior_correlations[0][i] + 0.25 * self.prior_correlations[1][i]
        });
        self.prior_correlations[1] = self.prior_correlations[0];
        self.prior_correlations[0] = current;

        let mut long = self.long_power;
        long.rotate_left(self.blocks % 256);
        let avg = mean(&long);
        for x in &mut long {
            *x -= avg;
        }
        [
            down as f64 / 128.0,
            skew,
            median,
            periodicity(&combined, 10),
            periodicity(&autocorrelation(&long), 30),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_sample(i: usize) -> u32 {
        let gate_block =
            i / 3072 * 3 + usize::from(i % 3072 >= 1023) + usize::from(i % 3072 >= 2046);
        let gate_sign = if i % 7 < 3 { 1.0 } else { -1.0 };
        let power_block = i / 768;
        let sample = match gate_block % 11 {
            0 => 0.0,
            1 => gate_sign * PCM_GATE,
            2 => gate_sign * PCM_GATE / 2.0,
            _ => {
                let gain = match power_block % 128 {
                    0 => 0.0,
                    127 => 1.0,
                    _ => [1.0, 0.7, 0.016, 0.3, 0.43, 0.02, 0.01][(power_block * 13 / 3) % 7],
                };
                let ramp = (i % 193) as f64 - 96.0;
                let noise = ((i % 3571) * 37 % 113) as f64 - 56.0;
                gain * (ramp + noise) / 256.0
            }
        };
        (sample as f32).to_bits()
    }

    #[test]
    fn continuous_physical_input_matches_independent_signal_processing() {
        let words: Vec<_> = (0..196_608).map(synthetic_sample).collect();
        let extract = |chunk| {
            let mut channel = PhysicalSpeech::default();
            let mut rows = Vec::new();
            for part in words.chunks(chunk) {
                channel.push(part, |features| rows.push(features));
            }
            rows
        };
        let rows = extract(997);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows, extract(words.len()));
        // Reference uses SciPy's causal firwin/lfilter plus NumPy rfft and correlate.
        for (index, reference) in [
            (
                0,
                [
                    0.183424519996801,
                    -0.278692918164082,
                    0.171875,
                    0.146708812580282,
                    0.163543441226576,
                    0.493727700300905,
                    0.470666196514790,
                ],
            ),
            (
                1,
                [
                    0.181995846961487,
                    -0.280971007508034,
                    0.1875,
                    0.155474044936710,
                    0.172197309417040,
                    0.469747175120621,
                    0.500100611498222,
                ],
            ),
            (
                3,
                [
                    0.184195245689384,
                    -0.280143109180542,
                    0.1875,
                    0.146378408640857,
                    0.162505289885738,
                    0.538234009605744,
                    0.528295966818405,
                ],
            ),
        ] {
            for (feature, (&value, expected)) in rows[index].iter().zip(reference).enumerate() {
                assert!(
                    (value - expected).abs() < 1e-10,
                    "window {index}, feature {feature}: {value} versus {expected}"
                );
            }
        }
    }
}
