// pattern: Functional Core
//! Exponent-difference packing and caller-supplied E-AC-3 U4/U7 row writing.

use super::bitio::{FrameBitWriter, FrameBitWriterError};

const MAX_GROUP_DIGITS: usize = 3;
const MAX_EXPONENT_DIFFERENCE: i8 = 2;
const MIN_EXPONENT_DIFFERENCE: i8 = -2;
const DIGIT_BASE: u16 = 5;

/// Returns the selected fullband exponent span for an active wire strategy.
pub(crate) const fn span_for_fullband_strategy(strategy: u8) -> Option<u8> {
    match strategy {
        1 => Some(1),
        2 => Some(2),
        3 => Some(4),
        _ => None,
    }
}

/// Returns the number of U7 words needed to pack the supplied difference count.
pub(crate) const fn packed_exponent_group_count(difference_count: usize) -> usize {
    difference_count.div_ceil(MAX_GROUP_DIGITS)
}

/// Error returned when a difference group is empty, too long, or out of range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExponentPackingError {
    ExponentInputError,
}

impl ExponentPackingError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ExponentInputError => "EXPONENT_INPUT_ERROR",
        }
    }
}

/// One F10.5 three-digit exponent-difference group encoded as a seven-bit value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackedExponentGroup(u8);

impl PackedExponentGroup {
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// One four-bit absolute exponent value supplied to an E-AC-3 row writer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eac3ExponentSeed(u8);

impl Eac3ExponentSeed {
    pub const fn new(value: u8) -> Option<Self> {
        if value <= 15 { Some(Self(value)) } else { None }
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

/// Cursor receipt for one E-AC-3 U4/U7 exponent row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eac3ExponentRowReceipt {
    start_bit: u32,
    end_bit: u32,
    group_count: u16,
}

impl Eac3ExponentRowReceipt {
    pub const fn start_bit(self) -> u32 {
        self.start_bit
    }

    pub const fn end_bit(self) -> u32 {
        self.end_bit
    }

    pub const fn group_count(self) -> u16 {
        self.group_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Eac3ExponentRowWriteError {
    SeedOutOfRange(u8),
    GroupCountOverflow,
    BitCountOverflow,
    Writer(FrameBitWriterError),
}

impl From<FrameBitWriterError> for Eac3ExponentRowWriteError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::Writer(error)
    }
}

/// Writes a caller-owned E-AC-3 absolute U4 seed and its U7 exponent groups.
///
/// The caller supplies the row strategy and group count. This helper writes no strategy field,
/// derives no exponent seed, and selects no group count. It preflights the full `4 + 7*N` extent
/// before changing the caller's cursor or storage.
pub fn write_eac3_exponent_row(
    writer: &mut FrameBitWriter<'_>,
    seed: u8,
    groups: &[PackedExponentGroup],
) -> Result<Eac3ExponentRowReceipt, Eac3ExponentRowWriteError> {
    let seed =
        Eac3ExponentSeed::new(seed).ok_or(Eac3ExponentRowWriteError::SeedOutOfRange(seed))?;
    let group_count =
        u16::try_from(groups.len()).map_err(|_| Eac3ExponentRowWriteError::GroupCountOverflow)?;
    let group_bits = u32::from(group_count)
        .checked_mul(7)
        .ok_or(Eac3ExponentRowWriteError::BitCountOverflow)?;
    let total_bits = 4_u32
        .checked_add(group_bits)
        .ok_or(Eac3ExponentRowWriteError::BitCountOverflow)?;
    let start_bit = writer.cursor().bit_offset();
    let end_bit = start_bit
        .checked_add(total_bits)
        .ok_or(Eac3ExponentRowWriteError::BitCountOverflow)?;
    writer.preflight_capacity(total_bits)?;

    writer.write_unsigned(u32::from(seed.value()), 4)?;
    for group in groups {
        writer.write_unsigned(u32::from(group.value()), 7)?;
    }

    Ok(Eac3ExponentRowReceipt {
        start_bit,
        end_bit,
        group_count,
    })
}

/// Packs one to three final exponent differences using F10.5.
///
/// Each difference must be in -2..=2. Missing tail digits use difference zero,
/// which maps to digit two. This helper owns no row, frame, or bitstream state.
pub fn pack_exponent_difference_group(
    differences: &[i8],
) -> Result<PackedExponentGroup, ExponentPackingError> {
    if differences.is_empty()
        || differences.len() > MAX_GROUP_DIGITS
        || differences.iter().any(|difference| {
            !(MIN_EXPONENT_DIFFERENCE..=MAX_EXPONENT_DIFFERENCE).contains(difference)
        })
    {
        return Err(ExponentPackingError::ExponentInputError);
    }

    let mut digits = [2_u16; MAX_GROUP_DIGITS];
    for (digit, difference) in digits.iter_mut().zip(differences) {
        *digit = u16::try_from(i16::from(*difference) + 2)
            .map_err(|_| ExponentPackingError::ExponentInputError)?;
    }

    let packed = DIGIT_BASE * DIGIT_BASE * digits[0] + DIGIT_BASE * digits[1] + digits[2];
    let encoded = u8::try_from(packed).map_err(|_| ExponentPackingError::ExponentInputError)?;
    Ok(PackedExponentGroup(encoded))
}

#[cfg(test)]
mod tests {
    use super::{
        Eac3ExponentRowWriteError, pack_exponent_difference_group, write_eac3_exponent_row,
    };
    use crate::codec::bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError};

    #[test]
    fn eac3_exponent_row_writes_seed_then_u7_groups_msb_first() {
        let groups = [
            pack_exponent_difference_group(&[-2, -2, -1]).unwrap(),
            pack_exponent_difference_group(&[2, 2, 2]).unwrap(),
        ];
        let mut storage = [0_u8; 3];
        let receipt = {
            let mut writer = FrameBitWriter::new(&mut storage, FrameBitCursor::new(0)).unwrap();
            let receipt = write_eac3_exponent_row(&mut writer, 5, &groups).unwrap();
            assert_eq!(writer.cursor().bit_offset(), 18);
            receipt
        };

        assert_eq!(storage, [0x50, 0x3f, 0x00]);
        assert_eq!(receipt.start_bit(), 0);
        assert_eq!(receipt.end_bit(), 18);
        assert_eq!(receipt.group_count(), 2);
    }

    #[test]
    fn eac3_exponent_row_preflights_seed_and_extent_before_mutation() {
        let group = pack_exponent_difference_group(&[0]).unwrap();
        let mut storage = [0xa5_u8; 2];
        let before = storage;
        let invalid_seed_error = {
            let mut writer = FrameBitWriter::new(&mut storage, FrameBitCursor::new(0)).unwrap();
            let error = write_eac3_exponent_row(&mut writer, 16, &[group]);
            assert_eq!(writer.cursor().bit_offset(), 0);
            error
        };
        assert_eq!(
            invalid_seed_error,
            Err(Eac3ExponentRowWriteError::SeedOutOfRange(16))
        );
        assert_eq!(storage, before);

        let capacity_error = {
            let mut writer = FrameBitWriter::new(&mut storage, FrameBitCursor::new(0)).unwrap();
            let error = write_eac3_exponent_row(&mut writer, 1, &[group, group]);
            assert_eq!(writer.cursor().bit_offset(), 0);
            error
        };
        assert_eq!(
            capacity_error,
            Err(Eac3ExponentRowWriteError::Writer(
                FrameBitWriterError::BufferTooSmall
            ))
        );
        assert_eq!(storage, before);
    }
}
