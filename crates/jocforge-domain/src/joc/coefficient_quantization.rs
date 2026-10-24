// pattern: Functional Core
//! Selected JOC coarse/fine coefficient quantization and trial restoration.

use crate::numeric::{FiniteF32, r32_mul};

const SCALE_BITS: u32 = 0x4700_0000; // 32768
const RESTORE_SCALE_BITS: u32 = 0x4180_0000; // 16
const I32_MIN_F32: f32 = -2_147_483_648.0;
const I32_MAX_EXCLUSIVE_F32: f32 = 2_147_483_648.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoefficientPrecision {
    Coarse,
    Fine,
}

impl CoefficientPrecision {
    pub fn from_wire_mode(mode: u8) -> Result<Self, CoefficientQuantizationError> {
        match mode {
            0 | 2 => Ok(Self::Coarse),
            1 | 3 => Ok(Self::Fine),
            _ => Err(CoefficientQuantizationError::UnsupportedMode),
        }
    }

    const fn center(self) -> i32 {
        match self {
            Self::Coarse => 48,
            Self::Fine => 96,
        }
    }

    const fn bias(self) -> i32 {
        match self {
            Self::Coarse => 205,
            Self::Fine => 102,
        }
    }

    const fn divisor(self) -> i32 {
        match self {
            Self::Coarse => 410,
            Self::Fine => 205,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoefficientQuantizationError {
    UnsupportedMode,
    NonFiniteInput,
    ScaledI32Range,
    NumericNonFinite,
    QuantizedCodeRange,
}

impl CoefficientQuantizationError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedMode => "JOC_QUANTIZATION_MODE",
            Self::NonFiniteInput => "JOC_QUANTIZATION_INPUT_NONFINITE",
            Self::ScaledI32Range => "JOC_QUANTIZATION_SCALED_I32_RANGE",
            Self::NumericNonFinite => "JOC_QUANTIZATION_NUMERIC_NONFINITE",
            Self::QuantizedCodeRange => "JOC_QUANTIZATION_CODE_RANGE",
        }
    }
}

impl std::fmt::Display for CoefficientQuantizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for CoefficientQuantizationError {}

pub fn quantize_selected_coefficient(
    value: f32,
    precision: CoefficientPrecision,
) -> Result<u16, CoefficientQuantizationError> {
    let finite = FiniteF32::new(value).map_err(|_| CoefficientQuantizationError::NonFiniteInput)?;
    let center = precision.center();
    if value == 0.0 {
        return u16::try_from(center).map_err(|_| CoefficientQuantizationError::QuantizedCodeRange);
    }
    let scaled = r32_mul(
        finite,
        FiniteF32::new(f32::from_bits(SCALE_BITS))
            .map_err(|_| CoefficientQuantizationError::NumericNonFinite)?,
    )
    .map_err(|_| CoefficientQuantizationError::ScaledI32Range)?
    .value();
    if !(I32_MIN_F32..I32_MAX_EXCLUSIVE_F32).contains(&scaled) {
        return Err(CoefficientQuantizationError::ScaledI32Range);
    }
    let integer = scaled.round_ties_even().clamp(-32_768.0, 32_767.0) as i32;
    let biased = if integer < 0 {
        integer - precision.bias()
    } else {
        integer + precision.bias()
    };
    let quantized = (biased / precision.divisor() + center).clamp(0, center * 2 - 1);
    u16::try_from(quantized).map_err(|_| CoefficientQuantizationError::QuantizedCodeRange)
}

/// Restores a trial code to the normalized coefficient domain. The stored
/// coefficient remains unchanged; this value exists only for score evaluation.
pub fn restore_trial_coefficient(
    quantized: u16,
    precision: CoefficientPrecision,
) -> Result<f32, CoefficientQuantizationError> {
    let max_code = match precision {
        CoefficientPrecision::Coarse => 95_u16,
        CoefficientPrecision::Fine => 191_u16,
    };
    if quantized > max_code {
        return Err(CoefficientQuantizationError::QuantizedCodeRange);
    }
    let index = match precision {
        CoefficientPrecision::Coarse => usize::from(quantized) * 2,
        CoefficientPrecision::Fine => usize::from(quantized),
    };
    let numerator =
        (i32::try_from(index).map_err(|_| CoefficientQuantizationError::QuantizedCodeRange)? - 96)
            * 205;
    let dequantized = FiniteF32::new(numerator as f32 / 32_768.0_f32)
        .map_err(|_| CoefficientQuantizationError::NumericNonFinite)?;
    r32_mul(
        dequantized,
        FiniteF32::new(f32::from_bits(RESTORE_SCALE_BITS))
            .map_err(|_| CoefficientQuantizationError::NumericNonFinite)?,
    )
    .map(FiniteF32::value)
    .map_err(|_| CoefficientQuantizationError::NumericNonFinite)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_boundaries_reject_nonfinite_and_scaled_i32_overflow() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                quantize_selected_coefficient(value, CoefficientPrecision::Fine),
                Err(CoefficientQuantizationError::NonFiniteInput)
            );
        }
        for value in [65_536.0_f32, -65_536.01_f32, f32::MAX, -f32::MAX] {
            assert_eq!(
                quantize_selected_coefficient(value, CoefficientPrecision::Fine),
                Err(CoefficientQuantizationError::ScaledI32Range)
            );
        }
        assert_eq!(
            quantize_selected_coefficient(0.0, CoefficientPrecision::Fine),
            Ok(96)
        );
        assert_eq!(
            quantize_selected_coefficient(-0.0, CoefficientPrecision::Coarse),
            Ok(48)
        );
    }
}
