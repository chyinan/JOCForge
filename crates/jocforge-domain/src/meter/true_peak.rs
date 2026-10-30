// pattern: Functional Core
//! Selected six-channel, two-stage 4x true-peak cascade and local peak metrics.

use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};
use serde_json::Value;
use sha2::{Digest, Sha256};

const CHANNEL_COUNT: usize = 6;
const SOURCE_SAMPLES: usize = 1024;
const TRUE_PEAK_SAMPLES: usize = 4096;
const FIR_STATE_WORDS: usize = 4628;
const SECOND_STAGE_START: usize = 1547;
const FIR_TAPS: usize = 12;
const TRUE_PEAK_PARAMETER_BYTES: &[u8] = include_bytes!("truepeak_parameters.json");
const TRUE_PEAK_PARAMETER_SHA256: &str =
    "f67e6fe86533af74acc261bf3908890e77a461198116c64b624dc91b69049757";
const LOG_PARAMETER_BYTES: &[u8] = include_bytes!("truepeak_log_parameters.json");
const LOG_PARAMETER_SHA256: &str =
    "044d42327f471b183b0a29caaa9db85f694a40c436a5afe3b83734dbf29bc266";
const TRUE_PEAK_PARAMETER_EXPANDED_SHA256: &str =
    "f4878bb808821078480754d676a94191954771757c07010982e97be8c571e84c";
const TRUE_PEAK_PARAMETER_COMPACT_SHA256: &str =
    "f9bdfb75951c9660c2d5498c1997e234880ac6b4f27bcb64f3e02af91d711cb4";
const TRUE_PEAK_FLOOR_WORD: u32 = 0xbf70_0000;
const TRUE_PEAK_CEILING_WORD: u32 = 0x3f70_0000;
const WINDOW_NORMALIZATION_M_WORD: u32 = 0x4f00_0000;
const WINDOW_NORMALIZATION_J_WORD: u32 = 0x3000_0000;
const TRUE_PEAK_THRESHOLD: f32 = 1.0e-6;
const TRUE_PEAK_CAP: f32 = 1.0e6;
const TRUE_PEAK_NOISE_WORD: u32 = 0x1e3c_e508;
const QUARTER_WORD: u32 = 0x3e80_0000;
const FIVE_HALVES_WORD: u32 = 0x4020_0000;
const ONE_THIRTY_SECOND_WORD: u32 = 0x3d00_0000;
const HOST_SCALE_WORD: u32 = 0x4300_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TruePeakError {
    Input,
    ParameterUnavailable,
    NumericFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LaneState {
    words: Vec<u32>,
}

/// Complete persistent true-peak FIR history and programme peak maxima.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TruePeakState {
    lanes: Vec<LaneState>,
    true_peak_internal_words: [u32; CHANNEL_COUNT],
    sample_peak_internal_words: [u32; CHANNEL_COUNT],
}

impl TruePeakState {
    pub fn cold() -> Self {
        Self {
            lanes: (0..CHANNEL_COUNT)
                .map(|_| LaneState {
                    words: vec![0; FIR_STATE_WORDS],
                })
                .collect(),
            true_peak_internal_words: [TRUE_PEAK_FLOOR_WORD; CHANNEL_COUNT],
            sample_peak_internal_words: [TRUE_PEAK_FLOOR_WORD; CHANNEL_COUNT],
        }
    }

    pub fn reset(&mut self) {
        *self = Self::cold();
    }

    /// Serializes the six FIR states in channel-major, sample-major F32LE order.
    pub fn history_f32le_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(CHANNEL_COUNT * FIR_STATE_WORDS * 4);
        for lane in &self.lanes {
            for word in &lane.words {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        bytes
    }

    pub const fn true_peak_internal_words(&self) -> &[u32; CHANNEL_COUNT] {
        &self.true_peak_internal_words
    }

    pub const fn sample_peak_internal_words(&self) -> &[u32; CHANNEL_COUNT] {
        &self.sample_peak_internal_words
    }

    pub fn final_true_peak_host_word(&self) -> Result<u32, TruePeakError> {
        final_host_word(&self.true_peak_internal_words)
    }

    pub fn final_sample_peak_host_word(&self) -> Result<u32, TruePeakError> {
        final_host_word(&self.sample_peak_internal_words)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TruePeakLaneMetrics {
    pub raw_peak_word: u32,
    pub first_absolute_maximum_index: usize,
    pub grouped_reported_index: i32,
    pub factor: u8,
    pub true_peak_internal_word: u32,
    pub true_peak_host_word: u32,
    pub sample_peak_raw_word: u32,
    pub sample_peak_first_index: usize,
    pub sample_peak_internal_word: u32,
    pub sample_peak_host_word: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TruePeakBlockResult {
    pub measurements: Vec<Vec<u32>>,
    pub lane_metrics: Vec<TruePeakLaneMetrics>,
    pub next_state: TruePeakState,
}

/// Processes one exact six-channel, 1024-sample block without mutating the prior state.
pub fn process_true_peak_block(
    state: &TruePeakState,
    input: &[Vec<u32>],
) -> Result<TruePeakBlockResult, TruePeakError> {
    let parameters = selected_parameters()?;
    process_true_peak_block_with_parameters(state, input, &parameters)
}

/// Processes one block with an owner-validated immutable parameter value.
///
/// This lets a caller use the direct approved provider projection without replacing the
/// true-peak owner or its state. It does not imply a complete Meter prepass or terminal receipt.
pub fn process_true_peak_block_with_parameters(
    state: &TruePeakState,
    input: &[Vec<u32>],
    parameters: &TruePeakParameters,
) -> Result<TruePeakBlockResult, TruePeakError> {
    validate_state(state)?;
    if input.len() != CHANNEL_COUNT
        || input
            .iter()
            .any(|row| row.len() != SOURCE_SAMPLES || row.iter().any(|word| !is_finite(*word)))
    {
        return Err(TruePeakError::Input);
    }

    let mut next_state = state.clone();
    let mut measurements = Vec::with_capacity(CHANNEL_COUNT);
    let mut lane_metrics = Vec::with_capacity(CHANNEL_COUNT);
    for (channel, source_row) in input.iter().enumerate() {
        let (measurement, next_history, factor) = if channel == 3 {
            (source_row.clone(), None, 1_u8)
        } else {
            let (samples, history) =
                interpolate_channel(&state.lanes[channel].words, source_row, parameters)?;
            (samples, Some(history), 4_u8)
        };
        if let Some(history) = next_history {
            next_state.lanes[channel].words = history;
        }
        let metrics = measure_channel(&measurement, source_row, factor, parameters)?;
        if f32::from_bits(metrics.true_peak_internal_word)
            > f32::from_bits(next_state.true_peak_internal_words[channel])
        {
            next_state.true_peak_internal_words[channel] = metrics.true_peak_internal_word;
        }
        if f32::from_bits(metrics.sample_peak_internal_word)
            > f32::from_bits(next_state.sample_peak_internal_words[channel])
        {
            next_state.sample_peak_internal_words[channel] = metrics.sample_peak_internal_word;
        }
        measurements.push(measurement);
        lane_metrics.push(metrics);
    }

    Ok(TruePeakBlockResult {
        measurements,
        lane_metrics,
        next_state,
    })
}

/// Immutable typed selected parameters validated from the authorized true-peak and log assets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TruePeakParameters {
    a: [u32; FIR_TAPS],
    b: [u32; FIR_TAPS],
    log: [u32; 11],
}

impl TruePeakParameters {
    pub const fn stage_a_words(&self) -> &[u32; FIR_TAPS] {
        &self.a
    }

    pub const fn stage_b_words(&self) -> &[u32; FIR_TAPS] {
        &self.b
    }

    pub const fn log_words(&self) -> &[u32; 11] {
        &self.log
    }
}

pub(crate) fn selected_parameters() -> Result<TruePeakParameters, TruePeakError> {
    validate_approved_provider_assets(TRUE_PEAK_PARAMETER_BYTES, LOG_PARAMETER_BYTES)
}

/// Validates the selected authorized true-peak and F06 log assets and binds their actual values.
///
/// Asset hashes, scope, profile and generated compact/expanded FIR words are checked before a
/// typed provider is returned. This does not imply a complete Meter provider set or prepass.
pub fn validate_approved_provider_assets(
    true_peak_bytes: &[u8],
    log_bytes: &[u8],
) -> Result<TruePeakParameters, TruePeakError> {
    if sha256(true_peak_bytes) != TRUE_PEAK_PARAMETER_SHA256 {
        return Err(TruePeakError::ParameterUnavailable);
    }
    let parameter: Value =
        serde_json::from_slice(true_peak_bytes).map_err(|_| TruePeakError::ParameterUnavailable)?;
    if parameter["classification"] != "AUTHORIZED_COMPILED_FUNCTIONAL_CONSTANTS"
        || parameter["delivery_scope"] != "INTERNAL_ONLY"
        || parameter["public_or_third_party_distribution"] != "NOT_AUTHORIZED"
        || parameter["sample_rate_hz"] != 48_000
        || parameter["logical_channel_order"]
            != serde_json::json!(["L", "R", "C", "LFE", "Ls", "Rs"])
    {
        return Err(TruePeakError::ParameterUnavailable);
    }
    let stage_a = words_from_hex(&parameter["stage_a_unique"]["words"], 6)?;
    let stage_b = words_from_hex(&parameter["stage_b_unique"]["words"], 5)?;
    let expanded_a = expand_mirrored_a(&stage_a);
    let expanded_b = expand_mirrored_b(&stage_b);
    let mut compact = Vec::with_capacity(44);
    for word in stage_a.iter().chain(&stage_b) {
        compact.extend_from_slice(&word.to_le_bytes());
    }
    let mut expanded = Vec::with_capacity(96);
    for word in expanded_a.iter().chain(&expanded_b) {
        expanded.extend_from_slice(&word.to_le_bytes());
    }
    if sha256(&compact) != TRUE_PEAK_PARAMETER_COMPACT_SHA256
        || sha256(&expanded) != TRUE_PEAK_PARAMETER_EXPANDED_SHA256
    {
        return Err(TruePeakError::ParameterUnavailable);
    }

    let log = validate_approved_log_constants(log_bytes)?;
    Ok(TruePeakParameters {
        a: expanded_a,
        b: expanded_b,
        log,
    })
}

/// Validates the direct F06 log constants into the exact typed word array consumed by Meter.
pub fn validate_approved_log_constants(bytes: &[u8]) -> Result<[u32; 11], TruePeakError> {
    if sha256(bytes) != LOG_PARAMETER_SHA256 {
        return Err(TruePeakError::ParameterUnavailable);
    }
    let log_parameters: Value =
        serde_json::from_slice(bytes).map_err(|_| TruePeakError::ParameterUnavailable)?;
    if log_parameters["schema"] != "meter-log-mathematical-constants.v1"
        || log_parameters["delivery_scope"] != "INTERNAL_ONLY"
        || log_parameters["public_or_third_party_distribution"] != "NOT_AUTHORIZED"
    {
        return Err(TruePeakError::ParameterUnavailable);
    }
    let words = words_from_hex(&log_parameters["words"], 11)?;
    let mut constants = [0; 11];
    constants.copy_from_slice(&words);
    Ok(constants)
}

pub(crate) fn selected_log_constants() -> Result<[u32; 11], TruePeakError> {
    validate_approved_log_constants(LOG_PARAMETER_BYTES)
}

fn words_from_hex(value: &Value, expected: usize) -> Result<Vec<u32>, TruePeakError> {
    let values = value
        .as_array()
        .filter(|values| values.len() == expected)
        .ok_or(TruePeakError::ParameterUnavailable)?;
    values
        .iter()
        .map(|value| {
            u32::from_str_radix(
                value.as_str().ok_or(TruePeakError::ParameterUnavailable)?,
                16,
            )
            .map_err(|_| TruePeakError::ParameterUnavailable)
        })
        .collect()
}

fn expand_mirrored_a(unique: &[u32]) -> [u32; FIR_TAPS] {
    std::array::from_fn(|index| unique[index.min(FIR_TAPS - 1 - index)])
}

fn expand_mirrored_b(unique: &[u32]) -> [u32; FIR_TAPS] {
    std::array::from_fn(|index| {
        if index < 10 {
            unique[index.min(9 - index)]
        } else {
            0
        }
    })
}

fn validate_state(state: &TruePeakState) -> Result<(), TruePeakError> {
    if state.lanes.len() != CHANNEL_COUNT
        || state.lanes.iter().any(|lane| {
            lane.words.len() != FIR_STATE_WORDS || lane.words.iter().any(|word| !is_finite(*word))
        })
        || state
            .true_peak_internal_words
            .iter()
            .chain(&state.sample_peak_internal_words)
            .any(|word| !is_finite(*word))
    {
        return Err(TruePeakError::Input);
    }
    Ok(())
}

fn is_finite(word: u32) -> bool {
    f32::from_bits(word).is_finite()
}

fn interpolate_channel(
    old_state: &[u32],
    source: &[u32],
    parameters: &TruePeakParameters,
) -> Result<(Vec<u32>, Vec<u32>), TruePeakError> {
    if old_state.len() != FIR_STATE_WORDS || source.len() != SOURCE_SAMPLES {
        return Err(TruePeakError::Input);
    }
    let mut state = old_state.to_vec();
    for (index, source_word) in source.iter().enumerate() {
        let quarter_scaled = multiply_words(*source_word, QUARTER_WORD)?;
        state[11 + index] = add_words(quarter_scaled, TRUE_PEAK_NOISE_WORD)?;
    }

    let first = fir(&state, SOURCE_SAMPLES, &parameters.a)?;
    for index in 0..SOURCE_SAMPLES {
        state[SECOND_STAGE_START + 9 + 2 * index] = first[index];
        state[SECOND_STAGE_START + 10 + 2 * index] = state[6 + index];
    }

    let first_history = state[1024..1035].to_vec();
    state[..11].copy_from_slice(&first_history);

    let second = fir(&state[SECOND_STAGE_START..], 2048, &parameters.b)?;
    let mut output = Vec::with_capacity(TRUE_PEAK_SAMPLES);
    for index in 0..2048 {
        output.push(second[index]);
        output.push(state[SECOND_STAGE_START + 5 + index]);
    }

    let second_history = state[SECOND_STAGE_START + 2048..SECOND_STAGE_START + 2057].to_vec();
    state[SECOND_STAGE_START..SECOND_STAGE_START + 9].copy_from_slice(&second_history);
    Ok((output, state))
}

fn fir(
    data: &[u32],
    output_count: usize,
    coefficients: &[u32; FIR_TAPS],
) -> Result<Vec<u32>, TruePeakError> {
    if data.len() < output_count.saturating_add(FIR_TAPS - 1) {
        return Err(TruePeakError::Input);
    }
    let mut output = Vec::with_capacity(output_count);
    for index in 0..output_count {
        let mut taps = [0; 4];
        for lane in 0..4 {
            let at_four = multiply_words(data[index + lane + 4], coefficients[lane + 4])?;
            let at_zero = multiply_words(data[index + lane], coefficients[lane])?;
            let first_sum = add_words(at_four, at_zero)?;
            let at_eight = multiply_words(data[index + lane + 8], coefficients[lane + 8])?;
            taps[lane] = add_words(first_sum, at_eight)?;
        }
        let even_lanes = add_words(taps[0], taps[2])?;
        let odd_lanes = add_words(taps[1], taps[3])?;
        output.push(add_words(even_lanes, odd_lanes)?);
    }
    Ok(output)
}

fn measure_channel(
    measurement: &[u32],
    source: &[u32],
    factor: u8,
    parameters: &TruePeakParameters,
) -> Result<TruePeakLaneMetrics, TruePeakError> {
    if source.len() != SOURCE_SAMPLES
        || (factor == 4 && measurement.len() != TRUE_PEAK_SAMPLES)
        || (factor == 1 && measurement.len() != SOURCE_SAMPLES)
    {
        return Err(TruePeakError::Input);
    }

    let (measurement_peak, first_absolute_maximum_index) = first_absolute_maximum(measurement)?;
    let factor_word = if factor == 1 {
        0x3f80_0000
    } else {
        0x4080_0000
    };
    let raw_peak_word = multiply_words(measurement_peak, factor_word)?;
    let grouped_base = (first_absolute_maximum_index / 4) * 4;
    let grouped_reported_index = i32::try_from(grouped_base / usize::from(factor))
        .map_err(|_| TruePeakError::NumericFailure)?
        - 7;
    let true_peak_internal_word = true_peak_metric(raw_peak_word, &parameters.log)?;
    let true_peak_host_word = multiply_words(true_peak_internal_word, HOST_SCALE_WORD)?;

    let (sample_peak_raw_word, sample_peak_first_index) = first_absolute_maximum(source)?;
    let sample_peak_internal_word = sample_peak_metric(sample_peak_raw_word, &parameters.log)?;
    let sample_peak_host_word = multiply_words(sample_peak_internal_word, HOST_SCALE_WORD)?;
    Ok(TruePeakLaneMetrics {
        raw_peak_word,
        first_absolute_maximum_index,
        grouped_reported_index,
        factor,
        true_peak_internal_word,
        true_peak_host_word,
        sample_peak_raw_word,
        sample_peak_first_index,
        sample_peak_internal_word,
        sample_peak_host_word,
    })
}

fn first_absolute_maximum(words: &[u32]) -> Result<(u32, usize), TruePeakError> {
    let first = *words.first().ok_or(TruePeakError::Input)?;
    if !is_finite(first) {
        return Err(TruePeakError::Input);
    }
    let mut maximum = first & 0x7fff_ffff;
    let mut maximum_index = 0;
    for (index, word) in words.iter().enumerate().skip(1) {
        if !is_finite(*word) {
            return Err(TruePeakError::Input);
        }
        let candidate = *word & 0x7fff_ffff;
        if f32::from_bits(candidate) > f32::from_bits(maximum) {
            maximum = candidate;
            maximum_index = index;
        }
    }
    Ok((maximum, maximum_index))
}

fn sample_peak_metric(peak_word: u32, log_constants: &[u32; 11]) -> Result<u32, TruePeakError> {
    let peak = f32::from_bits(peak_word);
    let internal = if peak < TRUE_PEAK_THRESHOLD {
        TRUE_PEAK_FLOOR_WORD
    } else {
        let value = if peak > 1.0 { 0x3f80_0000 } else { peak_word };
        let logarithm = ordered_log(value, 0, log_constants)?;
        let scaled = multiply_words(logarithm, log_constants[9])?;
        let metric = multiply_words(scaled, FIVE_HALVES_WORD)?;
        clamp_words(metric, TRUE_PEAK_FLOOR_WORD, TRUE_PEAK_CEILING_WORD)
    };
    Ok(internal)
}

fn true_peak_metric(raw_peak_word: u32, log_constants: &[u32; 11]) -> Result<u32, TruePeakError> {
    let raw_peak = f32::from_bits(raw_peak_word);
    if raw_peak < TRUE_PEAK_THRESHOLD {
        return Ok(TRUE_PEAK_FLOOR_WORD);
    }
    if raw_peak > TRUE_PEAK_CAP {
        return Ok(TRUE_PEAK_CEILING_WORD);
    }
    let quarter_peak = multiply_words(raw_peak_word, QUARTER_WORD)?;
    let logarithm = ordered_log(quarter_peak, 2, log_constants)?;
    let scaled = multiply_words(logarithm, log_constants[9])?;
    multiply_words(scaled, FIVE_HALVES_WORD)
}

pub(crate) fn energy_metric(
    mantissa_word: u32,
    offset: i32,
    log_constants: &[u32; 11],
) -> Result<u32, TruePeakError> {
    let mantissa = f32::from_bits(mantissa_word);
    if !mantissa.is_finite() || mantissa < 0.0 {
        return Err(TruePeakError::Input);
    }

    let shift = shift_word(mantissa_word);
    let offset_from_shift = offset
        .checked_sub(shift)
        .ok_or(TruePeakError::NumericFailure)?;
    if mantissa == 0.0 || offset_from_shift <= -39 {
        return Ok(TRUE_PEAK_FLOOR_WORD);
    }

    let a = if offset_from_shift < -32 {
        let normalized = multiply_words(power_of_two(shift)?, mantissa_word)?;
        let logarithm = ordered_log(
            normalized,
            offset_from_shift
                .checked_add(8)
                .ok_or(TruePeakError::NumericFailure)?,
            log_constants,
        )?;
        subtract_words(
            multiply_words(logarithm, log_constants[8])?,
            log_constants[10],
        )?
    } else {
        let logarithm = ordered_log(
            mantissa_word,
            offset.checked_add(1).ok_or(TruePeakError::NumericFailure)?,
            log_constants,
        )?;
        multiply_words(logarithm, log_constants[8])?
    };

    add_words(
        multiply_words(0x4000_0000, a)?,
        multiply_words(0x3f00_0000, a)?,
    )
}

/// Updates one normalized F06L energy accumulator value from an explicit power sum.
pub fn accumulate_normalized_energy(
    old_mantissa_word: u32,
    old_exponent: i32,
    power_sum_word: u32,
) -> Result<(u32, i32), TruePeakError> {
    let old_mantissa = f32::from_bits(old_mantissa_word);
    let power_sum = f32::from_bits(power_sum_word);
    if !old_mantissa.is_finite() || old_mantissa < 0.0 || !power_sum.is_finite() || power_sum < 0.0
    {
        return Err(TruePeakError::Input);
    }

    let shift = shift_word(power_sum_word);
    if old_exponent <= shift {
        let next_exponent = old_exponent
            .checked_sub(1)
            .ok_or(TruePeakError::NumericFailure)?;
        let scaled_power_sum = multiply_words(power_of_two(next_exponent)?, power_sum_word)?;
        let retained = multiply_words(0x3f00_0000, old_mantissa_word)?;
        let result = add_words(scaled_power_sum, retained)?;
        Ok((result, next_exponent))
    } else {
        let distance = old_exponent
            .checked_sub(shift)
            .and_then(|value| value.checked_add(1))
            .ok_or(TruePeakError::NumericFailure)?;
        let retained = if distance <= 31 {
            let negative_distance = distance
                .checked_neg()
                .ok_or(TruePeakError::NumericFailure)?;
            multiply_words(power_of_two(negative_distance)?, old_mantissa_word)?
        } else {
            0
        };
        let scaled_power_sum = multiply_words(power_of_two(shift - 1)?, power_sum_word)?;
        let result = add_words(scaled_power_sum, retained)?;
        let next_exponent = if f32::from_bits(result) == 0.0 {
            old_exponent
        } else {
            shift - 1
        };
        Ok((result, next_exponent))
    }
}

/// Normalizes one explicit F06L.8 window-end energy value.
pub fn normalize_energy_window(
    mantissa_word: u32,
    exponent: i32,
    window_samples: u32,
) -> Result<(u32, i32), TruePeakError> {
    let mantissa = f32::from_bits(mantissa_word);
    if !mantissa.is_finite() || mantissa < 0.0 || !(1..=48_000).contains(&window_samples) {
        return Err(TruePeakError::Input);
    }

    let samples_word = FiniteF32::new(window_samples as f32)
        .map(FiniteF32::to_bits)
        .map_err(|_| TruePeakError::NumericFailure)?;
    let reciprocal = multiply_words(
        divide_words(WINDOW_NORMALIZATION_M_WORD, samples_word)?,
        WINDOW_NORMALIZATION_J_WORD,
    )?;
    let shift = shift_word(reciprocal);
    let scaled_reciprocal = multiply_words(power_of_two(shift)?, reciprocal)?;
    let mut output_mantissa = multiply_words(scaled_reciprocal, mantissa_word)?;
    let output_exponent = shift
        .checked_add(exponent)
        .and_then(|value| value.checked_sub(16))
        .ok_or(TruePeakError::NumericFailure)?;

    if output_exponent < 0 {
        let normalization = output_exponent
            .checked_neg()
            .ok_or(TruePeakError::NumericFailure)?;
        output_mantissa = multiply_words(power_of_two(normalization)?, output_mantissa)?;
        Ok((output_mantissa, 0))
    } else {
        Ok((output_mantissa, output_exponent))
    }
}

/// Reduces one explicit ordered sequence of weighted energy values.
pub fn ordered_energy_reduction(
    source_exponents: &[i32],
    weights: &[u32],
    energies: &[u32],
) -> Result<(u32, i32), TruePeakError> {
    let count = source_exponents.len();
    if count > 16 || weights.len() != count || energies.len() != count {
        return Err(TruePeakError::Input);
    }
    if source_exponents
        .iter()
        .any(|exponent| !(0..=60).contains(exponent))
        || weights
            .iter()
            .chain(energies)
            .any(|word| !is_finite(*word) || f32::from_bits(*word) < 0.0)
    {
        return Err(TruePeakError::Input);
    }

    let mut accumulator = 0;
    let mut exponent = 48_i32;
    for ((source_exponent, weight), energy) in source_exponents.iter().zip(weights).zip(energies) {
        let weighted = multiply_words(*weight, *energy)?;
        let (previous_scale, added) = if exponent <= *source_exponent {
            let distance = source_exponent
                .checked_sub(exponent)
                .ok_or(TruePeakError::NumericFailure)?
                .min(30);
            exponent = exponent
                .checked_sub(1)
                .ok_or(TruePeakError::NumericFailure)?;
            let scale_exponent = distance
                .checked_neg()
                .and_then(|value| value.checked_sub(1))
                .ok_or(TruePeakError::NumericFailure)?;
            (
                0x3f00_0000,
                multiply_words(power_of_two(scale_exponent)?, weighted)?,
            )
        } else {
            let distance = exponent
                .checked_sub(*source_exponent)
                .ok_or(TruePeakError::NumericFailure)?
                .min(30);
            exponent = source_exponent
                .checked_sub(1)
                .ok_or(TruePeakError::NumericFailure)?;
            let scale_exponent = distance
                .checked_neg()
                .and_then(|value| value.checked_sub(1))
                .ok_or(TruePeakError::NumericFailure)?;
            (
                power_of_two(scale_exponent)?,
                multiply_words(weighted, 0x3f00_0000)?,
            )
        };
        accumulator = add_words(multiply_words(previous_scale, accumulator)?, added)?;
    }

    if exponent > 27 {
        let scale_exponent = exponent
            .checked_sub(27)
            .and_then(i32::checked_neg)
            .ok_or(TruePeakError::NumericFailure)?;
        accumulator = multiply_words(power_of_two(scale_exponent)?, accumulator)?;
        exponent = 27;
    }

    let scaled_accumulator = multiply_words(0x3e00_0000, accumulator)?;
    let shift = shift_word(scaled_accumulator);
    let normalization_shift = (30 - exponent).min(shift);
    let mantissa = multiply_words(power_of_two(normalization_shift - 3)?, accumulator)?;
    let result_exponent = normalization_shift
        .checked_add(exponent)
        .ok_or(TruePeakError::NumericFailure)?;
    Ok((mantissa, result_exponent))
}

fn ordered_log(value_word: u32, offset: i32, constants: &[u32; 11]) -> Result<u32, TruePeakError> {
    let value = f32::from_bits(value_word);
    if !value.is_finite() {
        return Err(TruePeakError::NumericFailure);
    }
    if value <= 0.0 {
        return Ok(0xbf80_0000);
    }
    let shift = shift_word(value_word);
    if shift >= 31 {
        let r = offset - shift;
        if r < -46 {
            return Ok(0xbf80_0000);
        }
        let scaled_r = integer_times_power_of_two(r, 25)?;
        let log10 = multiply_words(scaled_r, constants[7])?;
        return multiply_words(log10, 0x4000_0000);
    }

    let scaled = multiply_words(power_of_two(shift)?, value_word)?;
    let x = subtract_words(scaled, 0x3f80_0000)?;
    let mut p = subtract_words(constants[0], multiply_words(constants[1], x)?)?;
    for coefficient in &constants[2..=6] {
        p = subtract_words(*coefficient, multiply_words(p, x)?)?;
    }
    let px = multiply_words(p, x)?;
    let pxx = multiply_words(px, x)?;
    let polynomial = subtract_words(x, pxx)?;
    let exponent_offset = integer_times_power_of_two(offset - shift, 26)?;
    let bias = multiply_words(exponent_offset, constants[7])?;
    add_words(multiply_words(polynomial, ONE_THIRTY_SECOND_WORD)?, bias)
}

pub(crate) fn shift_word(word: u32) -> i32 {
    let doubled = word.wrapping_mul(2);
    let exponent = if doubled == 0 {
        -127
    } else {
        i32::try_from(doubled >> 24).expect("top byte fits i32") - 126
    };
    (-exponent).clamp(0, 60)
}

pub(crate) fn power_of_two(exponent: i32) -> Result<u32, TruePeakError> {
    if !(-126..=127).contains(&exponent) {
        return Err(TruePeakError::NumericFailure);
    }
    let exponent_bits = u32::try_from(exponent + 127).map_err(|_| TruePeakError::NumericFailure)?;
    Ok(exponent_bits << 23)
}

fn integer_times_power_of_two(value: i32, power: i16) -> Result<u32, TruePeakError> {
    let integer = i16::try_from(value).map_err(|_| TruePeakError::NumericFailure)?;
    let integer_word = f32::from(integer).to_bits();
    let scale = 2_f32.powi(i32::from(power)).to_bits();
    multiply_words(integer_word, scale)
}

fn clamp_words(value: u32, lower: u32, upper: u32) -> u32 {
    let value = f32::from_bits(value);
    if value > f32::from_bits(upper) {
        upper
    } else if value < f32::from_bits(lower) {
        lower
    } else {
        value.to_bits()
    }
}

fn final_host_word(values: &[u32; CHANNEL_COUNT]) -> Result<u32, TruePeakError> {
    let mut maximum = values[0];
    for candidate in values.iter().copied().skip(1) {
        if !is_finite(candidate) {
            return Err(TruePeakError::Input);
        }
        if f32::from_bits(candidate) > f32::from_bits(maximum) {
            maximum = candidate;
        }
    }
    multiply_words(maximum, HOST_SCALE_WORD)
}

pub(crate) fn multiply_words(left: u32, right: u32) -> Result<u32, TruePeakError> {
    let left = FiniteF32::new(f32::from_bits(left)).map_err(|_| TruePeakError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right)).map_err(|_| TruePeakError::NumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| TruePeakError::NumericFailure)
}

pub(crate) fn divide_words(numerator: u32, denominator: u32) -> Result<u32, TruePeakError> {
    let numerator = f32::from_bits(numerator);
    let denominator = f32::from_bits(denominator);
    if !numerator.is_finite() || !denominator.is_finite() || denominator == 0.0 {
        return Err(TruePeakError::NumericFailure);
    }
    FiniteF32::new(numerator / denominator)
        .map(FiniteF32::to_bits)
        .map_err(|_| TruePeakError::NumericFailure)
}

pub(crate) fn add_words(left: u32, right: u32) -> Result<u32, TruePeakError> {
    let left = FiniteF32::new(f32::from_bits(left)).map_err(|_| TruePeakError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right)).map_err(|_| TruePeakError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| TruePeakError::NumericFailure)
}

pub(crate) fn subtract_words(left: u32, right: u32) -> Result<u32, TruePeakError> {
    let left = FiniteF32::new(f32::from_bits(left)).map_err(|_| TruePeakError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right)).map_err(|_| TruePeakError::NumericFailure)?;
    r32_sub(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| TruePeakError::NumericFailure)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests;
