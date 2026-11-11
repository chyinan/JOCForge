// pattern: Functional Core
//! Bounded Flat7x I0 AC-3 BSI-tail serialization from caller-owned values.

use super::bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError};

pub(crate) const I0_BSI_TAIL_MINIMUM_BITS: u32 = 7;

const I0_AC3_HEADER_END_WITHOUT_COMPR: u32 = 62;
const I0_AC3_HEADER_END_WITH_COMPR: u32 = 70;
const AUDIO_PRODUCTION_MIX_LEVEL_WIDTH: u8 = 5;
const AUDIO_PRODUCTION_ROOM_TYPE_WIDTH: u8 = 2;
const TIMECODE_WIDTH: u8 = 14;
const ADDBSI_LENGTH_WIDTH: u8 = 6;
const MAX_ADDBSI_BYTES: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0Ac3AudioProductionInfo {
    pub mix_level: u8,
    pub room_type: u8,
}

/// Caller-owned values for the I0 BSI tail beginning after optional `compr`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0Ac3BsiTailInput<'a> {
    pub langcode: Option<u8>,
    pub audio_production: Option<Flat7xI0Ac3AudioProductionInfo>,
    pub copyright: bool,
    pub original_bitstream: bool,
    pub time_code_1: Option<u16>,
    pub time_code_2: Option<u16>,
    pub additional_bsi: Option<&'a [u8]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum I0Ac3TimeCodeField {
    First,
    Second,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xI0Ac3BsiTailError {
    HeaderEndCursorMismatch {
        actual: u32,
    },
    MixLevelOutOfRange {
        value: u8,
    },
    RoomTypeOutOfRange {
        value: u8,
    },
    TimeCodeOutOfRange {
        field: I0Ac3TimeCodeField,
        value: u16,
    },
    AdditionalBsiLengthOutOfRange {
        actual_bytes: usize,
    },
    ExtentOverflow,
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for Flat7xI0Ac3BsiTailError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Exact cursor extent emitted by one I0 BSI-tail serialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0Ac3BsiTailReceipt {
    start: FrameBitCursor,
    end: FrameBitCursor,
    additional_bsi_start: Option<FrameBitCursor>,
    additional_bsi_bit_length: Option<u32>,
    additional_bsi_byte_length: Option<u8>,
}

impl Flat7xI0Ac3BsiTailReceipt {
    pub const fn start_bit(self) -> u32 {
        self.start.bit_offset()
    }

    pub const fn end_bit(self) -> u32 {
        self.end.bit_offset()
    }

    pub const fn bit_length(self) -> u32 {
        self.end.bit_offset() - self.start.bit_offset()
    }

    pub const fn additional_bsi_start_bit(self) -> Option<u32> {
        match self.additional_bsi_start {
            Some(cursor) => Some(cursor.bit_offset()),
            None => None,
        }
    }

    pub const fn additional_bsi_bit_length(self) -> Option<u32> {
        self.additional_bsi_bit_length
    }

    pub const fn additional_bsi_byte_length(self) -> Option<u8> {
        self.additional_bsi_byte_length
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

/// Writes the caller-authored BSI tail after the approved I0 header segment.
///
/// The tail starts at bit 62 or 70, depending on whether the header included
/// `compr`. This fixed acmod=7 API omits the acmod=0-only dual-mono fields.
/// It validates every optional value and the complete bounded extent before
/// writing any field, then returns the exact emitted range.
pub fn write_flat7x_i0_ac3_bsi_tail(
    writer: &mut FrameBitWriter<'_>,
    input: Flat7xI0Ac3BsiTailInput<'_>,
) -> Result<Flat7xI0Ac3BsiTailReceipt, Flat7xI0Ac3BsiTailError> {
    let start = writer.cursor();
    if !matches!(
        start.bit_offset(),
        I0_AC3_HEADER_END_WITHOUT_COMPR | I0_AC3_HEADER_END_WITH_COMPR
    ) {
        return Err(Flat7xI0Ac3BsiTailError::HeaderEndCursorMismatch {
            actual: start.bit_offset(),
        });
    }

    validate_input(&input)?;
    let additional_bsi_byte_length = input
        .additional_bsi
        .map(|payload| {
            u8::try_from(payload.len()).map_err(|_| Flat7xI0Ac3BsiTailError::ExtentOverflow)
        })
        .transpose()?;
    let additional_bsi_bit_length = additional_bsi_byte_length
        .map(u32::from)
        .map(|length| {
            length
                .checked_mul(8)
                .ok_or(Flat7xI0Ac3BsiTailError::ExtentOverflow)
        })
        .transpose()?;
    let additional_bsi_payload_bits = additional_bsi_bit_length.unwrap_or(0);
    let optional_payload_bits = u32::from(input.langcode.is_some())
        .checked_mul(8)
        .and_then(|bits| {
            bits.checked_add(if input.audio_production.is_some() {
                u32::from(AUDIO_PRODUCTION_MIX_LEVEL_WIDTH + AUDIO_PRODUCTION_ROOM_TYPE_WIDTH)
            } else {
                0
            })
        })
        .and_then(|bits| {
            bits.checked_add(if input.time_code_1.is_some() {
                u32::from(TIMECODE_WIDTH)
            } else {
                0
            })
        })
        .and_then(|bits| {
            bits.checked_add(if input.time_code_2.is_some() {
                u32::from(TIMECODE_WIDTH)
            } else {
                0
            })
        })
        .and_then(|bits| {
            bits.checked_add(if input.additional_bsi.is_some() {
                u32::from(ADDBSI_LENGTH_WIDTH)
            } else {
                0
            })
        })
        .and_then(|bits| bits.checked_add(additional_bsi_payload_bits))
        .ok_or(Flat7xI0Ac3BsiTailError::ExtentOverflow)?;
    let bit_length = I0_BSI_TAIL_MINIMUM_BITS
        .checked_add(optional_payload_bits)
        .ok_or(Flat7xI0Ac3BsiTailError::ExtentOverflow)?;
    writer.preflight_capacity(bit_length)?;

    writer.write_bit(input.langcode.is_some())?;
    if let Some(langcode) = input.langcode {
        writer.write_unsigned(u32::from(langcode), 8)?;
    }

    writer.write_bit(input.audio_production.is_some())?;
    if let Some(audio_production) = input.audio_production {
        writer.write_unsigned(
            u32::from(audio_production.mix_level),
            AUDIO_PRODUCTION_MIX_LEVEL_WIDTH,
        )?;
        writer.write_unsigned(
            u32::from(audio_production.room_type),
            AUDIO_PRODUCTION_ROOM_TYPE_WIDTH,
        )?;
    }

    writer.write_bit(input.copyright)?;
    writer.write_bit(input.original_bitstream)?;
    writer.write_bit(input.time_code_1.is_some())?;
    if let Some(time_code) = input.time_code_1 {
        writer.write_unsigned(u32::from(time_code), TIMECODE_WIDTH)?;
    }
    writer.write_bit(input.time_code_2.is_some())?;
    if let Some(time_code) = input.time_code_2 {
        writer.write_unsigned(u32::from(time_code), TIMECODE_WIDTH)?;
    }
    writer.write_bit(input.additional_bsi.is_some())?;
    let additional_bsi_start = input.additional_bsi.map(|_| writer.cursor());
    if let (Some(payload), Some(byte_length)) = (input.additional_bsi, additional_bsi_byte_length) {
        writer.write_unsigned(u32::from(byte_length - 1), ADDBSI_LENGTH_WIDTH)?;
        let payload_start = writer.cursor();
        writer.write_bytes(payload)?;
        return Ok(Flat7xI0Ac3BsiTailReceipt {
            start,
            end: writer.cursor(),
            additional_bsi_start: Some(payload_start),
            additional_bsi_bit_length,
            additional_bsi_byte_length,
        });
    }

    Ok(Flat7xI0Ac3BsiTailReceipt {
        start,
        end: writer.cursor(),
        additional_bsi_start,
        additional_bsi_bit_length,
        additional_bsi_byte_length,
    })
}

fn validate_input(input: &Flat7xI0Ac3BsiTailInput<'_>) -> Result<(), Flat7xI0Ac3BsiTailError> {
    if let Some(audio_production) = input.audio_production {
        if audio_production.mix_level >= (1 << AUDIO_PRODUCTION_MIX_LEVEL_WIDTH) {
            return Err(Flat7xI0Ac3BsiTailError::MixLevelOutOfRange {
                value: audio_production.mix_level,
            });
        }
        if audio_production.room_type >= (1 << AUDIO_PRODUCTION_ROOM_TYPE_WIDTH) {
            return Err(Flat7xI0Ac3BsiTailError::RoomTypeOutOfRange {
                value: audio_production.room_type,
            });
        }
    }
    for (field, value) in [
        (I0Ac3TimeCodeField::First, input.time_code_1),
        (I0Ac3TimeCodeField::Second, input.time_code_2),
    ] {
        if let Some(value) = value {
            if value >= (1 << TIMECODE_WIDTH) {
                return Err(Flat7xI0Ac3BsiTailError::TimeCodeOutOfRange { field, value });
            }
        }
    }
    if let Some(payload) = input.additional_bsi {
        if !(1..=MAX_ADDBSI_BYTES).contains(&payload.len()) {
            return Err(Flat7xI0Ac3BsiTailError::AdditionalBsiLengthOutOfRange {
                actual_bytes: payload.len(),
            });
        }
    }
    Ok(())
}
