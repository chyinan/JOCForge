// pattern: Functional Core
//! F06U selected single-slot ungated mean and DI-off getter projection.

use crate::meter::true_peak::{
    TruePeakError, energy_metric, selected_log_constants, validate_approved_log_constants,
};
use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};
use sha2::{Digest, Sha256};

const MAX_COMMITTED_COUNT: u32 = 0x7fff_ffff;
const SIGNED_COUNT_BOUNDARY: u32 = 0x8000_0000;
const M_WORD: u32 = 0x4f00_0000;
const J_WORD: u32 = 0x3000_0000;
const ADMISSION_METRIC_WORD: u32 = 0xbf0c_0000;
const ZERO_WORD: u32 = 0;
const HALF_WORD: u32 = 0x3f00_0000;
const TWO_WORD: u32 = 0x4000_0000;
const HOST_SCALE_WORD: u32 = 0x4300_0000;
const LOUDNESS_FLOOR_WORD: u32 = 0xbf70_0000;
const SPEECH_SENTINEL_INTERNAL_WORD: u32 = 0xbc00_0000;
const THRESHOLD_WORD: u32 = 0x42c8_0000;
const STATE_HEADER: &[u8; 8] = b"UMSTATE1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UngatedMeanError {
    InvalidInput,
    InvalidState,
    UnsupportedConfiguration,
    ParameterUnavailable,
    NumericFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiOffGetterError {
    InvalidInput,
    UnsupportedConfiguration,
    NumericFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UngatedMeanState {
    count: u32,
    mean_word: u32,
    exponent: i32,
    phase: u32,
    cursor: u32,
    filled: u32,
    linear_energy_word: u32,
    result_word: u32,
}

impl UngatedMeanState {
    pub fn cold() -> Self {
        Self {
            count: 0,
            mean_word: ZERO_WORD,
            exponent: 30,
            phase: 0,
            cursor: 0,
            filled: 0,
            linear_energy_word: ZERO_WORD,
            result_word: LOUDNESS_FLOOR_WORD,
        }
    }

    pub const fn count(&self) -> u32 {
        self.count
    }

    pub const fn mean_word(&self) -> u32 {
        self.mean_word
    }

    pub const fn exponent(&self) -> i32 {
        self.exponent
    }

    pub const fn phase(&self) -> u32 {
        self.phase
    }

    pub const fn cursor(&self) -> u32 {
        self.cursor
    }

    pub const fn filled(&self) -> u32 {
        self.filled
    }

    pub const fn linear_energy_word(&self) -> u32 {
        self.linear_energy_word
    }

    pub const fn result_word(&self) -> u32 {
        self.result_word
    }

    /// Explicit value replacement for a caller constructing a staged state.
    #[must_use]
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    /// Explicit value replacement for a caller constructing a staged state.
    #[must_use]
    pub fn with_mean_word(mut self, mean_word: u32) -> Self {
        self.mean_word = mean_word;
        self
    }

    /// Explicit value replacement for a caller constructing a staged state.
    #[must_use]
    pub fn with_exponent(mut self, exponent: i32) -> Self {
        self.exponent = exponent;
        self
    }

    /// Explicit value replacement for a caller constructing a staged state.
    #[must_use]
    pub fn with_filled(mut self, filled: u32) -> Self {
        self.filled = filled;
        self
    }

    /// Serializes the complete F06U state in the approved 40-byte MEAN_STATE_V1 order.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(40);
        bytes.extend_from_slice(STATE_HEADER);
        bytes.extend_from_slice(&self.count.to_le_bytes());
        bytes.extend_from_slice(&self.mean_word.to_le_bytes());
        bytes.extend_from_slice(&self.exponent.to_le_bytes());
        bytes.extend_from_slice(&self.phase.to_le_bytes());
        bytes.extend_from_slice(&self.cursor.to_le_bytes());
        bytes.extend_from_slice(&self.filled.to_le_bytes());
        bytes.extend_from_slice(&self.linear_energy_word.to_le_bytes());
        bytes.extend_from_slice(&self.result_word.to_le_bytes());
        bytes
    }

    pub fn canonical_sha256(&self) -> String {
        format!("{:x}", Sha256::digest(self.canonical_bytes()))
    }

    fn validate(&self) -> Result<(), UngatedMeanError> {
        if self.count > MAX_COMMITTED_COUNT
            || self.phase != 0
            || self.cursor != 0
            || self.filled > 1
        {
            return Err(UngatedMeanError::InvalidState);
        }
        let mean = f32::from_bits(self.mean_word);
        let linear_energy = f32::from_bits(self.linear_energy_word);
        let result = f32::from_bits(self.result_word);
        if !mean.is_finite()
            || mean < 0.0
            || !linear_energy.is_finite()
            || linear_energy < 0.0
            || !result.is_finite()
        {
            return Err(UngatedMeanError::InvalidState);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UngatedMeanInput {
    pub mantissa_word: u32,
    pub exponent: i32,
    pub ungated_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UngatedMeanParameters {
    log_constants: [u32; 11],
}

impl UngatedMeanParameters {
    /// Loads the selected, hash-bound internal F06L log constants without loading FIR parameters.
    pub fn selected() -> Result<Self, UngatedMeanError> {
        let log_constants = selected_log_constants().map_err(|error| match error {
            TruePeakError::ParameterUnavailable => UngatedMeanError::ParameterUnavailable,
            TruePeakError::Input | TruePeakError::NumericFailure => {
                UngatedMeanError::ParameterUnavailable
            }
        })?;
        Ok(Self { log_constants })
    }

    /// Constructs F06U's immutable log dependency from the approved direct log asset.
    pub fn from_approved_log_asset(bytes: &[u8]) -> Result<Self, UngatedMeanError> {
        let log_constants =
            validate_approved_log_constants(bytes).map_err(|error| match error {
                TruePeakError::ParameterUnavailable => UngatedMeanError::ParameterUnavailable,
                TruePeakError::Input | TruePeakError::NumericFailure => {
                    UngatedMeanError::ParameterUnavailable
                }
            })?;
        Ok(Self { log_constants })
    }

    pub const fn log_words(&self) -> &[u32; 11] {
        &self.log_constants
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UngatedMeanStep {
    input_metric_word: u32,
    writer_called: bool,
    next_state: UngatedMeanState,
}

impl UngatedMeanStep {
    pub const fn input_metric_word(&self) -> u32 {
        self.input_metric_word
    }

    pub const fn writer_called(&self) -> bool {
        self.writer_called
    }

    pub const fn next_state(&self) -> &UngatedMeanState {
        &self.next_state
    }

    pub fn into_next_state(self) -> UngatedMeanState {
        self.next_state
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiOffGetterOutput {
    dialogue_word: u32,
    speech_percentage_word: u32,
    selected_internal_word: u32,
    selected_host_word: u32,
    internal_gating_kind: u32,
    public_gating_kind: u32,
    wrapper_config_mode: u32,
    normalized_mode: u32,
    speech_classifier_executed: bool,
}

impl DiOffGetterOutput {
    pub const fn dialogue_word(&self) -> u32 {
        self.dialogue_word
    }

    pub const fn speech_percentage_word(&self) -> u32 {
        self.speech_percentage_word
    }

    pub const fn selected_internal_word(&self) -> u32 {
        self.selected_internal_word
    }

    pub const fn selected_host_word(&self) -> u32 {
        self.selected_host_word
    }

    pub const fn internal_gating_kind(&self) -> u32 {
        self.internal_gating_kind
    }

    pub const fn public_gating_kind(&self) -> u32 {
        self.public_gating_kind
    }

    pub const fn wrapper_config_mode(&self) -> u32 {
        self.wrapper_config_mode
    }

    pub const fn normalized_mode(&self) -> u32 {
        self.normalized_mode
    }

    pub const fn speech_classifier_executed(&self) -> bool {
        self.speech_classifier_executed
    }
}

/// Runs the fixed period-one, capacity-one F06U admission and mean transition.
///
/// The input energy pair is explicit; this function does not own a meter-window
/// clock, derive energy from audio, or commit state outside its returned value.
pub fn process_ungated_mean(
    old: &UngatedMeanState,
    input: UngatedMeanInput,
    parameters: &UngatedMeanParameters,
) -> Result<UngatedMeanStep, UngatedMeanError> {
    old.validate()?;
    let mantissa = f32::from_bits(input.mantissa_word);
    if !mantissa.is_finite() || mantissa < 0.0 || !(0..=30).contains(&input.exponent) {
        return Err(UngatedMeanError::InvalidInput);
    }
    if !input.ungated_enabled {
        return Err(UngatedMeanError::UnsupportedConfiguration);
    }

    let metric_word = energy_metric(
        input.mantissa_word,
        4 - input.exponent,
        &parameters.log_constants,
    )
    .map_err(map_metric_error)?;
    let writer_called = f32::from_bits(metric_word) >= f32::from_bits(ADMISSION_METRIC_WORD);
    if !writer_called {
        return Ok(UngatedMeanStep {
            input_metric_word: metric_word,
            writer_called,
            next_state: old.clone(),
        });
    }

    let count_new = old.count.wrapping_add(1);
    let signed_count = i32::from_ne_bytes(count_new.to_ne_bytes());
    let count_word = finite_word(signed_count as f32)?;
    let q_word = multiply(divide(M_WORD, count_word)?, J_WORD)?;
    let mut exponent_new;
    let mut value;

    if old.exponent <= input.exponent {
        let scale_exponent = old
            .exponent
            .checked_sub(input.exponent)
            .ok_or(UngatedMeanError::NumericFailure)?;
        let scale = power_of_two(scale_exponent)?;
        let scaled_input = multiply(scale, input.mantissa_word)?;
        let delta = multiply(subtract(scaled_input, old.mean_word)?, q_word)?;
        value = add(old.mean_word, delta)?;
        exponent_new = old.exponent;
    } else {
        let distance = old
            .exponent
            .checked_sub(input.exponent)
            .ok_or(UngatedMeanError::NumericFailure)?;
        let negative_distance = distance
            .checked_neg()
            .ok_or(UngatedMeanError::NumericFailure)?;
        let retained_scale = power_of_two(
            negative_distance
                .checked_sub(1)
                .ok_or(UngatedMeanError::NumericFailure)?,
        )?;
        let retained = multiply(retained_scale, old.mean_word)?;
        let input_scale = power_of_two(negative_distance)?;
        let current_delta = subtract(input.mantissa_word, multiply(input_scale, old.mean_word)?)?;
        let current = multiply(multiply(current_delta, q_word)?, HALF_WORD)?;
        value = add(current, retained)?;
        exponent_new = input.exponent - 1;
    }

    if f32::from_bits(value) >= f32::from_bits(HALF_WORD) {
        value = multiply(value, HALF_WORD)?;
        exponent_new = exponent_new
            .checked_sub(1)
            .ok_or(UngatedMeanError::NumericFailure)?;
    }

    let phase_after_event = old
        .phase
        .checked_add(1)
        .ok_or(UngatedMeanError::NumericFailure)?;
    if phase_after_event != 1 {
        return Err(UngatedMeanError::NumericFailure);
    }
    let filled = old.filled.saturating_add(1).min(1);
    let t = if signed_count == 0 { 1 } else { signed_count };
    let min_exponent = 30.min(exponent_new);
    let count_scale = finite_word(t as f32)?;
    let ratio = multiply(multiply(count_word, J_WORD)?, divide(M_WORD, count_scale)?)?;
    let r = multiply(ratio, value)?;
    let combined_scale = power_of_two(
        min_exponent
            .checked_sub(exponent_new)
            .ok_or(UngatedMeanError::NumericFailure)?,
    )?;
    let combined = add(ZERO_WORD, multiply(combined_scale, r)?)?;
    let result_offset = 4_i32
        .checked_sub(min_exponent)
        .ok_or(UngatedMeanError::NumericFailure)?;
    let result_word = energy_metric(combined, result_offset, &parameters.log_constants)
        .map_err(map_metric_error)?;
    let negative_min_exponent = min_exponent
        .checked_neg()
        .ok_or(UngatedMeanError::NumericFailure)?;
    let linear_energy_word = multiply(
        multiply(TWO_WORD, combined)?,
        power_of_two(negative_min_exponent)?,
    )?;

    let committed_count = if count_new >= SIGNED_COUNT_BOUNDARY {
        count_new >> 1
    } else {
        count_new
    };
    let next_state = UngatedMeanState {
        count: committed_count,
        mean_word: value,
        exponent: exponent_new,
        phase: 0,
        cursor: 0,
        filled,
        linear_energy_word,
        result_word,
    };
    next_state.validate()?;
    Ok(UngatedMeanStep {
        input_metric_word: metric_word,
        writer_called,
        next_state,
    })
}

/// Projects the fixed 1770-4/DI-off getter and selected loudness branch.
pub fn di_off_getter(
    itu_host_word: Option<u32>,
    ungated_internal_word: u32,
    metering_mode: &str,
    dialogue_intelligence: bool,
    speech_threshold_word: u32,
) -> Result<DiOffGetterOutput, DiOffGetterError> {
    if metering_mode != "1770-4" || dialogue_intelligence || speech_threshold_word != THRESHOLD_WORD
    {
        return Err(DiOffGetterError::UnsupportedConfiguration);
    }
    let ungated = finite(ungated_internal_word).map_err(|_| DiOffGetterError::InvalidInput)?;
    let itu_internal_word = itu_host_word
        .map(|host| divide(host, HOST_SCALE_WORD))
        .transpose()
        .map_err(|_| DiOffGetterError::InvalidInput)?;
    let select_itu = itu_internal_word
        .is_some_and(|word| f32::from_bits(word) > f32::from_bits(LOUDNESS_FLOOR_WORD));
    let selected_internal_word = if select_itu {
        itu_internal_word.ok_or(DiOffGetterError::InvalidInput)?
    } else {
        ungated.to_bits()
    };
    let selected_host_word = multiply(selected_internal_word, HOST_SCALE_WORD)
        .map_err(|_| DiOffGetterError::NumericFailure)?;
    let dialogue_word = multiply(LOUDNESS_FLOOR_WORD, HOST_SCALE_WORD)
        .map_err(|_| DiOffGetterError::NumericFailure)?;
    let speech_percentage_word = multiply(SPEECH_SENTINEL_INTERNAL_WORD, HOST_SCALE_WORD)
        .map_err(|_| DiOffGetterError::NumericFailure)?;
    Ok(DiOffGetterOutput {
        dialogue_word,
        speech_percentage_word,
        selected_internal_word,
        selected_host_word,
        internal_gating_kind: u32::from(select_itu),
        public_gating_kind: 2 * u32::from(select_itu),
        wrapper_config_mode: 2,
        normalized_mode: 3,
        speech_classifier_executed: false,
    })
}

fn map_metric_error(error: TruePeakError) -> UngatedMeanError {
    match error {
        TruePeakError::Input => UngatedMeanError::InvalidInput,
        TruePeakError::ParameterUnavailable => UngatedMeanError::ParameterUnavailable,
        TruePeakError::NumericFailure => UngatedMeanError::NumericFailure,
    }
}

fn finite(word: u32) -> Result<FiniteF32, UngatedMeanError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| UngatedMeanError::NumericFailure)
}

fn finite_word(value: f32) -> Result<u32, UngatedMeanError> {
    FiniteF32::new(value)
        .map(FiniteF32::to_bits)
        .map_err(|_| UngatedMeanError::NumericFailure)
}

fn add(left: u32, right: u32) -> Result<u32, UngatedMeanError> {
    r32_add(finite(left)?, finite(right)?)
        .map(FiniteF32::to_bits)
        .map_err(|_| UngatedMeanError::NumericFailure)
}

fn subtract(left: u32, right: u32) -> Result<u32, UngatedMeanError> {
    r32_sub(finite(left)?, finite(right)?)
        .map(FiniteF32::to_bits)
        .map_err(|_| UngatedMeanError::NumericFailure)
}

fn multiply(left: u32, right: u32) -> Result<u32, UngatedMeanError> {
    r32_mul(finite(left)?, finite(right)?)
        .map(FiniteF32::to_bits)
        .map_err(|_| UngatedMeanError::NumericFailure)
}

fn divide(numerator: u32, denominator: u32) -> Result<u32, UngatedMeanError> {
    let numerator = finite(numerator)?.value();
    let denominator = finite(denominator)?.value();
    if denominator == 0.0 {
        return Err(UngatedMeanError::NumericFailure);
    }
    finite_word(numerator / denominator)
}

fn power_of_two(exponent: i32) -> Result<u32, UngatedMeanError> {
    if !(-126..=127).contains(&exponent) {
        return Err(UngatedMeanError::NumericFailure);
    }
    let exponent_bits =
        u32::try_from(exponent + 127).map_err(|_| UngatedMeanError::NumericFailure)?;
    Ok(exponent_bits << 23)
}
