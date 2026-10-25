// pattern: Functional Core
//! Selected F06ND peak scheduler, limiters and none dynamics.

use crate::drc::none_drc_dialnorm_offset::{
    NoneDrcDialnormOffsetError, none_drc_dialnorm_offset_word,
};
use crate::drc::none_drc_normalizer::{NoneDrcNormalizationError, normalize_none_drc_metric};
use crate::drc::none_drc_quantizer::{
    NoneDrcQuantizerError, NoneDrcQuantizerKind, quantize_none_drc_control,
};
use crate::numeric::{FiniteF32, R32Error, r32_add, r32_mul, r32_sub};

const BLOCK_COUNT: usize = 6;
const POSITIVE_ZERO: f32 = f32::from_bits(0x0000_0000);
const ONE: f32 = 1.0;
const HALF: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
struct NoneDynamicsState {
    level: f32,
    hold: u16,
    log_gain: f32,
}

impl NoneDynamicsState {
    const fn cold() -> Self {
        Self {
            level: ONE,
            hold: 0,
            log_gain: POSITIVE_ZERO,
        }
    }

    fn advance(limiter_gain: f32) -> Self {
        Self {
            level: ONE,
            hold: 0,
            log_gain: POSITIVE_ZERO.min(limiter_gain),
        }
    }

    fn words(self) -> (u32, u16, u32) {
        (self.level.to_bits(), self.hold, self.log_gain.to_bits())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LimiterState {
    gain: f32,
    hold: u16,
}

impl LimiterState {
    const fn cold() -> Self {
        Self {
            gain: POSITIVE_ZERO,
            hold: 0,
        }
    }

    fn words(self) -> (u32, u16) {
        (self.gain.to_bits(), self.hold)
    }
}

/// Selected continuous scheduler state for one programme.
#[derive(Debug, PartialEq)]
pub struct NoneDrcSchedulerState {
    rf: NoneDynamicsState,
    line: NoneDynamicsState,
    rf_limiter: LimiterState,
    line_limiter: LimiterState,
    peak_six: [f32; BLOCK_COUNT],
    generation: u64,
}

impl NoneDrcSchedulerState {
    pub const fn cold() -> Self {
        Self {
            rf: NoneDynamicsState::cold(),
            line: NoneDynamicsState::cold(),
            rf_limiter: LimiterState::cold(),
            line_limiter: LimiterState::cold(),
            peak_six: [POSITIVE_ZERO; BLOCK_COUNT],
            generation: 0,
        }
    }

    pub fn rf_dynamics_words(&self) -> (u32, u16, u32) {
        self.rf.words()
    }

    pub fn line_dynamics_words(&self) -> (u32, u16, u32) {
        self.line.words()
    }

    pub fn rf_limiter_words(&self) -> (u32, u16) {
        self.rf_limiter.words()
    }

    pub fn line_limiter_words(&self) -> (u32, u16) {
        self.line_limiter.words()
    }

    pub fn peak_six_words(&self) -> [u32; BLOCK_COUNT] {
        self.peak_six.map(f32::to_bits)
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    fn staged_next_state(&self) -> Self {
        Self {
            rf: self.rf,
            line: self.line,
            rf_limiter: self.rf_limiter,
            line_limiter: self.line_limiter,
            peak_six: self.peak_six,
            generation: self.generation,
        }
    }
}

/// One scheduler input row. RMS levels are included as validated peer output;
/// the selected scheduler consumes peak inputs and dialnorm only.
pub struct NoneDrcSchedulerFrameInput {
    pub dialnorm: i32,
    pub rms_levels_six: [FiniteF32; BLOCK_COUNT],
    pub peak_inputs_six: [FiniteF32; BLOCK_COUNT],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoneDrcSchedulerFrameOutput {
    pub rf_packed_word_u16: u16,
    pub rf_flag_u16: u16,
    pub line_packed_words_u16: [u16; BLOCK_COUNT],
    pub line_flags_u16: [u16; BLOCK_COUNT],
    pub rf_aux_packed_word_u16: u16,
    pub rf_limiter_packed_word_u16: u16,
    pub line_aux_packed_words_u16: [u16; BLOCK_COUNT],
    pub line_limiter_packed_words_u16: [u16; BLOCK_COUNT],
    pub line_minimum_signed_word: i16,
    pub peak_logs_six_words: [u32; BLOCK_COUNT],
    pub line_gain_words: [u32; BLOCK_COUNT],
}

#[derive(Debug, PartialEq)]
pub struct NoneDrcSchedulerFrameTransition {
    pub output: NoneDrcSchedulerFrameOutput,
    pub next_state: NoneDrcSchedulerState,
}

/// Lower-level failures from one selected scheduler frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoneDrcSchedulerError {
    Numeric(R32Error),
    GenerationOverflow,
    Dialnorm(NoneDrcDialnormOffsetError),
    Normalization(NoneDrcNormalizationError),
    Quantization(NoneDrcQuantizerError),
}

impl NoneDrcSchedulerError {
    /// Maps only the washed F06ND intermediate nonfinite/overflow class.
    ///
    /// Dialnorm, negative-input, unsupported-kind and other nested labels remain
    /// unbound and return `None`.
    pub const fn outcome_code(self) -> Option<&'static str> {
        match self {
            Self::Numeric(_)
            | Self::GenerationOverflow
            | Self::Normalization(NoneDrcNormalizationError::NumericNonFinite)
            | Self::Quantization(NoneDrcQuantizerError::NumericNonFinite) => {
                Some("NUMERIC_FAILURE")
            }
            Self::Dialnorm(_) | Self::Normalization(_) | Self::Quantization(_) => None,
        }
    }
}

impl From<R32Error> for NoneDrcSchedulerError {
    fn from(error: R32Error) -> Self {
        Self::Numeric(error)
    }
}

impl From<NoneDrcDialnormOffsetError> for NoneDrcSchedulerError {
    fn from(error: NoneDrcDialnormOffsetError) -> Self {
        Self::Dialnorm(error)
    }
}

impl From<NoneDrcNormalizationError> for NoneDrcSchedulerError {
    fn from(error: NoneDrcNormalizationError) -> Self {
        Self::Normalization(error)
    }
}

impl From<NoneDrcQuantizerError> for NoneDrcSchedulerError {
    fn from(error: NoneDrcQuantizerError) -> Self {
        Self::Quantization(error)
    }
}

/// Runs one selected six-block F06ND scheduler frame without statistics or
/// programme composition.
pub fn transition_none_drc_scheduler_frame(
    state: &NoneDrcSchedulerState,
    input: &NoneDrcSchedulerFrameInput,
) -> Result<NoneDrcSchedulerFrameTransition, NoneDrcSchedulerError> {
    // These finite RMS values belong to the preceding statistics producer.
    // This selected none scheduler deliberately does not consume them.

    let dialnorm_offset = f32::from_bits(none_drc_dialnorm_offset_word(input.dialnorm)?);
    let half_offset = r32_mul_values(dialnorm_offset, HALF)?;
    let mut peak_logs_six = [POSITIVE_ZERO; BLOCK_COUNT];
    for (block, peak_input) in input.peak_inputs_six.iter().enumerate() {
        let normalized = normalize_none_drc_metric(peak_input.value())?;
        peak_logs_six[block] = clamp(r32_add_values(normalized, half_offset)?);
    }

    let mut next_state = state.staged_next_state();
    next_state.peak_six = peak_logs_six;

    let rf_peak_log = peak_logs_six[1..]
        .iter()
        .copied()
        .fold(peak_logs_six[0], f32::min);
    let rf_limiter_input = clamp(r32_sub_values(rf_peak_log, r32_div_values(11.0, 288.0)?)?);
    next_state.rf_limiter = advance_limiter(state.rf_limiter, rf_limiter_input, LimiterKind::Rf)?;

    let mut line_packed_words_u16 = [0; BLOCK_COUNT];
    let mut line_limiter_packed_words_u16 = [0; BLOCK_COUNT];
    let mut line_gain_words = [0; BLOCK_COUNT];
    let mut line_code_minimum = i16::MAX;
    for block in 0..BLOCK_COUNT {
        next_state.line_limiter = advance_limiter(
            next_state.line_limiter,
            peak_logs_six[block],
            LimiterKind::Line,
        )?;
        next_state.line = NoneDynamicsState::advance(next_state.line_limiter.gain);
        let line_gain = next_state.line.log_gain;
        line_gain_words[block] = line_gain.to_bits();

        let line_code =
            quantize_none_drc_control(line_gain, NoneDrcQuantizerKind::Line)?.storage_word();
        line_packed_words_u16[block] = line_code;
        line_limiter_packed_words_u16[block] =
            quantize_none_drc_control(next_state.line_limiter.gain, NoneDrcQuantizerKind::Line)?
                .storage_word();
        line_code_minimum = line_code_minimum.min(signed_i16(line_code));
    }

    let mut rf_log_gain_minimum = ONE;
    for _ in 0..BLOCK_COUNT {
        next_state.rf = NoneDynamicsState::advance(next_state.rf_limiter.gain);
        rf_log_gain_minimum = rf_log_gain_minimum.min(next_state.rf.log_gain);
    }

    let rf_final =
        quantize_none_drc_control(rf_log_gain_minimum, NoneDrcQuantizerKind::Rf)?.storage_word();
    let rf_limiter =
        quantize_none_drc_control(next_state.rf_limiter.gain, NoneDrcQuantizerKind::Rf)?
            .storage_word();

    next_state.generation = state
        .generation
        .checked_add(1)
        .ok_or(NoneDrcSchedulerError::GenerationOverflow)?;
    let line_aux_packed_words_u16 = line_packed_words_u16;
    let output = NoneDrcSchedulerFrameOutput {
        rf_packed_word_u16: rf_final,
        rf_flag_u16: 1,
        line_packed_words_u16,
        line_flags_u16: [1; BLOCK_COUNT],
        rf_aux_packed_word_u16: rf_final,
        rf_limiter_packed_word_u16: rf_limiter,
        line_aux_packed_words_u16,
        line_limiter_packed_words_u16,
        line_minimum_signed_word: line_code_minimum,
        peak_logs_six_words: peak_logs_six.map(f32::to_bits),
        line_gain_words,
    };

    Ok(NoneDrcSchedulerFrameTransition { output, next_state })
}

#[derive(Clone, Copy)]
enum LimiterKind {
    Rf,
    Line,
}

impl LimiterKind {
    fn target_offset(self) -> Result<f32, R32Error> {
        match self {
            Self::Rf => r32_div_values(1.0, 128.0),
            Self::Line => r32_div_values(1.0, 256.0),
        }
    }

    fn increment(self) -> Result<f32, R32Error> {
        match self {
            Self::Rf => r32_div_values(3.0, 2_000.0),
            Self::Line => r32_div_values(1.0, 8_000.0),
        }
    }

    const fn hold_duration(self) -> u16 {
        match self {
            Self::Rf => 5,
            Self::Line => 30,
        }
    }
}

fn advance_limiter(
    state: LimiterState,
    peak_log: f32,
    kind: LimiterKind,
) -> Result<LimiterState, R32Error> {
    let mut value = r32_sub_values(peak_log, r32_div_values(7.0, 48.0)?)?;
    if value >= r32_div_values(1.0, 6.0)? {
        value = ONE;
    } else {
        value = r32_mul_values(value, 6.0)?;
    }

    let target = r32_sub_values(value, kind.target_offset()?)?;
    let incremented = r32_add_values(state.gain, kind.increment()?)?;
    if incremented <= target {
        if state.hold > 0 {
            Ok(LimiterState {
                gain: state.gain,
                hold: state.hold - 1,
            })
        } else {
            Ok(LimiterState {
                gain: incremented,
                hold: 0,
            })
        }
    } else {
        Ok(LimiterState {
            gain: target,
            hold: kind.hold_duration(),
        })
    }
}

fn signed_i16(bits: u16) -> i16 {
    i16::from_ne_bytes(bits.to_ne_bytes())
}

fn clamp(value: f32) -> f32 {
    (-ONE).max(value).min(ONE)
}

fn r32_mul_values(left: f32, right: f32) -> Result<f32, R32Error> {
    let left = FiniteF32::new(left).map_err(|_| R32Error::NonFiniteResult)?;
    let right = FiniteF32::new(right).map_err(|_| R32Error::NonFiniteResult)?;
    r32_mul(left, right)
        .map(FiniteF32::value)
        .map_err(|_| R32Error::NonFiniteResult)
}

fn r32_add_values(left: f32, right: f32) -> Result<f32, R32Error> {
    let left = FiniteF32::new(left).map_err(|_| R32Error::NonFiniteResult)?;
    let right = FiniteF32::new(right).map_err(|_| R32Error::NonFiniteResult)?;
    r32_add(left, right)
        .map(FiniteF32::value)
        .map_err(|_| R32Error::NonFiniteResult)
}

fn r32_sub_values(left: f32, right: f32) -> Result<f32, R32Error> {
    let left = FiniteF32::new(left).map_err(|_| R32Error::NonFiniteResult)?;
    let right = FiniteF32::new(right).map_err(|_| R32Error::NonFiniteResult)?;
    r32_sub(left, right)
        .map(FiniteF32::value)
        .map_err(|_| R32Error::NonFiniteResult)
}

fn r32_div_values(left: f32, right: f32) -> Result<f32, R32Error> {
    let result = left / right;
    FiniteF32::new(result)
        .map(FiniteF32::value)
        .map_err(|_| R32Error::NonFiniteResult)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_overflow_is_rejected_without_returning_a_transition() {
        let mut state = NoneDrcSchedulerState::cold();
        state.generation = u64::MAX;
        let zero = FiniteF32::new(POSITIVE_ZERO).unwrap();
        let input = NoneDrcSchedulerFrameInput {
            dialnorm: 24,
            rms_levels_six: [zero; BLOCK_COUNT],
            peak_inputs_six: [zero; BLOCK_COUNT],
        };

        assert_eq!(
            transition_none_drc_scheduler_frame(&state, &input),
            Err(NoneDrcSchedulerError::GenerationOverflow)
        );
        assert_eq!(state.generation(), u64::MAX);
    }
}
