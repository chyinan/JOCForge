// pattern: Functional Core
//! Selected F11.2 virtual-header fixed bit cost for acmod 7 and one programme.

/// Returns the fixed virtual-header cost for the selected surround EX presence.
///
/// The delivered formula is 91 bits plus 14 bits when the surround EX mode is
/// nonzero. Profile/header selection remains the caller's responsibility.
pub const fn virtual_header_fixed_bit_cost(surround_ex_mode: u8) -> u16 {
    if surround_ex_mode > 0 { 105 } else { 91 }
}
