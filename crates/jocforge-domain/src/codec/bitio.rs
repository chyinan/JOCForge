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

    /// Checks that a complete compound field group fits without changing bytes or cursor.
    pub fn preflight_capacity(&self, bit_count: u32) -> Result<(), FrameBitWriterError> {
        self.ensure_capacity(bit_count)
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
        let start = self.cursor.0;
        let end = start
            .checked_add(u32::from(width))
            .ok_or(FrameBitWriterError::CursorOverflow)?;
        let mut at = start;
        while at < end {
            let free = 8 - (at % 8);
            let take = free.min(end - at);
            let shift = free - take;
            let mask = ((1_u32 << take) - 1) << shift;
            let bits = ((value >> (end - at - take)) << shift) & mask;
            let byte = &mut self.storage[(at / 8) as usize];
            *byte = (*byte & !mask.to_le_bytes()[0]) | bits.to_le_bytes()[0];
            at += take;
        }
        self.cursor = FrameBitCursor(end);
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), FrameBitWriterError> {
        let byte_count =
            u32::try_from(bytes.len()).map_err(|_| FrameBitWriterError::CursorOverflow)?;
        let bit_count = byte_count
            .checked_mul(8)
            .ok_or(FrameBitWriterError::CursorOverflow)?;
        self.ensure_capacity(bit_count)?;
        let end = self
            .cursor
            .0
            .checked_add(bit_count)
            .ok_or(FrameBitWriterError::CursorOverflow)?;
        if bytes.is_empty() {
            return Ok(());
        }
        let start = (self.cursor.0 / 8) as usize;
        let offset = self.cursor.0 % 8;
        if offset == 0 {
            self.storage[start..start + bytes.len()].copy_from_slice(bytes);
        } else {
            let carry_shift = 8 - offset;
            let destination = &mut self.storage[start..=start + bytes.len()];
            let untouched = u8::MAX << carry_shift;
            destination[0] = (destination[0] & untouched) | (bytes[0] >> offset);
            for (dst, pair) in destination[1..bytes.len()].iter_mut().zip(bytes.windows(2)) {
                *dst = (pair[0] << carry_shift) | (pair[1] >> offset);
            }
            destination[bytes.len()] =
                (bytes[bytes.len() - 1] << carry_shift) | (destination[bytes.len()] & !untouched);
        }
        self.cursor = FrameBitCursor(end);
        Ok(())
    }

    /// Copies a packed MSB-first field, preserving bits outside the destination span.
    pub fn write_bit_slice(
        &mut self,
        source: &[u8],
        start_bit: u32,
        length: u32,
    ) -> Result<(), FrameBitWriterError> {
        let end = start_bit
            .checked_add(length)
            .ok_or(FrameBitWriterError::CursorOverflow)?;
        if end.div_ceil(8) as usize > source.len() {
            return Err(FrameBitWriterError::BufferTooSmall);
        }
        self.ensure_capacity(length)?;
        let offset = start_bit % 8;
        let mut first = (start_bit / 8) as usize;
        let mut remaining = length;
        if offset != 0 && remaining != 0 {
            let take = remaining.min(8 - offset);
            let value = (source[first] >> (8 - offset - take)) & (u8::MAX >> (8 - take));
            self.write_unsigned(u32::from(value), take as u8)?;
            remaining -= take;
            first += 1;
        }
        let whole = (remaining / 8) as usize;
        self.write_bytes(&source[first..first + whole])?;
        let tail = (remaining % 8) as u8;
        if tail != 0 {
            self.write_unsigned(u32::from(source[first + whole] >> (8 - tail)), tail)?;
        }
        Ok(())
    }

    pub fn skip_zeroes(&mut self, bit_count: u32) -> Result<(), FrameBitWriterError> {
        self.ensure_capacity(bit_count)?;
        let end = self
            .cursor
            .0
            .checked_add(bit_count)
            .ok_or(FrameBitWriterError::CursorOverflow)?;
        let head = (8 - self.cursor.0 % 8) % 8;
        self.write_unsigned(0, head.min(bit_count) as u8)?;
        let whole = (end - self.cursor.0) / 8;
        let start = (self.cursor.0 / 8) as usize;
        self.storage[start..start + whole as usize].fill(0);
        self.cursor.0 += whole * 8;
        self.write_unsigned(0, (end - self.cursor.0) as u8)?;
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

    #[test]
    fn bit_slices_copy_all_source_and_destination_alignments() {
        let source = [0x29, 0xeb, 0x45, 0x83, 0xfc];
        for start in 0..8 {
            for dest in 0..8 {
                for length in 0..=32 {
                    let mut actual = [0x96; 5];
                    let mut expected = actual;
                    let mut reference =
                        FrameBitWriter::new(&mut expected, FrameBitCursor::new(dest)).unwrap();
                    for bit in start..start + length {
                        reference
                            .write_bit(source[(bit / 8) as usize] & (0x80 >> (bit % 8)) != 0)
                            .unwrap();
                    }
                    let mut writer =
                        FrameBitWriter::new(&mut actual, FrameBitCursor::new(dest)).unwrap();
                    writer.write_bit_slice(&source, start, length).unwrap();
                    assert_eq!(writer.cursor().bit_offset(), dest + length);
                    assert_eq!(actual, expected);
                }
            }
        }
    }

    #[test]
    fn byte_chunks_preserve_surrounding_bits_for_every_alignment_and_width() {
        for offset in 0..8_u32 {
            for width in 0..=32_u8 {
                let mask = u32::MAX.checked_shr(32 - u32::from(width)).unwrap_or(0);
                for value in [0, mask, 0xa45b_92de & mask] {
                    let mut actual = [0xa5; 12];
                    let mut expected = actual;
                    let mut writer =
                        FrameBitWriter::new(&mut actual, FrameBitCursor::new(offset)).unwrap();
                    writer.write_unsigned(value, width).unwrap();
                    writer.write_bytes(&[0x38, 0xa9]).unwrap();
                    writer.skip_zeroes(21).unwrap();
                    let mut position = offset;
                    for (bits, n) in [(value, width), (0x38a9, 16), (0, 21)] {
                        for shift in (0..n).rev() {
                            let bit = 1 << (7 - position % 8);
                            let byte = &mut expected[(position / 8) as usize];
                            if bits >> shift & 1 != 0 {
                                *byte |= bit;
                            } else {
                                *byte &= !bit;
                            }
                            position += 1;
                        }
                    }
                    assert_eq!(writer.cursor().bit_offset(), position);
                    assert_eq!(actual, expected);
                }
            }
        }
    }
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
