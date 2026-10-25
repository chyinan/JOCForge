// pattern: Functional Core
//! Selected F06ND.6 RF and line packed-control quantizer.

use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const QUANTIZER_SCALE: f32 = 32_768.0;
const FRACTION_SCALE: f32 = f32::from_bits(0x3800_0000); // 2^-15
const Q0: f32 = f32::from_bits(0x3E2F_7469);
const Q1: f32 = f32::from_bits(0x3F2A_DD8F);
const Q2: f32 = f32::from_bits(0x3F7F_7A01);
const RF_ZERO_SENTINEL: u32 = 0xFFFF_FF00;

/// F06ND.6's selected two packed-control quantizer kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoneDrcQuantizerKind {
    Rf,
    Line,
}

impl NoneDrcQuantizerKind {
    pub const fn wire_value(self) -> u8 {
        match self {
            Self::Rf => 0,
            Self::Line => 1,
        }
    }

    const fn layout(self) -> QuantizerLayout {
        match self {
            Self::Rf => QuantizerLayout {
                multiplier: 1.0,
                exponent_mask: 0xF000,
                fraction_mask: 0x0FFF,
                fraction_shift: 3,
                bias: f32::from_bits(0x3C80_0000), // 1/64
                exponent_limit: 0x7000,
                exponent_step: 0x1000,
                output_fraction_mask: 0x3C00,
                output_fraction_shift: 2,
            },
            Self::Line => QuantizerLayout {
                multiplier: 2.0,
                exponent_mask: 0xE000,
                fraction_mask: 0x1FFF,
                fraction_shift: 2,
                bias: f32::from_bits(0x3C00_0000), // 1/128
                exponent_limit: 0x6000,
                exponent_step: 0x2000,
                output_fraction_mask: 0x3E00,
                output_fraction_shift: 1,
            },
        }
    }
}

impl TryFrom<u8> for NoneDrcQuantizerKind {
    type Error = NoneDrcQuantizerError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Rf),
            1 => Ok(Self::Line),
            _ => Err(NoneDrcQuantizerError::UnsupportedKind),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct QuantizerLayout {
    multiplier: f32,
    exponent_mask: u16,
    fraction_mask: u16,
    fraction_shift: u32,
    bias: f32,
    exponent_limit: u16,
    exponent_step: u16,
    output_fraction_mask: u16,
    output_fraction_shift: u32,
}

/// The full scalar result and the low word consumed by packed storage.
///
/// RF's zero sentinel is 0xFFFF_FF00 as a callee result and 0xFF00 when stored
/// as U16. Line zero remains 0x0000 in both representations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackedControlCode {
    callee_word: u32,
    storage_word: u16,
}

impl PackedControlCode {
    pub const fn callee_word(self) -> u32 {
        self.callee_word
    }

    pub const fn storage_word(self) -> u16 {
        self.storage_word
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoneDrcQuantizerError {
    UnsupportedKind,
    NonFiniteInput,
    NumericNonFinite,
}

impl NoneDrcQuantizerError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedKind => "NONE_DRC_QUANTIZER_KIND",
            Self::NonFiniteInput => "NONE_DRC_QUANTIZER_INPUT_NONFINITE",
            Self::NumericNonFinite => "NONE_DRC_QUANTIZER_NUMERIC_NONFINITE",
        }
    }
}

impl std::fmt::Display for NoneDrcQuantizerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for NoneDrcQuantizerError {}

/// Quantizes one finite RF or line control value according to F06ND.6.
///
/// The selected DRC_QUANTIZE3 binary32 parameter words are Q0=0x3E2F7469,
/// Q1=0x3F2ADD8F, and Q2=0x3F7F7A01. Each multiply/add/subtract in the
/// polynomial is rounded independently to binary32 in the specified order.
pub fn quantize_none_drc_control(
    value: f32,
    kind: NoneDrcQuantizerKind,
) -> Result<PackedControlCode, NoneDrcQuantizerError> {
    FiniteF32::new(value).map_err(|_| NoneDrcQuantizerError::NonFiniteInput)?;
    let layout = kind.layout();

    // Keep the two R32 operations separate. Overflow here remains signed
    // infinity and is saturated by the following RoundEven/Sat16 operation.
    let multiplied = layout.multiplier * value;
    let scaled = multiplied * QUANTIZER_SCALE;
    let quantized = saturating_round_even_i16(scaled);
    let quantized_bits = u16::from_ne_bytes(quantized.to_ne_bytes());

    let mut exponent = signed_i16(quantized_bits & layout.exponent_mask);
    let fraction = quantized_bits & layout.fraction_mask;
    let signed_fraction = signed_i16(fraction << layout.fraction_shift);
    let fraction_value = checked_r32_mul(f32::from(signed_fraction), FRACTION_SCALE)?;
    let x = checked_r32_sub(fraction_value, 1.0)?;

    let x_squared = checked_r32_mul(x, x)?;
    let quadratic_term = checked_r32_mul(x_squared, Q0)?;
    let linear_term = checked_r32_mul(x, Q1)?;
    let quadratic_and_linear = checked_r32_add(quadratic_term, linear_term)?;
    let polynomial = checked_r32_add(quadratic_and_linear, Q2)?;
    let biased_polynomial = checked_r32_add(polynomial, layout.bias)?;

    let packed_fraction = if biased_polynomial >= 1.0 {
        if exponent >= signed_i16(layout.exponent_limit) {
            QUANTIZER_SCALE
        } else {
            exponent = exponent.wrapping_add(signed_i16(layout.exponent_step));
            0.0
        }
    } else {
        biased_polynomial
    };

    let rounded_fraction =
        saturating_round_even_i16(checked_r32_mul(packed_fraction, QUANTIZER_SCALE)?);
    let rounded_bits = u16::from_ne_bytes(rounded_fraction.to_ne_bytes());
    let fraction_code =
        signed_i16(rounded_bits & layout.output_fraction_mask) >> layout.output_fraction_shift;
    let combined = exponent.saturating_add(fraction_code);
    let storage_word = u16::from_ne_bytes(combined.to_ne_bytes());

    if kind == NoneDrcQuantizerKind::Rf && storage_word == 0 {
        Ok(PackedControlCode {
            callee_word: RF_ZERO_SENTINEL,
            storage_word: 0xFF00,
        })
    } else {
        Ok(PackedControlCode {
            callee_word: u32::from(storage_word),
            storage_word,
        })
    }
}

fn signed_i16(bits: u16) -> i16 {
    i16::from_ne_bytes(bits.to_ne_bytes())
}

fn saturating_round_even_i16(value: f32) -> i16 {
    let rounded = value.round_ties_even();
    if rounded <= f32::from(i16::MIN) {
        i16::MIN
    } else if rounded >= f32::from(i16::MAX) {
        i16::MAX
    } else {
        rounded as i16
    }
}

fn checked_r32_mul(left: f32, right: f32) -> Result<f32, NoneDrcQuantizerError> {
    let left = FiniteF32::new(left).map_err(|_| NoneDrcQuantizerError::NumericNonFinite)?;
    let right = FiniteF32::new(right).map_err(|_| NoneDrcQuantizerError::NumericNonFinite)?;
    r32_mul(left, right)
        .map(FiniteF32::value)
        .map_err(|_| NoneDrcQuantizerError::NumericNonFinite)
}

fn checked_r32_add(left: f32, right: f32) -> Result<f32, NoneDrcQuantizerError> {
    let left = FiniteF32::new(left).map_err(|_| NoneDrcQuantizerError::NumericNonFinite)?;
    let right = FiniteF32::new(right).map_err(|_| NoneDrcQuantizerError::NumericNonFinite)?;
    r32_add(left, right)
        .map(FiniteF32::value)
        .map_err(|_| NoneDrcQuantizerError::NumericNonFinite)
}

fn checked_r32_sub(left: f32, right: f32) -> Result<f32, NoneDrcQuantizerError> {
    let left = FiniteF32::new(left).map_err(|_| NoneDrcQuantizerError::NumericNonFinite)?;
    let right = FiniteF32::new(right).map_err(|_| NoneDrcQuantizerError::NumericNonFinite)?;
    r32_sub(left, right)
        .map(FiniteF32::value)
        .map_err(|_| NoneDrcQuantizerError::NumericNonFinite)
}
