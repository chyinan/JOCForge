// pattern: Functional Core
//! F06I synthetic integrated-meter state, event, histogram, and absolute-gate transition.

use super::true_peak::{
    TruePeakError, add_words, divide_words, energy_metric, multiply_words,
    power_of_two as f32_power_of_two, selected_log_constants, shift_word, subtract_words,
    validate_approved_log_constants,
};
use crate::numeric::FiniteF32;
use sha2::{Digest, Sha256};

const PERIOD: u32 = 5;
const GROUP_COUNT: usize = 4;
const GROUP_WIDTH: usize = 5;
const RECENT_COUNT: usize = GROUP_COUNT * GROUP_WIDTH;
const BIN_COUNT: usize = 3040;
const BIN_MIDPOINT: usize = 1520;
const MAX_ABSOLUTE_COUNT: u32 = 0x7fff_ffff;
const M_WORD: u32 = 0x4f00_0000;
const J_WORD: u32 = 0x3000_0000;
const ZERO_WORD: u32 = 0;
const HALF_WORD: u32 = 0x3f00_0000;
const FLOOR_WORD: u32 = 0xbf70_0000;
const ABSOLUTE_GATE_WORD: u32 = 0xbf0c_0000;
const ABSOLUTE_THRESHOLD_OFFSET_WORD: u32 = 0xbda0_0000;
const BIN_OFFSET_WORD: u32 = 0xbf34_0000;
const BIN_SCALE_WORD: u32 = 0x3f00_0000;
const BIN_STEP_WORD: u32 = 0x3780_0000;
const BIN_BIAS_WORD: u32 = 0x3080_0000;
const STATE_HEADER: &[u8; 8] = b"MINTG1\0\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegratedMeterError {
    InvalidInput,
    InvalidState,
    ParameterUnavailable,
    NumericFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedMeterParameters {
    period: u32,
    log_constants: [u32; 11],
}

impl IntegratedMeterParameters {
    /// Loads the selected 48 kHz F06I profile and verifies its generated event period.
    pub fn selected() -> Result<Self, IntegratedMeterError> {
        let log_constants = selected_log_constants().map_err(map_numeric_error)?;
        Self::from_log_constants(log_constants)
    }

    /// Constructs the selected F06I parameter set from the approved direct log asset.
    pub fn from_approved_log_asset(bytes: &[u8]) -> Result<Self, IntegratedMeterError> {
        let log_constants = validate_approved_log_constants(bytes).map_err(map_numeric_error)?;
        Self::from_log_constants(log_constants)
    }

    fn from_log_constants(log_constants: [u32; 11]) -> Result<Self, IntegratedMeterError> {
        let period = selected_period()?;
        if period != PERIOD {
            return Err(IntegratedMeterError::ParameterUnavailable);
        }
        Ok(Self {
            period,
            log_constants,
        })
    }

    pub const fn period(&self) -> u32 {
        self.period
    }

    pub const fn log_words(&self) -> &[u32; 11] {
        &self.log_constants
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegratedMeterInput {
    pub mantissa_word: u32,
    pub exponent: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedMeterState {
    period: u32,
    counter: u32,
    recent_mantissas: [u32; RECENT_COUNT],
    recent_exponents: [i32; RECENT_COUNT],
    counts: [u32; BIN_COUNT],
    bin_means: [u32; BIN_COUNT],
    absolute_count: u32,
    absolute_mean_word: u32,
    absolute_exponent: i32,
    result_word: u32,
}

impl IntegratedMeterState {
    pub fn cold() -> Self {
        Self {
            period: PERIOD,
            counter: 0,
            recent_mantissas: [ZERO_WORD; RECENT_COUNT],
            recent_exponents: [0; RECENT_COUNT],
            counts: [0; BIN_COUNT],
            bin_means: [ZERO_WORD; BIN_COUNT],
            absolute_count: 0,
            absolute_mean_word: ZERO_WORD,
            absolute_exponent: 48,
            result_word: FLOOR_WORD,
        }
    }

    pub const fn period(&self) -> u32 {
        self.period
    }

    pub const fn counter(&self) -> u32 {
        self.counter
    }

    pub const fn result_word(&self) -> u32 {
        self.result_word
    }

    pub const fn absolute_count(&self) -> u32 {
        self.absolute_count
    }

    pub fn histogram_samples(&self) -> u64 {
        self.counts.iter().map(|count| u64::from(*count)).sum()
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(24_512);
        bytes.extend_from_slice(STATE_HEADER);
        bytes.extend_from_slice(&self.period.to_le_bytes());
        bytes.extend_from_slice(&self.counter.to_le_bytes());
        for word in self.recent_mantissas {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for exponent in self.recent_exponents {
            bytes.extend_from_slice(&exponent.to_le_bytes());
        }
        for count in self.counts {
            bytes.extend_from_slice(&count.to_le_bytes());
        }
        for word in self.bin_means {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(&self.absolute_count.to_le_bytes());
        bytes.extend_from_slice(&self.absolute_mean_word.to_le_bytes());
        bytes.extend_from_slice(&self.absolute_exponent.to_le_bytes());
        bytes.extend_from_slice(&self.result_word.to_le_bytes());
        bytes
    }

    pub fn canonical_sha256(&self) -> String {
        format!("{:x}", Sha256::digest(self.canonical_bytes()))
    }

    /// Explicit synthetic state patch used to reproduce an approved test state.
    #[must_use]
    pub fn with_counts_fill(mut self, value: u32) -> Self {
        self.counts.fill(value);
        self
    }

    /// Explicit synthetic state replacement for exercising the counter boundary.
    #[must_use]
    pub fn with_counter(mut self, value: u32) -> Self {
        self.counter = value;
        self
    }

    /// Explicit synthetic state replacement for exercising selected-profile validation.
    #[must_use]
    pub fn with_period(mut self, value: u32) -> Self {
        self.period = value;
        self
    }

    /// Explicit synthetic state patch used to reproduce an approved test state.
    #[must_use]
    pub fn with_absolute_count(mut self, value: u32) -> Self {
        self.absolute_count = value;
        self
    }

    /// Explicit synthetic state patch used to reproduce an approved test state.
    #[must_use]
    pub fn with_absolute_mean_word(mut self, value: u32) -> Self {
        self.absolute_mean_word = value;
        self
    }

    /// Explicit synthetic state patch used to reproduce an approved test state.
    #[must_use]
    pub fn with_absolute_exponent(mut self, value: i32) -> Self {
        self.absolute_exponent = value;
        self
    }

    fn validate(&self, expected_period: u32) -> Result<(), IntegratedMeterError> {
        if self.period != expected_period
            || self.counter >= 4 * self.period
            || self.absolute_count > MAX_ABSOLUTE_COUNT
        {
            return Err(IntegratedMeterError::InvalidState);
        }
        if self
            .recent_mantissas
            .iter()
            .chain(&self.bin_means)
            .chain([&self.absolute_mean_word])
            .any(|word| {
                let value = f32::from_bits(*word);
                !value.is_finite() || value < 0.0
            })
            || !f32::from_bits(self.result_word).is_finite()
        {
            return Err(IntegratedMeterError::InvalidState);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegratedMeterEvent {
    bucket: u16,
    counts_halved: bool,
    absolute_admitted: bool,
    threshold_word: u32,
    begin: u16,
    total: u64,
}

impl IntegratedMeterEvent {
    pub const fn bucket(&self) -> u16 {
        self.bucket
    }

    pub const fn counts_halved(&self) -> bool {
        self.counts_halved
    }

    pub const fn absolute_admitted(&self) -> bool {
        self.absolute_admitted
    }

    pub const fn threshold_word(&self) -> u32 {
        self.threshold_word
    }

    pub const fn begin(&self) -> u16 {
        self.begin
    }

    pub const fn total(&self) -> u64 {
        self.total
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedMeterStep {
    next_state: IntegratedMeterState,
    event: Option<IntegratedMeterEvent>,
}

impl IntegratedMeterStep {
    pub const fn next_state(&self) -> &IntegratedMeterState {
        &self.next_state
    }

    pub const fn event(&self) -> Option<&IntegratedMeterEvent> {
        self.event.as_ref()
    }

    pub fn into_next_state(self) -> IntegratedMeterState {
        self.next_state
    }
}

/// Processes one explicit normalized energy pair and returns its complete staged state.
pub fn process_integrated_meter(
    old: &IntegratedMeterState,
    input: IntegratedMeterInput,
    parameters: &IntegratedMeterParameters,
) -> Result<IntegratedMeterStep, IntegratedMeterError> {
    old.validate(parameters.period)?;
    let mantissa = f32::from_bits(input.mantissa_word);
    if !mantissa.is_finite() || mantissa < 0.0 || !(0..=30).contains(&input.exponent) {
        return Err(IntegratedMeterError::InvalidInput);
    }

    let next_counter = old
        .counter
        .checked_add(1)
        .ok_or(IntegratedMeterError::NumericFailure)?;
    let slot = usize::try_from(old.counter % parameters.period)
        .map_err(|_| IntegratedMeterError::InvalidState)?;
    let mut next_state = old.clone();
    let insertion = (GROUP_COUNT - 1) * GROUP_WIDTH + slot;
    next_state.recent_mantissas[insertion] = input.mantissa_word;
    next_state.recent_exponents[insertion] = input.exponent;
    next_state.counter = next_counter;

    let event = if next_counter == (GROUP_COUNT as u32) * parameters.period {
        Some(process_event(&mut next_state, parameters)?)
    } else {
        None
    };

    if next_state.counter % parameters.period == 0 {
        shift_recent_groups(&mut next_state);
        if next_state.counter == (GROUP_COUNT as u32) * parameters.period {
            next_state.counter -= parameters.period;
        }
    }
    next_state.validate(parameters.period)?;
    Ok(IntegratedMeterStep { next_state, event })
}

pub fn integrated_histogram_bin(metric_word: u32) -> Result<u16, IntegratedMeterError> {
    let metric = f32::from_bits(metric_word);
    if !metric.is_finite() {
        return Err(IntegratedMeterError::InvalidInput);
    }
    let shifted = subtract(metric_word, BIN_OFFSET_WORD)?;
    let half = multiply(shifted, BIN_SCALE_WORD)?;
    let fine = multiply(half, BIN_STEP_WORD)?;
    let value = add(fine, BIN_BIAS_WORD)?;
    let value = f32::from_bits(value);
    let index = if value < 1.0 {
        let scaled = multiply(word_from_f32(value)?, M_WORD)?;
        let scaled_value = f32::from_bits(scaled);
        let lower = -f32::from_bits(M_WORD);
        let bounded = if scaled_value < lower {
            lower.to_bits()
        } else {
            scaled
        };
        round_to_i64(bounded)?
    } else {
        i64::from(i32::MAX)
    };
    let shifted_index = index >> 2;
    let maximum_index =
        i64::try_from(BIN_COUNT - 1).map_err(|_| IntegratedMeterError::NumericFailure)?;
    let bounded_index = shifted_index.clamp(0, maximum_index);
    u16::try_from(bounded_index).map_err(|_| IntegratedMeterError::NumericFailure)
}

fn process_event(
    state: &mut IntegratedMeterState,
    parameters: &IntegratedMeterParameters,
) -> Result<IntegratedMeterEvent, IntegratedMeterError> {
    let mut mantissa = ZERO_WORD;
    let mut exponent = 48_i32;
    for group in 0..GROUP_COUNT {
        for position in 0..GROUP_WIDTH {
            let index = group * GROUP_WIDTH + position;
            (mantissa, exponent) = merge_normalized_energy(
                mantissa,
                exponent,
                state.recent_mantissas[index],
                state.recent_exponents[index],
            )?;
        }
    }

    let window_samples =
        i32::try_from(4 * parameters.period).map_err(|_| IntegratedMeterError::NumericFailure)?;
    let window_scale = multiply(divide(M_WORD, word_from_i32(window_samples)?)?, J_WORD)?;
    mantissa = multiply(mantissa, window_scale)?;
    let metric = energy_metric(
        mantissa,
        4_i32
            .checked_sub(exponent)
            .ok_or(IntegratedMeterError::NumericFailure)?,
        &parameters.log_constants,
    )
    .map_err(map_numeric_error)?;
    let bucket = integrated_histogram_bin(metric)?;
    let bucket_index = usize::from(bucket);
    let counts_halved = state.counts[bucket_index] > 32_767;
    if counts_halved {
        for count in &mut state.counts {
            *count >>= 1;
        }
    }
    state.counts[bucket_index] = state.counts[bucket_index]
        .checked_add(1)
        .ok_or(IntegratedMeterError::NumericFailure)?;
    update_bin_mean(state, bucket_index, mantissa, exponent)?;

    let mut absolute_admitted = false;
    if f32::from_bits(metric) > f32::from_bits(ABSOLUTE_GATE_WORD) {
        absolute_admitted = true;
        update_absolute_mean(state, mantissa, exponent)?;
    }
    if state.absolute_count == MAX_ABSOLUTE_COUNT {
        state.absolute_count = MAX_ABSOLUTE_COUNT >> 1;
    }

    let absolute_metric = energy_metric(
        state.absolute_mean_word,
        state
            .absolute_exponent
            .checked_neg()
            .ok_or(IntegratedMeterError::NumericFailure)?,
        &parameters.log_constants,
    )
    .map_err(map_numeric_error)?;
    let threshold_word = max_word(
        ABSOLUTE_GATE_WORD,
        add(absolute_metric, ABSOLUTE_THRESHOLD_OFFSET_WORD)?,
    );

    let mut begin = usize::from(integrated_histogram_bin(threshold_word)?);
    if begin < BIN_COUNT {
        let metric_offset = if begin < BIN_MIDPOINT { -1 } else { 3 };
        let start_metric = energy_metric(
            state.bin_means[begin],
            metric_offset,
            &parameters.log_constants,
        )
        .map_err(map_numeric_error)?;
        if f32::from_bits(start_metric) < f32::from_bits(threshold_word) {
            begin += 1;
        }
    }
    let total: u64 = state
        .counts
        .iter()
        .skip(begin.min(BIN_COUNT))
        .map(|count| u64::from(*count))
        .sum();
    let result = if total == 0 {
        FLOOR_WORD
    } else {
        integrated_loudness_result(state, begin, total, &parameters.log_constants)?
    };
    state.result_word = result;

    Ok(IntegratedMeterEvent {
        bucket,
        counts_halved,
        absolute_admitted,
        threshold_word,
        begin: u16::try_from(begin).map_err(|_| IntegratedMeterError::NumericFailure)?,
        total,
    })
}

fn merge_normalized_energy(
    mantissa: u32,
    exponent: i32,
    next_mantissa: u32,
    next_exponent: i32,
) -> Result<(u32, i32), IntegratedMeterError> {
    let difference = exponent
        .checked_sub(next_exponent)
        .ok_or(IntegratedMeterError::NumericFailure)?;
    let (value, merged_exponent) = if difference <= 0 {
        let retained_exponent = difference
            .checked_sub(1)
            .ok_or(IntegratedMeterError::NumericFailure)?;
        let term = if retained_exponent > -32 {
            multiply(power_of_two(retained_exponent)?, next_mantissa)?
        } else {
            ZERO_WORD
        };
        let combined = add(multiply(HALF_WORD, mantissa)?, term)?;
        (
            combined,
            exponent
                .checked_sub(1)
                .ok_or(IntegratedMeterError::NumericFailure)?,
        )
    } else {
        let retained_exponent = difference
            .checked_neg()
            .and_then(|value| value.checked_sub(1))
            .ok_or(IntegratedMeterError::NumericFailure)?;
        let term = if retained_exponent > -32 {
            multiply(power_of_two(retained_exponent)?, mantissa)?
        } else {
            ZERO_WORD
        };
        let combined = add(term, multiply(HALF_WORD, next_mantissa)?)?;
        (
            combined,
            next_exponent
                .checked_sub(1)
                .ok_or(IntegratedMeterError::NumericFailure)?,
        )
    };

    let normalization_shift = (30_i32
        .checked_sub(merged_exponent)
        .ok_or(IntegratedMeterError::NumericFailure)?)
    .min(shift_word(value));
    let normalized = multiply(value, power_of_two(normalization_shift)?)?;
    let output_exponent = merged_exponent
        .checked_add(normalization_shift)
        .ok_or(IntegratedMeterError::NumericFailure)?;
    Ok((normalized, output_exponent))
}

fn update_bin_mean(
    state: &mut IntegratedMeterState,
    bucket: usize,
    mantissa: u32,
    exponent: i32,
) -> Result<(), IntegratedMeterError> {
    let mean_exponent: i32 = if bucket >= BIN_MIDPOINT { 1 } else { 5 };
    let scale_exponent = mean_exponent
        .checked_sub(exponent)
        .ok_or(IntegratedMeterError::NumericFailure)?;
    let scale = power_of_two(scale_exponent)?;
    let scaled = multiply(scale, mantissa)?;
    let difference = subtract(scaled, state.bin_means[bucket])?;
    let count_word = word_from_u32(state.counts[bucket])?;
    let inverse_count = divide(M_WORD, count_word)?;
    let delta = multiply(inverse_count, difference)?;
    state.bin_means[bucket] = add(multiply(delta, J_WORD)?, state.bin_means[bucket])?;
    Ok(())
}

fn update_absolute_mean(
    state: &mut IntegratedMeterState,
    mantissa: u32,
    exponent: i32,
) -> Result<(), IntegratedMeterError> {
    let count = state
        .absolute_count
        .checked_add(1)
        .ok_or(IntegratedMeterError::NumericFailure)?;
    let count_word = word_from_u32(count)?;
    let q = multiply(divide(M_WORD, count_word)?, J_WORD)?;
    let mut shift = shift_word(mantissa);
    let mut adjusted_exponent = shift
        .checked_add(exponent)
        .and_then(|value| value.checked_sub(4))
        .ok_or(IntegratedMeterError::NumericFailure)?;
    if adjusted_exponent > 31 {
        adjusted_exponent = 31;
        shift = 35_i32
            .checked_sub(exponent)
            .ok_or(IntegratedMeterError::NumericFailure)?;
    }
    let normalized = multiply(mantissa, power_of_two(shift)?)?;
    let old_mean = state.absolute_mean_word;
    let old_exponent = state.absolute_exponent;
    let (new_mean, new_exponent) = if adjusted_exponent <= old_exponent {
        let distance = old_exponent
            .checked_sub(adjusted_exponent)
            .ok_or(IntegratedMeterError::NumericFailure)?;
        let current = if distance < 32 {
            subtract(
                normalized,
                multiply(
                    power_of_two(
                        adjusted_exponent
                            .checked_sub(old_exponent)
                            .ok_or(IntegratedMeterError::NumericFailure)?,
                    )?,
                    old_mean,
                )?,
            )?
        } else {
            normalized
        };
        let delta = multiply(current, q)?;
        let previous_distance = distance
            .checked_add(1)
            .ok_or(IntegratedMeterError::NumericFailure)?;
        let previous = if previous_distance < 32 {
            multiply(
                power_of_two(
                    adjusted_exponent
                        .checked_sub(old_exponent)
                        .and_then(|value| value.checked_sub(1))
                        .ok_or(IntegratedMeterError::NumericFailure)?,
                )?,
                old_mean,
            )?
        } else {
            ZERO_WORD
        };
        (
            add(multiply(HALF_WORD, delta)?, previous)?,
            adjusted_exponent
                .checked_sub(1)
                .ok_or(IntegratedMeterError::NumericFailure)?,
        )
    } else {
        let distance = adjusted_exponent
            .checked_sub(old_exponent)
            .ok_or(IntegratedMeterError::NumericFailure)?;
        let delta = if distance < 32 {
            subtract(
                multiply(
                    power_of_two(
                        old_exponent
                            .checked_sub(adjusted_exponent)
                            .ok_or(IntegratedMeterError::NumericFailure)?,
                    )?,
                    normalized,
                )?,
                old_mean,
            )?
        } else {
            negate(old_mean)?
        };
        let updated = multiply(multiply(delta, q)?, HALF_WORD)?;
        let retained = multiply(HALF_WORD, old_mean)?;
        (
            add(updated, retained)?,
            old_exponent
                .checked_sub(1)
                .ok_or(IntegratedMeterError::NumericFailure)?,
        )
    };

    let renormalization_shift = shift_word(new_mean);
    let normalized_exponent = renormalization_shift
        .checked_add(new_exponent)
        .ok_or(IntegratedMeterError::NumericFailure)?;
    if normalized_exponent < 31 {
        state.absolute_mean_word = multiply(new_mean, power_of_two(renormalization_shift)?)?;
        state.absolute_exponent = normalized_exponent;
    } else {
        state.absolute_mean_word = new_mean;
        state.absolute_exponent = new_exponent;
    }
    state.absolute_count = count;
    Ok(())
}

fn integrated_loudness_result(
    state: &IntegratedMeterState,
    begin: usize,
    total: u64,
    log_constants: &[u32; 11],
) -> Result<u32, IntegratedMeterError> {
    let low = if begin < BIN_MIDPOINT {
        histogram_sum(state, begin, BIN_MIDPOINT, total)?
    } else {
        ZERO_WORD
    };
    let low_shift = shift_word(low);
    let low_normalized = multiply(low, power_of_two(low_shift)?)?;
    let low_exponent = low_shift
        .checked_add(4)
        .ok_or(IntegratedMeterError::NumericFailure)?;

    let high_begin = begin.max(BIN_MIDPOINT);
    let high = histogram_sum(state, high_begin, BIN_COUNT, total)?;
    let high_shift = shift_word(high);
    let (combined, exponent) = if high_shift >= low_exponent {
        let value = add(
            multiply(HALF_WORD, low_normalized)?,
            multiply(power_of_two(low_exponent - 1)?, high)?,
        )?;
        (value, low_exponent - 1)
    } else {
        let distance = low_exponent
            .checked_sub(high_shift)
            .and_then(|value| value.checked_add(1))
            .ok_or(IntegratedMeterError::NumericFailure)?;
        let previous = if distance < 32 {
            multiply(
                power_of_two(
                    distance
                        .checked_neg()
                        .ok_or(IntegratedMeterError::NumericFailure)?,
                )?,
                low_normalized,
            )?
        } else {
            ZERO_WORD
        };
        (
            add(previous, multiply(power_of_two(high_shift - 1)?, high)?)?,
            high_shift - 1,
        )
    };
    let offset = 3_i32
        .checked_sub(exponent)
        .ok_or(IntegratedMeterError::NumericFailure)?;
    energy_metric(combined, offset, log_constants).map_err(map_numeric_error)
}

fn histogram_sum(
    state: &IntegratedMeterState,
    start: usize,
    end: usize,
    total: u64,
) -> Result<u32, IntegratedMeterError> {
    let mut value = ZERO_WORD;
    if start >= end {
        return Ok(value);
    }
    let total_word = word_from_u64(total)?;
    let reciprocal = divide(M_WORD, total_word)?;
    for index in start..end {
        let count = word_from_u32(state.counts[index])?;
        let term = multiply(
            multiply(multiply(count, reciprocal)?, J_WORD)?,
            state.bin_means[index],
        )?;
        value = add(value, term)?;
    }
    Ok(value)
}

fn shift_recent_groups(state: &mut IntegratedMeterState) {
    let old_mantissas = state.recent_mantissas;
    let old_exponents = state.recent_exponents;
    for group in 0..GROUP_COUNT - 1 {
        let destination = group * GROUP_WIDTH;
        let source = (group + 1) * GROUP_WIDTH;
        state.recent_mantissas[destination..destination + GROUP_WIDTH]
            .copy_from_slice(&old_mantissas[source..source + GROUP_WIDTH]);
        state.recent_exponents[destination..destination + GROUP_WIDTH]
            .copy_from_slice(&old_exponents[source..source + GROUP_WIDTH]);
    }
    state.recent_mantissas[(GROUP_COUNT - 1) * GROUP_WIDTH..].fill(ZERO_WORD);
    state.recent_exponents[(GROUP_COUNT - 1) * GROUP_WIDTH..].fill(48);
}

fn selected_period() -> Result<u32, IntegratedMeterError> {
    let source_samples = 48_000_i32
        .checked_shl(13)
        .ok_or(IntegratedMeterError::NumericFailure)?;
    let first = multiply(
        multiply(word_from_i32(source_samples)?, J_WORD)?,
        divide(word_from_f32(1.0)?, word_from_f32(80.0)?)?,
    )?;
    let small_scale = divide(J_WORD, word_from_f32(50.0)?)?;
    let small = multiply(word_from_f32(48_000.0)?, small_scale)?;
    let samples = round_to_i64(multiply(small, M_WORD)?)?;
    if samples <= 0 {
        return Err(IntegratedMeterError::NumericFailure);
    }
    let ratio = multiply(
        multiply(divide(M_WORD, word_from_i64(samples)?)?, J_WORD)?,
        first,
    )?;
    let ratio = add(ratio, 0x3380_0000)?;
    let frames = round_to_i64(multiply(ratio, M_WORD)?)? >> 8;
    let four_frames = frames
        .checked_mul(4)
        .ok_or(IntegratedMeterError::NumericFailure)?;
    let v = add(
        multiply(word_from_i64(four_frames)?, 0x2f00_0000)?,
        0x3080_0000,
    )?;
    u32::try_from(round_to_i64(multiply(v, M_WORD)?)? >> 2)
        .map_err(|_| IntegratedMeterError::NumericFailure)
}

fn round_to_i64(word: u32) -> Result<i64, IntegratedMeterError> {
    let value = f32::from_bits(word);
    if !value.is_finite() {
        return Err(IntegratedMeterError::NumericFailure);
    }
    let rounded = f64::from(value).round_ties_even();
    if rounded < i64::MIN as f64 || rounded >= 9_223_372_036_854_775_808.0 {
        return Err(IntegratedMeterError::NumericFailure);
    }
    Ok(rounded as i64)
}

fn word_from_i32(value: i32) -> Result<u32, IntegratedMeterError> {
    word_from_f32(value as f32)
}

fn word_from_i64(value: i64) -> Result<u32, IntegratedMeterError> {
    word_from_f32(value as f32)
}

fn word_from_u32(value: u32) -> Result<u32, IntegratedMeterError> {
    word_from_f32(value as f32)
}

fn word_from_u64(value: u64) -> Result<u32, IntegratedMeterError> {
    word_from_f32(value as f32)
}

fn word_from_f32(value: f32) -> Result<u32, IntegratedMeterError> {
    FiniteF32::new(value)
        .map(FiniteF32::to_bits)
        .map_err(|_| IntegratedMeterError::NumericFailure)
}

fn finite(word: u32) -> Result<FiniteF32, IntegratedMeterError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| IntegratedMeterError::NumericFailure)
}

fn add(left: u32, right: u32) -> Result<u32, IntegratedMeterError> {
    add_words(left, right).map_err(map_numeric_error)
}

fn subtract(left: u32, right: u32) -> Result<u32, IntegratedMeterError> {
    subtract_words(left, right).map_err(map_numeric_error)
}

fn multiply(left: u32, right: u32) -> Result<u32, IntegratedMeterError> {
    multiply_words(left, right).map_err(map_numeric_error)
}

fn divide(numerator: u32, denominator: u32) -> Result<u32, IntegratedMeterError> {
    divide_words(numerator, denominator).map_err(map_numeric_error)
}

fn power_of_two(exponent: i32) -> Result<u32, IntegratedMeterError> {
    f32_power_of_two(exponent).map_err(map_numeric_error)
}

fn negate(word: u32) -> Result<u32, IntegratedMeterError> {
    let value = finite(word)?;
    word_from_f32(-value.value())
}

fn max_word(left: u32, right: u32) -> u32 {
    if f32::from_bits(left) < f32::from_bits(right) {
        right
    } else {
        left
    }
}

fn map_numeric_error(error: TruePeakError) -> IntegratedMeterError {
    match error {
        TruePeakError::Input => IntegratedMeterError::InvalidInput,
        TruePeakError::ParameterUnavailable => IntegratedMeterError::ParameterUnavailable,
        TruePeakError::NumericFailure => IntegratedMeterError::NumericFailure,
    }
}
