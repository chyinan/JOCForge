// pattern: Functional Core
//! One stateless F06NR overlap-window transition and its 128-word next carry.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const PARAMETERS: &[u8] = include_bytes!("ordinary_transform_math_parameters.json");
const PARAMETERS_SHA256: &str = "f1fb1a16b3caedab6ddc583201b9454755fd8871ecdf7881a6a3ffbad10fddf1";
const WINDOW_WORDS_SHA256: &str =
    "45e8cbe4c07bd1b59bc009ed6ac77d7911a3a309b01308131da97e640da95b23";
const WINDOW_WORDS: usize = 256;
const CARRY_WORDS: usize = 128;
const SIGN_BIT: u32 = 0x8000_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecondaryOverlapError {
    InputError,
    ParameterUnavailable,
    ParameterError,
    NumericFailure,
}

impl SecondaryOverlapError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputError => "SECONDARY_INPUT_ERROR",
            Self::ParameterUnavailable => "PARAMETER_UNAVAILABLE",
            Self::ParameterError => "PARAMETER_IDENTITY_ERROR",
            Self::NumericFailure => "NUMERIC_FAILURE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecondaryOverlapParameters {
    window_words: [u32; WINDOW_WORDS],
    provider_sha256: [u8; 32],
    window_words_sha256: [u8; 32],
}

impl SecondaryOverlapParameters {
    /// Loads the selected math-generated F08 window from a hash-pinned provider.
    pub fn selected() -> Result<Self, SecondaryOverlapError> {
        if sha256_hex(PARAMETERS) != PARAMETERS_SHA256 {
            return Err(SecondaryOverlapError::ParameterUnavailable);
        }
        let root: Value = serde_json::from_slice(PARAMETERS)
            .map_err(|_| SecondaryOverlapError::ParameterError)?;
        let root = root
            .as_object()
            .ok_or(SecondaryOverlapError::ParameterError)?;
        if text(root, "schema")? != "mathematical-transform-parameters.v1"
            || text(root, "profile_id")? != "math-generated-v1"
            || text(root, "data_role")? != "ALGORITHM_PARAMETERS_NOT_TEST_OR_SONG_LOOKUP"
            || text(root, "origin")? != "MATHEMATICAL_FORMULAS_GENERATED_IN_THIS_TASK"
            || root
                .get("private_vendor_key_included")
                .and_then(Value::as_bool)
                != Some(false)
        {
            return Err(SecondaryOverlapError::ParameterError);
        }
        let fields = root
            .get("fields")
            .and_then(Value::as_object)
            .ok_or(SecondaryOverlapError::ParameterError)?;
        let window_values = fields
            .get("window")
            .and_then(Value::as_array)
            .ok_or(SecondaryOverlapError::ParameterError)?;
        if window_values.len() != WINDOW_WORDS {
            return Err(SecondaryOverlapError::ParameterError);
        }
        let mut window_words = [0_u32; WINDOW_WORDS];
        for (index, encoded) in window_values.iter().enumerate() {
            let word = encoded
                .as_u64()
                .and_then(|value| u32::try_from(value).ok())
                .ok_or(SecondaryOverlapError::ParameterError)?;
            if !f32::from_bits(word).is_finite() {
                return Err(SecondaryOverlapError::ParameterError);
            }
            window_words[index] = word;
        }
        let window_bytes = little_endian_words(&window_words);
        if sha256_hex(&window_bytes) != WINDOW_WORDS_SHA256 {
            return Err(SecondaryOverlapError::ParameterError);
        }
        Ok(Self {
            window_words,
            provider_sha256: Sha256::digest(PARAMETERS).into(),
            window_words_sha256: Sha256::digest(window_bytes).into(),
        })
    }

    pub const fn provider_sha256(&self) -> &[u8; 32] {
        &self.provider_sha256
    }

    pub const fn window_words_sha256(&self) -> &[u8; 32] {
        &self.window_words_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecondaryOverlapCarry([u32; CARRY_WORDS]);

impl SecondaryOverlapCarry {
    pub const fn cold() -> Self {
        Self([0; CARRY_WORDS])
    }

    pub fn from_words(words: [u32; CARRY_WORDS]) -> Result<Self, SecondaryOverlapError> {
        if words.iter().any(|word| !f32::from_bits(*word).is_finite()) {
            return Err(SecondaryOverlapError::InputError);
        }
        Ok(Self(words))
    }

    pub const fn words(&self) -> &[u32; CARRY_WORDS] {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecondaryOverlapResult {
    output_words: [u32; WINDOW_WORDS],
    carry_after: SecondaryOverlapCarry,
}

impl SecondaryOverlapResult {
    pub const fn output_words(&self) -> &[u32; WINDOW_WORDS] {
        &self.output_words
    }

    pub const fn carry_after(&self) -> &SecondaryOverlapCarry {
        &self.carry_after
    }

    pub fn into_parts(self) -> ([u32; WINDOW_WORDS], SecondaryOverlapCarry) {
        (self.output_words, self.carry_after)
    }
}

/// Produces one 256-word overlap output and the next 128-word inverse carry.
/// It neither commits caller state nor performs a private EOF flush.
pub fn overlap_secondary_block(
    transformed_words: &[u32],
    carry_before: &SecondaryOverlapCarry,
    parameters: &SecondaryOverlapParameters,
) -> Result<SecondaryOverlapResult, SecondaryOverlapError> {
    if transformed_words.len() != WINDOW_WORDS
        || transformed_words
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SecondaryOverlapError::InputError);
    }

    let mut output_words = [0_u32; WINDOW_WORDS];
    let mut written = [false; WINDOW_WORDS];
    for index in 0..64 {
        let first_high = parameters.window_words[255 - 2 * index];
        let first_low = parameters.window_words[2 * index];
        let first_old = carry_before.0[index];
        let first_current = transformed_words[192 + index];
        store_once(
            &mut output_words,
            &mut written,
            2 * index,
            subtract(
                negate(multiply(first_old, first_high)?),
                multiply(first_current, first_low)?,
            )?,
        )?;
        store_once(
            &mut output_words,
            &mut written,
            255 - 2 * index,
            subtract(
                multiply(first_high, first_current)?,
                multiply(first_old, first_low)?,
            )?,
        )?;

        let second_low = parameters.window_words[2 * index + 1];
        let second_high = parameters.window_words[254 - 2 * index];
        let second_old = carry_before.0[127 - index];
        let second_current = transformed_words[63 - index];
        store_once(
            &mut output_words,
            &mut written,
            2 * index + 1,
            add(
                multiply(second_current, second_low)?,
                multiply(second_old, second_high)?,
            )?,
        )?;
        store_once(
            &mut output_words,
            &mut written,
            254 - 2 * index,
            subtract(
                multiply(second_old, second_low)?,
                multiply(second_current, second_high)?,
            )?,
        )?;
    }
    if written.iter().any(|was_written| !was_written) {
        return Err(SecondaryOverlapError::NumericFailure);
    }

    let mut carry_words = [0_u32; CARRY_WORDS];
    carry_words.copy_from_slice(&transformed_words[64..192]);
    Ok(SecondaryOverlapResult {
        output_words,
        carry_after: SecondaryOverlapCarry::from_words(carry_words)?,
    })
}

fn store_once(
    output: &mut [u32; WINDOW_WORDS],
    written: &mut [bool; WINDOW_WORDS],
    index: usize,
    value: u32,
) -> Result<(), SecondaryOverlapError> {
    if index >= output.len() || written[index] {
        return Err(SecondaryOverlapError::NumericFailure);
    }
    output[index] = value;
    written[index] = true;
    Ok(())
}

fn negate(word: u32) -> u32 {
    word ^ SIGN_BIT
}

fn add(left: u32, right: u32) -> Result<u32, SecondaryOverlapError> {
    r32_add(finite(left)?, finite(right)?)
        .map(|value| value.value().to_bits())
        .map_err(|_| SecondaryOverlapError::NumericFailure)
}

fn subtract(left: u32, right: u32) -> Result<u32, SecondaryOverlapError> {
    r32_sub(finite(left)?, finite(right)?)
        .map(|value| value.value().to_bits())
        .map_err(|_| SecondaryOverlapError::NumericFailure)
}

fn multiply(left: u32, right: u32) -> Result<u32, SecondaryOverlapError> {
    r32_mul(finite(left)?, finite(right)?)
        .map(|value| value.value().to_bits())
        .map_err(|_| SecondaryOverlapError::NumericFailure)
}

fn finite(word: u32) -> Result<FiniteF32, SecondaryOverlapError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| SecondaryOverlapError::NumericFailure)
}

fn text<'a>(fields: &'a Map<String, Value>, key: &str) -> Result<&'a str, SecondaryOverlapError> {
    fields
        .get(key)
        .and_then(Value::as_str)
        .ok_or(SecondaryOverlapError::ParameterError)
}

fn little_endian_words(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}
