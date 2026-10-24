// pattern: Functional Core
//! Stateless raw exponent mapping from a caller-supplied POWER bit word.

/// Maps one U32 POWER bit word, bias, and cap to a signed raw exponent.
///
/// The word is consumed as integer bits and is never converted to floating point.
/// This helper does not produce POWER, select a route, or produce bias or cap.
/// Runtime vectors cover only the listed POWER words, biases, and cap values.
pub fn raw_exponent_from_power_bits(power_word: u32, bias: i16, cap: i16) -> i16 {
    let doubled_word = power_word.wrapping_mul(2);
    let source_exponent = if doubled_word == 0 {
        -127
    } else {
        i16::from(doubled_word.to_be_bytes()[0]) - 126
    };
    let difference = (-source_exponent).clamp(0, 60);
    let biased = bias.wrapping_add(difference);
    let capped = if biased > cap { cap } else { biased };
    capped >> 1
}
