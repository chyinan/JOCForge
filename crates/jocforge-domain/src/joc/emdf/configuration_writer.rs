// pattern: Functional Core
//! Selected F13.3 typed configuration writer and bounded descriptor bit budget.
//!
//! This module models only the approved semantic configuration projection and
//! the bounded bit output described by the washed F13C configuration-writer
//! package. It does not model parent transaction or native status codes.

/// Typed fields consumed by the selected F13.3 configuration writer.
///
/// Gated-out fields are `None`; validation rejects values outside the approved
/// semantic ranges before any output is created.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Configuration {
    sample_offset: Option<u32>,
    duration: Option<u32>,
    group_id: Option<u32>,
    codec_data_present: bool,
    discard_unknown: bool,
    frame_aligned: Option<bool>,
    create_duplicate: Option<bool>,
    remove_duplicate: Option<bool>,
    priority: Option<u8>,
    processing_allowed: Option<u8>,
}

impl Configuration {
    /// Creates a configuration only when all fields are representable and
    /// branch-consistent with the approved ten-field projection.
    #[allow(clippy::too_many_arguments)]
    pub const fn try_new(
        sample_offset: Option<u32>,
        duration: Option<u32>,
        group_id: Option<u32>,
        codec_data_present: u8,
        discard_unknown: u8,
        frame_aligned: Option<u8>,
        create_duplicate: Option<u8>,
        remove_duplicate: Option<u8>,
        priority: Option<u8>,
        processing_allowed: Option<u8>,
    ) -> Option<Self> {
        if matches!(sample_offset, Some(value) if value > 2_047)
            || matches!(duration, Some(value) if value > 4_196_351)
            || matches!(group_id, Some(value) if value > 87_379)
        {
            return None;
        }

        let Some(codec_data_present) = decode_presence(codec_data_present) else {
            return None;
        };
        let Some(discard_unknown) = decode_presence(discard_unknown) else {
            return None;
        };
        if !optional_presence_is_valid(frame_aligned)
            || !optional_presence_is_valid(create_duplicate)
            || !optional_presence_is_valid(remove_duplicate)
        {
            return None;
        }
        let frame_aligned = decode_optional_presence(frame_aligned).into_option();
        let create_duplicate = decode_optional_presence(create_duplicate).into_option();
        let remove_duplicate = decode_optional_presence(remove_duplicate).into_option();

        let alignment_is_consumed = !discard_unknown && sample_offset.is_none();
        if alignment_is_consumed != frame_aligned.is_some() {
            return None;
        }

        let alignment_is_true = matches!(frame_aligned, Some(true));
        let duplicate_fields_are_consumed = alignment_is_consumed && alignment_is_true;
        if duplicate_fields_are_consumed != create_duplicate.is_some()
            || duplicate_fields_are_consumed != remove_duplicate.is_some()
        {
            return None;
        }

        let priority_is_consumed =
            !discard_unknown && (sample_offset.is_some() || alignment_is_true);
        if priority_is_consumed {
            if !matches!(priority, Some(value) if value <= 31)
                || !matches!(processing_allowed, Some(value) if value <= 3)
            {
                return None;
            }
        } else if priority.is_some() || processing_allowed.is_some() {
            return None;
        }

        Some(Self {
            sample_offset,
            duration,
            group_id,
            codec_data_present,
            discard_unknown,
            frame_aligned,
            create_duplicate,
            remove_duplicate,
            priority,
            processing_allowed,
        })
    }

    /// Selected ID11 configuration from the washed semantic binding.
    pub const fn selected_id11() -> Self {
        Self {
            sample_offset: None,
            duration: None,
            group_id: Some(0),
            codec_data_present: false,
            discard_unknown: false,
            frame_aligned: Some(false),
            create_duplicate: None,
            remove_duplicate: None,
            priority: None,
            processing_allowed: None,
        }
    }

    /// Selected ID14 configuration from the washed semantic binding.
    pub const fn selected_id14() -> Self {
        Self {
            sample_offset: None,
            duration: None,
            group_id: Some(0),
            codec_data_present: false,
            discard_unknown: false,
            frame_aligned: Some(true),
            create_duplicate: Some(false),
            remove_duplicate: Some(false),
            priority: Some(0),
            processing_allowed: Some(0),
        }
    }

    /// Selected ID2 configuration from the washed semantic binding.
    pub const fn selected_id2() -> Self {
        Self {
            sample_offset: None,
            duration: None,
            group_id: None,
            codec_data_present: false,
            discard_unknown: true,
            frame_aligned: None,
            create_duplicate: None,
            remove_duplicate: None,
            priority: None,
            processing_allowed: None,
        }
    }

    /// Selected ID1 configuration from the washed semantic binding.
    pub const fn selected_id1() -> Self {
        Self {
            sample_offset: None,
            duration: Some(1_536),
            group_id: None,
            codec_data_present: false,
            discard_unknown: false,
            frame_aligned: Some(false),
            create_duplicate: None,
            remove_duplicate: None,
            priority: None,
            processing_allowed: None,
        }
    }
}

const fn decode_presence(value: u8) -> Option<bool> {
    match value {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

const fn optional_presence_is_valid(value: Option<u8>) -> bool {
    match value {
        Some(0 | 1) | None => true,
        Some(_) => false,
    }
}

#[derive(Clone, Copy)]
enum OptionalPresence {
    Absent,
    False,
    True,
}

impl OptionalPresence {
    const fn into_option(self) -> Option<bool> {
        match self {
            Self::Absent => None,
            Self::False => Some(false),
            Self::True => Some(true),
        }
    }
}

const fn decode_optional_presence(value: Option<u8>) -> OptionalPresence {
    match value {
        Some(1) => OptionalPresence::True,
        Some(0) => OptionalPresence::False,
        None | Some(_) => OptionalPresence::Absent,
    }
}

/// Bounded output from one local writer attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitWriteOutcome {
    bytes: Vec<u8>,
    bits_written: usize,
    complete: bool,
}

impl BitWriteOutcome {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn bits_written(&self) -> u64 {
        u64::try_from(self.bits_written).unwrap_or(u64::MAX)
    }

    pub fn rounded_bytes_written(&self) -> u64 {
        u64::try_from(self.bits_written.div_ceil(8)).unwrap_or(u64::MAX)
    }

    pub const fn is_complete(&self) -> bool {
        self.complete
    }
}

/// Writes the selected semantic configuration MSB-first into a byte budget.
///
/// A capacity exhaustion returns the retained bit prefix with `is_complete()`
/// false. This is a local bit-writer result, not a native status or a parent
/// transaction/rollback contract.
pub fn write_configuration(configuration: Configuration, capacity_bytes: u32) -> BitWriteOutcome {
    let mut writer = BitWriter::new(capacity_bytes);
    let complete = write_configuration_bits(&mut writer, configuration);
    writer.finish(complete)
}

/// Writes a selected sub-31 payload ID, its configuration and opaque body
/// through one bounded output cursor.
pub fn write_descriptor(
    payload_id: u8,
    configuration: Configuration,
    body: &[u8],
    capacity_bytes: u32,
) -> Option<BitWriteOutcome> {
    if payload_id >= 31 {
        return None;
    }

    let mut writer = BitWriter::new(capacity_bytes);
    let complete = writer.write(u32::from(payload_id), 5)
        && write_configuration_bits(&mut writer, configuration)
        && write_variable(&mut writer, u32::try_from(body.len()).ok()?, 8)
        && body.iter().all(|byte| writer.write(u32::from(*byte), 8));
    Some(writer.finish(complete))
}

fn write_configuration_bits(writer: &mut BitWriter, configuration: Configuration) -> bool {
    for (present, value, width) in [
        (
            configuration.sample_offset.is_some(),
            configuration.sample_offset,
            11,
        ),
        (configuration.duration.is_some(), configuration.duration, 11),
        (configuration.group_id.is_some(), configuration.group_id, 2),
    ] {
        if !writer.write(u32::from(present), 1) {
            return false;
        }
        if let Some(value) = value {
            if !write_variable(writer, value, width) {
                return false;
            }
        }
    }

    if !writer.write(u32::from(configuration.codec_data_present), 1)
        || (configuration.codec_data_present && !writer.write(0, 8))
        || !writer.write(u32::from(configuration.discard_unknown), 1)
    {
        return false;
    }

    if configuration.discard_unknown {
        return true;
    }

    if configuration.sample_offset.is_none() {
        let aligned = configuration.frame_aligned.unwrap_or(false);
        if !writer.write(u32::from(aligned), 1) {
            return false;
        }
        if aligned
            && (!writer.write(
                u32::from(configuration.create_duplicate.unwrap_or(false)),
                1,
            ) || !writer.write(
                u32::from(configuration.remove_duplicate.unwrap_or(false)),
                1,
            ))
        {
            return false;
        }
    }

    if configuration.sample_offset.is_some() || matches!(configuration.frame_aligned, Some(true)) {
        return writer.write(u32::from(configuration.priority.unwrap_or(0)), 5)
            && writer.write(u32::from(configuration.processing_allowed.unwrap_or(0)), 2);
    }

    true
}

fn write_variable(writer: &mut BitWriter, value: u32, width: u8) -> bool {
    let mut groups = 1_u32;
    let mut threshold = 1_u32 << width;
    let mut previous = 0_u32;
    while value >= threshold {
        previous = threshold;
        let Some(next) = threshold
            .checked_add(1)
            .and_then(|v| v.checked_mul(1_u32 << width))
        else {
            return false;
        };
        threshold = next;
        groups += 1;
        if groups > 16 {
            return false;
        }
    }

    let adjusted = value - previous;
    for index in (0..groups).rev() {
        if !writer.write(adjusted >> (u32::from(width) * index), width)
            || !writer.write(u32::from(index != 0), 1)
        {
            return false;
        }
    }
    true
}

struct BitWriter {
    bytes: Vec<u8>,
    capacity_bits: usize,
    bits_written: usize,
}

impl BitWriter {
    fn new(capacity_bytes: u32) -> Self {
        let capacity_bytes = usize::try_from(capacity_bytes).unwrap_or(usize::MAX);
        Self {
            bytes: Vec::with_capacity(capacity_bytes.min(4_096)),
            capacity_bits: capacity_bytes.saturating_mul(8),
            bits_written: 0,
        }
    }

    fn write(&mut self, value: u32, width: u8) -> bool {
        let Some(end) = self.bits_written.checked_add(usize::from(width)) else {
            return false;
        };
        if end > self.capacity_bits {
            return false;
        }
        for shift in (0..width).rev() {
            if self.bits_written % 8 == 0 {
                self.bytes.push(0);
            }
            let bit = ((value >> shift) & 1) as u8;
            let offset = 7 - (self.bits_written % 8);
            let last = self.bytes.len() - 1;
            self.bytes[last] |= bit << offset;
            self.bits_written += 1;
        }
        true
    }

    fn finish(self, complete: bool) -> BitWriteOutcome {
        BitWriteOutcome {
            bytes: self.bytes,
            bits_written: self.bits_written,
            complete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Configuration, write_configuration, write_descriptor};

    #[test]
    fn selected_configurations_match_washed_bit_budgets() {
        for (configuration, expected_bits) in [
            (Configuration::selected_id11(), "001000000"),
            (Configuration::selected_id14(), "001000001000000000"),
            (Configuration::selected_id2(), "00001"),
            (Configuration::selected_id1(), "011100000000000000"),
        ] {
            let output = write_configuration(configuration, 64);
            assert!(output.is_complete());
            assert_eq!(
                bits_as_text(output.bytes(), output.bits_written()),
                expected_bits
            );
        }
    }

    #[test]
    fn approved_limits_and_gated_fields_are_enforced_before_writing() {
        assert!(
            Configuration::try_new(
                Some(2_047),
                None,
                None,
                0,
                0,
                None,
                None,
                None,
                Some(31),
                Some(3)
            )
            .is_some()
        );
        assert!(
            Configuration::try_new(
                Some(2_048),
                None,
                None,
                0,
                0,
                None,
                None,
                None,
                Some(0),
                Some(0)
            )
            .is_none()
        );
        assert!(
            Configuration::try_new(
                None,
                Some(4_196_352),
                None,
                0,
                0,
                Some(0),
                None,
                None,
                None,
                None
            )
            .is_none()
        );
        assert!(
            Configuration::try_new(
                None,
                None,
                Some(87_380),
                0,
                0,
                Some(0),
                None,
                None,
                None,
                None
            )
            .is_none()
        );
        assert!(
            Configuration::try_new(None, None, None, 0, 1, Some(0), None, None, None, None)
                .is_none()
        );
        assert!(
            Configuration::try_new(None, None, None, 2, 0, Some(0), None, None, None, None)
                .is_none()
        );
    }

    #[test]
    fn zero_capacity_retains_no_bits_and_reports_local_incompleteness() {
        let output = write_configuration(Configuration::selected_id11(), 0);
        assert_eq!(output.bits_written(), 0);
        assert!(output.bytes().is_empty());
        assert!(!output.is_complete());
    }

    #[test]
    fn descriptor_budget_preserves_the_local_partial_prefix() {
        let body = vec![0x5a; 512];
        let partial = write_descriptor(14, Configuration::selected_id14(), &body, 512).unwrap();
        assert_eq!(partial.bits_written(), 4_089);
        assert!(!partial.is_complete());

        let complete = write_descriptor(14, Configuration::selected_id14(), &body, 1_024).unwrap();
        assert_eq!(complete.bits_written(), 4_137);
        assert_eq!(complete.rounded_bytes_written(), 518);
        assert!(complete.is_complete());
    }

    #[test]
    fn unsupported_extended_payload_id_is_rejected_before_output_creation() {
        assert!(write_descriptor(31, Configuration::selected_id11(), &[], 64).is_none());
    }

    fn bits_as_text(bytes: &[u8], bit_count: u64) -> String {
        (0..usize::try_from(bit_count).unwrap())
            .map(|index| {
                if (bytes[index / 8] >> (7 - (index % 8))) & 1 == 0 {
                    '0'
                } else {
                    '1'
                }
            })
            .collect()
    }
}
