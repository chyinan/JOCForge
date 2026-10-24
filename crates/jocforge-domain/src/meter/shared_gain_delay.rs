// pattern: Functional Core
//! Selected shared post-render gain delay for the independent meter path.
//!
//! This module owns only the DV32 leaf state and computes one immutable
//! callback transition. Programme ownership, transaction commit, source
//! scheduling, and meter finalization remain outside this boundary.

use crate::numeric::{FiniteF32, R32Error, r32_add, r32_mul, r32_sub};

pub const SHARED_GAIN_DELAY_ROWS: usize = 6;
pub const SHARED_GAIN_DELAY_CALLBACK_SAMPLES: usize = 1_536;
pub const SHARED_GAIN_DELAY_BLOCK_SAMPLES: usize = 32;

const BLOCK_COUNT: usize = SHARED_GAIN_DELAY_CALLBACK_SAMPLES / SHARED_GAIN_DELAY_BLOCK_SAMPLES;
const STATE_HEADER: &[u8; 8] = b"SDVSTATE";
const STATE_BYTES: usize = 812;
const ONE_WORD: u32 = 0x3f80_0000;
const HALF_WORD: u32 = 0x3f00_0000;
const TWO_WORD: u32 = 0x4000_0000;
const TWO_FIFTY_SIX_WORD: u32 = 0x4380_0000;
const GAIN_SCALE_WORD: u32 = 0x3b80_0000;
const NEGATIVE_ONE_WORD: u32 = 0xbf80_0000;
const HOLD_LIMIT: u32 = 3;

// Approved selected 48 kHz shared-DV32 parameter words, in c0..c4 order.
const COEFFICIENT_WORDS: [u32; 5] = [
    0x3f7d_1000,
    0x3c3c_0000,
    0x3f7f_4200,
    0x3b3e_0000,
    0x3f69_6600,
];
const RECIPROCAL_PIVOT_WORD: u32 = 0x3f35_0600;
const TARGET_PIVOT_WORD: u32 = 0x3f57_6600;
const ENVELOPE_WEIGHT_WORD: u32 = 0x3f18_2000;
const SELECTED_KERNEL_Q15: [u16; SHARED_GAIN_DELAY_BLOCK_SAMPLES] = [
    20, 177, 491, 958, 1_573, 2_331, 3_224, 4_244, 5_381, 6_624, 7_961, 9_379, 10_864, 12_403,
    13_980, 15_580, 17_188, 18_788, 20_365, 21_904, 23_389, 24_807, 26_144, 27_387, 28_524, 29_544,
    30_437, 31_195, 31_810, 32_277, 32_591, 32_748,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SharedGainDelayError {
    ParameterUnavailable,
    InputShape,
    InputNonFinite,
    StateInvalid,
    CounterOverflow,
    NumericFailure,
}

/// Callback position owned only by the selected shared gain-delay leaf.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SharedGainDelayCallbackIndex(u32);

impl SharedGainDelayCallbackIndex {
    const COLD: Self = Self(0);

    pub const fn value(self) -> u32 {
        self.0
    }

    fn checked_next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedGainDelayParameterState {
    peak_word: u32,
    previous_gain_word: u32,
    hold: u32,
    envelope_word: u32,
    smoothed_gain_word: u32,
    target_gain_word: u32,
    output_level_word: u32,
    blocks_processed: u32,
}

impl SharedGainDelayParameterState {
    pub const fn peak_word(&self) -> u32 {
        self.peak_word
    }

    pub const fn previous_gain_word(&self) -> u32 {
        self.previous_gain_word
    }

    pub const fn hold(&self) -> u32 {
        self.hold
    }

    pub const fn envelope_word(&self) -> u32 {
        self.envelope_word
    }

    pub const fn smoothed_gain_word(&self) -> u32 {
        self.smoothed_gain_word
    }

    pub const fn target_gain_word(&self) -> u32 {
        self.target_gain_word
    }

    pub const fn output_level_word(&self) -> u32 {
        self.output_level_word
    }

    pub const fn blocks_processed(&self) -> u32 {
        self.blocks_processed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedGainDelayState {
    callback_index: SharedGainDelayCallbackIndex,
    parameters: SharedGainDelayParameterState,
    history_words: [[u32; SHARED_GAIN_DELAY_BLOCK_SAMPLES]; SHARED_GAIN_DELAY_ROWS],
}

impl SharedGainDelayState {
    pub const fn cold() -> Self {
        Self {
            callback_index: SharedGainDelayCallbackIndex::COLD,
            parameters: SharedGainDelayParameterState {
                peak_word: 0,
                previous_gain_word: ONE_WORD,
                hold: 0,
                envelope_word: ONE_WORD,
                smoothed_gain_word: ONE_WORD,
                target_gain_word: ONE_WORD,
                output_level_word: GAIN_SCALE_WORD,
                blocks_processed: 0,
            },
            history_words: [[0; SHARED_GAIN_DELAY_BLOCK_SAMPLES]; SHARED_GAIN_DELAY_ROWS],
        }
    }

    pub fn reset(&mut self) {
        *self = Self::cold();
    }

    pub const fn callback_index(&self) -> SharedGainDelayCallbackIndex {
        self.callback_index
    }

    pub const fn parameters(&self) -> &SharedGainDelayParameterState {
        &self.parameters
    }

    pub const fn history_words(
        &self,
    ) -> &[[u32; SHARED_GAIN_DELAY_BLOCK_SAMPLES]; SHARED_GAIN_DELAY_ROWS] {
        &self.history_words
    }

    /// Serializes the approved portable leaf-state layout. This is not a native ABI.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(STATE_BYTES);
        bytes.extend_from_slice(STATE_HEADER);
        push_u32(&mut bytes, self.callback_index.value());
        push_u32(&mut bytes, self.parameters.peak_word);
        push_u32(&mut bytes, self.parameters.previous_gain_word);
        push_u32(&mut bytes, self.parameters.hold);
        push_u32(&mut bytes, self.parameters.envelope_word);
        push_u32(&mut bytes, self.parameters.smoothed_gain_word);
        push_u32(&mut bytes, self.parameters.target_gain_word);
        push_u32(&mut bytes, self.parameters.output_level_word);
        push_u32(&mut bytes, self.parameters.blocks_processed);
        for row in &self.history_words {
            for word in row {
                push_u32(&mut bytes, *word);
            }
        }
        debug_assert_eq!(bytes.len(), STATE_BYTES);
        bytes
    }
}

#[derive(Clone, Debug)]
pub struct SharedGainDelayParameters {
    coefficients: [FiniteF32; 5],
    reciprocal_pivot: FiniteF32,
    target_pivot: FiniteF32,
    envelope_weight: FiniteF32,
    kernel: [FiniteF32; SHARED_GAIN_DELAY_BLOCK_SAMPLES],
}

impl SharedGainDelayParameters {
    /// Builds the selected provider after the caller supplies the authenticated
    /// `receiver_blend32` Q15 record.
    pub fn selected(kernel_q15: &[u16]) -> Result<Self, SharedGainDelayError> {
        if kernel_q15 != SELECTED_KERNEL_Q15 {
            return Err(SharedGainDelayError::ParameterUnavailable);
        }

        let coefficients = COEFFICIENT_WORDS.map(finite_parameter_word);
        let coefficients = collect_results(coefficients)?;
        let mut kernel = [finite_constant(0.0)?; SHARED_GAIN_DELAY_BLOCK_SAMPLES];
        for (destination, q15) in kernel.iter_mut().zip(kernel_q15.iter().copied()) {
            *destination = FiniteF32::new(f32::from(q15) / 32_768.0)
                .map_err(|_| SharedGainDelayError::ParameterUnavailable)?;
        }

        Ok(Self {
            coefficients,
            reciprocal_pivot: finite_parameter_word(RECIPROCAL_PIVOT_WORD)?,
            target_pivot: finite_parameter_word(TARGET_PIVOT_WORD)?,
            envelope_weight: finite_parameter_word(ENVELOPE_WEIGHT_WORD)?,
            kernel,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct SharedGainDelayTransition {
    output_words: Vec<Vec<u32>>,
    after: SharedGainDelayState,
}

impl SharedGainDelayTransition {
    pub fn output_words(&self) -> &[Vec<u32>] {
        &self.output_words
    }

    pub const fn after(&self) -> &SharedGainDelayState {
        &self.after
    }

    pub fn into_parts(self) -> (Vec<Vec<u32>>, SharedGainDelayState) {
        (self.output_words, self.after)
    }
}

#[derive(Clone, Copy)]
struct ParameterTransition {
    after: SharedGainDelayParameterState,
}

/// Computes one 6x1536 selected callback without mutating the committed leaf state.
pub fn stage_shared_gain_delay_callback(
    before: &SharedGainDelayState,
    input_words: &[Vec<u32>],
    parameters: &SharedGainDelayParameters,
) -> Result<SharedGainDelayTransition, SharedGainDelayError> {
    validate_state(before)?;
    validate_input(input_words)?;
    before
        .callback_index
        .checked_next()
        .ok_or(SharedGainDelayError::CounterOverflow)?;
    before
        .parameters
        .blocks_processed
        .checked_add(u32::try_from(BLOCK_COUNT).map_err(|_| SharedGainDelayError::CounterOverflow)?)
        .ok_or(SharedGainDelayError::CounterOverflow)?;

    let mut output_words = (0..SHARED_GAIN_DELAY_ROWS)
        .map(|_| vec![0; SHARED_GAIN_DELAY_CALLBACK_SAMPLES])
        .collect::<Vec<_>>();
    let mut after = before.clone();

    for block_index in 0..BLOCK_COUNT {
        let sample_offset = block_index * SHARED_GAIN_DELAY_BLOCK_SAMPLES;
        let transition =
            update_parameters(&after.parameters, input_words, sample_offset, parameters)?;
        filter_block(
            &after.history_words,
            &mut output_words,
            sample_offset,
            &after.parameters,
            &transition.after,
            parameters,
        )?;
        copy_block_to_history(&mut after.history_words, input_words, sample_offset);
        after.parameters = transition.after;
    }

    after.callback_index = before
        .callback_index
        .checked_next()
        .ok_or(SharedGainDelayError::CounterOverflow)?;
    Ok(SharedGainDelayTransition {
        output_words,
        after,
    })
}

fn validate_state(state: &SharedGainDelayState) -> Result<(), SharedGainDelayError> {
    let parameters = state.parameters;
    for word in [
        parameters.peak_word,
        parameters.previous_gain_word,
        parameters.envelope_word,
        parameters.smoothed_gain_word,
        parameters.target_gain_word,
        parameters.output_level_word,
    ] {
        let value = f32::from_bits(word);
        if !value.is_finite() || value < 0.0 {
            return Err(SharedGainDelayError::StateInvalid);
        }
    }
    if parameters.output_level_word != GAIN_SCALE_WORD || parameters.hold > i32::MAX as u32 {
        return Err(SharedGainDelayError::StateInvalid);
    }
    if state
        .history_words
        .iter()
        .flatten()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SharedGainDelayError::StateInvalid);
    }
    Ok(())
}

fn validate_input(input_words: &[Vec<u32>]) -> Result<(), SharedGainDelayError> {
    if input_words.len() != SHARED_GAIN_DELAY_ROWS
        || input_words
            .iter()
            .any(|row| row.len() != SHARED_GAIN_DELAY_CALLBACK_SAMPLES)
    {
        return Err(SharedGainDelayError::InputShape);
    }
    if input_words
        .iter()
        .flatten()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SharedGainDelayError::InputNonFinite);
    }
    Ok(())
}

fn update_parameters(
    before: &SharedGainDelayParameterState,
    input_words: &[Vec<u32>],
    sample_offset: usize,
    parameters: &SharedGainDelayParameters,
) -> Result<ParameterTransition, SharedGainDelayError> {
    let zero = finite_constant(0.0)?;
    let mut current_peak = zero;
    for row in input_words {
        for word in &row[sample_offset..sample_offset + SHARED_GAIN_DELAY_BLOCK_SAMPLES] {
            let magnitude = finite_word(*word & 0x7fff_ffff, SharedGainDelayError::InputNonFinite)?;
            if magnitude.value() > current_peak.value() {
                current_peak = magnitude;
            }
        }
    }

    let old_peak = finite_state_word(before.peak_word)?;
    let old_envelope = finite_state_word(before.envelope_word)?;
    let scaled_peak =
        r32_mul(max_first(current_peak, old_peak), old_envelope).map_err(map_numeric_error)?;
    let attenuation = reciprocal_attenuation(scaled_peak, parameters.reciprocal_pivot)?;

    let old_smoothed = finite_state_word(before.smoothed_gain_word)?;
    let old_target = finite_state_word(before.target_gain_word)?;
    let maximum = r32_mul(old_envelope, attenuation).map_err(map_numeric_error)?;
    let previous_gain = old_smoothed;
    let mut hold = before.hold;
    let mut target = old_target;
    let mut smoothed = old_smoothed;

    if old_smoothed.value() > maximum.value() {
        target = attenuation;
        hold = 0;
        smoothed = maximum;
    } else if hold < HOLD_LIMIT {
        hold = hold
            .checked_add(1)
            .ok_or(SharedGainDelayError::CounterOverflow)?;
    } else {
        let attenuation_term =
            r32_mul(parameters.coefficients[4], attenuation).map_err(map_numeric_error)?;
        let target_retain = r32_sub(attenuation, attenuation_term).map_err(map_numeric_error)?;
        let target_smooth =
            r32_mul(parameters.coefficients[4], old_target).map_err(map_numeric_error)?;
        target = r32_add(target_retain, target_smooth).map_err(map_numeric_error)?;

        let candidate = r32_mul(target, old_envelope).map_err(map_numeric_error)?;
        if candidate.value() > smoothed.value() {
            smoothed = candidate;
        }
        if smoothed.value() > maximum.value() {
            smoothed = maximum;
        }
    }

    let envelope = if target.value() < parameters.target_pivot.value() {
        let term0 = r32_mul(target, old_envelope).map_err(map_numeric_error)?;
        let term1 = r32_mul(term0, parameters.envelope_weight).map_err(map_numeric_error)?;
        let term2 = r32_mul(term1, finite_parameter_word(TWO_WORD)?).map_err(map_numeric_error)?;
        let term3 = r32_mul(term2, parameters.coefficients[1]).map_err(map_numeric_error)?;
        let retain =
            r32_mul(old_envelope, parameters.coefficients[0]).map_err(map_numeric_error)?;
        r32_add(term3, retain).map_err(map_numeric_error)?
    } else {
        let retain =
            r32_mul(old_envelope, parameters.coefficients[2]).map_err(map_numeric_error)?;
        r32_add(retain, parameters.coefficients[3]).map_err(map_numeric_error)?
    };

    let blocks_processed = before
        .blocks_processed
        .checked_add(1)
        .ok_or(SharedGainDelayError::CounterOverflow)?;
    Ok(ParameterTransition {
        after: SharedGainDelayParameterState {
            peak_word: current_peak.to_bits(),
            previous_gain_word: previous_gain.to_bits(),
            hold,
            envelope_word: envelope.to_bits(),
            smoothed_gain_word: smoothed.to_bits(),
            target_gain_word: target.to_bits(),
            output_level_word: GAIN_SCALE_WORD,
            blocks_processed,
        },
    })
}

fn reciprocal_attenuation(
    scaled_peak: FiniteF32,
    reciprocal_pivot: FiniteF32,
) -> Result<FiniteF32, SharedGainDelayError> {
    let one = finite_parameter_word(ONE_WORD)?;
    if scaled_peak.value() <= one.value() {
        return Ok(one);
    }

    let half = finite_parameter_word(HALF_WORD)?;
    let two = finite_parameter_word(TWO_WORD)?;
    let mut normalized = scaled_peak;
    let mut exponent = 0_i32;
    while normalized.value() >= one.value() {
        normalized = r32_mul(normalized, half).map_err(map_numeric_error)?;
        exponent = exponent
            .checked_sub(1)
            .ok_or(SharedGainDelayError::NumericFailure)?;
    }

    let biased_exponent = exponent
        .checked_add(128)
        .filter(|value| (1..=254).contains(value))
        .ok_or(SharedGainDelayError::NumericFailure)?;
    let exponent_bits =
        u32::try_from(biased_exponent).map_err(|_| SharedGainDelayError::NumericFailure)?;
    let scale = finite_parameter_word(exponent_bits << 23)?;

    let mut reciprocal = if normalized.value() <= reciprocal_pivot.value() {
        r32_mul(r32_sub(one, normalized).map_err(map_numeric_error)?, two)
            .map_err(map_numeric_error)?
    } else {
        r32_sub(one, r32_mul(half, normalized).map_err(map_numeric_error)?)
            .map_err(map_numeric_error)?
    };
    for _ in 0..4 {
        let product = r32_mul(reciprocal, normalized).map_err(map_numeric_error)?;
        let difference = r32_sub(half, product).map_err(map_numeric_error)?;
        let correction = r32_mul(
            r32_mul(difference, reciprocal).map_err(map_numeric_error)?,
            two,
        )
        .map_err(map_numeric_error)?;
        reciprocal = r32_add(reciprocal, correction).map_err(map_numeric_error)?;
    }

    r32_mul(r32_mul(one, reciprocal).map_err(map_numeric_error)?, scale).map_err(map_numeric_error)
}

fn filter_block(
    history_words: &[[u32; SHARED_GAIN_DELAY_BLOCK_SAMPLES]; SHARED_GAIN_DELAY_ROWS],
    output_words: &mut [Vec<u32>],
    sample_offset: usize,
    before: &SharedGainDelayParameterState,
    after: &SharedGainDelayParameterState,
    parameters: &SharedGainDelayParameters,
) -> Result<(), SharedGainDelayError> {
    let alpha = r32_mul(
        finite_state_word(before.output_level_word)?,
        finite_state_word(after.previous_gain_word)?,
    )
    .map_err(map_numeric_error)?;
    let beta = r32_mul(
        finite_state_word(after.smoothed_gain_word)?,
        finite_parameter_word(GAIN_SCALE_WORD)?,
    )
    .map_err(map_numeric_error)?;
    let positive_limit = finite_parameter_word(ONE_WORD)?;
    let negative_limit = finite_parameter_word(NEGATIVE_ONE_WORD)?;
    let two_fifty_six = finite_parameter_word(TWO_FIFTY_SIX_WORD)?;
    let gain_scale = finite_parameter_word(GAIN_SCALE_WORD)?;

    for row in 0..SHARED_GAIN_DELAY_ROWS {
        for sample in 0..SHARED_GAIN_DELAY_BLOCK_SAMPLES {
            let history = finite_state_word(history_words[row][sample])?;
            let history_beta = r32_mul(history, beta).map_err(map_numeric_error)?;
            let a = r32_mul(parameters.kernel[sample], history_beta).map_err(map_numeric_error)?;
            let history_alpha = r32_mul(history, alpha).map_err(map_numeric_error)?;
            let b = r32_mul(
                parameters.kernel[SHARED_GAIN_DELAY_BLOCK_SAMPLES - 1 - sample],
                history_alpha,
            )
            .map_err(map_numeric_error)?;
            let sum = r32_add(a, b).map_err(map_numeric_error)?;
            let intermediate = r32_mul(sum, two_fifty_six).map_err(map_numeric_error)?;
            let upper_clamped = min_first(positive_limit, intermediate);
            let clipped = max_first(negative_limit, upper_clamped);
            let output_scaled = r32_mul(clipped, gain_scale).map_err(map_numeric_error)?;
            let output = r32_mul(output_scaled, two_fifty_six).map_err(map_numeric_error)?;
            output_words[row][sample_offset + sample] = output.to_bits();
        }
    }

    Ok(())
}

fn copy_block_to_history(
    history_words: &mut [[u32; SHARED_GAIN_DELAY_BLOCK_SAMPLES]; SHARED_GAIN_DELAY_ROWS],
    input_words: &[Vec<u32>],
    sample_offset: usize,
) {
    for row in 0..SHARED_GAIN_DELAY_ROWS {
        history_words[row].copy_from_slice(
            &input_words[row][sample_offset..sample_offset + SHARED_GAIN_DELAY_BLOCK_SAMPLES],
        );
    }
}

fn finite_state_word(word: u32) -> Result<FiniteF32, SharedGainDelayError> {
    finite_word(word, SharedGainDelayError::StateInvalid)
}

fn finite_parameter_word(word: u32) -> Result<FiniteF32, SharedGainDelayError> {
    finite_word(word, SharedGainDelayError::ParameterUnavailable)
}

fn finite_word(word: u32, error: SharedGainDelayError) -> Result<FiniteF32, SharedGainDelayError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| error)
}

fn finite_constant(value: f32) -> Result<FiniteF32, SharedGainDelayError> {
    FiniteF32::new(value).map_err(|_| SharedGainDelayError::NumericFailure)
}

fn collect_results<const N: usize>(
    values: [Result<FiniteF32, SharedGainDelayError>; N],
) -> Result<[FiniteF32; N], SharedGainDelayError> {
    let mut iterator = values.into_iter();
    let mut result = [finite_constant(0.0)?; N];
    for destination in &mut result {
        *destination = iterator
            .next()
            .expect("array iterator has the declared length")?;
    }
    Ok(result)
}

fn map_numeric_error(_: R32Error) -> SharedGainDelayError {
    SharedGainDelayError::NumericFailure
}

fn max_first(left: FiniteF32, right: FiniteF32) -> FiniteF32 {
    if right.value() > left.value() {
        right
    } else {
        left
    }
}

fn min_first(left: FiniteF32, right: FiniteF32) -> FiniteF32 {
    if right.value() < left.value() {
        right
    } else {
        left
    }
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use serde_json::Value;
    use sha2::{Digest, Sha256};

    use super::{
        SELECTED_KERNEL_Q15, SHARED_GAIN_DELAY_BLOCK_SAMPLES, SHARED_GAIN_DELAY_CALLBACK_SAMPLES,
        SHARED_GAIN_DELAY_ROWS, SharedGainDelayCallbackIndex, SharedGainDelayError,
        SharedGainDelayParameters, SharedGainDelayState, stage_shared_gain_delay_callback,
    };

    fn selected_parameters() -> SharedGainDelayParameters {
        SharedGainDelayParameters::selected(&SELECTED_KERNEL_Q15).unwrap()
    }

    fn constant_input(word: u32) -> Vec<Vec<u32>> {
        vec![vec![word; SHARED_GAIN_DELAY_CALLBACK_SAMPLES]; SHARED_GAIN_DELAY_ROWS]
    }

    fn callback_recipe(scale: f32) -> Vec<Vec<u32>> {
        (0..SHARED_GAIN_DELAY_ROWS)
            .map(|row| {
                (0..SHARED_GAIN_DELAY_CALLBACK_SAMPLES)
                    .map(|sample| {
                        let centered = i32::try_from(sample % 17).unwrap() - 8;
                        let base = ((row + 1) as f32 * centered as f32) / 16.0_f32;
                        (base * scale).to_bits()
                    })
                    .collect()
            })
            .collect()
    }

    fn bytes_sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn output_sha256(rows: &[Vec<u32>]) -> String {
        let mut bytes = Vec::with_capacity(
            SHARED_GAIN_DELAY_ROWS * SHARED_GAIN_DELAY_CALLBACK_SAMPLES * size_of::<u32>(),
        );
        for word in rows.iter().flatten() {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes_sha256(&bytes)
    }

    #[test]
    fn cold_state_matches_the_portable_selected_layout() {
        let state = SharedGainDelayState::cold();
        let bytes = state.canonical_bytes();
        assert_eq!(bytes.len(), 812);
        assert_eq!(&bytes[..8], b"SDVSTATE");
        let expected_header_words = [
            0x0000_0000,
            0x0000_0000,
            0x3f80_0000,
            0x0000_0000,
            0x3f80_0000,
            0x3f80_0000,
            0x3f80_0000,
            0x3b80_0000,
            0x0000_0000,
        ];
        for (bytes, expected) in bytes[8..44].chunks_exact(4).zip(expected_header_words) {
            assert_eq!(u32::from_le_bytes(bytes.try_into().unwrap()), expected);
        }
        assert!(bytes[44..].iter().all(|byte| *byte == 0));
        assert_eq!(state.callback_index().value(), 0);
        assert_eq!(state.parameters().blocks_processed(), 0);
        assert!(
            state
                .history_words()
                .iter()
                .flatten()
                .all(|word| *word == 0)
        );

        let parameters = selected_parameters();
        let mut advanced =
            stage_shared_gain_delay_callback(&state, &constant_input(0x4000_0000), &parameters)
                .unwrap()
                .after()
                .clone();
        assert_ne!(advanced, state);
        advanced.reset();
        assert_eq!(advanced, state);
    }

    #[test]
    fn selected_parameter_constructor_rejects_kernel_drift() {
        let mut drifted = SELECTED_KERNEL_Q15;
        drifted[0] += 1;
        assert!(matches!(
            SharedGainDelayParameters::selected(&drifted),
            Err(SharedGainDelayError::ParameterUnavailable)
        ));
    }

    #[test]
    fn approved_continuous_callbacks_match_all_six_hash_bound_cases() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/shared_gain_delay/continuous_callbacks.json"
        ))
        .unwrap();
        let parameters = selected_parameters();
        let mut state = SharedGainDelayState::cold();

        for case in fixture["cases"].as_array().unwrap() {
            if case["reset_before"].as_bool().unwrap() {
                state.reset();
            }
            assert_eq!(
                bytes_sha256(&state.canonical_bytes()),
                case["before_state_sha256"].as_str().unwrap()
            );
            let before_callback = state.callback_index();

            let input = callback_recipe(case["scale"].as_f64().unwrap() as f32);
            let transition = stage_shared_gain_delay_callback(&state, &input, &parameters).unwrap();
            assert_eq!(
                output_sha256(transition.output_words()),
                case["output_sha256"].as_str().unwrap()
            );
            assert_eq!(
                bytes_sha256(&transition.after().canonical_bytes()),
                case["after_state_sha256"].as_str().unwrap()
            );
            assert_eq!(
                transition.after().callback_index(),
                before_callback.checked_next().unwrap()
            );
            state = transition.into_parts().1;
        }
    }

    #[test]
    fn continuous_callbacks_use_the_previous_thirty_two_samples() {
        let parameters = selected_parameters();
        let first = stage_shared_gain_delay_callback(
            &SharedGainDelayState::cold(),
            &constant_input(0x3e80_0000),
            &parameters,
        )
        .unwrap();

        for row in first.output_words() {
            assert!(
                row[..SHARED_GAIN_DELAY_BLOCK_SAMPLES]
                    .iter()
                    .all(|word| *word == 0)
            );
            assert!(
                row[SHARED_GAIN_DELAY_BLOCK_SAMPLES..]
                    .iter()
                    .all(|word| *word == 0x3e80_0000)
            );
        }
        assert_eq!(first.after().callback_index().value(), 1);
        assert_eq!(first.after().parameters().peak_word(), 0x3e80_0000);
        assert_eq!(first.after().parameters().previous_gain_word(), 0x3f80_0000);
        assert_eq!(first.after().parameters().hold(), 3);
        assert_eq!(first.after().parameters().envelope_word(), 0x3f80_0000);
        assert_eq!(first.after().parameters().smoothed_gain_word(), 0x3f80_0000);
        assert_eq!(first.after().parameters().target_gain_word(), 0x3f80_0000);
        assert_eq!(first.after().parameters().output_level_word(), 0x3b80_0000);
        assert_eq!(first.after().parameters().blocks_processed(), 48);

        let second = stage_shared_gain_delay_callback(
            first.after(),
            &constant_input(0x3f00_0000),
            &parameters,
        )
        .unwrap();
        for row in second.output_words() {
            assert!(
                row[..SHARED_GAIN_DELAY_BLOCK_SAMPLES]
                    .iter()
                    .all(|word| *word == 0x3e80_0000)
            );
            assert!(
                row[SHARED_GAIN_DELAY_BLOCK_SAMPLES..]
                    .iter()
                    .all(|word| *word == 0x3f00_0000)
            );
        }
        assert_eq!(second.after().callback_index().value(), 2);
        assert_eq!(second.after().parameters().blocks_processed(), 96);
        assert!(
            second
                .after()
                .history_words()
                .iter()
                .flatten()
                .all(|word| *word == 0x3f00_0000)
        );
    }

    #[test]
    fn zero_scale_preserves_signed_zero_in_history_and_delayed_output() {
        let parameters = selected_parameters();
        let mut input = constant_input(0);
        for (row_index, row) in input.iter_mut().enumerate() {
            for (sample_index, word) in row.iter_mut().enumerate() {
                let signed_source = (i32::try_from(row_index).unwrap() + 1)
                    * (i32::try_from(sample_index % 17).unwrap() - 8);
                *word = if signed_source < 0 { 0x8000_0000 } else { 0 };
            }
        }

        let transition =
            stage_shared_gain_delay_callback(&SharedGainDelayState::cold(), &input, &parameters)
                .unwrap();
        assert!(
            transition
                .output_words()
                .iter()
                .flatten()
                .all(|word| f32::from_bits(*word) == 0.0)
        );
        assert!(
            transition
                .output_words()
                .iter()
                .flatten()
                .any(|word| *word == 0x8000_0000)
        );
        for (row_index, history) in transition.after().history_words().iter().enumerate() {
            assert_eq!(
                history,
                &input[row_index]
                    [SHARED_GAIN_DELAY_CALLBACK_SAMPLES - SHARED_GAIN_DELAY_BLOCK_SAMPLES..]
            );
        }
        assert_eq!(transition.after().parameters().peak_word(), 0);
        assert_eq!(transition.after().parameters().blocks_processed(), 48);
    }

    #[test]
    fn invalid_input_is_rejected_without_mutating_the_prior_state() {
        let parameters = selected_parameters();
        let before = SharedGainDelayState::cold();
        let snapshot = before.clone();
        let mut input = constant_input(0);
        input[4][91] = f32::NAN.to_bits();
        assert!(matches!(
            stage_shared_gain_delay_callback(&before, &input, &parameters),
            Err(SharedGainDelayError::InputNonFinite)
        ));
        assert_eq!(before, snapshot);

        input[4][91] = 0;
        input.pop();
        assert!(matches!(
            stage_shared_gain_delay_callback(&before, &input, &parameters),
            Err(SharedGainDelayError::InputShape)
        ));
        assert_eq!(before, snapshot);
    }

    #[test]
    fn attenuation_path_reduces_gain_without_changing_the_input_owner() {
        let parameters = selected_parameters();
        let before = SharedGainDelayState::cold();
        let input = constant_input(0x4000_0000);
        let transition = stage_shared_gain_delay_callback(&before, &input, &parameters).unwrap();

        assert_eq!(transition.after().parameters().peak_word(), 0x4000_0000);
        assert!(f32::from_bits(transition.after().parameters().smoothed_gain_word()) < 1.0);
        assert!(f32::from_bits(transition.after().parameters().target_gain_word()) < 1.0);
        assert!(f32::from_bits(transition.after().parameters().envelope_word()) < 1.0);
        assert_eq!(before, SharedGainDelayState::cold());
        assert!(input.iter().flatten().all(|word| *word == 0x4000_0000));
    }

    #[test]
    fn callback_and_block_counter_overflow_are_rejected_before_staging() {
        let parameters = selected_parameters();
        let input = constant_input(0);
        let mut callback_overflow = SharedGainDelayState::cold();
        callback_overflow.callback_index = SharedGainDelayCallbackIndex(u32::MAX);
        assert!(matches!(
            stage_shared_gain_delay_callback(&callback_overflow, &input, &parameters),
            Err(SharedGainDelayError::CounterOverflow)
        ));

        let mut block_overflow = SharedGainDelayState::cold();
        block_overflow.parameters.blocks_processed = u32::MAX - 47;
        assert!(matches!(
            stage_shared_gain_delay_callback(&block_overflow, &input, &parameters),
            Err(SharedGainDelayError::CounterOverflow)
        ));
    }
}
