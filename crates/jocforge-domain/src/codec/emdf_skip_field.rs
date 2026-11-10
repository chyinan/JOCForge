// pattern: Functional Core
//! Serializes the selected Flat7x D0 block-two EMDF skip-field syntax.
//!
//! ETSI TS 102 366 V1.4.1 clauses 4.4.3.58–60 define the presence bit,
//! 9-bit byte length and corresponding skip-field bytes. The Flat7x target
//! supplies the single-field block-two cap and whole-AU overflow policy.

use crate::{
    codec::bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    profile::{Flat7xEmdfOverflowPolicy, Flat7xI0D0TargetPlan},
};

const SKIPL_WIDTH_BITS: u8 = 9;
const SKIPLE_WIDTH_BITS: u8 = 1;
const SKIPL_MAX_BYTES: u16 = (1_u16 << SKIPL_WIDTH_BITS) - 1;
const FLAT7X_D0_BLOCK_ZERO_BASED: u8 = 2;
const SINGLE_SKIP_FIELD_COUNT: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum D0EmdfSkipFieldWriteError {
    InvalidTargetContract,
    MissingContainer,
    ContainerTooLarge,
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for D0EmdfSkipFieldWriteError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes a complete Flat7x D0 block-two skip-field syntax at the caller cursor.
///
/// Flat7x requires a complete container, so this API emits `skiple=1`, its U9
/// `skipl` byte count, then exactly that many dummy bytes. A missing container
/// rejects before any bits are written. The caller positions the cursor after
/// delta-bit allocation and before mantissas; other count and channel domains
/// remain separate.
pub fn write_flat7x_d0_block2_skipfld(
    writer: &mut FrameBitWriter<'_>,
    complete_emdf_container: &[u8],
) -> Result<FrameBitCursor, D0EmdfSkipFieldWriteError> {
    let target = Flat7xI0D0TargetPlan::approved_contract().d0_emdf_skip_field();
    if target.zero_based_audio_block() != FLAT7X_D0_BLOCK_ZERO_BASED
        || target.field_count() != SINGLE_SKIP_FIELD_COUNT
        || target.fragmentation_allowed()
        || target.overflow_policy() != Flat7xEmdfOverflowPolicy::RejectWholeAccessUnitBeforePublish
        || target.maximum_complete_container_bytes() > SKIPL_MAX_BYTES
    {
        return Err(D0EmdfSkipFieldWriteError::InvalidTargetContract);
    }

    // The Flat7x target requires an EMDF container; the generic ETSI
    // skiple=0 absent-field form is outside this target-specific API.
    if complete_emdf_container.is_empty() {
        return Err(D0EmdfSkipFieldWriteError::MissingContainer);
    }
    if complete_emdf_container.len() > usize::from(target.maximum_complete_container_bytes()) {
        return Err(D0EmdfSkipFieldWriteError::ContainerTooLarge);
    }

    let length_bytes = u32::try_from(complete_emdf_container.len())
        .map_err(|_| D0EmdfSkipFieldWriteError::ContainerTooLarge)?;
    let payload_bits = length_bytes
        .checked_mul(8)
        .ok_or(D0EmdfSkipFieldWriteError::ContainerTooLarge)?;
    let field_bits = u32::from(SKIPLE_WIDTH_BITS + SKIPL_WIDTH_BITS)
        .checked_add(payload_bits)
        .ok_or(D0EmdfSkipFieldWriteError::ContainerTooLarge)?;

    writer.preflight_capacity(field_bits)?;
    writer.write_bit(true)?;
    writer.write_unsigned(length_bytes, SKIPL_WIDTH_BITS)?;
    writer.write_bytes(complete_emdf_container)?;
    Ok(writer.cursor())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_bits(bytes: &[u8], start_bit: u32, width: u8) -> u32 {
        (0..u32::from(width)).fold(0, |value, offset| {
            let bit = start_bit + offset;
            let byte = bytes[(bit / 8) as usize];
            (value << 1) | u32::from((byte >> (7 - bit % 8)) & 1)
        })
    }

    fn read_bytes(bytes: &[u8], start_bit: u32, byte_count: usize) -> Vec<u8> {
        (0..byte_count)
            .map(|index| read_bits(bytes, start_bit + u32::try_from(index * 8).unwrap(), 8) as u8)
            .collect()
    }

    #[test]
    fn writes_skiple_then_u9_byte_count_and_container_at_an_unaligned_cursor() {
        let start = 3;
        let container = (0..16).map(|value| 0x80 | value).collect::<Vec<_>>();
        let initial = vec![0xa5; 20];
        let mut bytes = initial.clone();
        let end;
        {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
            end = write_flat7x_d0_block2_skipfld(&mut writer, &container).unwrap();
        }

        assert_eq!(read_bits(&bytes, start, SKIPLE_WIDTH_BITS), 1);
        assert_eq!(
            read_bits(
                &bytes,
                start + u32::from(SKIPLE_WIDTH_BITS),
                SKIPL_WIDTH_BITS
            ),
            16
        );
        assert_eq!(
            read_bytes(
                &bytes,
                start + u32::from(SKIPLE_WIDTH_BITS + SKIPL_WIDTH_BITS),
                16
            ),
            container
        );
        assert_eq!(end.bit_offset(), start + 1 + 9 + 16 * 8);
        for bit in 0..u32::try_from(bytes.len() * 8).unwrap() {
            let in_field = (start..end.bit_offset()).contains(&bit);
            if !in_field {
                assert_eq!(read_bits(&bytes, bit, 1), read_bits(&initial, bit, 1));
            }
        }
    }

    #[test]
    fn accepts_the_one_byte_length_boundary() {
        let container = [0xe1_u8];
        let mut bytes = [0xa5_u8; 3];
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();

        let end = write_flat7x_d0_block2_skipfld(&mut writer, &container).unwrap();

        assert_eq!(read_bits(&bytes, 0, SKIPLE_WIDTH_BITS), 1);
        assert_eq!(
            read_bits(&bytes, u32::from(SKIPLE_WIDTH_BITS), SKIPL_WIDTH_BITS),
            1
        );
        assert_eq!(
            read_bytes(&bytes, u32::from(SKIPLE_WIDTH_BITS + SKIPL_WIDTH_BITS), 1),
            container
        );
        assert_eq!(end.bit_offset(), 18);
    }

    #[test]
    fn accepts_the_511_byte_boundary_and_encodes_its_u9_length() {
        let container = vec![0x5a; usize::from(SKIPL_MAX_BYTES)];
        let start = 1;
        let mut bytes = vec![0xa5; 514];
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();

        let end = write_flat7x_d0_block2_skipfld(&mut writer, &container).unwrap();

        assert_eq!(read_bits(&bytes, start, SKIPLE_WIDTH_BITS), 1);
        assert_eq!(
            read_bits(
                &bytes,
                start + u32::from(SKIPLE_WIDTH_BITS),
                SKIPL_WIDTH_BITS
            ),
            511
        );
        assert_eq!(
            read_bytes(
                &bytes,
                start + u32::from(SKIPLE_WIDTH_BITS + SKIPL_WIDTH_BITS),
                container.len()
            ),
            container
        );
        assert_eq!(end.bit_offset(), start + 1 + 9 + 511 * 8);
    }

    #[test]
    fn rejects_missing_container_and_512_byte_container_before_writing() {
        for container in [Vec::new(), vec![0x5a; 512]] {
            let start = 5;
            let mut bytes = vec![0xa5; 520];
            let before = bytes.clone();
            let result;
            let end;
            {
                let mut writer =
                    FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
                result = write_flat7x_d0_block2_skipfld(&mut writer, &container);
                end = writer.cursor();
            }

            assert_eq!(end.bit_offset(), start);
            assert_eq!(bytes, before);
            assert_eq!(
                result,
                Err(if container.is_empty() {
                    D0EmdfSkipFieldWriteError::MissingContainer
                } else {
                    D0EmdfSkipFieldWriteError::ContainerTooLarge
                })
            );
        }
    }

    #[test]
    fn preflights_the_complete_field_before_mutating_frame_bits() {
        let container = [0x58_u8; 1];
        let start = 3;
        let mut bytes = [0xa5_u8; 2];
        let before = bytes;
        let result;
        let end;
        {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
            result = write_flat7x_d0_block2_skipfld(&mut writer, &container);
            end = writer.cursor();
        }

        assert_eq!(
            result,
            Err(D0EmdfSkipFieldWriteError::BitWriter(
                FrameBitWriterError::BufferTooSmall
            ))
        );
        assert_eq!(end.bit_offset(), start);
        assert_eq!(bytes, before);
    }
}
