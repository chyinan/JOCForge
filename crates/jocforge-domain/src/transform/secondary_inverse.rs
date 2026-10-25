// pattern: Functional Core
//! Stateless long/short inverse256 spectral split, rotations and clamped output.

use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

use super::secondary_fft::{
    SecondaryFftError, SecondaryFftParameters, SecondaryFftSize, execute_secondary_fft,
};

const INVERSE_WORDS: usize = 256;
const HALF_WORD: u32 = 0x3f00_0000;
const ONE_WORD: u32 = 0x3f80_0000;
const TWO_WORD: u32 = 0x4000_0000;
const NEGATIVE_ONE_WORD: u32 = 0xbf80_0000;
const SIGN_BIT: u32 = 0x8000_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecondaryInverseError {
    InputError,
    ParameterUnavailable,
    NumericFailure,
    UnsupportedProfile,
}

impl SecondaryInverseError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputError => "SECONDARY_INPUT_ERROR",
            Self::ParameterUnavailable => "PARAMETER_UNAVAILABLE",
            Self::NumericFailure => "NUMERIC_FAILURE",
            Self::UnsupportedProfile => "UNSUPPORTED_PROFILE",
        }
    }
}

/// Performs the selected long or short secondary inverse256 without history.
/// F06NR owns frame selection, overlap carry and carrier placement.
pub fn execute_secondary_inverse256(
    spectrum_words: &[u32],
    short: bool,
    parameters: &SecondaryFftParameters,
) -> Result<[u32; INVERSE_WORDS], SecondaryInverseError> {
    if spectrum_words.len() != INVERSE_WORDS
        || spectrum_words
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SecondaryInverseError::InputError);
    }

    if short {
        execute_short_inverse(spectrum_words, parameters)
    } else {
        execute_long_inverse(spectrum_words, parameters)
    }
}

fn execute_long_inverse(
    spectrum_words: &[u32],
    parameters: &SecondaryFftParameters,
) -> Result<[u32; INVERSE_WORDS], SecondaryInverseError> {
    let mut descriptor_input = [0_u32; INVERSE_WORDS];
    for index in 0..INVERSE_WORDS / 2 {
        descriptor_input[2 * index] = multiply(spectrum_words[255 - 2 * index], HALF_WORD)?;
        descriptor_input[2 * index + 1] =
            negate(multiply(spectrum_words[254 - 2 * index], HALF_WORD)?);
    }
    let transformed =
        inverse_descriptor(SecondaryFftSize::Points128, &descriptor_input, parameters)?;
    let mut output = [0_u32; INVERSE_WORDS];
    let mut written = [false; INVERSE_WORDS];
    for index in 0..INVERSE_WORDS / 2 {
        store_once(
            &mut output,
            &mut written,
            index,
            clamp_to_unit(multiply(transformed[2 * index], TWO_WORD)?)?,
        )?;
        store_once(
            &mut output,
            &mut written,
            255 - index,
            clamp_to_unit(multiply(transformed[2 * index + 1], TWO_WORD)?)?,
        )?;
    }
    verify_complete_output(&written)?;
    Ok(output)
}

fn execute_short_inverse(
    spectrum_words: &[u32],
    parameters: &SecondaryFftParameters,
) -> Result<[u32; INVERSE_WORDS], SecondaryInverseError> {
    let mut descriptor_a = [0_u32; INVERSE_WORDS / 2];
    let mut descriptor_b = [0_u32; INVERSE_WORDS / 2];
    for index in 0..INVERSE_WORDS / 4 {
        descriptor_a[2 * index] = multiply(spectrum_words[254 - 4 * index], HALF_WORD)?;
        descriptor_a[2 * index + 1] = negate(multiply(spectrum_words[252 - 4 * index], HALF_WORD)?);
        descriptor_b[2 * index] = multiply(spectrum_words[255 - 4 * index], HALF_WORD)?;
        descriptor_b[2 * index + 1] = negate(multiply(spectrum_words[253 - 4 * index], HALF_WORD)?);
    }

    let transformed_a = inverse_descriptor(SecondaryFftSize::Points64, &descriptor_a, parameters)?;
    let transformed_b = inverse_descriptor(SecondaryFftSize::Points64, &descriptor_b, parameters)?;
    let mut output = [0_u32; INVERSE_WORDS];
    let mut written = [false; INVERSE_WORDS];
    for index in 0..INVERSE_WORDS / 4 {
        store_once(
            &mut output,
            &mut written,
            index,
            clamp_to_unit(multiply(transformed_a[2 * index], TWO_WORD)?)?,
        )?;
        store_once(
            &mut output,
            &mut written,
            255 - index,
            clamp_to_unit(multiply(transformed_a[2 * index + 1], TWO_WORD)?)?,
        )?;
        store_once(
            &mut output,
            &mut written,
            64 + index,
            clamp_to_unit(multiply(transformed_b[2 * index], TWO_WORD)?)?,
        )?;
        store_once(
            &mut output,
            &mut written,
            191 - index,
            clamp_to_unit(multiply(transformed_b[2 * index + 1], TWO_WORD)?)?,
        )?;
    }
    verify_complete_output(&written)?;
    Ok(output)
}

fn inverse_descriptor(
    fft_size: SecondaryFftSize,
    input_words: &[u32],
    parameters: &SecondaryFftParameters,
) -> Result<Vec<u32>, SecondaryInverseError> {
    let complex_points = usize::from(fft_size.complex_points());
    if input_words.len() != 2 * complex_points {
        return Err(SecondaryInverseError::InputError);
    }
    let (pre_a, pre_b, post_sine) = parameters.pre_post_words(fft_size);
    if pre_a.len() != complex_points
        || pre_b.len() != complex_points
        || post_sine.len() != complex_points
    {
        return Err(SecondaryInverseError::ParameterUnavailable);
    }

    let mut work = vec![0_u32; input_words.len()];
    for index in 0..complex_points {
        let front = input_words[2 * index];
        let back = input_words[2 * complex_points - 1 - 2 * index];
        work[2 * index] = add(
            multiply(back, pre_a[index])?,
            multiply(front, pre_b[index])?,
        )?;
        work[2 * index + 1] = subtract(
            multiply(back, pre_b[index])?,
            multiply(front, pre_a[index])?,
        )?;
    }

    let transformed = execute_secondary_fft(fft_size, &work, parameters).map_err(map_fft_error)?;
    let mut output = vec![0_u32; input_words.len()];
    for index in 0..complex_points {
        let even = transformed[2 * index];
        let odd = transformed[2 * index + 1];
        let sine = post_sine[index];
        let cosine = if index == 0 {
            ONE_WORD
        } else {
            post_sine[complex_points - index]
        };
        output[2 * index] = multiply(
            add(multiply(even, cosine)?, multiply(odd, sine)?)?,
            TWO_WORD,
        )?;
        output[2 * complex_points - 1 - 2 * index] = multiply(
            subtract(multiply(even, sine)?, multiply(odd, cosine)?)?,
            TWO_WORD,
        )?;
    }
    Ok(output)
}

fn clamp_to_unit(word: u32) -> Result<u32, SecondaryInverseError> {
    let value = f32::from_bits(word);
    if !value.is_finite() {
        return Err(SecondaryInverseError::NumericFailure);
    }
    if value < -1.0 {
        Ok(NEGATIVE_ONE_WORD)
    } else if value > 1.0 {
        Ok(ONE_WORD)
    } else {
        Ok(word)
    }
}

fn store_once(
    output: &mut [u32; INVERSE_WORDS],
    written: &mut [bool; INVERSE_WORDS],
    index: usize,
    word: u32,
) -> Result<(), SecondaryInverseError> {
    if index >= output.len() || written[index] {
        return Err(SecondaryInverseError::NumericFailure);
    }
    output[index] = word;
    written[index] = true;
    Ok(())
}

fn verify_complete_output(written: &[bool; INVERSE_WORDS]) -> Result<(), SecondaryInverseError> {
    if written.iter().all(|value| *value) {
        Ok(())
    } else {
        Err(SecondaryInverseError::NumericFailure)
    }
}

fn negate(word: u32) -> u32 {
    word ^ SIGN_BIT
}

fn add(left: u32, right: u32) -> Result<u32, SecondaryInverseError> {
    let left = finite(left)?;
    let right = finite(right)?;
    r32_add(left, right)
        .map(|value| value.value().to_bits())
        .map_err(|_| SecondaryInverseError::NumericFailure)
}

fn subtract(left: u32, right: u32) -> Result<u32, SecondaryInverseError> {
    let left = finite(left)?;
    let right = finite(right)?;
    r32_sub(left, right)
        .map(|value| value.value().to_bits())
        .map_err(|_| SecondaryInverseError::NumericFailure)
}

fn multiply(left: u32, right: u32) -> Result<u32, SecondaryInverseError> {
    let left = finite(left)?;
    let right = finite(right)?;
    r32_mul(left, right)
        .map(|value| value.value().to_bits())
        .map_err(|_| SecondaryInverseError::NumericFailure)
}

fn finite(word: u32) -> Result<FiniteF32, SecondaryInverseError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| SecondaryInverseError::NumericFailure)
}

fn map_fft_error(error: SecondaryFftError) -> SecondaryInverseError {
    match error {
        SecondaryFftError::InputError => SecondaryInverseError::InputError,
        SecondaryFftError::ParameterUnavailable | SecondaryFftError::ParameterError => {
            SecondaryInverseError::ParameterUnavailable
        }
        SecondaryFftError::NumericFailure => SecondaryInverseError::NumericFailure,
        SecondaryFftError::UnsupportedProfile => SecondaryInverseError::UnsupportedProfile,
    }
}
