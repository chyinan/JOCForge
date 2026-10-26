// pattern: Functional Core
//! Reader-target error adapter for the shared exact-rational trig primitive.

use super::ReaderTargetError;
use crate::numeric::exact_trigonometry::{ExactTrigonometryError, exact_r64_to_r32};

/// Rounded panner words and the actual number of enclosure steps for each series.
pub(super) struct MathematicalR64R32 {
    pub(super) cosine: f32,
    pub(super) sine: f32,
    #[allow(dead_code)]
    pub(super) cosine_series_steps: u64,
    #[allow(dead_code)]
    pub(super) sine_series_steps: u64,
}

/// Preserves the reader-target error boundary while reusing the shared numeric implementation.
pub(super) fn mathematical_r64_r32(angle: f32) -> Result<MathematicalR64R32, ReaderTargetError> {
    let result = exact_r64_to_r32(angle).map_err(|error| match error {
        ExactTrigonometryError::AngleOutOfRange => ReaderTargetError::ProfileUnsupported,
        ExactTrigonometryError::NumericFailure => ReaderTargetError::NumericFailure,
    })?;
    Ok(MathematicalR64R32 {
        cosine: result.cosine,
        sine: result.sine,
        cosine_series_steps: result.cosine_series_steps,
        sine_series_steps: result.sine_series_steps,
    })
}

#[cfg(test)]
mod tests {
    use super::mathematical_r64_r32;
    use serde_json::Value;

    const TRIGONOMETRY_KAT: &str =
        include_str!("../../../../tests/fixtures/reader_target/panner-trigonometry-kat.json");

    #[test]
    fn direct_angles_match_the_rational_enclosure_kat() {
        let vectors: Value = serde_json::from_str(TRIGONOMETRY_KAT).unwrap();
        for case in vectors["direct_angles"].as_array().unwrap() {
            let angle_bits =
                u32::from_str_radix(case["angle_f32_hex"].as_str().unwrap(), 16).unwrap();
            let result = mathematical_r64_r32(f32::from_bits(angle_bits)).unwrap();
            let expected_cosine =
                u32::from_str_radix(case["cosine_f32_hex"].as_str().unwrap(), 16).unwrap();
            let expected_sine =
                u32::from_str_radix(case["sine_f32_hex"].as_str().unwrap(), 16).unwrap();
            assert_eq!(result.cosine.to_bits(), expected_cosine, "{case}");
            assert_eq!(result.sine.to_bits(), expected_sine, "{case}");
        }
    }
}
