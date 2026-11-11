// pattern: Functional Core
//! ETSI AC-3 CRC16 field generation for caller-owned structural frame bytes.

use super::crc16::eac3_crc16;

const AC3_CRC16_POLYNOMIAL: u16 = 0x8005;
pub(crate) const AC3_SYNCWORD: u16 = 0x0b77;
const X_INVERSE_MOD_POLYNOMIAL: u16 = 0xc002;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Ac3Crc16GenerationError {
    SuffixLengthOverflow,
    FrameTooShort,
    FrameBitLengthOverflow,
    Crc2ResidualNonzero { residual: u16 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Ac3Crc1Generation {
    suffix_crc: u16,
    shift_factor: u16,
    value: u16,
}

impl Ac3Crc1Generation {
    #[cfg(test)]
    pub const fn suffix_crc(self) -> u16 {
        self.suffix_crc
    }

    #[cfg(test)]
    pub const fn shift_factor(self) -> u16 {
        self.shift_factor
    }

    pub const fn value(self) -> u16 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Ac3Crc2Generation {
    value: u16,
    residual: u16,
    crcrsv_before: bool,
    crcrsv_after: bool,
    crcrsv_toggled: bool,
    crcrsv_bit_offset: u32,
    field_start_byte: u16,
}

impl Ac3Crc2Generation {
    pub const fn value(self) -> u16 {
        self.value
    }

    pub const fn residual(self) -> u16 {
        self.residual
    }

    pub const fn crcrsv_before(self) -> bool {
        self.crcrsv_before
    }

    pub const fn crcrsv_after(self) -> bool {
        self.crcrsv_after
    }

    pub const fn crcrsv_toggled(self) -> bool {
        self.crcrsv_toggled
    }

    pub const fn crcrsv_bit_offset(self) -> u32 {
        self.crcrsv_bit_offset
    }

    pub const fn field_start_byte(self) -> u16 {
        self.field_start_byte
    }
}

/// Generates the leading AC-3 CRC1 field from the bytes after the CRC1 slot.
///
/// The approved Wash contract defines `CRC1 = CRC(S) * x^-(16+8|S|) mod P`,
/// where `S` is the caller's suffix from byte 4 through the ETSI 5/8 boundary.
pub(crate) fn generate_ac3_crc1_from_suffix(
    suffix: &[u8],
) -> Result<Ac3Crc1Generation, Ac3Crc16GenerationError> {
    let suffix_bytes =
        u32::try_from(suffix.len()).map_err(|_| Ac3Crc16GenerationError::SuffixLengthOverflow)?;
    let exponent = suffix_bytes
        .checked_mul(8)
        .and_then(|bits| bits.checked_add(16))
        .ok_or(Ac3Crc16GenerationError::SuffixLengthOverflow)?;
    let suffix_crc = eac3_crc16(suffix);
    let shift_factor = gf2_power(X_INVERSE_MOD_POLYNOMIAL, exponent);
    let value = gf2_multiply_mod(suffix_crc, shift_factor);
    Ok(Ac3Crc1Generation {
        suffix_crc,
        shift_factor,
        value,
    })
}

/// Generates and residual-checks the trailing AC-3 CRC2 field in place.
///
/// The caller provides a complete synthetic or production-candidate frame with
/// the `crcrsv` bit already set. If the initial CRC2 equals the syncword, this
/// applies the approved rule by toggling `crcrsv` and recomputing CRC2. This
/// function validates CRC geometry only; it does not parse or generate audio.
pub(crate) fn generate_ac3_crc2_for_frame(
    frame: &mut [u8],
) -> Result<Ac3Crc2Generation, Ac3Crc16GenerationError> {
    if frame.len() < 5 {
        return Err(Ac3Crc16GenerationError::FrameTooShort);
    }
    let frame_bytes =
        u32::try_from(frame.len()).map_err(|_| Ac3Crc16GenerationError::FrameBitLengthOverflow)?;
    let frame_bits = frame_bytes
        .checked_mul(8)
        .ok_or(Ac3Crc16GenerationError::FrameBitLengthOverflow)?;
    let crcrsv_bit_offset = frame_bits
        .checked_sub(17)
        .ok_or(Ac3Crc16GenerationError::FrameTooShort)?;
    let field_start = frame
        .len()
        .checked_sub(2)
        .ok_or(Ac3Crc16GenerationError::FrameTooShort)?;
    let field_start_byte =
        u16::try_from(field_start).map_err(|_| Ac3Crc16GenerationError::FrameBitLengthOverflow)?;
    let crcrsv_before = read_frame_bit(frame, crcrsv_bit_offset);
    let crc2_before_adjustment = eac3_crc16(&frame[2..field_start]);
    let crcrsv_toggled = crcrsv_collision_requires_toggle(crc2_before_adjustment);
    let crcrsv_after = if crcrsv_toggled {
        !crcrsv_before
    } else {
        crcrsv_before
    };
    if crcrsv_toggled {
        write_frame_bit(frame, crcrsv_bit_offset, crcrsv_after);
    }

    let value = eac3_crc16(&frame[2..field_start]);
    frame[field_start..].copy_from_slice(&value.to_be_bytes());
    let residual = eac3_crc16(&frame[2..]);
    if residual != 0 {
        return Err(Ac3Crc16GenerationError::Crc2ResidualNonzero { residual });
    }
    Ok(Ac3Crc2Generation {
        value,
        residual,
        crcrsv_before,
        crcrsv_after,
        crcrsv_toggled,
        crcrsv_bit_offset,
        field_start_byte,
    })
}

pub(crate) const fn crcrsv_collision_requires_toggle(crc2_before_adjustment: u16) -> bool {
    crc2_before_adjustment == AC3_SYNCWORD
}

fn gf2_power(mut base: u16, mut exponent: u32) -> u16 {
    let mut result = 1_u16;
    while exponent != 0 {
        if exponent & 1 != 0 {
            result = gf2_multiply_mod(result, base);
        }
        exponent >>= 1;
        if exponent != 0 {
            base = gf2_multiply_mod(base, base);
        }
    }
    result
}

fn gf2_multiply_mod(mut left: u16, mut right: u16) -> u16 {
    let mut product = 0_u16;
    for _ in 0..16 {
        if right & 1 != 0 {
            product ^= left;
        }
        right >>= 1;
        let high_bit = left & 0x8000 != 0;
        left <<= 1;
        if high_bit {
            left ^= AC3_CRC16_POLYNOMIAL;
        }
    }
    product
}

fn read_frame_bit(frame: &[u8], bit_offset: u32) -> bool {
    let byte_index = usize::try_from(bit_offset / 8).unwrap_or(usize::MAX);
    let shift = 7 - bit_offset % 8;
    frame[byte_index] & (1_u8 << shift) != 0
}

fn write_frame_bit(frame: &mut [u8], bit_offset: u32, value: bool) {
    let byte_index = usize::try_from(bit_offset / 8).unwrap_or(usize::MAX);
    let shift = 7 - bit_offset % 8;
    let mask = 1_u8 << shift;
    if value {
        frame[byte_index] |= mask;
    } else {
        frame[byte_index] &= !mask;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        crcrsv_collision_requires_toggle, generate_ac3_crc1_from_suffix,
        generate_ac3_crc2_for_frame,
    };
    use crate::integrity::crc16::eac3_crc16;

    fn washed_pattern_byte(index: usize, seed: u8) -> u8 {
        let value = index * 37 + 11 + usize::from(seed) * 29 + (index >> 3) * 13;
        (value & 0xff) as u8
    }

    fn suffix(seed: u8, start: usize, end: usize) -> Vec<u8> {
        (start..end)
            .map(|index| washed_pattern_byte(index, seed))
            .collect()
    }

    #[test]
    fn crc1_target_geometry_matches_wash_independent_scalar_crosscheck() {
        let suffix = suffix(17, 4, 1_600);
        let generated = generate_ac3_crc1_from_suffix(&suffix).unwrap();
        assert_eq!(generated.suffix_crc(), 0x0ad2);
        assert_eq!(generated.shift_factor(), 0x6e87);
        assert_eq!(generated.value(), 0x6837);
    }

    #[test]
    fn crc1_handles_word_aligned_and_odd_word_truncation_boundaries() {
        let aligned = suffix(7, 4, 80);
        let odd_words = suffix(11, 4, 40);
        assert_eq!(
            generate_ac3_crc1_from_suffix(&aligned).unwrap().value(),
            0x8860
        );
        assert_eq!(
            generate_ac3_crc1_from_suffix(&odd_words).unwrap().value(),
            0x08ad
        );
    }

    #[test]
    fn crc2_generation_writes_big_endian_and_verifies_zero_residual() {
        let mut frame: Vec<_> = (0..128)
            .map(|index| washed_pattern_byte(index, 7))
            .collect();
        frame[..2].copy_from_slice(&0x0b77_u16.to_be_bytes());
        let crcrsv_bit = u32::try_from(frame.len() * 8 - 17).unwrap();
        let generation = generate_ac3_crc2_for_frame(&mut frame).unwrap();
        assert_eq!(generation.residual(), 0);
        assert_eq!(eac3_crc16(&frame[2..]), 0);
        assert_eq!(&frame[frame.len() - 2..], &generation.value().to_be_bytes());
        assert_eq!(generation.crcrsv_bit_offset(), crcrsv_bit);
        assert_eq!(generation.field_start_byte(), 126);
    }

    #[test]
    fn syncword_collision_requires_the_approved_crcrsv_adjustment() {
        assert!(crcrsv_collision_requires_toggle(0x0b77));
        assert!(!crcrsv_collision_requires_toggle(0xade9));
    }
}
