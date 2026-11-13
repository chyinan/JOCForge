// pattern: Functional Core
//! Flat7x D0 long-transform spectrum rows and per-channel gain-range decisions.

use super::bap::{FLAT7X_D0_BAP_BIN_COUNT, FLAT7X_D0_BAP_CHANNEL_COUNT};
use crate::transform::{
    long_mdct::forward_long_mdct_512, math_generated_parameters::MathGeneratedTransformParameters,
};

pub const D0_PCM_SAMPLES_PER_BLOCK: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flat7xD0PcmWindow {
    pub previous_samples: [f32; D0_PCM_SAMPLES_PER_BLOCK],
    pub new_samples: [f32; D0_PCM_SAMPLES_PER_BLOCK],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flat7xD0PcmBlockInput {
    /// D0 coded-channel order: Ls, Rs, Lrs, Rrs. No LFE row is present.
    pub channels: [Flat7xD0PcmWindow; FLAT7X_D0_BAP_CHANNEL_COUNT],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0SpectrumError {
    InvalidPcm,
    NumericFailure,
}

impl Flat7xD0SpectrumError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidPcm => "D0_PCM_INPUT_ERROR",
            Self::NumericFailure => "D0_TRANSFORM_NUMERIC_FAILURE",
        }
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0SpectrumBlock {
    coefficient_bits_by_channel: [[u32; FLAT7X_D0_BAP_BIN_COUNT]; FLAT7X_D0_BAP_CHANNEL_COUNT],
    gain_range_codes: [u8; FLAT7X_D0_BAP_CHANNEL_COUNT],
}

impl Flat7xD0SpectrumBlock {
    pub const fn coefficient_bits_by_channel(
        &self,
    ) -> &[[u32; FLAT7X_D0_BAP_BIN_COUNT]; FLAT7X_D0_BAP_CHANNEL_COUNT] {
        &self.coefficient_bits_by_channel
    }

    pub const fn gain_range_codes(&self) -> &[u8; FLAT7X_D0_BAP_CHANNEL_COUNT] {
        &self.gain_range_codes
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Functional Core: transforms one four-channel D0 block and derives ETSI `gainrng` codes.
///
/// The caller supplies the preceding 256 samples for each channel. The output keeps the four
/// coded-channel order and truncates the 256-coefficient long transform to the selected 253 active
/// bins. It does not install overlap history or produce exponent/mantissa syntax.
pub fn transform_flat7x_d0_pcm_block(
    input: &Flat7xD0PcmBlockInput,
    parameters: &MathGeneratedTransformParameters,
) -> Result<Flat7xD0SpectrumBlock, Flat7xD0SpectrumError> {
    let mut coefficient_bits_by_channel =
        [[0_u32; FLAT7X_D0_BAP_BIN_COUNT]; FLAT7X_D0_BAP_CHANNEL_COUNT];
    let mut gain_range_codes = [0_u8; FLAT7X_D0_BAP_CHANNEL_COUNT];

    for (channel, window) in input.channels.iter().enumerate() {
        if window
            .previous_samples
            .iter()
            .chain(window.new_samples.iter())
            .any(|sample| !sample.is_finite() || !(-1.0..=1.0).contains(sample))
        {
            return Err(Flat7xD0SpectrumError::InvalidPcm);
        }

        gain_range_codes[channel] = gain_range_code(window);
        let transformed = forward_long_mdct_512(
            &window.previous_samples,
            &window.new_samples,
            parameters.window_f32_bits(),
        )
        .map_err(|_| Flat7xD0SpectrumError::NumericFailure)?;
        for (destination, coefficient) in coefficient_bits_by_channel[channel]
            .iter_mut()
            .zip(transformed.iter())
        {
            *destination = coefficient.to_bits();
        }
    }

    Ok(Flat7xD0SpectrumBlock {
        coefficient_bits_by_channel,
        gain_range_codes,
    })
}

fn gain_range_code(window: &Flat7xD0PcmWindow) -> u8 {
    let maximum = window
        .previous_samples
        .iter()
        .chain(window.new_samples.iter())
        .map(|sample| sample.abs())
        .fold(0.0_f32, f32::max);
    if maximum >= 0.5 {
        0
    } else if maximum >= 0.25 {
        1
    } else if maximum >= 0.125 {
        2
    } else {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::{D0_PCM_SAMPLES_PER_BLOCK, Flat7xD0PcmWindow, gain_range_code};

    fn window(level: f32) -> Flat7xD0PcmWindow {
        Flat7xD0PcmWindow {
            previous_samples: [level; D0_PCM_SAMPLES_PER_BLOCK],
            new_samples: [0.0; D0_PCM_SAMPLES_PER_BLOCK],
        }
    }

    #[test]
    fn gain_range_uses_table_6_33a_thresholds_on_the_complete_window() {
        assert_eq!(gain_range_code(&window(0.5)), 0);
        assert_eq!(gain_range_code(&window(0.25)), 1);
        assert_eq!(gain_range_code(&window(0.125)), 2);
        assert_eq!(gain_range_code(&window(0.124)), 3);
    }
}
