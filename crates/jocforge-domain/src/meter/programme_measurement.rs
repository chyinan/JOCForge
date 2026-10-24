// pattern: Functional Core
#![allow(dead_code)] // Called by the future whole-meter finalize coordinator.
//! Final selected meter leaf values to one immutable programme measurement proposal.

use super::{
    integrated_meter::IntegratedMeterState,
    measurement_receipt::{
        MeasurementReceiptError, ProgrammeGatingKind, ProgrammeMeasurementReceipt,
        ProgrammeMeasurementReceiptInput,
    },
    true_peak::{TruePeakError, TruePeakState, multiply_words},
    ungated_mean::{DiOffGetterError, UngatedMeanState, di_off_getter},
};
use crate::numeric::{FiniteF32, r32_sub, trunc_f32_to_i32};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const HOST_SCALE_WORD: u32 = 0x4300_0000;
const NEGATIVE_ONE_TWENTY_WORD: u32 = 0xc2f0_0000;
const HALF_WORD: u32 = 0x3f00_0000;
const DIALNORM_SENTINEL_TOLERANCE: f32 = 0.000_001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgrammeMeasurementError {
    Identity,
    Getter,
    Peak,
    Numeric,
    Receipt,
    Gating,
    Canonical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MeterFinalizationIdentity<'a> {
    pub(crate) programme_id: &'a str,
    pub(crate) attempt: u64,
    pub(crate) input_sha256: [u8; 32],
    pub(crate) config_sha256: [u8; 32],
    pub(crate) meter_terminal_sha256: [u8; 32],
    pub(crate) normal_callbacks: u64,
    pub(crate) zero_drains: u64,
    pub(crate) energy_windows: u64,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgrammeMeasurementProposal {
    receipt: ProgrammeMeasurementReceipt,
    measurement: ProgrammeMeasurementValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgrammeMeasurementValue {
    dialogue_word: u32,
    speech_percentage_word: u32,
    itu_word: u32,
    ungated_word: u32,
    true_peak_word: u32,
    sample_peak_word: u32,
    selected_word: u32,
    gating_kind: ProgrammeGatingKind,
    dialnorm: u8,
    canonical_sha256: [u8; 32],
}

impl ProgrammeMeasurementProposal {
    pub const fn receipt(&self) -> &ProgrammeMeasurementReceipt {
        &self.receipt
    }

    pub const fn speech_percentage_word(&self) -> u32 {
        self.measurement.speech_percentage_word
    }

    pub const fn ungated_word(&self) -> u32 {
        self.measurement.ungated_word
    }

    pub const fn sample_peak_word(&self) -> u32 {
        self.measurement.sample_peak_word
    }

    pub const fn dialnorm(&self) -> u8 {
        self.measurement.dialnorm
    }

    pub const fn measurement(&self) -> &ProgrammeMeasurementValue {
        &self.measurement
    }

    pub fn into_receipt(self) -> ProgrammeMeasurementReceipt {
        self.receipt
    }
}

impl ProgrammeMeasurementValue {
    #[allow(clippy::too_many_arguments)]
    fn from_words(
        dialogue_word: u32,
        speech_percentage_word: u32,
        itu_word: u32,
        ungated_word: u32,
        true_peak_word: u32,
        sample_peak_word: u32,
        selected_word: u32,
        gating_kind: ProgrammeGatingKind,
        dialnorm: u8,
    ) -> Result<Self, ProgrammeMeasurementError> {
        if [
            dialogue_word,
            speech_percentage_word,
            itu_word,
            ungated_word,
            true_peak_word,
            sample_peak_word,
            selected_word,
        ]
        .into_iter()
        .any(|word| !f32::from_bits(word).is_finite())
            || !(1..=31).contains(&dialnorm)
        {
            return Err(ProgrammeMeasurementError::Canonical);
        }
        let canonical_bytes = canonical_measurement_bytes(
            dialogue_word,
            speech_percentage_word,
            itu_word,
            ungated_word,
            true_peak_word,
            sample_peak_word,
            selected_word,
            gating_kind,
            dialnorm,
        )?;
        Ok(Self {
            dialogue_word,
            speech_percentage_word,
            itu_word,
            ungated_word,
            true_peak_word,
            sample_peak_word,
            selected_word,
            gating_kind,
            dialnorm,
            canonical_sha256: Sha256::digest(canonical_bytes).into(),
        })
    }

    pub const fn dialogue_word(&self) -> u32 {
        self.dialogue_word
    }

    pub const fn itu_word(&self) -> u32 {
        self.itu_word
    }

    pub const fn true_peak_word(&self) -> u32 {
        self.true_peak_word
    }

    pub const fn selected_word(&self) -> u32 {
        self.selected_word
    }

    pub const fn gating_kind(&self) -> ProgrammeGatingKind {
        self.gating_kind
    }

    pub const fn canonical_sha256(&self) -> [u8; 32] {
        self.canonical_sha256
    }

    pub fn canonical_sha256_hex(&self) -> String {
        hex(&self.canonical_sha256)
    }
}

pub(crate) fn finalize_selected_measurement(
    identity: MeterFinalizationIdentity<'_>,
    true_peak: &TruePeakState,
    integrated: &IntegratedMeterState,
    ungated: &UngatedMeanState,
) -> Result<ProgrammeMeasurementProposal, ProgrammeMeasurementError> {
    if identity.programme_id.is_empty() {
        return Err(ProgrammeMeasurementError::Identity);
    }
    let itu_word =
        multiply_words(integrated.result_word(), HOST_SCALE_WORD).map_err(map_peak_error)?;
    let getter = di_off_getter(
        Some(itu_word),
        ungated.result_word(),
        "1770-4",
        false,
        100.0_f32.to_bits(),
    )
    .map_err(map_getter_error)?;
    let gating_kind = match getter.public_gating_kind() {
        0 => ProgrammeGatingKind::Ungated,
        2 => ProgrammeGatingKind::Level,
        _ => return Err(ProgrammeMeasurementError::Gating),
    };
    let true_peak_word = true_peak
        .final_true_peak_host_word()
        .map_err(map_peak_error)?;
    let sample_peak_word = true_peak
        .final_sample_peak_host_word()
        .map_err(map_peak_error)?;
    let ungated_word =
        multiply_words(ungated.result_word(), HOST_SCALE_WORD).map_err(map_peak_error)?;
    let dialnorm = dialnorm_from_selected_lufs(getter.selected_host_word())?;
    let measurement = ProgrammeMeasurementValue::from_words(
        getter.dialogue_word(),
        getter.speech_percentage_word(),
        itu_word,
        ungated_word,
        true_peak_word,
        sample_peak_word,
        getter.selected_host_word(),
        gating_kind,
        dialnorm,
    )?;
    let receipt =
        ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
            programme_id: identity.programme_id,
            attempt: identity.attempt,
            input_sha256: identity.input_sha256,
            config_sha256: identity.config_sha256,
            dialogue_word: getter.dialogue_word(),
            itu_word,
            true_peak_word,
            gating_kind: gating_kind.as_str(),
            selected_lufs_word: getter.selected_host_word(),
            meter_terminal_sha256: identity.meter_terminal_sha256,
            normal_callbacks: identity.normal_callbacks,
            zero_drains: identity.zero_drains,
            energy_windows: identity.energy_windows,
        })
        .map_err(map_receipt_error)?;
    Ok(ProgrammeMeasurementProposal {
        receipt,
        measurement,
    })
}

#[allow(clippy::too_many_arguments)]
fn canonical_measurement_bytes(
    dialogue_word: u32,
    speech_percentage_word: u32,
    itu_word: u32,
    ungated_word: u32,
    true_peak_word: u32,
    sample_peak_word: u32,
    selected_word: u32,
    gating_kind: ProgrammeGatingKind,
    dialnorm: u8,
) -> Result<Vec<u8>, ProgrammeMeasurementError> {
    let mut fields = BTreeMap::<String, Value>::new();
    for (name, word) in [
        ("dialogue_host", dialogue_word),
        ("itu_host", itu_word),
        ("sample_peak_host", sample_peak_word),
        ("selected_host", selected_word),
        ("speech_percentage_host", speech_percentage_word),
        ("true_peak_host", true_peak_word),
        ("ungated_host", ungated_word),
    ] {
        fields.insert(name.to_owned(), Value::from(f32::from_bits(word)));
    }
    fields.insert(
        "gating_kind".to_owned(),
        Value::from(gating_kind.public_code()),
    );
    fields.insert("dialnorm".to_owned(), Value::from(dialnorm));
    serde_json::to_vec(&fields).map_err(|_| ProgrammeMeasurementError::Canonical)
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

pub fn dialnorm_from_selected_lufs(
    selected_lufs_word: u32,
) -> Result<u8, ProgrammeMeasurementError> {
    let selected = FiniteF32::new(f32::from_bits(selected_lufs_word))
        .map_err(|_| ProgrammeMeasurementError::Numeric)?;
    let negative_one_twenty = FiniteF32::new(f32::from_bits(NEGATIVE_ONE_TWENTY_WORD))
        .map_err(|_| ProgrammeMeasurementError::Numeric)?;
    let sentinel_distance = r32_sub(selected, negative_one_twenty)
        .map_err(|_| ProgrammeMeasurementError::Numeric)?
        .value()
        .abs();
    if sentinel_distance < DIALNORM_SENTINEL_TOLERANCE {
        return Ok(31);
    }
    let shifted = r32_sub(
        selected,
        FiniteF32::new(f32::from_bits(HALF_WORD))
            .map_err(|_| ProgrammeMeasurementError::Numeric)?,
    )
    .map_err(|_| ProgrammeMeasurementError::Numeric)?;
    let truncated = trunc_f32_to_i32(shifted)
        .map_err(|_| ProgrammeMeasurementError::Numeric)?
        .value()
        .clamp(-31, -1);
    u8::try_from(-truncated).map_err(|_| ProgrammeMeasurementError::Numeric)
}

fn map_getter_error(_: DiOffGetterError) -> ProgrammeMeasurementError {
    ProgrammeMeasurementError::Getter
}

fn map_peak_error(_: TruePeakError) -> ProgrammeMeasurementError {
    ProgrammeMeasurementError::Peak
}

fn map_receipt_error(_: MeasurementReceiptError) -> ProgrammeMeasurementError {
    ProgrammeMeasurementError::Receipt
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const UNIT_VECTORS: &[u8] =
        include_bytes!("../../tests/fixtures/f04_component/unit-vectors.json");

    #[test]
    fn approved_dialnorm_vectors_match() {
        let document: Value = serde_json::from_slice(UNIT_VECTORS).unwrap();
        let cases = document["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|case| case["entry_id"] == "dialnorm" || case["functional_id"] == "F06.3");
        let mut checked = 0;
        for case in cases {
            let input = case["input"]["selected_lufs"]
                .as_f64()
                .expect("dialnorm input") as f32;
            let expected = u8::try_from(case["expected"]["dialnorm"].as_u64().unwrap()).unwrap();
            assert_eq!(
                dialnorm_from_selected_lufs(input.to_bits()).unwrap(),
                expected
            );
            checked += 1;
        }
        assert_eq!(checked, 7);
    }

    #[test]
    fn cold_leaf_states_finalize_to_the_selected_absence_measurement() {
        let proposal = finalize_selected_measurement(
            MeterFinalizationIdentity {
                programme_id: "cold-finalize-test",
                attempt: 0,
                input_sha256: [0x11; 32],
                config_sha256: [0x22; 32],
                meter_terminal_sha256: [0x33; 32],
                normal_callbacks: 4_247,
                zero_drains: 3,
                energy_windows: 6_794,
            },
            &TruePeakState::cold(),
            &IntegratedMeterState::cold(),
            &UngatedMeanState::cold(),
        )
        .unwrap();
        assert_eq!(
            proposal.receipt().gating_kind(),
            ProgrammeGatingKind::Ungated
        );
        assert_eq!(
            proposal.receipt().selected_lufs_word(),
            (-120.0_f32).to_bits()
        );
        assert_eq!(proposal.receipt().dialogue_word(), (-120.0_f32).to_bits());
        assert_eq!(proposal.dialnorm(), 31);
        assert_eq!(proposal.receipt().meter_terminal_sha256(), &[0x33; 32]);
        assert_eq!(
            proposal.measurement().gating_kind(),
            ProgrammeGatingKind::Ungated
        );
        assert_eq!(
            proposal.measurement().selected_word(),
            (-120.0_f32).to_bits()
        );
    }

    #[test]
    fn approved_frozen_measurement_value_matches_its_canonical_digest() {
        let value = ProgrammeMeasurementValue::from_words(
            (-120.0_f32).to_bits(),
            (-1.0_f32).to_bits(),
            (-23.5_f32).to_bits(),
            (-22.75_f32).to_bits(),
            (-1.5_f32).to_bits(),
            (-2.0_f32).to_bits(),
            (-23.5_f32).to_bits(),
            ProgrammeGatingKind::Level,
            24,
        )
        .unwrap();
        assert_eq!(
            value.canonical_sha256_hex(),
            "8ca4448baf29a9d75141985e64d2f5f4c69f94d16b60e41561564f7699946fe9"
        );
    }
}
