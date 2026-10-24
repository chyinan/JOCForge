// pattern: Functional Core
//! Stateless F10.5 packing for one group of up to three exponent differences.

const MAX_GROUP_DIGITS: usize = 3;
const MAX_EXPONENT_DIFFERENCE: i8 = 2;
const MIN_EXPONENT_DIFFERENCE: i8 = -2;
const DIGIT_BASE: u16 = 5;

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
