// pattern: Functional Core
//! Boundary for bounded MSB bit input and output. It may depend only on immutable codec contracts.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameBitCursor(u32);

impl FrameBitCursor {
    pub const fn new(bit_offset: u32) -> Self {
        Self(bit_offset)
    }

    pub const fn bit_offset(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameBitWriterError {
    Width,
    Value,
    CursorOverflow,
    BufferTooSmall,
}

/// Bounded high-bit-first writer over caller-owned frame scratch.
///
/// The writer preserves all bits outside the written range. It owns no codec
/// state and does not infer field widths or frame syntax.
pub struct FrameBitWriter<'a> {
    storage: &'a mut [u8],
    cursor: FrameBitCursor,
}

impl<'a> FrameBitWriter<'a> {
    pub fn new(storage: &'a mut [u8], cursor: FrameBitCursor) -> Result<Self, FrameBitWriterError> {
        let storage_bits = storage_bits(storage)?;
        if u64::from(cursor.bit_offset()) > storage_bits {
            return Err(FrameBitWriterError::BufferTooSmall);
        }
        Ok(Self { storage, cursor })
    }

    pub const fn cursor(&self) -> FrameBitCursor {
        self.cursor
    }

    pub fn write_bit(&mut self, value: bool) -> Result<(), FrameBitWriterError> {
        self.write_unsigned(u32::from(value), 1)
    }

    pub fn write_unsigned(&mut self, value: u32, width: u8) -> Result<(), FrameBitWriterError> {
        if width > 32 {
            return Err(FrameBitWriterError::Width);
        }
        if width < 32 && value >= (1_u32 << width) {
            return Err(FrameBitWriterError::Value);
        }
        self.ensure_capacity(u32::from(width))?;
        for index in 0..u32::from(width) {
            let shift = u32::from(width) - index - 1;
            self.write_one((value >> shift) & 1 != 0)?;
        }
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), FrameBitWriterError> {
        let byte_count =
            u32::try_from(bytes.len()).map_err(|_| FrameBitWriterError::CursorOverflow)?;
        let bit_count = byte_count
            .checked_mul(8)
            .ok_or(FrameBitWriterError::CursorOverflow)?;
        self.ensure_capacity(bit_count)?;
        for byte in bytes {
            self.write_unsigned(u32::from(*byte), 8)?;
        }
        Ok(())
    }

    pub fn skip_zeroes(&mut self, bit_count: u32) -> Result<(), FrameBitWriterError> {
        self.ensure_capacity(bit_count)?;
        for _ in 0..bit_count {
            self.write_one(false)?;
        }
        Ok(())
    }

    fn ensure_capacity(&self, bit_count: u32) -> Result<(), FrameBitWriterError> {
        let end = u64::from(self.cursor.bit_offset())
            .checked_add(u64::from(bit_count))
            .ok_or(FrameBitWriterError::CursorOverflow)?;
        if end > storage_bits(self.storage)? {
            return Err(FrameBitWriterError::BufferTooSmall);
        }
        Ok(())
    }

    fn write_one(&mut self, value: bool) -> Result<(), FrameBitWriterError> {
        let bit_offset = self.cursor.bit_offset();
        let byte_index =
            usize::try_from(bit_offset / 8).map_err(|_| FrameBitWriterError::CursorOverflow)?;
        let shift = 7 - (bit_offset % 8);
        let mask = 1_u8 << shift;
        let destination = self
            .storage
            .get_mut(byte_index)
            .ok_or(FrameBitWriterError::BufferTooSmall)?;
        if value {
            *destination |= mask;
        } else {
            *destination &= !mask;
        }
        self.cursor = FrameBitCursor(
            bit_offset
                .checked_add(1)
                .ok_or(FrameBitWriterError::CursorOverflow)?,
        );
        Ok(())
    }
}

fn storage_bits(storage: &[u8]) -> Result<u64, FrameBitWriterError> {
    u64::try_from(storage.len())
        .map_err(|_| FrameBitWriterError::CursorOverflow)?
        .checked_mul(8)
        .ok_or(FrameBitWriterError::CursorOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const BOUNDARY: &[u8] = include_bytes!("../../tests/fixtures/frame_bit_writer/BOUNDARY.json");

    fn decode_hex(value: &str) -> Vec<u8> {
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let digit = |byte| match byte {
                    b'0'..=b'9' => byte - b'0',
                    b'a'..=b'f' => byte - b'a' + 10,
                    _ => panic!("invalid test hex"),
                };
                (digit(pair[0]) << 4) | digit(pair[1])
            })
            .collect()
    }

    #[test]
    fn unaligned_msb_fields_preserve_surrounding_bits() {
        let document: Value = serde_json::from_slice(BOUNDARY).unwrap();
        let case = &document["cases"][0];
        let mut storage = decode_hex(case["initial_hex"].as_str().unwrap());
        let start = u32::try_from(case["start_bit"].as_u64().unwrap()).unwrap();
        let mut writer = FrameBitWriter::new(&mut storage, FrameBitCursor::new(start)).unwrap();
        writer
            .write_unsigned(
                u32::try_from(case["field_value"].as_u64().unwrap()).unwrap(),
                u8::try_from(case["field_width"].as_u64().unwrap()).unwrap(),
            )
            .unwrap();
        writer
            .write_bytes(&decode_hex(case["bytes_hex"].as_str().unwrap()))
            .unwrap();
        assert_eq!(writer.cursor().bit_offset(), case["end_bit"]);
        assert_eq!(storage, decode_hex(case["expected_hex"].as_str().unwrap()));
    }

    #[test]
    fn invalid_width_value_and_capacity_do_not_advance() {
        let mut storage = [0_u8; 1];
        let mut writer = FrameBitWriter::new(&mut storage, FrameBitCursor::new(7)).unwrap();
        assert_eq!(writer.write_unsigned(2, 1), Err(FrameBitWriterError::Value));
        assert_eq!(writer.cursor().bit_offset(), 7);
        assert_eq!(
            writer.write_unsigned(0, 33),
            Err(FrameBitWriterError::Width)
        );
        assert_eq!(writer.cursor().bit_offset(), 7);
        assert_eq!(
            writer.write_unsigned(0b11, 2),
            Err(FrameBitWriterError::BufferTooSmall)
        );
        assert_eq!(writer.cursor().bit_offset(), 7);
    }
}
