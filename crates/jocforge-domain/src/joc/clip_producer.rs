// pattern: Functional Core
//! Explicit sample-boundary clip peak/hold, delay, attenuation and compensation producer.

use std::fmt;

pub const CLIP_CHANNELS: usize = 6;
pub const CLIP_FRAME_SAMPLES: usize = 1_536;
const LFE_CHANNEL: usize = 3;
const PREFIX_SAMPLES: usize = 1_280;
const TAIL_SAMPLES: usize = CLIP_FRAME_SAMPLES - PREFIX_SAMPLES;
const DELAY_SAMPLES: usize = 256;
const DELAY_RING_SAMPLES: usize = 512;
const CLIP_LEVELS: usize = 128;
const ATTACK_HOLD_CALLS: u32 = 5;

pub type ClipSampleFrame = [[f32; CLIP_FRAME_SAMPLES]; CLIP_CHANNELS];
type DelayRing = [[u32; DELAY_RING_SAMPLES]; CLIP_CHANNELS];

/// Explicit ordinary functional constants and lookup tables for this clip slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipFunctionalParameters {
    threshold_words: [u32; CLIP_LEVELS],
    attenuation_words: [u32; CLIP_LEVELS],
    constants: ClipFunctionalConstants,
}

/// F32 constants are kept as words so arithmetic starts from the supplied binary32 values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClipFunctionalConstants {
    pub polynomial_a: u32,
    pub polynomial_b: u32,
    pub polynomial_c: u32,
    pub exponent_scale: u32,
    pub release_step: u32,
    pub level_bias: u32,
}

impl ClipFunctionalParameters {
    pub fn new(
        threshold_words: [u32; CLIP_LEVELS],
        attenuation_words: [u32; CLIP_LEVELS],
        constants: ClipFunctionalConstants,
    ) -> Result<Self, ClipProducerError> {
        let thresholds_valid = threshold_words
            .iter()
            .all(|word| f32::from_bits(*word).is_finite());
        let attenuation_valid = attenuation_words.iter().all(|word| {
            let value = f32::from_bits(*word);
            value.is_finite() && value > 0.0
        });
        let constants_valid = [
            constants.polynomial_a,
            constants.polynomial_b,
            constants.polynomial_c,
            constants.exponent_scale,
            constants.release_step,
            constants.level_bias,
        ]
        .iter()
        .all(|word| f32::from_bits(*word).is_finite());
        if !thresholds_valid || !attenuation_valid || !constants_valid {
            return Err(ClipProducerError::InvalidFunctionalParameters);
        }
        Ok(Self {
            threshold_words,
            attenuation_words,
            constants,
        })
    }

    pub const fn threshold_words(&self) -> &[u32; CLIP_LEVELS] {
        &self.threshold_words
    }

    pub const fn attenuation_words(&self) -> &[u32; CLIP_LEVELS] {
        &self.attenuation_words
    }

    pub const fn constants(&self) -> ClipFunctionalConstants {
        self.constants
    }
}

/// Local sample/state validation failures. No native status equivalence is implied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipProducerError {
    InvalidFunctionalParameters,
    InvalidGainExponent,
    InvalidEnvelopeSeed,
    NonFiniteSample,
    NonFiniteIntermediate,
}

impl fmt::Display for ClipProducerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidFunctionalParameters => "invalid explicit clip parameter provider",
            Self::InvalidGainExponent => "clip gain exponent is outside the F32 power domain",
            Self::InvalidEnvelopeSeed => "clip state seed is outside the selected owner domain",
            Self::NonFiniteSample => "clip input frame contains a non-finite sample",
            Self::NonFiniteIntermediate => "clip processing produced a non-finite intermediate",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ClipProducerError {}

/// Persistent state for one explicitly owned clip producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipProducerState {
    previous_tail_peak_word: u32,
    hold_counter: u32,
    envelope_level_word: u32,
    gain_exponent: i32,
    normalization_word: u32,
    previous_attenuation_word: u32,
    delay_ring_words: DelayRing,
    delay_read_index: u16,
    delay_write_index: u16,
}

impl ClipProducerState {
    /// Constructs the cold selected owner with an independent 256-sample delay ring.
    pub fn cold(
        gain_exponent: i32,
        parameters: &ClipFunctionalParameters,
    ) -> Result<Self, ClipProducerError> {
        if !(-126..=127).contains(&gain_exponent) {
            return Err(ClipProducerError::InvalidGainExponent);
        }
        let normalization = r32_div(gain_exponent as f32, 48.0);
        if !normalization.is_finite() {
            return Err(ClipProducerError::NonFiniteIntermediate);
        }
        Ok(Self {
            previous_tail_peak_word: 0.0_f32.to_bits(),
            hold_counter: 0,
            envelope_level_word: 0.0_f32.to_bits(),
            gain_exponent,
            normalization_word: normalization.to_bits(),
            previous_attenuation_word: parameters.attenuation_words[0],
            delay_ring_words: [[0; DELAY_RING_SAMPLES]; CLIP_CHANNELS],
            delay_read_index: DELAY_SAMPLES as u16,
            delay_write_index: 0,
        })
    }

    /// Restores the explicit envelope fields while retaining the remaining cold fields.
    pub fn with_envelope_seed(
        mut self,
        level_word: u32,
        hold_counter: u32,
    ) -> Result<Self, ClipProducerError> {
        if !f32::from_bits(level_word).is_finite() || hold_counter > ATTACK_HOLD_CALLS {
            return Err(ClipProducerError::InvalidEnvelopeSeed);
        }
        self.envelope_level_word = level_word;
        self.hold_counter = hold_counter;
        Ok(self)
    }

    pub const fn previous_tail_peak_word(&self) -> u32 {
        self.previous_tail_peak_word
    }

    pub const fn hold_counter(&self) -> u32 {
        self.hold_counter
    }

    pub const fn envelope_level_word(&self) -> u32 {
        self.envelope_level_word
    }

    pub const fn gain_exponent(&self) -> i32 {
        self.gain_exponent
    }

    pub const fn normalization_word(&self) -> u32 {
        self.normalization_word
    }

    pub const fn previous_attenuation_word(&self) -> u32 {
        self.previous_attenuation_word
    }

    pub const fn delay_read_index(&self) -> u16 {
        self.delay_read_index
    }

    pub const fn delay_write_index(&self) -> u16 {
        self.delay_write_index
    }

    /// Produces the sample-derived clip index without advancing wrapper delay state.
    pub fn produce_clip_index(
        &self,
        frame: &ClipSampleFrame,
        parameters: &ClipFunctionalParameters,
    ) -> Result<ClipProducerOutput, ClipProducerError> {
        let transition = self.clip_transition(frame, parameters)?;
        let mut after_state = self.clone();
        after_state.previous_tail_peak_word = transition.next_tail_peak.to_bits();
        after_state.hold_counter = transition.next_hold;
        after_state.envelope_level_word = transition.next_level.to_bits();
        Ok(ClipProducerOutput {
            clip_index_u32: transition.clip_index_u32,
            compensated_frame: None,
            after_state,
        })
    }

    /// Processes one owned wrapper frame, including delay/compensation, and returns an after-state proposal.
    /// The caller decides whether to commit that proposal; this API owns no parent transaction.
    pub fn process_wrapper(
        &self,
        frame: &ClipSampleFrame,
        parameters: &ClipFunctionalParameters,
    ) -> Result<ClipProducerOutput, ClipProducerError> {
        let transition = self.clip_transition(frame, parameters)?;
        let attenuation = f32::from_bits(transition.attenuation_word);
        let previous_attenuation = f32::from_bits(self.previous_attenuation_word);
        let ramp_increment = r32_div(r32_sub(attenuation, previous_attenuation), 256.0_f32);
        let one_sixteenth = 1.0_f32 / 16.0_f32;
        let gain_scale = power_of_two(self.gain_exponent);
        let fixed_factor = r32_div(one_sixteenth, attenuation);
        let read_start = usize::from(self.delay_read_index) % DELAY_RING_SAMPLES;
        let write_start = usize::from(self.delay_write_index) % DELAY_RING_SAMPLES;
        let mut compensated = vec![[0.0_f32; CLIP_FRAME_SAMPLES]; CLIP_CHANNELS].into_boxed_slice();
        let mut current_attenuation = previous_attenuation;

        for sample_index in 0..DELAY_SAMPLES {
            let factor = r32_div(one_sixteenth, current_attenuation);
            for channel in 0..CLIP_CHANNELS {
                let delayed_sample =
                    f32::from_bits(self.delay_ring_words[channel][read_start + sample_index]);
                compensated[channel][sample_index] =
                    compensate_sample(factor, delayed_sample, gain_scale);
            }
            current_attenuation = r32_add(current_attenuation, ramp_increment);
        }

        for sample_index in DELAY_SAMPLES..CLIP_FRAME_SAMPLES {
            let source_index = sample_index - DELAY_SAMPLES;
            for (channel, row) in frame.iter().enumerate() {
                compensated[channel][sample_index] =
                    compensate_sample(fixed_factor, row[source_index], gain_scale);
            }
        }

        let mut next_state = self.clone();
        for (channel, row) in frame.iter().enumerate() {
            for tail_index in 0..TAIL_SAMPLES {
                next_state.delay_ring_words[channel][write_start + tail_index] =
                    row[PREFIX_SAMPLES + tail_index].to_bits();
            }
        }
        next_state.previous_tail_peak_word = transition.next_tail_peak.to_bits();
        next_state.hold_counter = transition.next_hold;
        next_state.envelope_level_word = transition.next_level.to_bits();
        next_state.previous_attenuation_word = attenuation.to_bits();
        next_state.delay_read_index = next_delay_index(self.delay_read_index);
        next_state.delay_write_index = next_delay_index(self.delay_write_index);

        Ok(ClipProducerOutput {
            clip_index_u32: transition.clip_index_u32,
            compensated_frame: Some(compensated),
            after_state: next_state,
        })
    }

    fn clip_transition(
        &self,
        frame: &ClipSampleFrame,
        parameters: &ClipFunctionalParameters,
    ) -> Result<ClipTransition, ClipProducerError> {
        if frame.iter().flatten().any(|sample| !sample.is_finite()) {
            return Err(ClipProducerError::NonFiniteSample);
        }

        let mut decision_peak = f32::from_bits(self.previous_tail_peak_word);
        let mut next_tail_peak = 0.0_f32;
        for (channel, row) in frame.iter().enumerate() {
            if channel == LFE_CHANNEL {
                continue;
            }
            for (sample_index, sample) in row.iter().enumerate() {
                let magnitude = sample.abs();
                if magnitude > decision_peak {
                    decision_peak = magnitude;
                }
                if sample_index >= PREFIX_SAMPLES && magnitude > next_tail_peak {
                    next_tail_peak = magnitude;
                }
            }
        }

        let transformed = transform_peak(decision_peak, parameters.constants)?;
        let normalization = f32::from_bits(self.normalization_word);
        let difference = r32_sub(transformed, normalization);
        let scaled_difference = f64::from(difference) * 6.0_f64;
        let clamped_difference = scaled_difference.clamp(-1.0_f64, 1.0_f64) as f32;
        let candidate = r32_sub(
            clamped_difference,
            f32::from_bits(parameters.constants.level_bias),
        );
        let old_level = f32::from_bits(self.envelope_level_word);
        let release = r32_add(old_level, f32::from_bits(parameters.constants.release_step));
        if !candidate.is_finite() || !release.is_finite() {
            return Err(ClipProducerError::NonFiniteIntermediate);
        }
        let (next_level, next_hold) = if release > candidate {
            (candidate, ATTACK_HOLD_CALLS)
        } else if self.hold_counter != 0 {
            (old_level, self.hold_counter - 1)
        } else {
            (release, 0)
        };
        let lookup_x = if next_level < 0.0 || (next_level == 0.0 && next_level.is_sign_negative()) {
            next_level
        } else {
            0.0
        };
        let mut threshold_index = 0_usize;
        while threshold_index < CLIP_LEVELS - 1
            && lookup_x < f32::from_bits(parameters.threshold_words[threshold_index])
        {
            threshold_index += 1;
        }
        Ok(ClipTransition {
            clip_index_u32: 128 + u32::try_from(threshold_index).unwrap_or(127),
            attenuation_word: parameters.attenuation_words[threshold_index],
            next_tail_peak,
            next_level,
            next_hold,
        })
    }
}

/// Clip index and delayed/attenuated sample rows from one clip-owner tick.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipProducerOutput {
    clip_index_u32: u32,
    compensated_frame: Option<Box<[[f32; CLIP_FRAME_SAMPLES]]>>,
    after_state: ClipProducerState,
}

impl ClipProducerOutput {
    pub const fn clip_index_u32(&self) -> u32 {
        self.clip_index_u32
    }

    pub fn compensated_frame(&self) -> Option<&[[f32; CLIP_FRAME_SAMPLES]]> {
        self.compensated_frame.as_deref()
    }

    pub const fn after_state(&self) -> &ClipProducerState {
        &self.after_state
    }
}

struct ClipTransition {
    clip_index_u32: u32,
    attenuation_word: u32,
    next_tail_peak: f32,
    next_level: f32,
    next_hold: u32,
}

fn transform_peak(peak: f32, constants: ClipFunctionalConstants) -> Result<f32, ClipProducerError> {
    let clamped_peak = peak.clamp(-1.0_f32, 1.0_f32);
    let doubled_word = clamped_peak.to_bits().wrapping_mul(2);
    let exponent = if doubled_word == 0 {
        -127
    } else {
        i32::try_from(doubled_word >> 24).unwrap_or(0) - 126
    };
    let headroom = (-exponent).clamp(0, 60);
    if headroom >= 48 || peak == 0.0 {
        return Ok(1.0_f32);
    }

    let scaled_peak = r32_mul(peak, power_of_two(headroom));
    let a = f32::from_bits(constants.polynomial_a);
    let b = f32::from_bits(constants.polynomial_b);
    let c = f32::from_bits(constants.polynomial_c);
    let d = f32::from_bits(constants.exponent_scale);
    let peak_times_b = r32_mul(scaled_peak, b);
    let a_minus_peak_b = r32_sub(a, peak_times_b);
    let peak_squared = r32_mul(scaled_peak, scaled_peak);
    let peak_squared_times_c = r32_mul(peak_squared, c);
    let polynomial = r32_add(a_minus_peak_b, peak_squared_times_c);
    let scaled_polynomial = r32_mul(polynomial, 4.0_f32);
    let scaled_headroom = r32_mul(headroom as f32, d);
    let transformed = r32_add(scaled_polynomial, scaled_headroom);
    if transformed.is_finite() {
        Ok(transformed)
    } else {
        Err(ClipProducerError::NonFiniteIntermediate)
    }
}

fn compensate_sample(factor: f32, sample: f32, gain_scale: f32) -> f32 {
    let attenuated = r32_mul(factor, sample);
    r32_mul(attenuated, gain_scale)
}

fn power_of_two(exponent: i32) -> f32 {
    f32::from_bits(u32::try_from(127 + exponent).unwrap_or(127) << 23)
}

fn next_delay_index(index: u16) -> u16 {
    let start = usize::from(index) % DELAY_RING_SAMPLES;
    u16::try_from(start + DELAY_SAMPLES).unwrap_or(DELAY_RING_SAMPLES as u16)
}

#[inline(never)]
fn r32_add(left: f32, right: f32) -> f32 {
    left + right
}

#[inline(never)]
fn r32_sub(left: f32, right: f32) -> f32 {
    left - right
}

#[inline(never)]
fn r32_mul(left: f32, right: f32) -> f32 {
    left * right
}

#[inline(never)]
fn r32_div(left: f32, right: f32) -> f32 {
    left / right
}
