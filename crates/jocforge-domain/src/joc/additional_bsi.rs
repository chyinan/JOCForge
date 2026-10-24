// pattern: Functional Core
//! Selected additional-BSI count, two-byte payload prefix, and literal import copy.

/// Fixed payload extent in the selected one-substream additional-BSI path.
pub const ADDITIONAL_BSI_PAYLOAD_BYTES: usize = 64;

/// An admitted selected-profile count whose constructor status was zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedAdditionalBsiCount(u8);

impl ValidatedAdditionalBsiCount {
    pub const fn as_i32(self) -> i32 {
        self.0 as i32
    }
}

/// Scalar and observable slot effects from the selected configuration constructor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfigurationCountAdmission {
    return_i32: i32,
    count_slot_after_i32: i32,
    prefix_write_observed: bool,
    validated_count: Option<ValidatedAdditionalBsiCount>,
}

impl ConfigurationCountAdmission {
    pub const fn return_i32(self) -> i32 {
        self.return_i32
    }

    pub const fn count_slot_after_i32(self) -> i32 {
        self.count_slot_after_i32
    }

    pub const fn prefix_write_observed(self) -> bool {
        self.prefix_write_observed
    }

    pub const fn validated_count(self) -> Option<ValidatedAdditionalBsiCount> {
        self.validated_count
    }
}

/// Models the selected count slot result and observed prefix-write ordering.
///
/// This is a scalar outcome projection. It does not materialize the unavailable
/// header/selector prefix bytes or claim whole-record rollback semantics.
pub fn admit_selected_configuration_count(
    explicit_count_i32: Option<i32>,
    count_slot_before_i32: i32,
) -> ConfigurationCountAdmission {
    let candidate = explicit_count_i32.unwrap_or(11);
    if let (true, Ok(count)) = ((1..=16).contains(&candidate), u8::try_from(candidate)) {
        ConfigurationCountAdmission {
            return_i32: 0,
            count_slot_after_i32: candidate,
            prefix_write_observed: true,
            validated_count: Some(ValidatedAdditionalBsiCount(count)),
        }
    } else {
        ConfigurationCountAdmission {
            return_i32: 220,
            count_slot_after_i32: count_slot_before_i32,
            prefix_write_observed: true,
            validated_count: None,
        }
    }
}

/// Derives the selected wrapper count from a bounded rate code and signed-I16 labels.
///
/// `None` is a local Rust input-domain rejection; the washed package does not bind
/// a native error code for invalid rate codes or channel-list extents.
pub fn derive_selected_bsi_count(rate_code: u8, channel_codes_i16: &[i16]) -> Option<i32> {
    if rate_code > 5 || !(1..=128).contains(&channel_codes_i16.len()) {
        return None;
    }
    let lfe_present = channel_codes_i16
        .iter()
        .any(|code| *code >= 0 && ((*code >> 8) & 127) == 4);
    let non_lfe_count = if rate_code == 0 { 11 } else { 15 };
    Some(non_lfe_count + i32::from(lfe_present))
}

/// The selected one-substream additional-BSI record before importer copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdditionalBsiRecord {
    enable_u16: u16,
    length_minus_one_u16: u16,
    payload_u8: [u8; ADDITIONAL_BSI_PAYLOAD_BYTES],
}

impl AdditionalBsiRecord {
    pub const fn enable_u16(&self) -> u16 {
        self.enable_u16
    }

    pub const fn length_minus_one_u16(&self) -> u16 {
        self.length_minus_one_u16
    }

    pub const fn payload_u8(&self) -> &[u8; ADDITIONAL_BSI_PAYLOAD_BYTES] {
        &self.payload_u8
    }
}

/// The literal field projection copied into the importer working header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedAdditionalBsi {
    enable_u16: u16,
    length_minus_one_u16: u16,
    payload_u8: [u8; ADDITIONAL_BSI_PAYLOAD_BYTES],
}

impl ImportedAdditionalBsi {
    pub const fn enable_u16(&self) -> u16 {
        self.enable_u16
    }

    pub const fn length_minus_one_u16(&self) -> u16 {
        self.length_minus_one_u16
    }

    pub const fn payload_u8(&self) -> &[u8; ADDITIONAL_BSI_PAYLOAD_BYTES] {
        &self.payload_u8
    }
}

/// Produces the selected enable/length fields and writes only payload bytes 0 and 1.
pub fn produce_selected_additional_bsi(
    count: ValidatedAdditionalBsiCount,
    payload_before_u8: &[u8; ADDITIONAL_BSI_PAYLOAD_BYTES],
) -> AdditionalBsiRecord {
    let mut payload_u8 = *payload_before_u8;
    payload_u8[0] = 1;
    payload_u8[1] = count.0;
    AdditionalBsiRecord {
        enable_u16: 1,
        length_minus_one_u16: 1,
        payload_u8,
    }
}

/// Copies the selected U16 fields and all 64 payload bytes literally.
pub fn import_selected_additional_bsi(source: &AdditionalBsiRecord) -> ImportedAdditionalBsi {
    ImportedAdditionalBsi {
        enable_u16: source.enable_u16,
        length_minus_one_u16: source.length_minus_one_u16,
        payload_u8: source.payload_u8,
    }
}
