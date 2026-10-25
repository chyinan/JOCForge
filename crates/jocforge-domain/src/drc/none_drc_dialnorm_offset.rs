// pattern: Functional Core
//! Selected F06ND dialnorm-to-offset scalar mapping.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoneDrcDialnormOffsetError {
    UnsupportedProfile,
}

impl NoneDrcDialnormOffsetError {
    pub const fn code(self) -> &'static str {
        "UNSUPPORTED_PROFILE"
    }
}

impl std::fmt::Display for NoneDrcDialnormOffsetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for NoneDrcDialnormOffsetError {}

/// Returns the binary32 word for the selected `R32((31-d)/144)` mapping.
///
/// F06ND selects only integer dialnorm values in `1..=31`; values outside
/// that domain are rejected as `UNSUPPORTED_PROFILE`.
pub fn none_drc_dialnorm_offset_word(dialnorm: i32) -> Result<u32, NoneDrcDialnormOffsetError> {
    if !(1..=31).contains(&dialnorm) {
        return Err(NoneDrcDialnormOffsetError::UnsupportedProfile);
    }

    let numerator = (31 - dialnorm) as f32;
    Ok((numerator / 144.0_f32).to_bits())
}
