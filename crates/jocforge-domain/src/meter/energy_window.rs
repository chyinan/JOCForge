// pattern: Functional Core
//! Selected K-weighted energy-window state and complete-1024 block transition.

use super::{
    k_matrix::KMatrixProvider,
    true_peak::{
        TruePeakError, accumulate_normalized_energy, add_words, multiply_words,
        normalize_energy_window, ordered_energy_reduction,
    },
};

const CHANNEL_COUNT: usize = 6;
const STAGE_COUNT: usize = 2;
const MATRIX_ROWS: usize = 8;
const SIMD_LANES: usize = 4;
const COMPLETE_BLOCK_SAMPLES: usize = 1_024;
const ENERGY_WINDOW_SAMPLES: usize = 960;
const SEGMENT_ALIGNMENT: usize = 8;
const COLD_EXPONENT: i32 = 48;
const EIGHTH_WORD: u32 = 0x3e00_0000;
const ONE_THIRTY_SECOND_WORD: u32 = 0x3d00_0000;
const HALF_WORD: u32 = 0x3f00_0000;
const ZERO_WORD: u32 = 0;
const SURROUND_WEIGHT_WORD: u32 = 0x3f34_7ae1;
const CHANNEL_WEIGHTS: [u32; CHANNEL_COUNT] = [
    HALF_WORD,
    HALF_WORD,
    HALF_WORD,
    ZERO_WORD,
    SURROUND_WEIGHT_WORD,
    SURROUND_WEIGHT_WORD,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnergyWindowError {
    InputShape,
    InputNonFinite,
    StateInvalid,
    ParameterInvalid,
    NumericFailure,
}

/// Persistent selected K-filter and incomplete 960-sample window state.
///
/// This value does not own the upstream valid-audio scheduler or any meter
/// commit authority. It advances only from complete six-lane 1024-sample
/// blocks supplied by that scheduler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnergyWindowState {
    k_state_words: [[[u32; SIMD_LANES]; STAGE_COUNT]; CHANNEL_COUNT],
    accumulator_mantissa_words: [u32; CHANNEL_COUNT],
    accumulator_exponents: [i32; CHANNEL_COUNT],
    samples_in_window: u16,
}

impl EnergyWindowState {
    pub fn cold(provider: &KMatrixProvider) -> Result<Self, EnergyWindowError> {
        let parameters = KFilterParameters::from_provider(provider)?;
        let k_state_words = std::array::from_fn(|_| parameters.cold_state_words);
        let state = Self {
            k_state_words,
            accumulator_mantissa_words: [ZERO_WORD; CHANNEL_COUNT],
            accumulator_exponents: [COLD_EXPONENT; CHANNEL_COUNT],
            samples_in_window: 0,
        };
        state.validate(&parameters)?;
        Ok(state)
    }

    pub const fn k_state_words(&self) -> &[[[u32; SIMD_LANES]; STAGE_COUNT]; CHANNEL_COUNT] {
        &self.k_state_words
    }

    pub const fn accumulator_mantissa_words(&self) -> &[u32; CHANNEL_COUNT] {
        &self.accumulator_mantissa_words
    }

    pub const fn accumulator_exponents(&self) -> &[i32; CHANNEL_COUNT] {
        &self.accumulator_exponents
    }

    pub const fn samples_in_window(&self) -> u16 {
        self.samples_in_window
    }

    fn validate(&self, parameters: &KFilterParameters) -> Result<(), EnergyWindowError> {
        let samples = usize::from(self.samples_in_window);
        if samples >= ENERGY_WINDOW_SAMPLES || samples % 64 != 0 {
            return Err(EnergyWindowError::StateInvalid);
        }
        for channel in 0..CHANNEL_COUNT {
            let mantissa = f32::from_bits(self.accumulator_mantissa_words[channel]);
            if !mantissa.is_finite()
                || mantissa < 0.0
                || !(0..=60).contains(&self.accumulator_exponents[channel])
            {
                return Err(EnergyWindowError::StateInvalid);
            }
            if samples == 0
                && (self.accumulator_mantissa_words[channel] != ZERO_WORD
                    || self.accumulator_exponents[channel] != COLD_EXPONENT)
            {
                return Err(EnergyWindowError::StateInvalid);
            }
            for stage in 0..STAGE_COUNT {
                for lane in 0..SIMD_LANES {
                    let word = self.k_state_words[channel][stage][lane];
                    let value = f32::from_bits(word);
                    let lower = f32::from_bits(parameters.lower_words[lane]);
                    let upper = f32::from_bits(parameters.upper_words[lane]);
                    if !value.is_finite() || value < lower || value > upper {
                        return Err(EnergyWindowError::StateInvalid);
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnergyRecord {
    mantissa_word: u32,
    exponent: i32,
}

impl EnergyRecord {
    pub const fn mantissa_word(self) -> u32 {
        self.mantissa_word
    }

    pub const fn exponent(self) -> i32 {
        self.exponent
    }
}

/// Linear proposal containing the complete next energy-window state.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct EnergyWindowProposal {
    next_state: EnergyWindowState,
    energy_records: Vec<EnergyRecord>,
}

impl EnergyWindowProposal {
    pub const fn next_state(&self) -> &EnergyWindowState {
        &self.next_state
    }

    pub fn energy_records(&self) -> &[EnergyRecord] {
        &self.energy_records
    }

    pub fn into_parts(self) -> (EnergyWindowState, Vec<EnergyRecord>) {
        (self.next_state, self.energy_records)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct KFilterParameters {
    matrix_words: [[[u32; SIMD_LANES]; MATRIX_ROWS]; STAGE_COUNT],
    cold_state_words: [[u32; SIMD_LANES]; STAGE_COUNT],
    lower_words: [u32; SIMD_LANES],
    upper_words: [u32; SIMD_LANES],
    noise_word: u32,
}

impl KFilterParameters {
    fn from_provider(provider: &KMatrixProvider) -> Result<Self, EnergyWindowError> {
        let flat_matrix = provider.matrix_words();
        let matrix_words = std::array::from_fn(|stage| {
            std::array::from_fn(|row| {
                std::array::from_fn(|lane| {
                    flat_matrix[stage * MATRIX_ROWS * SIMD_LANES + row * SIMD_LANES + lane]
                })
            })
        });
        let flat_cold = provider.cold_state_words();
        let cold_state_words = std::array::from_fn(|stage| {
            std::array::from_fn(|lane| flat_cold[stage * SIMD_LANES + lane])
        });
        let parameters = Self {
            matrix_words,
            cold_state_words,
            lower_words: *provider.lower_words(),
            upper_words: *provider.upper_words(),
            noise_word: provider.noise_word(),
        };
        parameters.validate()?;
        Ok(parameters)
    }

    fn validate(&self) -> Result<(), EnergyWindowError> {
        if self
            .matrix_words
            .iter()
            .flatten()
            .flatten()
            .chain(self.cold_state_words.iter().flatten())
            .chain(&self.lower_words)
            .chain(&self.upper_words)
            .chain([&self.noise_word])
            .any(|word| !f32::from_bits(*word).is_finite())
        {
            return Err(EnergyWindowError::ParameterInvalid);
        }
        for lane in 0..SIMD_LANES {
            if f32::from_bits(self.lower_words[lane]) > f32::from_bits(self.upper_words[lane]) {
                return Err(EnergyWindowError::ParameterInvalid);
            }
        }
        Ok(())
    }
}

struct KFilterSegment {
    #[cfg(test)]
    output_words: Vec<u32>,
    next_state_words: [[u32; SIMD_LANES]; STAGE_COUNT],
    power_sum_word: u32,
}

/// Stages one complete six-lane 1024-sample block.
///
/// A block may finish one or two 960-sample records depending on the prior
/// partial window. K-filter state continues across both the internal split and
/// completed energy-window boundaries.
pub fn stage_energy_window_block(
    before: &EnergyWindowState,
    input_rows: &[&[u32]; CHANNEL_COUNT],
    provider: &KMatrixProvider,
) -> Result<EnergyWindowProposal, EnergyWindowError> {
    let parameters = KFilterParameters::from_provider(provider)?;
    before.validate(&parameters)?;
    if input_rows
        .iter()
        .any(|row| row.len() != COMPLETE_BLOCK_SAMPLES)
    {
        return Err(EnergyWindowError::InputShape);
    }
    if input_rows
        .iter()
        .flat_map(|row| row.iter())
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(EnergyWindowError::InputNonFinite);
    }

    let mut next_state = before.clone();
    let mut energy_records = Vec::with_capacity(2);
    let mut block_cursor = 0;
    while block_cursor < COMPLETE_BLOCK_SAMPLES {
        let samples_in_window = usize::from(next_state.samples_in_window);
        let take =
            (ENERGY_WINDOW_SAMPLES - samples_in_window).min(COMPLETE_BLOCK_SAMPLES - block_cursor);
        if take == 0 || take % SEGMENT_ALIGNMENT != 0 {
            return Err(EnergyWindowError::StateInvalid);
        }

        for (channel, input_row) in input_rows.iter().enumerate() {
            let segment = process_k_filter_segment(
                &input_row[block_cursor..block_cursor + take],
                next_state.k_state_words[channel],
                &parameters,
            )?;
            next_state.k_state_words[channel] = segment.next_state_words;
            let (mantissa_word, exponent) = accumulate_normalized_energy(
                next_state.accumulator_mantissa_words[channel],
                next_state.accumulator_exponents[channel],
                segment.power_sum_word,
            )
            .map_err(map_energy_error)?;
            next_state.accumulator_mantissa_words[channel] = mantissa_word;
            next_state.accumulator_exponents[channel] = exponent;
        }

        block_cursor += take;
        next_state.samples_in_window = u16::try_from(samples_in_window + take)
            .map_err(|_| EnergyWindowError::NumericFailure)?;
        if usize::from(next_state.samples_in_window) == ENERGY_WINDOW_SAMPLES {
            let mut normalized_mantissas = [ZERO_WORD; CHANNEL_COUNT];
            let mut normalized_exponents = [0_i32; CHANNEL_COUNT];
            for channel in 0..CHANNEL_COUNT {
                let (mantissa, exponent) = normalize_energy_window(
                    next_state.accumulator_mantissa_words[channel],
                    next_state.accumulator_exponents[channel],
                    ENERGY_WINDOW_SAMPLES as u32,
                )
                .map_err(map_energy_error)?;
                normalized_mantissas[channel] = mantissa;
                normalized_exponents[channel] = exponent;
            }
            let (mantissa_word, exponent) = ordered_energy_reduction(
                &normalized_exponents,
                &CHANNEL_WEIGHTS,
                &normalized_mantissas,
            )
            .map_err(map_energy_error)?;
            energy_records.push(EnergyRecord {
                mantissa_word,
                exponent,
            });
            next_state.accumulator_mantissa_words = [ZERO_WORD; CHANNEL_COUNT];
            next_state.accumulator_exponents = [COLD_EXPONENT; CHANNEL_COUNT];
            next_state.samples_in_window = 0;
        }
    }
    next_state.validate(&parameters)?;
    Ok(EnergyWindowProposal {
        next_state,
        energy_records,
    })
}

fn process_k_filter_segment(
    input_words: &[u32],
    old_state_words: [[u32; SIMD_LANES]; STAGE_COUNT],
    parameters: &KFilterParameters,
) -> Result<KFilterSegment, EnergyWindowError> {
    if input_words.is_empty()
        || input_words.len() > COMPLETE_BLOCK_SAMPLES
        || input_words.len() % SEGMENT_ALIGNMENT != 0
        || input_words
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(EnergyWindowError::InputShape);
    }

    let mut output_words = Vec::with_capacity(input_words.len());
    for word in input_words {
        output_words.push(multiply_words(*word, EIGHTH_WORD).map_err(map_energy_error)?);
    }
    let mut next_state_words = old_state_words;
    for stage in 0..STAGE_COUNT {
        let mut previous = old_state_words[stage];
        for offset in (0..output_words.len()).step_by(SIMD_LANES) {
            let mut noisy_input = [ZERO_WORD; SIMD_LANES];
            for lane in 0..SIMD_LANES {
                noisy_input[lane] = add_words(output_words[offset + lane], parameters.noise_word)
                    .map_err(map_energy_error)?;
            }
            let mut accumulator = [ZERO_WORD; SIMD_LANES];
            for (lane, accumulator_word) in accumulator.iter_mut().enumerate() {
                *accumulator_word =
                    multiply_words(noisy_input[0], parameters.matrix_words[stage][0][lane])
                        .map_err(map_energy_error)?;
                for (row, word) in noisy_input.iter().enumerate().skip(1) {
                    let product = multiply_words(*word, parameters.matrix_words[stage][row][lane])
                        .map_err(map_energy_error)?;
                    *accumulator_word =
                        add_words(*accumulator_word, product).map_err(map_energy_error)?;
                }
                for (row, word) in previous.iter().enumerate() {
                    let product = multiply_words(
                        *word,
                        parameters.matrix_words[stage][row + SIMD_LANES][lane],
                    )
                    .map_err(map_energy_error)?;
                    *accumulator_word =
                        add_words(*accumulator_word, product).map_err(map_energy_error)?;
                }
            }
            output_words[offset..offset + SIMD_LANES].copy_from_slice(&accumulator);
            previous = [
                noisy_input[3],
                noisy_input[2],
                accumulator[3],
                accumulator[2],
            ];
        }
        next_state_words[stage] = std::array::from_fn(|lane| {
            clamp_word(
                previous[lane],
                parameters.lower_words[lane],
                parameters.upper_words[lane],
            )
        });
    }

    for word in &mut output_words {
        *word = multiply_words(*word, ONE_THIRTY_SECOND_WORD).map_err(map_energy_error)?;
    }
    let power_sum_word = ordered_power_sum(&output_words)?;
    Ok(KFilterSegment {
        #[cfg(test)]
        output_words,
        next_state_words,
        power_sum_word,
    })
}

fn ordered_power_sum(output_words: &[u32]) -> Result<u32, EnergyWindowError> {
    if output_words.is_empty() || output_words.len() % SEGMENT_ALIGNMENT != 0 {
        return Err(EnergyWindowError::InputShape);
    }
    let mut first = [ZERO_WORD; SIMD_LANES];
    let mut second = [ZERO_WORD; SIMD_LANES];
    for group in output_words.chunks_exact(SEGMENT_ALIGNMENT) {
        for lane in 0..SIMD_LANES {
            let first_square =
                multiply_words(group[lane], group[lane]).map_err(map_energy_error)?;
            first[lane] = add_words(first[lane], first_square).map_err(map_energy_error)?;
            let second_square = multiply_words(group[lane + SIMD_LANES], group[lane + SIMD_LANES])
                .map_err(map_energy_error)?;
            second[lane] = add_words(second[lane], second_square).map_err(map_energy_error)?;
        }
    }
    let mut total = [ZERO_WORD; SIMD_LANES];
    for lane in 0..SIMD_LANES {
        total[lane] = add_words(first[lane], second[lane]).map_err(map_energy_error)?;
    }
    let even = add_words(total[0], total[2]).map_err(map_energy_error)?;
    let odd = add_words(total[1], total[3]).map_err(map_energy_error)?;
    add_words(even, odd).map_err(map_energy_error)
}

fn clamp_word(value: u32, lower: u32, upper: u32) -> u32 {
    let value_float = f32::from_bits(value);
    let lower_bounded = if value_float < f32::from_bits(lower) {
        lower
    } else {
        value
    };
    if f32::from_bits(lower_bounded) > f32::from_bits(upper) {
        upper
    } else {
        lower_bounded
    }
}

fn map_energy_error(error: TruePeakError) -> EnergyWindowError {
    match error {
        TruePeakError::Input => EnergyWindowError::StateInvalid,
        TruePeakError::ParameterUnavailable => EnergyWindowError::ParameterInvalid,
        TruePeakError::NumericFailure => EnergyWindowError::NumericFailure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meter::k_matrix::{KMatrixMode, KMatrixProfile, generate_k_matrix};
    use serde_json::Value;
    use sha2::{Digest, Sha256};

    fn selected_provider() -> KMatrixProvider {
        generate_k_matrix(KMatrixProfile {
            sample_rate_hz: 48_000,
            mode: KMatrixMode::Modern,
            stage_count: 2,
        })
        .unwrap()
    }

    fn synthetic_parameters(provider: &KMatrixProvider) -> KFilterParameters {
        let matrix_words = std::array::from_fn(|stage| {
            std::array::from_fn(|row| {
                std::array::from_fn(|lane| {
                    let magnitude = ((stage + 1) * (row + 1) * (lane + 1)) as f32;
                    let sign = if (row + lane) % 2 == 0 {
                        1.0_f32
                    } else {
                        -1.0_f32
                    };
                    (sign * magnitude / 4_096.0_f32).to_bits()
                })
            })
        });
        let flat_cold = provider.cold_state_words();
        KFilterParameters {
            matrix_words,
            cold_state_words: std::array::from_fn(|stage| {
                std::array::from_fn(|lane| flat_cold[stage * SIMD_LANES + lane])
            }),
            lower_words: *provider.lower_words(),
            upper_words: *provider.upper_words(),
            noise_word: provider.noise_word(),
        }
    }

    fn recipe(samples: usize, last_sample_override: Option<f32>) -> Vec<u32> {
        let mut words = (0..samples)
            .map(|index| {
                let centered = i32::try_from(index % 17).unwrap() - 8;
                (centered as f32 / 16.0_f32).to_bits()
            })
            .collect::<Vec<_>>();
        if let Some(value) = last_sample_override {
            *words.last_mut().unwrap() = value.to_bits();
        }
        words
    }

    fn words_sha256(words: &[u32]) -> String {
        let mut bytes = Vec::with_capacity(words.len() * 4);
        for word in words {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        format!("{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn approved_parameterized_k_filter_sequence_matches_all_three_input_recipes() {
        let provider = selected_provider();
        let parameters = synthetic_parameters(&provider);
        let mut state = parameters.cold_state_words;
        let fixture: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/energy_window/parameterized_filter_sequences.json"
        ))
        .unwrap();

        for (expected_block, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
            assert_eq!(
                case["block_index"].as_u64().unwrap(),
                u64::try_from(expected_block).unwrap()
            );
            let samples = usize::try_from(case["samples"].as_u64().unwrap()).unwrap();
            let override_value = case["last_sample_override"]
                .as_f64()
                .map(|value| value as f32);
            let expected_state = std::array::from_fn(|stage| {
                std::array::from_fn(|lane| {
                    u32::try_from(
                        case["expected_next_state_words"][stage][lane]
                            .as_u64()
                            .unwrap(),
                    )
                    .unwrap()
                })
            });
            let input = recipe(samples, override_value);
            let result = process_k_filter_segment(&input, state, &parameters).unwrap();
            assert_eq!(
                words_sha256(&result.output_words),
                case["expected_output_sha256"].as_str().unwrap()
            );
            assert_eq!(result.next_state_words, expected_state);
            assert_eq!(
                result.power_sum_word,
                u32::try_from(case["expected_power_sum_word"].as_u64().unwrap()).unwrap()
            );
            state = result.next_state_words;
        }
    }

    #[test]
    fn complete_blocks_cross_960_boundaries_without_resetting_k_state() {
        let provider = selected_provider();
        let cold = EnergyWindowState::cold(&provider).unwrap();
        let rows = std::array::from_fn::<_, CHANNEL_COUNT, _>(|channel| {
            (0..COMPLETE_BLOCK_SAMPLES)
                .map(|sample| {
                    let numerator = ((sample + channel * 3) % 29) as f32 - 14.0;
                    (numerator / 32_768.0_f32).to_bits()
                })
                .collect::<Vec<_>>()
        });
        let views = std::array::from_fn(|channel| rows[channel].as_slice());

        let first = stage_energy_window_block(&cold, &views, &provider).unwrap();
        assert_eq!(first.energy_records().len(), 1);
        assert_eq!(first.next_state().samples_in_window(), 64);
        assert_eq!(cold.samples_in_window(), 0);
        assert_ne!(first.next_state().k_state_words(), cold.k_state_words());

        let second = stage_energy_window_block(first.next_state(), &views, &provider).unwrap();
        assert_eq!(second.energy_records().len(), 1);
        assert_eq!(second.next_state().samples_in_window(), 128);
        assert_ne!(
            second.next_state().k_state_words(),
            first.next_state().k_state_words()
        );
        assert!(second.energy_records().iter().all(|record| {
            f32::from_bits(record.mantissa_word()).is_finite()
                && f32::from_bits(record.mantissa_word()) >= 0.0
                && (0..=60).contains(&record.exponent())
        }));
    }

    #[test]
    fn invalid_shape_and_nonfinite_input_leave_before_state_unchanged() {
        let provider = selected_provider();
        let before = EnergyWindowState::cold(&provider).unwrap();
        let before_copy = before.clone();
        let short_rows = std::array::from_fn::<_, CHANNEL_COUNT, _>(|_| {
            vec![ZERO_WORD; COMPLETE_BLOCK_SAMPLES - 1]
        });
        let short_views = std::array::from_fn(|channel| short_rows[channel].as_slice());
        assert_eq!(
            stage_energy_window_block(&before, &short_views, &provider),
            Err(EnergyWindowError::InputShape)
        );

        let mut nonfinite_rows =
            std::array::from_fn::<_, CHANNEL_COUNT, _>(|_| vec![ZERO_WORD; COMPLETE_BLOCK_SAMPLES]);
        nonfinite_rows[4][711] = f32::NAN.to_bits();
        let nonfinite_views = std::array::from_fn(|channel| nonfinite_rows[channel].as_slice());
        assert_eq!(
            stage_energy_window_block(&before, &nonfinite_views, &provider),
            Err(EnergyWindowError::InputNonFinite)
        );
        assert_eq!(before, before_copy);
    }
}
