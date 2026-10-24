// pattern: Functional Core
//! Hash-pinned selected-profile analysis and codec policy parameters.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

const POLICY_PARAMETER_BYTES: usize = 4559;
const POLICY_PARAMETER_SHA256: &str =
    "a99d077900058eb4a862f5ea29bc87408bf6d7a4f4cc42e3da66ae6c68a93c6a";
const POLICY_PARAMETER_SCHEMA: &str = "authorized-policy-functional-parameters.v1";
const SELECTED_PROFILE_ID: &str = "eac3-joc-48k-1024k-acmod7-uncoupled-nonspx-policy-v1";

/// Caller-supplied profile identity used to admit the fixed parameter provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyProfile<'a> {
    pub id: &'a str,
    pub sample_rate_hz: u32,
    pub data_rate_kbps: u32,
    pub acmod: u8,
    pub coupling: bool,
    pub spx: bool,
    pub gain_layout_mask: u32,
    pub refresh_mode: u8,
    pub normalization_option: u8,
    pub normalization_gate: u8,
}

impl PolicyProfile<'static> {
    /// Returns the sole profile admitted by the washed policy parameter file.
    pub const fn selected() -> Self {
        Self {
            id: SELECTED_PROFILE_ID,
            sample_rate_hz: 48_000,
            data_rate_kbps: 1024,
            acmod: 7,
            coupling: false,
            spx: false,
            gain_layout_mask: 7,
            refresh_mode: 1,
            normalization_option: 1,
            normalization_gate: 0,
        }
    }
}

/// Provider failures mapped by the washed F09PA contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyParameterError {
    ParameterUnavailable,
    UnsupportedProfile,
    InvalidParameterDocument,
}

impl PolicyParameterError {
    /// Returns the stable public error code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::ParameterUnavailable => "PARAMETER_UNAVAILABLE",
            Self::UnsupportedProfile => "UNSUPPORTED_PROFILE",
            Self::InvalidParameterDocument => "POLICY_PARAMETER_ERROR",
        }
    }
}

/// Immutable exact-bit values supplied for the selected profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyParameters {
    d06_normal_threshold_bits: u32,
    d06_weights: [i32; 4],
    d06_normalization_factor_bits: u32,
    d07_polynomial_bits: [u32; 5],
    d07_anchors_bits: [u32; 4],
    d07_thresholds_bits: [u32; 3],
    d07_hysteresis_bits: u32,
}

impl PolicyParameters {
    pub const fn d06_normal_threshold_bits(&self) -> u32 {
        self.d06_normal_threshold_bits
    }

    pub const fn d06_weights(&self) -> &[i32; 4] {
        &self.d06_weights
    }

    pub const fn d06_normalization_factor_bits(&self) -> u32 {
        self.d06_normalization_factor_bits
    }

    pub const fn d07_polynomial_bits(&self) -> &[u32; 5] {
        &self.d07_polynomial_bits
    }

    pub const fn d07_anchors_bits(&self) -> &[u32; 4] {
        &self.d07_anchors_bits
    }

    pub const fn d07_thresholds_bits(&self) -> &[u32; 3] {
        &self.d07_thresholds_bits
    }

    pub const fn d07_hysteresis_bits(&self) -> u32 {
        self.d07_hysteresis_bits
    }

    /// Serializes the 19 values in asset order using I32/F32 little endian.
    pub fn serialized_words_le(&self) -> [u8; 76] {
        let mut bytes = [0; 76];
        let mut offset = 0;
        append_u32(&mut bytes, &mut offset, self.d06_normal_threshold_bits);
        for value in self.d06_weights {
            append_i32(&mut bytes, &mut offset, value);
        }
        append_u32(&mut bytes, &mut offset, self.d06_normalization_factor_bits);
        for value in self.d07_polynomial_bits {
            append_u32(&mut bytes, &mut offset, value);
        }
        for value in self.d07_anchors_bits {
            append_u32(&mut bytes, &mut offset, value);
        }
        for value in self.d07_thresholds_bits {
            append_u32(&mut bytes, &mut offset, value);
        }
        append_u32(&mut bytes, &mut offset, self.d07_hysteresis_bits);
        debug_assert_eq!(offset, bytes.len());
        bytes
    }
}

/// Loads the sole selected provider from caller-supplied bytes.
///
/// The provider file identity is checked before JSON parsing. No filesystem,
/// environment, lifecycle, or mutable global state is accessed.
pub fn load_policy_parameters(
    profile: PolicyProfile<'_>,
    provider_file: Option<&[u8]>,
) -> Result<PolicyParameters, PolicyParameterError> {
    let provider_file = provider_file.ok_or(PolicyParameterError::ParameterUnavailable)?;
    if !is_selected_profile(profile) {
        return Err(PolicyParameterError::UnsupportedProfile);
    }
    if provider_file.len() != POLICY_PARAMETER_BYTES
        || sha256_hex(provider_file) != POLICY_PARAMETER_SHA256
    {
        return Err(PolicyParameterError::InvalidParameterDocument);
    }

    let document: Value = serde_json::from_slice(provider_file)
        .map_err(|_| PolicyParameterError::InvalidParameterDocument)?;
    if document.get("schema").and_then(Value::as_str) != Some(POLICY_PARAMETER_SCHEMA) {
        return Err(PolicyParameterError::InvalidParameterDocument);
    }

    let provider_profile = document
        .get("selected_profile")
        .ok_or(PolicyParameterError::UnsupportedProfile)?;
    if !provider_profile_matches(provider_profile) {
        return Err(PolicyParameterError::UnsupportedProfile);
    }

    let assets = document
        .get("assets")
        .ok_or(PolicyParameterError::ParameterUnavailable)?
        .as_array()
        .ok_or(PolicyParameterError::InvalidParameterDocument)?;
    if assets.len() != 7 {
        return Err(PolicyParameterError::InvalidParameterDocument);
    }

    let d06_normal_threshold_bits = f32_asset::<1>(&assets[0], "D06-T-NORMAL", 0, 1)?[0];
    let d06_weights = i32_asset::<4>(&assets[1], "D06-W", 1)?;
    let d06_normalization_factor_bits = f32_asset::<1>(&assets[2], "D06-N", 2, 1)?[0];
    let d07_polynomial_bits = f32_asset(&assets[3], "D07-P", 3, 5)?;
    validate_polynomial_element_names(&assets[3])?;
    let d07_anchors_bits = f32_asset(&assets[4], "D07-A", 4, 4)?;
    let d07_thresholds_bits = f32_asset(&assets[5], "D07-T", 5, 3)?;
    let d07_hysteresis_bits = f32_asset::<1>(&assets[6], "D07-H", 6, 1)?[0];

    Ok(PolicyParameters {
        d06_normal_threshold_bits,
        d06_weights,
        d06_normalization_factor_bits,
        d07_polynomial_bits,
        d07_anchors_bits,
        d07_thresholds_bits,
        d07_hysteresis_bits,
    })
}

fn is_selected_profile(profile: PolicyProfile<'_>) -> bool {
    profile == PolicyProfile::selected()
}

fn provider_profile_matches(value: &Value) -> bool {
    let Some(profile) = value.as_object() else {
        return false;
    };
    exact_keys(
        profile,
        &[
            "id",
            "sample_rate_hz",
            "data_rate_kbps",
            "acmod",
            "coupling",
            "spx",
            "gain_layout_mask",
            "refresh_mode",
            "normalization_option",
            "normalization_gate",
        ],
    ) && profile.get("id").and_then(Value::as_str) == Some(SELECTED_PROFILE_ID)
        && profile.get("sample_rate_hz").and_then(Value::as_u64) == Some(48_000)
        && profile.get("data_rate_kbps").and_then(Value::as_u64) == Some(1024)
        && profile.get("acmod").and_then(Value::as_u64) == Some(7)
        && profile.get("coupling").and_then(Value::as_bool) == Some(false)
        && profile.get("spx").and_then(Value::as_bool) == Some(false)
        && profile.get("gain_layout_mask").and_then(Value::as_u64) == Some(7)
        && profile.get("refresh_mode").and_then(Value::as_u64) == Some(1)
        && profile.get("normalization_option").and_then(Value::as_u64) == Some(1)
        && profile.get("normalization_gate").and_then(Value::as_u64) == Some(0)
}

fn f32_asset<const N: usize>(
    asset: &Value,
    expected_id: &str,
    expected_order: u64,
    expected_count: usize,
) -> Result<[u32; N], PolicyParameterError> {
    validate_asset_header(asset, expected_id, "F32", expected_order)?;
    let words = required(asset, "u32_words_hex")?
        .as_array()
        .ok_or(PolicyParameterError::InvalidParameterDocument)?;
    if words.len() != expected_count || words.len() != N {
        return Err(PolicyParameterError::InvalidParameterDocument);
    }
    let mut output = [0; N];
    for (destination, word) in output.iter_mut().zip(words) {
        let encoded = word
            .as_str()
            .ok_or(PolicyParameterError::InvalidParameterDocument)?;
        let digits = encoded
            .strip_prefix("0x")
            .ok_or(PolicyParameterError::InvalidParameterDocument)?;
        if digits.len() != 8 {
            return Err(PolicyParameterError::InvalidParameterDocument);
        }
        *destination = u32::from_str_radix(digits, 16)
            .map_err(|_| PolicyParameterError::InvalidParameterDocument)?;
    }
    Ok(output)
}

fn i32_asset<const N: usize>(
    asset: &Value,
    expected_id: &str,
    expected_order: u64,
) -> Result<[i32; N], PolicyParameterError> {
    validate_asset_header(asset, expected_id, "I32", expected_order)?;
    let values = required(asset, "i32_values")?
        .as_array()
        .ok_or(PolicyParameterError::InvalidParameterDocument)?;
    if values.len() != N {
        return Err(PolicyParameterError::InvalidParameterDocument);
    }
    let mut output = [0; N];
    for (destination, value) in output.iter_mut().zip(values) {
        let value = value
            .as_i64()
            .ok_or(PolicyParameterError::InvalidParameterDocument)?;
        *destination =
            i32::try_from(value).map_err(|_| PolicyParameterError::InvalidParameterDocument)?;
    }
    Ok(output)
}

fn validate_asset_header(
    asset: &Value,
    expected_id: &str,
    expected_type: &str,
    expected_order: u64,
) -> Result<(), PolicyParameterError> {
    let object = asset
        .as_object()
        .ok_or(PolicyParameterError::InvalidParameterDocument)?;
    let asset_id = object
        .get("asset_id")
        .ok_or(PolicyParameterError::ParameterUnavailable)?
        .as_str()
        .ok_or(PolicyParameterError::InvalidParameterDocument)?;
    if asset_id != expected_id
        || object.get("type").and_then(Value::as_str) != Some(expected_type)
        || object.get("consumer_order").and_then(Value::as_u64) != Some(expected_order)
    {
        return Err(PolicyParameterError::InvalidParameterDocument);
    }
    Ok(())
}

fn validate_polynomial_element_names(asset: &Value) -> Result<(), PolicyParameterError> {
    let names = required(asset, "element_names")?
        .as_array()
        .ok_or(PolicyParameterError::InvalidParameterDocument)?;
    let expected = [
        "offset",
        "linear",
        "quadratic",
        "headroom_step",
        "result_scale",
    ];
    if names.len() != expected.len()
        || names
            .iter()
            .zip(expected)
            .any(|(name, expected_name)| name.as_str() != Some(expected_name))
    {
        return Err(PolicyParameterError::InvalidParameterDocument);
    }
    Ok(())
}

fn required<'a>(object: &'a Value, key: &str) -> Result<&'a Value, PolicyParameterError> {
    object
        .get(key)
        .ok_or(PolicyParameterError::ParameterUnavailable)
}

fn exact_keys(object: &Map<String, Value>, expected: &[&str]) -> bool {
    object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn append_u32(output: &mut [u8; 76], offset: &mut usize, value: u32) {
    let bytes = value.to_le_bytes();
    output[*offset..*offset + bytes.len()].copy_from_slice(&bytes);
    *offset += bytes.len();
}

fn append_i32(output: &mut [u8; 76], offset: &mut usize, value: i32) {
    let bytes = value.to_le_bytes();
    output[*offset..*offset + bytes.len()].copy_from_slice(&bytes);
    *offset += bytes.len();
}
