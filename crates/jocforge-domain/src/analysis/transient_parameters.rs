// pattern: Functional Core
//! Identity-first typed provider for the selected F07 D05 parameter asset.

use serde_json::Value;
use sha2::{Digest, Sha256};

const PROVIDER_BYTES: usize = 7_548;
const PROVIDER_SHA256: &str = "130cd1a446ea8413a1d0d4c5eea631e890b7f1cd675975efa92697e258402991";
const PROVIDER_SCHEMA: &str = "d05-authorized-functional-constants.v1";
const CANONICAL_PARAMETER_BYTES: usize = 132;
const CANONICAL_PARAMETER_SHA256: &str =
    "05bd7906942e0d786a4e31bcd7ca4352b1bb41d63e30e14b723c1c6ab4f86eee";
const GROUP_ORDER: [&str; 6] = ["D05-P", "D05-A", "D05-S", "D05-O", "D05-M", "D05-C"];
const THREE_SECTION_FIELDS: [&str; 15] = [
    "section0.a0",
    "section0.a1",
    "section0.b0",
    "section0.b1",
    "section0.b2",
    "section1.a0",
    "section1.a1",
    "section1.b0",
    "section1.b1",
    "section1.b2",
    "section2.a0",
    "section2.a1",
    "section2.b0",
    "section2.b1",
    "section2.b2",
];
const LEVEL_FIELDS: [&str; 3] = ["whole_block", "half_block", "quarter_block"];

/// Failures returned by the selected F07 D05 provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransientParameterError {
    ParameterUnavailable,
    InvalidDocument,
}

impl TransientParameterError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ParameterUnavailable => "PARAMETER_UNAVAILABLE",
            Self::InvalidDocument => "TRANSIENT_PARAMETER_ERROR",
        }
    }
}

/// The exact binary32 words supplied for the selected F07 transient profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransientParameters {
    two_section_words: [[u32; 5]; 2],
    three_section_words: [[u32; 5]; 3],
    short_threshold_words: [u32; 3],
    other_threshold_words: [u32; 3],
    minimum_power_word: u32,
    history_clear_threshold_word: u32,
}

impl TransientParameters {
    pub const fn two_section_words(&self) -> &[[u32; 5]; 2] {
        &self.two_section_words
    }

    pub const fn three_section_words(&self) -> &[[u32; 5]; 3] {
        &self.three_section_words
    }

    pub const fn short_threshold_words(&self) -> &[u32; 3] {
        &self.short_threshold_words
    }

    pub const fn other_threshold_words(&self) -> &[u32; 3] {
        &self.other_threshold_words
    }

    pub const fn minimum_power_word(&self) -> u32 {
        self.minimum_power_word
    }

    pub const fn history_clear_threshold_word(&self) -> u32 {
        self.history_clear_threshold_word
    }

    /// Serializes all 33 parameter words in the F07A group order as F32LE.
    pub fn serialized_words_le(&self) -> [u8; CANONICAL_PARAMETER_BYTES] {
        let mut bytes = [0; CANONICAL_PARAMETER_BYTES];
        let mut offset = 0;
        for section in self.two_section_words {
            for word in section {
                append_word(&mut bytes, &mut offset, word);
            }
        }
        for section in self.three_section_words {
            for word in section {
                append_word(&mut bytes, &mut offset, word);
            }
        }
        for word in self.short_threshold_words {
            append_word(&mut bytes, &mut offset, word);
        }
        for word in self.other_threshold_words {
            append_word(&mut bytes, &mut offset, word);
        }
        append_word(&mut bytes, &mut offset, self.minimum_power_word);
        append_word(&mut bytes, &mut offset, self.history_clear_threshold_word);
        debug_assert_eq!(offset, bytes.len());
        bytes
    }
}

/// Loads the one hash-pinned D05 provider. File and schema I/O remain caller-owned.
pub fn load_transient_parameters(
    provider_file: Option<&[u8]>,
) -> Result<TransientParameters, TransientParameterError> {
    let provider_file = provider_file.ok_or(TransientParameterError::ParameterUnavailable)?;
    if provider_file.len() != PROVIDER_BYTES || sha256_hex(provider_file) != PROVIDER_SHA256 {
        return Err(TransientParameterError::InvalidDocument);
    }

    let document: Value = serde_json::from_slice(provider_file)
        .map_err(|_| TransientParameterError::InvalidDocument)?;
    validate_document_header(&document)?;
    let groups = document
        .get("groups")
        .and_then(Value::as_object)
        .ok_or(TransientParameterError::InvalidDocument)?;
    if groups.len() != GROUP_ORDER.len() {
        return Err(TransientParameterError::InvalidDocument);
    }

    let two = group_words::<10>(&document, "D05-P", &two_section_field_order())?;
    let three = group_words::<15>(&document, "D05-A", &three_section_field_order())?;
    let short = group_words::<3>(&document, "D05-S", &level_field_order())?;
    let other = group_words::<3>(&document, "D05-O", &level_field_order())?;
    let minimum = group_words::<1>(&document, "D05-M", &["minimum_filtered_peak_gate"])?[0];
    let clear = group_words::<1>(
        &document,
        "D05-C",
        &["block_end_abs_history_sum_clear_threshold"],
    )?[0];

    let parameters = TransientParameters {
        two_section_words: [
            [two[0], two[1], two[2], two[3], two[4]],
            [two[5], two[6], two[7], two[8], two[9]],
        ],
        three_section_words: [
            [three[0], three[1], three[2], three[3], three[4]],
            [three[5], three[6], three[7], three[8], three[9]],
            [three[10], three[11], three[12], three[13], three[14]],
        ],
        short_threshold_words: short,
        other_threshold_words: other,
        minimum_power_word: minimum,
        history_clear_threshold_word: clear,
    };

    let serialization = document
        .get("serialization")
        .ok_or(TransientParameterError::InvalidDocument)?;
    if serialization.get("word_byte_order").and_then(Value::as_str) != Some("little-endian")
        || serialization.get("slots").and_then(Value::as_u64) != Some(33)
        || serialization.get("bytes").and_then(Value::as_u64) != Some(132)
        || serialization
            .get("distinct_f32_words")
            .and_then(Value::as_u64)
            != Some(28)
        || serialization.get("sha256").and_then(Value::as_str) != Some(CANONICAL_PARAMETER_SHA256)
        || sha256_hex(&parameters.serialized_words_le()) != CANONICAL_PARAMETER_SHA256
    {
        return Err(TransientParameterError::InvalidDocument);
    }
    Ok(parameters)
}

fn validate_document_header(document: &Value) -> Result<(), TransientParameterError> {
    let group_order = document
        .get("group_order")
        .and_then(Value::as_array)
        .ok_or(TransientParameterError::InvalidDocument)?;
    let group_order_matches = group_order.len() == GROUP_ORDER.len()
        && group_order
            .iter()
            .zip(GROUP_ORDER)
            .all(|(actual, expected)| actual.as_str() == Some(expected));
    if document.get("schema").and_then(Value::as_str) != Some(PROVIDER_SCHEMA)
        || document.get("classification").and_then(Value::as_str)
            != Some("AUTHORIZED_NONSECRET_AUDIO_FUNCTIONAL_CONSTANTS")
        || document.get("delivery_scope").and_then(Value::as_str)
            != Some("INTERNAL_RESEARCH_AND_INTERNAL_IMPLEMENTATION_ONLY")
        || document
            .get("public_or_third_party_distribution_authorized")
            .and_then(Value::as_bool)
            != Some(false)
        || document
            .get("secret_or_authentication_material")
            .and_then(Value::as_bool)
            != Some(false)
        || document
            .get("private_vendor_key_included")
            .and_then(Value::as_bool)
            != Some(false)
        || document
            .get("other_profile_constants_authorized_by_this_artifact")
            .and_then(Value::as_bool)
            != Some(false)
        || !group_order_matches
    {
        return Err(TransientParameterError::InvalidDocument);
    }
    Ok(())
}

fn group_words<const N: usize>(
    document: &Value,
    group_name: &str,
    expected_fields: &[&str],
) -> Result<[u32; N], TransientParameterError> {
    let group = document
        .get("groups")
        .and_then(|groups| groups.get(group_name))
        .ok_or(TransientParameterError::InvalidDocument)?;
    let field_order = group
        .get("field_order")
        .and_then(Value::as_array)
        .ok_or(TransientParameterError::InvalidDocument)?;
    if group.get("type").and_then(Value::as_str) != Some("binary32")
        || field_order.len() != expected_fields.len()
        || field_order
            .iter()
            .zip(expected_fields)
            .any(|(actual, expected)| actual.as_str() != Some(expected))
        || group.get("count").and_then(Value::as_u64) != Some(N as u64)
    {
        return Err(TransientParameterError::InvalidDocument);
    }

    let word_values = group
        .get("f32_words_u32")
        .and_then(Value::as_array)
        .ok_or(TransientParameterError::InvalidDocument)?;
    if word_values.len() != N {
        return Err(TransientParameterError::InvalidDocument);
    }
    let mut words = [0; N];
    for (index, value) in word_values.iter().enumerate() {
        let word = u32::try_from(
            value
                .as_u64()
                .ok_or(TransientParameterError::InvalidDocument)?,
        )
        .map_err(|_| TransientParameterError::InvalidDocument)?;
        if !f32::from_bits(word).is_finite() {
            return Err(TransientParameterError::InvalidDocument);
        }
        words[index] = word;
    }

    let group_bytes = words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    if group.get("f32le_sha256").and_then(Value::as_str) != Some(sha256_hex(&group_bytes).as_str())
        || group.get("distinct_f32_words").and_then(Value::as_u64)
            != Some(
                u64::try_from(
                    words
                        .iter()
                        .copied()
                        .collect::<std::collections::HashSet<_>>()
                        .len(),
                )
                .unwrap_or(u64::MAX),
            )
    {
        return Err(TransientParameterError::InvalidDocument);
    }
    Ok(words)
}

fn two_section_field_order() -> [&'static str; 10] {
    [
        "section0.a0",
        "section0.a1",
        "section0.b0",
        "section0.b1",
        "section0.b2",
        "section1.a0",
        "section1.a1",
        "section1.b0",
        "section1.b1",
        "section1.b2",
    ]
}

fn three_section_field_order() -> [&'static str; 15] {
    THREE_SECTION_FIELDS
}

fn level_field_order() -> [&'static str; 3] {
    LEVEL_FIELDS
}

fn append_word(bytes: &mut [u8; CANONICAL_PARAMETER_BYTES], offset: &mut usize, word: u32) {
    bytes[*offset..*offset + 4].copy_from_slice(&word.to_le_bytes());
    *offset += 4;
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
