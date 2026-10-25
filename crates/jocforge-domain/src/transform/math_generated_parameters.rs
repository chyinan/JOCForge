// pattern: Functional Core
//! Identity-gated math-generated transform parameters for the selected F08 route.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

const PROVIDER_SHA256: &str = "f1fb1a16b3caedab6ddc583201b9454755fd8871ecdf7881a6a3ffbad10fddf1";
const REQUIRED_FIELDS: [&str; 11] = [
    "window",
    "long_cosine",
    "long_sine",
    "short_cosine",
    "short_sine",
    "spectrum_permutation",
    "butterfly",
    "lfe_cosine",
    "lfe_sine",
    "lfe_recurrence",
    "lfe_permutation",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MathGeneratedTransformParameterError {
    ParameterUnavailable,
    ParameterIdentityError,
}

impl MathGeneratedTransformParameterError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ParameterUnavailable => "PARAMETER_UNAVAILABLE",
            Self::ParameterIdentityError => "PARAMETER_IDENTITY_ERROR",
        }
    }
}

/// Immutable typed view of the selected `math-generated-v1` provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MathGeneratedTransformParameters {
    window_f32_bits: [u32; 256],
    long_cosine_f32_bits: [u32; 128],
    long_sine_f32_bits: [u32; 128],
    short_cosine_f32_bits: [u32; 64],
    short_sine_f32_bits: [u32; 64],
    spectrum_permutation: [u16; 128],
    butterfly_f32_bits: [u32; 64],
    lfe_cosine_f32_bits: [u32; 8],
    lfe_sine_f32_bits: [u32; 8],
    lfe_recurrence_f32_bits: [u32; 8],
    lfe_permutation: [u32; 8],
}

impl MathGeneratedTransformParameters {
    /// Parses and validates the one approved math-generated parameter file.
    ///
    /// Missing required fields return `PARAMETER_UNAVAILABLE`. Schema, identity,
    /// dimensions, finite-value, and permutation mismatches return
    /// `PARAMETER_IDENTITY_ERROR`.
    pub fn from_provider_bytes(bytes: &[u8]) -> Result<Self, MathGeneratedTransformParameterError> {
        if bytes.is_empty() {
            return Err(MathGeneratedTransformParameterError::ParameterUnavailable);
        }
        let document: Value = serde_json::from_slice(bytes)
            .map_err(|_| MathGeneratedTransformParameterError::ParameterIdentityError)?;
        let fields = document
            .get("fields")
            .and_then(Value::as_object)
            .ok_or(MathGeneratedTransformParameterError::ParameterIdentityError)?;
        for name in REQUIRED_FIELDS {
            if !fields.contains_key(name) {
                return Err(MathGeneratedTransformParameterError::ParameterUnavailable);
            }
        }

        if document.get("schema").and_then(Value::as_str)
            != Some("mathematical-transform-parameters.v1")
            || document.get("profile_id").and_then(Value::as_str) != Some("math-generated-v1")
            || document.get("data_role").and_then(Value::as_str)
                != Some("ALGORITHM_PARAMETERS_NOT_TEST_OR_SONG_LOOKUP")
            || document.get("origin").and_then(Value::as_str)
                != Some("MATHEMATICAL_FORMULAS_GENERATED_IN_THIS_TASK")
            || document
                .get("private_vendor_key_included")
                .and_then(Value::as_bool)
                != Some(false)
            || sha256_hex(bytes) != PROVIDER_SHA256
        {
            return Err(MathGeneratedTransformParameterError::ParameterIdentityError);
        }

        let window_f32_bits = read_f32_words::<256>(fields, "window")?;
        let long_cosine_f32_bits = read_f32_words::<128>(fields, "long_cosine")?;
        let long_sine_f32_bits = read_f32_words::<128>(fields, "long_sine")?;
        let short_cosine_f32_bits = read_f32_words::<64>(fields, "short_cosine")?;
        let short_sine_f32_bits = read_f32_words::<64>(fields, "short_sine")?;
        let spectrum_permutation = read_u16_words::<128>(fields, "spectrum_permutation")?;
        let butterfly_f32_bits = read_f32_words::<64>(fields, "butterfly")?;
        let lfe_cosine_f32_bits = read_f32_words::<8>(fields, "lfe_cosine")?;
        let lfe_sine_f32_bits = read_f32_words::<8>(fields, "lfe_sine")?;
        let lfe_recurrence_f32_bits = read_f32_words::<8>(fields, "lfe_recurrence")?;
        let lfe_permutation = read_u32_words::<8>(fields, "lfe_permutation")?;

        if butterfly_f32_bits[0] != 0
            || butterfly_f32_bits[63] != 0
            || !is_spectrum_permutation(&spectrum_permutation)
            || lfe_permutation.iter().any(|index| *index >= 8)
        {
            return Err(MathGeneratedTransformParameterError::ParameterIdentityError);
        }

        Ok(Self {
            window_f32_bits,
            long_cosine_f32_bits,
            long_sine_f32_bits,
            short_cosine_f32_bits,
            short_sine_f32_bits,
            spectrum_permutation,
            butterfly_f32_bits,
            lfe_cosine_f32_bits,
            lfe_sine_f32_bits,
            lfe_recurrence_f32_bits,
            lfe_permutation,
        })
    }

    pub fn selected_math_generated_v1() -> Result<Self, MathGeneratedTransformParameterError> {
        Self::from_provider_bytes(include_bytes!("ordinary_transform_math_parameters.json"))
    }

    pub const fn window_f32_bits(&self) -> &[u32; 256] {
        &self.window_f32_bits
    }

    pub const fn long_cosine_f32_bits(&self) -> &[u32; 128] {
        &self.long_cosine_f32_bits
    }

    pub const fn long_sine_f32_bits(&self) -> &[u32; 128] {
        &self.long_sine_f32_bits
    }

    pub const fn short_cosine_f32_bits(&self) -> &[u32; 64] {
        &self.short_cosine_f32_bits
    }

    pub const fn short_sine_f32_bits(&self) -> &[u32; 64] {
        &self.short_sine_f32_bits
    }

    pub const fn spectrum_permutation(&self) -> &[u16; 128] {
        &self.spectrum_permutation
    }

    pub const fn butterfly_f32_bits(&self) -> &[u32; 64] {
        &self.butterfly_f32_bits
    }

    pub const fn lfe_cosine_f32_bits(&self) -> &[u32; 8] {
        &self.lfe_cosine_f32_bits
    }

    pub const fn lfe_sine_f32_bits(&self) -> &[u32; 8] {
        &self.lfe_sine_f32_bits
    }

    pub const fn lfe_recurrence_f32_bits(&self) -> &[u32; 8] {
        &self.lfe_recurrence_f32_bits
    }

    pub const fn lfe_permutation(&self) -> &[u32; 8] {
        &self.lfe_permutation
    }
}

fn read_f32_words<const N: usize>(
    fields: &Map<String, Value>,
    name: &str,
) -> Result<[u32; N], MathGeneratedTransformParameterError> {
    let values = fields
        .get(name)
        .and_then(Value::as_array)
        .ok_or(MathGeneratedTransformParameterError::ParameterIdentityError)?;
    if values.len() != N {
        return Err(MathGeneratedTransformParameterError::ParameterIdentityError);
    }
    let mut words = [0_u32; N];
    for (word, value) in words.iter_mut().zip(values) {
        let raw = value
            .as_u64()
            .and_then(|number| u32::try_from(number).ok())
            .ok_or(MathGeneratedTransformParameterError::ParameterIdentityError)?;
        if !f32::from_bits(raw).is_finite() {
            return Err(MathGeneratedTransformParameterError::ParameterIdentityError);
        }
        *word = raw;
    }
    Ok(words)
}

fn read_u16_words<const N: usize>(
    fields: &Map<String, Value>,
    name: &str,
) -> Result<[u16; N], MathGeneratedTransformParameterError> {
    let values = fields
        .get(name)
        .and_then(Value::as_array)
        .ok_or(MathGeneratedTransformParameterError::ParameterIdentityError)?;
    if values.len() != N {
        return Err(MathGeneratedTransformParameterError::ParameterIdentityError);
    }
    let mut words = [0_u16; N];
    for (word, value) in words.iter_mut().zip(values) {
        *word = value
            .as_u64()
            .and_then(|number| u16::try_from(number).ok())
            .ok_or(MathGeneratedTransformParameterError::ParameterIdentityError)?;
    }
    Ok(words)
}

fn read_u32_words<const N: usize>(
    fields: &Map<String, Value>,
    name: &str,
) -> Result<[u32; N], MathGeneratedTransformParameterError> {
    let values = fields
        .get(name)
        .and_then(Value::as_array)
        .ok_or(MathGeneratedTransformParameterError::ParameterIdentityError)?;
    if values.len() != N {
        return Err(MathGeneratedTransformParameterError::ParameterIdentityError);
    }
    let mut words = [0_u32; N];
    for (word, value) in words.iter_mut().zip(values) {
        *word = value
            .as_u64()
            .and_then(|number| u32::try_from(number).ok())
            .ok_or(MathGeneratedTransformParameterError::ParameterIdentityError)?;
    }
    Ok(words)
}

fn is_spectrum_permutation(values: &[u16; 128]) -> bool {
    let mut complete = [false; 128];
    let mut even_short = [false; 64];
    for (slot, value) in values.iter().copied().enumerate() {
        let index = usize::from(value);
        if index >= complete.len() || complete[index] {
            return false;
        }
        complete[index] = true;
        if slot % 2 == 0 {
            let short_index = usize::from(value);
            if short_index >= even_short.len() || even_short[short_index] {
                return false;
            }
            even_short[short_index] = true;
        }
    }
    complete.into_iter().all(|present| present) && even_short.into_iter().all(|present| present)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
