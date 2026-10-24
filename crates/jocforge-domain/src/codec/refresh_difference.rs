// pattern: Functional Core
//! F09E D06-E strict index-to-I16 refresh-difference map.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefreshDifferenceError {
    IndexOutOfRange,
}

impl RefreshDifferenceError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::IndexOutOfRange => "EXPONENT_INPUT_ERROR",
        }
    }
}

/// Maps one D06-E refresh-difference index to its selected I16 value.
///
/// This is a stateless integer map only; it does not perform exponent refresh,
/// coefficient-plane processing, BAP allocation, or policy-provider loading.
pub fn map_refresh_difference(index: i16) -> Result<i16, RefreshDifferenceError> {
    match index {
        0..=1 => Ok(0),
        2 => Ok(1),
        3..=31 => Ok(2 * index - 4),
        _ => Err(RefreshDifferenceError::IndexOutOfRange),
    }
}
