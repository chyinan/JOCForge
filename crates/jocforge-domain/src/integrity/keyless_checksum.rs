// pattern: Functional Core
//! Public keyless checksum profile validation and checksum primitives.

use serde_json::Value;
use sha2::{Digest, Sha256};

const PROFILE_ID: &str = "keyless-crc32-crc8-crc16.v1";
const EXPECTED_PROVIDER_SHA256: &str =
    "57b117ab76c597ad6f6d65af65eecc17d2df78cf0874d092fe1f67585e47bd75";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IntegritySha256([u8; 32]);

impl IntegritySha256 {
    pub(crate) const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(64);
        for byte in self.0 {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }
}

pub(crate) fn sha256(bytes: &[u8]) -> IntegritySha256 {
    IntegritySha256::from_bytes(Sha256::digest(bytes).into())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeylessIntegrityProvider {
    provider_sha256: IntegritySha256,
}

impl KeylessIntegrityProvider {
    pub const fn profile_id(&self) -> &'static str {
        PROFILE_ID
    }

    pub const fn provider_sha256(&self) -> IntegritySha256 {
        self.provider_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeylessProviderError {
    UnsupportedProfile,
    ProviderValues,
}

/// Validates the only materialized integrity provider without accessing files or credentials.
pub fn validate_keyless_provider(
    provider_json: &[u8],
) -> Result<KeylessIntegrityProvider, KeylessProviderError> {
    let value: Value =
        serde_json::from_slice(provider_json).map_err(|_| KeylessProviderError::ProviderValues)?;
    let root = object(&value)?;

    let profile_id = string_field(root, "profile_id")?;
    if profile_id != PROFILE_ID {
        return Err(KeylessProviderError::UnsupportedProfile);
    }

    require_keys(
        root,
        &[
            "schema",
            "profile_id",
            "crc32",
            "crc8",
            "crc16",
            "frame_bytes",
            "suppressed_sources",
            "primary_bytes",
            "secondary_bytes",
            "private_authentication_material",
        ],
    )?;
    require_string(root, "schema", "public-integrity-provider.v1")?;
    require_string(
        root,
        "private_authentication_material",
        "BLOCKED_AND_EXCLUDED",
    )?;
    require_u64(root, "frame_bytes", 4096)?;
    require_u64(root, "suppressed_sources", 2)?;
    require_u64(root, "primary_bytes", 4)?;
    require_u64(root, "secondary_bytes", 1)?;

    let crc32 = object(
        root.get("crc32")
            .ok_or(KeylessProviderError::ProviderValues)?,
    )?;
    require_keys(
        crc32,
        &[
            "name",
            "init",
            "reflected_polynomial",
            "xorout",
            "check_123456789",
        ],
    )?;
    require_string(crc32, "name", "CRC32/ISO-HDLC")?;
    require_u64(crc32, "init", u64::from(u32::MAX))?;
    require_u64(crc32, "reflected_polynomial", 0xedb8_8320)?;
    require_u64(crc32, "xorout", u64::from(u32::MAX))?;
    require_u64(crc32, "check_123456789", 0xcbf4_3926)?;

    let crc8 = object(
        root.get("crc8")
            .ok_or(KeylessProviderError::ProviderValues)?,
    )?;
    require_keys(
        crc8,
        &["name", "init", "polynomial", "xorout", "check_123456789"],
    )?;
    require_string(crc8, "name", "CRC8/SMBUS")?;
    require_u64(crc8, "init", 0)?;
    require_u64(crc8, "polynomial", 0x07)?;
    require_u64(crc8, "xorout", 0)?;
    require_u64(crc8, "check_123456789", 0xf4)?;

    let crc16 = object(
        root.get("crc16")
            .ok_or(KeylessProviderError::ProviderValues)?,
    )?;
    require_keys(
        crc16,
        &[
            "name",
            "init",
            "polynomial",
            "reflected",
            "xorout",
            "coverage_generate",
            "coverage_verify",
            "residual",
        ],
    )?;
    require_string(crc16, "name", "EAC3 CRC16")?;
    require_u64(crc16, "init", 0)?;
    require_u64(crc16, "polynomial", 0x8005)?;
    require_bool(crc16, "reflected", false)?;
    require_u64(crc16, "xorout", 0)?;
    require_string(crc16, "coverage_generate", "frame[2:4094]")?;
    require_string(crc16, "coverage_verify", "frame[2:4096]")?;
    require_u64(crc16, "residual", 0)?;

    let canonical = serde_json::to_vec(&value).map_err(|_| KeylessProviderError::ProviderValues)?;
    let provider_sha256 = sha256(&canonical);
    if provider_sha256.to_hex() != EXPECTED_PROVIDER_SHA256 {
        return Err(KeylessProviderError::ProviderValues);
    }

    Ok(KeylessIntegrityProvider { provider_sha256 })
}

/// CRC32/ISO-HDLC with reflected polynomial 0xedb88320.
pub fn crc32_iso_hdlc(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    crc ^ u32::MAX
}

/// CRC8/SMBUS, MSB-first with polynomial 0x07.
pub fn crc8_smbus(bytes: &[u8]) -> u8 {
    let mut crc = 0_u8;
    for byte in bytes {
        crc ^= *byte;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 {
                (crc << 1) ^ 0x07
            } else {
                crc << 1
            };
        }
    }
    crc
}

fn object(value: &Value) -> Result<&serde_json::Map<String, Value>, KeylessProviderError> {
    value
        .as_object()
        .ok_or(KeylessProviderError::ProviderValues)
}

fn require_keys(
    value: &serde_json::Map<String, Value>,
    expected: &[&str],
) -> Result<(), KeylessProviderError> {
    if value.len() == expected.len() && expected.iter().all(|key| value.contains_key(*key)) {
        Ok(())
    } else {
        Err(KeylessProviderError::ProviderValues)
    }
}

fn string_field<'a>(
    value: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a str, KeylessProviderError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or(KeylessProviderError::ProviderValues)
}

fn require_string(
    value: &serde_json::Map<String, Value>,
    key: &str,
    expected: &str,
) -> Result<(), KeylessProviderError> {
    if string_field(value, key)? == expected {
        Ok(())
    } else {
        Err(KeylessProviderError::ProviderValues)
    }
}

fn require_u64(
    value: &serde_json::Map<String, Value>,
    key: &str,
    expected: u64,
) -> Result<(), KeylessProviderError> {
    if value.get(key).and_then(Value::as_u64) == Some(expected) {
        Ok(())
    } else {
        Err(KeylessProviderError::ProviderValues)
    }
}

fn require_bool(
    value: &serde_json::Map<String, Value>,
    key: &str,
    expected: bool,
) -> Result<(), KeylessProviderError> {
    if value.get(key).and_then(Value::as_bool) == Some(expected) {
        Ok(())
    } else {
        Err(KeylessProviderError::ProviderValues)
    }
}
