// pattern: Functional Core
//! Measurement-receipt and explicit ID1-value projection to the selected configuration.

use crate::{
    meter::{
        measurement_receipt::{ProgrammeGatingKind, ProgrammeMeasurementReceipt},
        measurement_selection::Id1PreferredValue,
    },
    numeric::{FiniteF32, r32_add, trunc_f32_to_i32},
    transaction::{AttemptId, ProgrammeId},
};
use serde_json::Value;

const ABSENT: i32 = -512;
const LOUDNESS_MIN: i32 = -116;
const LOUDNESS_MAX: i32 = 11;
const TRUE_PEAK_MIN: i32 = -232;
const TRUE_PEAK_MAX: i32 = 23;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Id1ConfigurationError {
    NonfiniteMeasurement,
    Numeric,
    UnsupportedSelectedGating,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Id1MeasurementConfiguration {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    values: [i64; 9],
    measurement_input_sha256: [u8; 32],
    measurement_config_sha256: [u8; 32],
    meter_terminal_sha256: [u8; 32],
}

impl Id1MeasurementConfiguration {
    #[allow(dead_code)] // No production caller until the internal provider/receipt crosswalk is bound.
    pub(crate) fn from_selected_di_off_measurement(
        receipt: &ProgrammeMeasurementReceipt,
        preferred_value: Id1PreferredValue,
    ) -> Result<Self, Id1ConfigurationError> {
        if receipt.gating_kind() == ProgrammeGatingKind::Dialogue {
            return Err(Id1ConfigurationError::UnsupportedSelectedGating);
        }
        Self::from_measurement(receipt, preferred_value)
    }

    #[allow(dead_code)] // Shared conversion core for selected admission and isolated tests.
    fn from_measurement(
        receipt: &ProgrammeMeasurementReceipt,
        preferred_value: Id1PreferredValue,
    ) -> Result<Self, Id1ConfigurationError> {
        let values = [
            -2,
            1,
            i64::from(preferred_value.value()),
            0,
            i64::from(half_db_code(receipt.itu_word(), false)?),
            i64::from(half_db_code(receipt.dialogue_word(), false)?),
            i64::from(ABSENT),
            i64::from(half_db_code(receipt.true_peak_word(), true)?),
            i64::from(ABSENT),
        ];
        Ok(Self {
            programme_id: receipt.programme_id().clone(),
            attempt: receipt.attempt(),
            values,
            measurement_input_sha256: *receipt.input_sha256(),
            measurement_config_sha256: *receipt.config_sha256(),
            meter_terminal_sha256: *receipt.meter_terminal_sha256(),
        })
    }

    #[cfg(test)]
    pub(crate) fn from_measurement_for_test(
        receipt: &ProgrammeMeasurementReceipt,
        preferred_value: Id1PreferredValue,
    ) -> Result<Self, Id1ConfigurationError> {
        Self::from_measurement(receipt, preferred_value)
    }

    pub const fn values(&self) -> &[i64; 9] {
        &self.values
    }

    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> AttemptId {
        self.attempt
    }

    pub fn json_values(&self) -> Vec<Value> {
        self.values.into_iter().map(Value::from).collect()
    }

    pub const fn measurement_input_sha256(&self) -> &[u8; 32] {
        &self.measurement_input_sha256
    }

    pub const fn measurement_config_sha256(&self) -> &[u8; 32] {
        &self.measurement_config_sha256
    }

    pub const fn meter_terminal_sha256(&self) -> &[u8; 32] {
        &self.meter_terminal_sha256
    }
}

pub fn half_db_code(word: u32, true_peak: bool) -> Result<i32, Id1ConfigurationError> {
    let value = FiniteF32::new(f32::from_bits(word))
        .map_err(|_| Id1ConfigurationError::NonfiniteMeasurement)?;
    let shifted = r32_add(
        value,
        FiniteF32::new(120.0).map_err(|_| Id1ConfigurationError::Numeric)?,
    )
    .map_err(|_| Id1ConfigurationError::Numeric)?;
    if shifted.value() <= f32::from_bits(0x3400_0000) {
        return Ok(ABSENT);
    }
    let doubled = r32_add(value, value).map_err(|_| Id1ConfigurationError::Numeric)?;
    let bias = if value.value() < 0.0 { -0.5 } else { 0.5 };
    let biased = r32_add(
        doubled,
        FiniteF32::new(bias).map_err(|_| Id1ConfigurationError::Numeric)?,
    )
    .map_err(|_| Id1ConfigurationError::Numeric)?;
    let integer = trunc_f32_to_i32(biased)
        .map_err(|_| Id1ConfigurationError::Numeric)?
        .value();
    let (minimum, maximum) = if true_peak {
        (TRUE_PEAK_MIN, TRUE_PEAK_MAX)
    } else {
        (LOUDNESS_MIN, LOUDNESS_MAX)
    };
    Ok(integer.clamp(minimum, maximum))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meter::measurement_receipt::{
        ProgrammeMeasurementReceipt, ProgrammeMeasurementReceiptInput,
    };

    const UNIT_VECTORS: &[u8] =
        include_bytes!("../../../tests/fixtures/f04_component/unit-vectors.json");

    #[test]
    fn approved_half_db_vectors_match_all_measurement_words() {
        let document: Value = serde_json::from_slice(UNIT_VECTORS).unwrap();
        let cases = document["cases"]
            .as_array()
            .expect("unit vector cases are an array")
            .iter()
            .filter(|case| case["entry_id"] == "id1-half-db");
        let mut checked = 0;
        for case in cases {
            let true_peak = case["input"]["true_peak"].as_bool().unwrap();
            let words = case["input"]["measurement_u32"].as_array().unwrap();
            let expected = case["expected"]["codes"].as_array().unwrap();
            assert_eq!(words.len(), expected.len());
            for (word, expected) in words.iter().zip(expected) {
                assert_eq!(
                    half_db_code(u32::try_from(word.as_u64().unwrap()).unwrap(), true_peak)
                        .unwrap(),
                    i32::try_from(expected.as_i64().unwrap()).unwrap()
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 22);
    }

    #[test]
    fn measurement_receipt_projects_the_exact_nine_id1_fields() {
        let receipt =
            ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
                programme_id: crate::transaction::test_programme_id("id1-measurement-test"),
                attempt: AttemptId::new(3),
                input_sha256: [0x11; 32],
                config_sha256: [0x22; 32],
                dialogue_word: (-23.0_f32).to_bits(),
                itu_word: (-24.0_f32).to_bits(),
                true_peak_word: (-1.0_f32).to_bits(),
                gating_kind: "LEVEL",
                selected_lufs_word: (-24.0_f32).to_bits(),
                meter_terminal_sha256: [0x33; 32],
                normal_callbacks: 4_247,
                zero_drains: 3,
                energy_windows: 6_794,
            })
            .unwrap();
        let configuration = Id1MeasurementConfiguration::from_selected_di_off_measurement(
            &receipt,
            Id1PreferredValue::from_measurement_i32(0),
        )
        .unwrap();
        assert_eq!(
            configuration.values(),
            &[-2, 1, 0, 0, -48, -46, -512, -2, -512]
        );
        assert_eq!(configuration.measurement_input_sha256(), &[0x11; 32]);
        assert_eq!(configuration.measurement_config_sha256(), &[0x22; 32]);
        assert_eq!(configuration.meter_terminal_sha256(), &[0x33; 32]);
    }

    #[test]
    fn id1_preferred_value_is_explicit_and_selected_gating_remains_separate() {
        let receipt =
            ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
                programme_id: crate::transaction::test_programme_id("dialogue-gating-test"),
                attempt: AttemptId::new(0),
                input_sha256: [0x11; 32],
                config_sha256: [0x22; 32],
                dialogue_word: (-23.0_f32).to_bits(),
                itu_word: (-24.0_f32).to_bits(),
                true_peak_word: (-1.0_f32).to_bits(),
                gating_kind: "DIALOGUE",
                selected_lufs_word: (-24.0_f32).to_bits(),
                meter_terminal_sha256: [0x33; 32],
                normal_callbacks: 4_247,
                zero_drains: 3,
                energy_windows: 6_794,
            })
            .unwrap();
        assert_eq!(
            Id1MeasurementConfiguration::from_measurement(
                &receipt,
                Id1PreferredValue::from_measurement_i32(1),
            )
            .unwrap()
            .values()[2],
            1
        );
        assert_eq!(
            Id1MeasurementConfiguration::from_measurement(
                &receipt,
                Id1PreferredValue::from_measurement_i32(0),
            )
            .unwrap()
            .values()[2],
            0
        );
        assert_eq!(
            Id1MeasurementConfiguration::from_selected_di_off_measurement(
                &receipt,
                Id1PreferredValue::from_measurement_i32(1),
            ),
            Err(Id1ConfigurationError::UnsupportedSelectedGating)
        );
    }
}
