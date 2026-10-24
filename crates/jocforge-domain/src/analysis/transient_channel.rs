// pattern: Functional Core
//! One-lane F07.2/F07.3 selected D05 transient block transition.

use super::mean_square::{MeanSquareError, checked_mean_square_256};
use super::transient_parameters::TransientParameters;
use crate::numeric::{FiniteF32, r32_add, r32_mul};

/// The selected F07 transient producer consumes one 256-sample block per lane.
pub const TRANSIENT_BLOCK_SAMPLES: usize = 256;
const PEAK_STATISTICS_WORDS: usize = 18;
const QUEUE_LENGTH: usize = 4;
const STATE_SERIALIZED_BYTES: usize = 124;
const QUARTER_SAMPLES: usize = 64;
const POSITIVE_ZERO_BITS: u32 = 0;
const ONE_BITS: u32 = 0x3f80_0000;
const HALF_BITS: u32 = 0x3f00_0000;
const ABS_MAGNITUDE_MASK: u32 = 0x7fff_ffff;

/// Selected one-lane transient state carried between caller-ordered blocks.
#[derive(Debug, Eq, PartialEq)]
pub struct TransientChannelState {
    two_history: [[u32; 2]; 2],
    three_history: [[u32; 2]; 3],
    peak_statistics: [u32; PEAK_STATISTICS_WORDS],
    previous_mean: u32,
    short_queue: [u8; QUEUE_LENGTH],
    other_queue: [u8; QUEUE_LENGTH],
}

impl TransientChannelState {
    /// Creates the F07 selected-lane cold state.
    pub const fn cold() -> Self {
        Self {
            two_history: [[POSITIVE_ZERO_BITS; 2]; 2],
            three_history: [[POSITIVE_ZERO_BITS; 2]; 3],
            peak_statistics: [POSITIVE_ZERO_BITS; PEAK_STATISTICS_WORDS],
            previous_mean: POSITIVE_ZERO_BITS,
            short_queue: [0; QUEUE_LENGTH],
            other_queue: [0; QUEUE_LENGTH],
        }
    }

    /// Serializes 29 F32 words followed by the short and other U8 queues.
    pub fn canonical_bytes(&self) -> [u8; STATE_SERIALIZED_BYTES] {
        let mut bytes = [0; STATE_SERIALIZED_BYTES];
        let mut offset = 0;
        for section in self.two_history {
            for word in section {
                append_word(&mut bytes, &mut offset, word);
            }
        }
        for section in self.three_history {
            for word in section {
                append_word(&mut bytes, &mut offset, word);
            }
        }
        for word in self.peak_statistics {
            append_word(&mut bytes, &mut offset, word);
        }
        append_word(&mut bytes, &mut offset, self.previous_mean);
        bytes[offset..offset + QUEUE_LENGTH].copy_from_slice(&self.short_queue);
        offset += QUEUE_LENGTH;
        bytes[offset..offset + QUEUE_LENGTH].copy_from_slice(&self.other_queue);
        offset += QUEUE_LENGTH;
        debug_assert_eq!(offset, STATE_SERIALIZED_BYTES);
        bytes
    }
}

/// Errors returned by the selected one-lane F07 block transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransientChannelError {
    InputError,
    NumericFailure,
}

impl TransientChannelError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputError => "F07_INPUT_ERROR",
            Self::NumericFailure => "F07_NUMERIC_FAILURE",
        }
    }
}

/// The selected F07.2/F07.3 outputs for one channel block.
#[derive(Debug, Eq, PartialEq)]
pub struct TransientBlockOutput {
    filtered_words: [u32; TRANSIENT_BLOCK_SAMPLES],
    mean_word: u32,
    peak_positions: [u8; 4],
    short_source: u8,
    other_source: u8,
    short_consumed: u8,
    other_consumed: u8,
    short_tiers: [u8; 3],
    other_tiers: [u8; 3],
    next_state: TransientChannelState,
}

impl TransientBlockOutput {
    pub const fn filtered_words(&self) -> &[u32; TRANSIENT_BLOCK_SAMPLES] {
        &self.filtered_words
    }

    pub const fn mean_word(&self) -> u32 {
        self.mean_word
    }

    pub const fn peak_positions(&self) -> &[u8; 4] {
        &self.peak_positions
    }

    pub const fn short_source(&self) -> u8 {
        self.short_source
    }

    pub const fn other_source(&self) -> u8 {
        self.other_source
    }

    pub const fn short_consumed(&self) -> u8 {
        self.short_consumed
    }

    pub const fn other_consumed(&self) -> u8 {
        self.other_consumed
    }

    pub const fn short_tiers(&self) -> &[u8; 3] {
        &self.short_tiers
    }

    pub const fn other_tiers(&self) -> &[u8; 3] {
        &self.other_tiers
    }

    pub const fn next_state(&self) -> &TransientChannelState {
        &self.next_state
    }

    pub fn into_next_state(self) -> TransientChannelState {
        self.next_state
    }
}

/// Processes one caller-owned installed-audio block for one full-band channel.
///
/// This helper advances only the supplied lane state. It does not run F07.1 input
/// history, the other four full-band lanes, F08 transforms, source-frame ownership,
/// or the parent F07/F12 transaction.
pub fn process_transient_channel_block(
    state: TransientChannelState,
    input_words: &[u32; TRANSIENT_BLOCK_SAMPLES],
    parameters: &TransientParameters,
) -> Result<TransientBlockOutput, TransientChannelError> {
    let mut input_samples = [0.0_f32; TRANSIENT_BLOCK_SAMPLES];
    for (sample, word) in input_samples.iter_mut().zip(input_words) {
        let value = f32::from_bits(*word);
        if !value.is_finite() {
            return Err(TransientChannelError::InputError);
        }
        *sample = value;
    }

    let mean_word = checked_mean_square_256(&input_samples).map_err(map_mean_square_error)?;
    let mut next_state = state;
    let mut two_stage_output = input_samples;
    for (history, coefficients) in next_state
        .two_history
        .iter_mut()
        .zip(parameters.two_section_words())
    {
        two_stage_output = apply_two_section(&two_stage_output, history, coefficients)?;
    }
    clear_histories(
        &mut next_state.two_history,
        parameters.history_clear_threshold_word(),
    )?;

    let mut filtered = two_stage_output;
    for (history, coefficients) in next_state
        .three_history
        .iter_mut()
        .zip(parameters.three_section_words())
    {
        filtered = apply_paired_section(&filtered, history, coefficients)?;
    }
    clear_histories(
        &mut next_state.three_history,
        parameters.history_clear_threshold_word(),
    )?;

    let (segment_peaks, peak_positions) = segment_peaks(&filtered)?;
    let half_peaks = [
        max_nonnegative(segment_peaks[0], segment_peaks[1]),
        max_nonnegative(segment_peaks[2], segment_peaks[3]),
    ];
    let full_peak = max_nonnegative(half_peaks[0], half_peaks[1]);
    let previous_mean = next_state.previous_mean;
    let mut updated_statistics = next_state.peak_statistics;
    updated_statistics[2] = full_peak;
    updated_statistics[8] = half_peaks[0];
    updated_statistics[9] = half_peaks[1];
    updated_statistics[14..18].copy_from_slice(&segment_peaks);

    let short_tiers = tier_flags(&updated_statistics, parameters.short_threshold_words())?;
    let other_tiers = tier_flags(&updated_statistics, parameters.other_threshold_words())?;
    let min_gate = finite_from_bits(parameters.minimum_power_word())?;
    let full_peak_value = finite_from_bits(updated_statistics[2])?;
    let minimum_rejects = min_gate.value() > full_peak_value.value();
    let previous_mean_value = finite_from_bits(previous_mean)?;
    let mean_value = finite_from_bits(mean_word)?;
    let half_mean = multiply(finite_from_bits(HALF_BITS)?, mean_value)?;
    let short_mean_rejects = previous_mean_value.value() > half_mean.value();
    let other_mean_rejects = previous_mean_value.value() > mean_value.value();
    let short_source = short_tiers[0] && !(minimum_rejects || short_mean_rejects);
    let other_source =
        other_tiers.iter().any(|tier| *tier) && !(minimum_rejects || other_mean_rejects);

    let short_consumed = next_state.short_queue[0];
    let other_consumed = next_state.other_queue[0];
    next_state.short_queue.copy_within(1..QUEUE_LENGTH, 0);
    next_state.other_queue.copy_within(1..QUEUE_LENGTH, 0);
    next_state.short_queue[QUEUE_LENGTH - 1] = u8::from(short_source);
    next_state.other_queue[QUEUE_LENGTH - 1] = u8::from(other_source);

    let old_statistics = next_state.peak_statistics;
    next_state.peak_statistics = updated_statistics;
    next_state.peak_statistics[0] = old_statistics[1];
    next_state.peak_statistics[1] = updated_statistics[2];
    next_state.peak_statistics[6] = updated_statistics[8];
    next_state.peak_statistics[7] = updated_statistics[9];
    next_state.peak_statistics[12] = updated_statistics[16];
    next_state.peak_statistics[13] = updated_statistics[17];
    next_state.previous_mean = mean_word;

    Ok(TransientBlockOutput {
        filtered_words: filtered.map(f32::to_bits),
        mean_word,
        peak_positions,
        short_source: u8::from(short_source),
        other_source: u8::from(other_source),
        short_consumed,
        other_consumed,
        short_tiers: short_tiers.map(u8::from),
        other_tiers: other_tiers.map(u8::from),
        next_state,
    })
}

fn apply_two_section(
    input: &[f32; TRANSIENT_BLOCK_SAMPLES],
    history: &mut [u32; 2],
    coefficients: &[u32; 5],
) -> Result<[f32; TRANSIENT_BLOCK_SAMPLES], TransientChannelError> {
    let a0 = finite_from_bits(coefficients[0])?;
    let a1 = finite_from_bits(coefficients[1])?;
    let b0 = finite_from_bits(coefficients[2])?;
    let b1 = finite_from_bits(coefficients[3])?;
    let b2 = finite_from_bits(coefficients[4])?;
    let mut h0 = finite_from_bits(history[0])?;
    let mut h1 = finite_from_bits(history[1])?;
    let mut output = [0.0_f32; TRANSIENT_BLOCK_SAMPLES];

    for (index, sample) in input.iter().enumerate() {
        let x = finite_from_f32(*sample)?;
        let y = add(multiply(x, b0)?, h0)?;
        let next_h0 = add(add(multiply(x, b1)?, h1)?, multiply(y, a0)?)?;
        let next_h1 = add(multiply(x, b2)?, multiply(y, a1)?)?;
        output[index] = y.value();
        h0 = next_h0;
        h1 = next_h1;
    }

    history[0] = h0.to_bits();
    history[1] = h1.to_bits();
    Ok(output)
}

fn apply_paired_section(
    input: &[f32; TRANSIENT_BLOCK_SAMPLES],
    history: &mut [u32; 2],
    coefficients: &[u32; 5],
) -> Result<[f32; TRANSIENT_BLOCK_SAMPLES], TransientChannelError> {
    let a0 = finite_from_bits(coefficients[0])?;
    let a1 = finite_from_bits(coefficients[1])?;
    let b0 = finite_from_bits(coefficients[2])?;
    let b1 = finite_from_bits(coefficients[3])?;
    let b2 = finite_from_bits(coefficients[4])?;
    let quadratic_q = add(multiply(b0, a0)?, b1)?;
    let quadratic_z = add(multiply(b0, a1)?, b2)?;
    let one = finite_from_bits(ONE_BITS)?;
    let zero = finite_from_bits(POSITIVE_ZERO_BITS)?;
    let state_h0_coefficients = [one, a0, add(multiply(a0, a0)?, a1)?, multiply(a1, a0)?];
    let state_h1_coefficients = [zero, one, a0, a1];
    let input_x0_coefficients = [
        b0,
        quadratic_q,
        add(multiply(quadratic_q, a0)?, quadratic_z)?,
        multiply(quadratic_q, a1)?,
    ];
    let input_x1_coefficients = [zero, b0, quadratic_q, quadratic_z];

    let mut h0 = finite_from_bits(history[0])?;
    let mut h1 = finite_from_bits(history[1])?;
    let mut output = [0.0_f32; TRANSIENT_BLOCK_SAMPLES];
    for (pair_index, pair) in input.chunks_exact(2).enumerate() {
        let x0 = finite_from_f32(pair[0])?;
        let x1 = finite_from_f32(pair[1])?;
        let mut values = [zero; 4];
        for index in 0..4 {
            values[index] = add(
                add(
                    add(
                        multiply(x1, input_x1_coefficients[index])?,
                        multiply(x0, input_x0_coefficients[index])?,
                    )?,
                    multiply(h0, state_h0_coefficients[index])?,
                )?,
                multiply(h1, state_h1_coefficients[index])?,
            )?;
        }
        output[pair_index * 2] = values[0].value();
        output[pair_index * 2 + 1] = values[1].value();
        h0 = values[2];
        h1 = values[3];
    }

    history[0] = h0.to_bits();
    history[1] = h1.to_bits();
    Ok(output)
}

fn clear_histories<const N: usize>(
    histories: &mut [[u32; 2]; N],
    clear_threshold_bits: u32,
) -> Result<(), TransientChannelError> {
    let mut absolute_sum = finite_from_bits(POSITIVE_ZERO_BITS)?;
    for section in histories.iter() {
        for word in section {
            let absolute = *word & ABS_MAGNITUDE_MASK;
            absolute_sum = add(absolute_sum, finite_from_bits(absolute)?)?;
        }
    }
    let threshold = finite_from_bits(clear_threshold_bits)?;
    if absolute_sum.to_bits() != POSITIVE_ZERO_BITS && absolute_sum.value() < threshold.value() {
        histories.fill([POSITIVE_ZERO_BITS; 2]);
    }
    Ok(())
}

fn segment_peaks(
    filtered: &[f32; TRANSIENT_BLOCK_SAMPLES],
) -> Result<([u32; 4], [u8; 4]), TransientChannelError> {
    let mut peaks = [POSITIVE_ZERO_BITS; 4];
    let mut positions = [0_u8; 4];
    for segment in 0..4 {
        let start = segment * QUARTER_SAMPLES;
        let mut peak = 0.0_f32;
        let mut position = start;
        for (offset, sample) in filtered[start..start + QUARTER_SAMPLES].iter().enumerate() {
            let magnitude = sample.abs();
            if magnitude >= peak {
                peak = magnitude;
                position = start + offset;
            }
        }
        peaks[segment] = peak.min(1.0_f32).to_bits();
        positions[segment] =
            u8::try_from(position).map_err(|_| TransientChannelError::NumericFailure)?;
    }
    Ok((peaks, positions))
}

fn max_nonnegative(left_bits: u32, right_bits: u32) -> u32 {
    let left = f32::from_bits(left_bits);
    let right = f32::from_bits(right_bits);
    if left >= right { left_bits } else { right_bits }
}

fn tier_flags(
    statistics: &[u32; PEAK_STATISTICS_WORDS],
    thresholds: &[u32; 3],
) -> Result<[bool; 3], TransientChannelError> {
    let mut tiers = [false; 3];
    for (level, count) in [1_usize, 2, 4].into_iter().enumerate() {
        let base = 6 * level;
        let threshold = finite_from_bits(thresholds[level])?;
        for offset in 0..count {
            let peak = finite_from_bits(statistics[base + offset + 2])?;
            let comparison = finite_from_bits(statistics[base + offset + 1])?;
            if multiply(peak, threshold)?.value() > comparison.value() {
                tiers[level] = true;
                break;
            }
        }
    }
    Ok(tiers)
}

fn finite_from_bits(bits: u32) -> Result<FiniteF32, TransientChannelError> {
    FiniteF32::new(f32::from_bits(bits)).map_err(|_| TransientChannelError::NumericFailure)
}

fn finite_from_f32(value: f32) -> Result<FiniteF32, TransientChannelError> {
    FiniteF32::new(value).map_err(|_| TransientChannelError::NumericFailure)
}

fn multiply(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, TransientChannelError> {
    r32_mul(left, right).map_err(|_| TransientChannelError::NumericFailure)
}

fn add(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, TransientChannelError> {
    r32_add(left, right).map_err(|_| TransientChannelError::NumericFailure)
}

fn map_mean_square_error(error: MeanSquareError) -> TransientChannelError {
    match error {
        MeanSquareError::NonFiniteInput => TransientChannelError::InputError,
        MeanSquareError::NonFiniteIntermediate => TransientChannelError::NumericFailure,
    }
}

fn append_word(bytes: &mut [u8; STATE_SERIALIZED_BYTES], offset: &mut usize, word: u32) {
    bytes[*offset..*offset + 4].copy_from_slice(&word.to_le_bytes());
    *offset += 4;
}
