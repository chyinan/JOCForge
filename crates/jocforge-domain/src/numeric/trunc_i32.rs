// pattern: Functional Core
//! N00 truncation toward zero from validated finite binary32 to typed I32.

use super::FiniteF32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TruncI32Error {
    OutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TruncatedI32(i32);

impl TruncatedI32 {
    pub const fn value(self) -> i32 {
        self.0
    }
}

/// Truncates toward zero and rejects values outside I32.
///
/// ```compile_fail
/// use jocforge_domain::numeric::trunc_f32_to_i32;
/// trunc_f32_to_i32(1.5_f32);
/// ```
pub fn trunc_f32_to_i32(value: FiniteF32) -> Result<TruncatedI32, TruncI32Error> {
    let truncated = value.value().trunc();
    let exact = f64::from(truncated);
    if exact < f64::from(i32::MIN) || exact > f64::from(i32::MAX) {
        return Err(TruncI32Error::OutOfRange);
    }
    Ok(TruncatedI32(truncated as i32))
}
