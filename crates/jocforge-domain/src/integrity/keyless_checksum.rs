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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmdfProtectionBitRange {
    start_bit: u64,
    end_bit: u64,
}

impl EmdfProtectionBitRange {
    pub const fn start_bit(self) -> u64 {
        self.start_bit
    }

    pub const fn end_bit(self) -> u64 {
        self.end_bit
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedEmdfContainer {
    bytes: Vec<u8>,
    primary_range: EmdfProtectionBitRange,
    secondary_range: EmdfProtectionBitRange,
    primary_crc32: u32,
    secondary_crc8: u8,
    input_sha256: IntegritySha256,
    output_sha256: IntegritySha256,
}

impl ProtectedEmdfContainer {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn primary_range(&self) -> EmdfProtectionBitRange {
        self.primary_range
    }

    pub const fn secondary_range(&self) -> EmdfProtectionBitRange {
        self.secondary_range
    }

    pub const fn primary_crc32(&self) -> u32 {
        self.primary_crc32
    }

    pub const fn secondary_crc8(&self) -> u8 {
        self.secondary_crc8
    }

    pub const fn input_sha256(&self) -> IntegritySha256 {
        self.input_sha256
    }

    pub const fn output_sha256(&self) -> IntegritySha256 {
        self.output_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmdfProtectionError {
    UnsupportedProvider,
    ContainerShape,
    TerminalLayout,
    SlotRange,
    SlotNotZero,
}

/// F14K patches F13's zero-slot container using the serializer's dynamic MSB bit ranges.
///
/// It does not embed the container in an E-AC-3 frame or write CRC16; F12 owns those bytes.
pub fn protect_emdf_container(
    provider: &KeylessIntegrityProvider,
    zero_slot_container: &[u8],
    terminal_layout: &Value,
) -> Result<ProtectedEmdfContainer, EmdfProtectionError> {
    if provider.profile_id() != PROFILE_ID {
        return Err(EmdfProtectionError::UnsupportedProvider);
    }
    if !(5..=511).contains(&zero_slot_container.len()) || zero_slot_container[..2] != [0x58, 0x38] {
        return Err(EmdfProtectionError::ContainerShape);
    }
    let body_bytes = u16::from_be_bytes([zero_slot_container[2], zero_slot_container[3]]) as usize;
    if zero_slot_container.len() != body_bytes + 4
        || terminal_layout.get("wire_bytes").and_then(Value::as_u64)
            != Some(zero_slot_container.len() as u64)
        || terminal_layout.get("body_bytes").and_then(Value::as_u64) != Some(body_bytes as u64)
        || terminal_layout.get("bit_order").and_then(Value::as_str) != Some("MSB-first")
        || terminal_layout["primary_length_code"]
            .get("byte_count")
            .and_then(Value::as_u64)
            != Some(4)
        || terminal_layout["secondary_length_code"]
            .get("byte_count")
            .and_then(Value::as_u64)
            != Some(1)
    {
        return Err(EmdfProtectionError::ContainerShape);
    }

    let total_bits = zero_slot_container
        .len()
        .checked_mul(8)
        .and_then(|bits| u64::try_from(bits).ok())
        .ok_or(EmdfProtectionError::ContainerShape)?;
    let primary_range = bit_range(terminal_layout.get("primary_protection_range_bits"))?;
    let secondary_range = bit_range(terminal_layout.get("secondary_protection_range_bits"))?;
    let padding_range = bit_range(terminal_layout.get("terminal_padding_range_bits"))?;
    let (terminal_value, terminal_start, terminal_width) =
        layout_field(terminal_layout.get("terminal_id"))?;
    let (primary_value, primary_start, primary_width) =
        layout_field(terminal_layout.get("primary_length_code"))?;
    let (secondary_value, secondary_start, secondary_width) =
        layout_field(terminal_layout.get("secondary_length_code"))?;
    let fields_are_contiguous = terminal_start
        .checked_add(terminal_width)
        .is_some_and(|end| end == primary_start)
        && primary_start
            .checked_add(primary_width)
            .is_some_and(|end| end == secondary_start)
        && secondary_start
            .checked_add(secondary_width)
            .is_some_and(|end| end == primary_range.start_bit);
    let ranges_are_valid = primary_range.start_bit >= 32
        && primary_range.end_bit - primary_range.start_bit == 32
        && secondary_range.start_bit == primary_range.end_bit
        && secondary_range.end_bit - secondary_range.start_bit == 8
        && padding_range.start_bit == secondary_range.end_bit
        && padding_range.end_bit == total_bits
        && terminal_layout
            .get("terminal_padding_bits")
            .and_then(Value::as_u64)
            == Some(padding_range.end_bit - padding_range.start_bit);
    if terminal_value != 0
        || terminal_width != 5
        || primary_value != 2
        || primary_width != 2
        || secondary_value != 1
        || secondary_width != 2
        || primary_start.checked_add(primary_width) != Some(secondary_start)
        || !fields_are_contiguous
        || !ranges_are_valid
    {
        return Err(EmdfProtectionError::SlotRange);
    }
    let terminal_id_range = EmdfProtectionBitRange {
        start_bit: terminal_start,
        end_bit: terminal_start + terminal_width,
    };
    let primary_code_range = EmdfProtectionBitRange {
        start_bit: primary_start,
        end_bit: primary_start + primary_width,
    };
    let secondary_code_range = EmdfProtectionBitRange {
        start_bit: secondary_start,
        end_bit: secondary_start + secondary_width,
    };
    if read_msb_bits(zero_slot_container, terminal_id_range) != Some(terminal_value)
        || read_msb_bits(zero_slot_container, primary_code_range) != Some(primary_value)
        || read_msb_bits(zero_slot_container, secondary_code_range) != Some(secondary_value)
    {
        return Err(EmdfProtectionError::TerminalLayout);
    }
    if read_msb_bits(zero_slot_container, primary_range).ok_or(EmdfProtectionError::SlotRange)? != 0
        || read_msb_bits(zero_slot_container, secondary_range)
            .ok_or(EmdfProtectionError::SlotRange)?
            != 0
    {
        return Err(EmdfProtectionError::SlotNotZero);
    }

    let primary_crc32 = crc32_iso_hdlc(&zero_slot_container[4..]);
    let primary_bytes = primary_crc32.to_be_bytes();
    let secondary_crc8 = crc8_smbus(&primary_bytes);
    let mut bytes = zero_slot_container.to_vec();
    write_msb_bits(&mut bytes, primary_range, u64::from(primary_crc32))
        .ok_or(EmdfProtectionError::SlotRange)?;
    write_msb_bits(&mut bytes, secondary_range, u64::from(secondary_crc8))
        .ok_or(EmdfProtectionError::SlotRange)?;
    let input_sha256 = sha256(zero_slot_container);
    let output_sha256 = sha256(&bytes);

    Ok(ProtectedEmdfContainer {
        bytes,
        primary_range,
        secondary_range,
        primary_crc32,
        secondary_crc8,
        input_sha256,
        output_sha256,
    })
}

fn bit_range(value: Option<&Value>) -> Result<EmdfProtectionBitRange, EmdfProtectionError> {
    let values = value
        .and_then(Value::as_array)
        .filter(|values| values.len() == 2)
        .ok_or(EmdfProtectionError::TerminalLayout)?;
    let start_bit = values[0]
        .as_u64()
        .ok_or(EmdfProtectionError::TerminalLayout)?;
    let end_bit = values[1]
        .as_u64()
        .ok_or(EmdfProtectionError::TerminalLayout)?;
    if end_bit < start_bit {
        return Err(EmdfProtectionError::TerminalLayout);
    }
    Ok(EmdfProtectionBitRange { start_bit, end_bit })
}

fn layout_field(value: Option<&Value>) -> Result<(u64, u64, u64), EmdfProtectionError> {
    let field = value
        .and_then(Value::as_object)
        .ok_or(EmdfProtectionError::TerminalLayout)?;
    let value = field
        .get("value")
        .and_then(Value::as_u64)
        .ok_or(EmdfProtectionError::TerminalLayout)?;
    let start_bit = field
        .get("start_bit")
        .and_then(Value::as_u64)
        .ok_or(EmdfProtectionError::TerminalLayout)?;
    let width_bits = field
        .get("width_bits")
        .and_then(Value::as_u64)
        .ok_or(EmdfProtectionError::TerminalLayout)?;
    Ok((value, start_bit, width_bits))
}

fn read_msb_bits(bytes: &[u8], range: EmdfProtectionBitRange) -> Option<u64> {
    let width = range.end_bit.checked_sub(range.start_bit)?;
    if width > 64 || range.end_bit > u64::try_from(bytes.len()).ok()?.checked_mul(8)? {
        return None;
    }
    let mut value = 0_u64;
    for bit in range.start_bit..range.end_bit {
        let byte = usize::try_from(bit / 8).ok()?;
        let shift = 7 - u32::try_from(bit % 8).ok()?;
        value = (value << 1) | u64::from((bytes.get(byte)? >> shift) & 1);
    }
    Some(value)
}

fn write_msb_bits(bytes: &mut [u8], range: EmdfProtectionBitRange, value: u64) -> Option<()> {
    let width = range.end_bit.checked_sub(range.start_bit)?;
    if width > 64 || range.end_bit > u64::try_from(bytes.len()).ok()?.checked_mul(8)? {
        return None;
    }
    for offset in 0..width {
        let bit = range.start_bit.checked_add(offset)?;
        let byte = usize::try_from(bit / 8).ok()?;
        let shift = 7 - u32::try_from(bit % 8).ok()?;
        let mask = 1_u8.checked_shl(shift)?;
        let output_bit = (value >> (width - offset - 1)) & 1 != 0;
        let destination = bytes.get_mut(byte)?;
        if output_bit {
            *destination |= mask;
        } else {
            *destination &= !mask;
        }
    }
    Some(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn provider() -> KeylessIntegrityProvider {
        validate_keyless_provider(include_bytes!(
            "../../tests/fixtures/integrity_owner/integrity-public-provider.json"
        ))
        .expect("approved public provider fixture")
    }

    fn unaligned_terminal_layout() -> Value {
        json!({
            "bit_order": "MSB-first",
            "terminal_id": { "value": 0, "start_bit": 32, "width_bits": 5 },
            "primary_length_code": { "value": 2, "start_bit": 37, "width_bits": 2, "byte_count": 4 },
            "secondary_length_code": { "value": 1, "start_bit": 39, "width_bits": 2, "byte_count": 1 },
            "primary_protection_range_bits": [41, 73],
            "secondary_protection_range_bits": [73, 81],
            "terminal_padding_range_bits": [81, 88],
            "terminal_padding_bits": 7,
            "wire_bytes": 11,
            "body_bytes": 7
        })
    }

    #[test]
    fn protects_dynamic_non_byte_aligned_terminal_slots() {
        let zero_slots = [0x58, 0x38, 0x00, 0x07, 0x04, 0x80, 0, 0, 0, 0, 0];
        let layout = unaligned_terminal_layout();
        let protected = protect_emdf_container(&provider(), &zero_slots, &layout).unwrap();

        assert_eq!(protected.bytes().len(), zero_slots.len());
        assert_eq!(protected.primary_range().start_bit(), 41);
        assert_eq!(protected.secondary_range().start_bit(), 73);
        assert_eq!(
            read_msb_bits(protected.bytes(), protected.primary_range()),
            Some(u64::from(protected.primary_crc32()))
        );
        assert_eq!(
            read_msb_bits(protected.bytes(), protected.secondary_range()),
            Some(u64::from(protected.secondary_crc8()))
        );
        assert_eq!(protected.primary_crc32(), crc32_iso_hdlc(&zero_slots[4..]));
        assert_eq!(
            protected.secondary_crc8(),
            crc8_smbus(&protected.primary_crc32().to_be_bytes())
        );
    }

    #[test]
    fn rejects_out_of_range_dynamic_slots_and_nonzero_slots() {
        let zero_slots = [0x58, 0x38, 0x00, 0x07, 0x04, 0x80, 0, 0, 0, 0, 0];
        let mut bad_range = unaligned_terminal_layout();
        bad_range["primary_protection_range_bits"] = json!([57, 89]);
        bad_range["secondary_protection_range_bits"] = json!([89, 97]);
        assert_eq!(
            protect_emdf_container(&provider(), &zero_slots, &bad_range),
            Err(EmdfProtectionError::SlotRange)
        );

        let mut nonzero_slot = zero_slots;
        nonzero_slot[5] |= 0x01;
        assert_eq!(
            protect_emdf_container(&provider(), &nonzero_slot, &unaligned_terminal_layout()),
            Err(EmdfProtectionError::SlotNotZero)
        );
    }
}
