// pattern: Functional Core
//! Stateless writer for the washed three-byte internal DRC control buffer.
//!
//! This value is not joined to the command-2 queue, selected BSI metadata, or an AU bit offset.

const INTERNAL_CONTROL_BUFFER_BYTES: usize = 3;
const PREFIX_RECORD_BYTES: usize = 9;
const DRC_CODE_WIDTH: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrcControlStartBit(u8);

impl DrcControlStartBit {
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrcControlBuffer([u8; INTERNAL_CONTROL_BUFFER_BYTES]);

impl DrcControlBuffer {
    pub const fn as_bytes(&self) -> &[u8; INTERNAL_CONTROL_BUFFER_BYTES] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrcControlBufferError {
    RecordPrefixTooShort,
    CodeOutOfRange,
    FieldsOutsideBuffer,
}

/// Computes the relative start bit after the washed optional-field prefix.
pub fn drc_control_field_start_bit(
    record: &[u8],
) -> Result<DrcControlStartBit, DrcControlBufferError> {
    if record.len() < PREFIX_RECORD_BYTES {
        return Err(DrcControlBufferError::RecordPrefixTooShort);
    }
    let mut position = 2_u8;
    if record[0] == 3 {
        position += 4;
    }
    position += 1;
    if record[2] != 0 {
        position += 16;
    }
    position += 1;
    if record[4] == 0 {
        position += 1;
        if record[5] != 0 {
            position += 4;
        }
    } else {
        position += 4;
    }
    position += 1;
    if record[8] != 0 {
        position += 5;
    }
    Ok(DrcControlStartBit(position))
}

/// Encodes presence, Line DRC, and RF DRC at their relative positions in the internal buffer.
///
/// Bit offset zero is the most-significant bit of byte zero. Input code values are explicit
/// three-bit values; profile-to-code mapping is outside this function's scope.
pub fn encode_drc_control_buffer(
    record: &[u8],
    present: bool,
    line_code: u8,
    rf_code: u8,
) -> Result<DrcControlBuffer, DrcControlBufferError> {
    if line_code >= 1 << DRC_CODE_WIDTH || rf_code >= 1 << DRC_CODE_WIDTH {
        return Err(DrcControlBufferError::CodeOutOfRange);
    }
    let start_bit = drc_control_field_start_bit(record)?.value();
    let final_bit = start_bit
        .checked_add(6)
        .ok_or(DrcControlBufferError::FieldsOutsideBuffer)?;
    let buffer_bits = u8::try_from(INTERNAL_CONTROL_BUFFER_BYTES * 8)
        .map_err(|_| DrcControlBufferError::FieldsOutsideBuffer)?;
    if final_bit >= buffer_bits {
        return Err(DrcControlBufferError::FieldsOutsideBuffer);
    }

    let mut output = [0_u8; INTERNAL_CONTROL_BUFFER_BYTES];
    write_bit(&mut output, start_bit, present);
    write_code(&mut output, start_bit + 1, line_code);
    write_code(&mut output, start_bit + 4, rf_code);
    Ok(DrcControlBuffer(output))
}

fn write_code(output: &mut [u8; INTERNAL_CONTROL_BUFFER_BYTES], start_bit: u8, value: u8) {
    for bit_index in 0_u8..DRC_CODE_WIDTH {
        let shift = DRC_CODE_WIDTH - bit_index - 1;
        write_bit(output, start_bit + bit_index, value & (1 << shift) != 0);
    }
}

fn write_bit(output: &mut [u8; INTERNAL_CONTROL_BUFFER_BYTES], bit_offset: u8, value: bool) {
    if value {
        let byte_index = usize::from(bit_offset / 8);
        let bit_in_byte = 7 - bit_offset % 8;
        output[byte_index] |= 1 << bit_in_byte;
    }
}
