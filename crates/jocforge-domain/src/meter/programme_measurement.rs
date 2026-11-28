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
use crate::transaction::{AttemptId, ProgrammeId};
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MeterFinalizationIdentity {
    pub(crate) programme_id: ProgrammeId,
    pub(crate) attempt: AttemptId,
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

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgrammeMeasurementValue {
    dialogue_word: u32,
    // The public floor shares its wire value with the absence sentinel.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    measured_at_dialogue_floor: bool,
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
        dialogue: Option<u32>,
        speech_percentage_word: u32,
        itu_word: u32,
        ungated_word: u32,
        true_peak_word: u32,
        sample_peak_word: u32,
        selected_word: u32,
        gating_kind: ProgrammeGatingKind,
        dialnorm: u8,
    ) -> Result<Self, ProgrammeMeasurementError> {
        let dialogue_word = dialogue.unwrap_or(NEGATIVE_ONE_TWENTY_WORD);
        let measured_at_dialogue_floor = dialogue == Some(NEGATIVE_ONE_TWENTY_WORD);
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
            measured_at_dialogue_floor,
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
            measured_at_dialogue_floor,
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

    /// Check the exact stored float words against the original finalized measurement.
    pub(crate) fn validate_cached(&self) -> Result<(), ProgrammeMeasurementError> {
        let checked = Self::from_words(
            self.measured_dialogue_word(),
            self.speech_percentage_word,
            self.itu_word,
            self.ungated_word,
            self.true_peak_word,
            self.sample_peak_word,
            self.selected_word,
            self.gating_kind,
            self.dialnorm,
        )?;
        if checked != *self || dialnorm_from_selected_lufs(self.selected_word)? != self.dialnorm {
            return Err(ProgrammeMeasurementError::Canonical);
        }
        Ok(())
    }

    /// Percentage of report ticks accepted by the dialogue and programme activity gates.
    pub fn speech_percentage(&self) -> Option<f32> {
        let percent = f32::from_bits(self.speech_percentage_word);
        (percent >= 0.0).then_some(percent)
    }

    pub const fn dialogue_word(&self) -> u32 {
        self.dialogue_word
    }

    /// Candidate integrated level when accepted audio contributed energy.
    pub fn dialogue_loudness(&self) -> Option<f32> {
        self.measured_dialogue_word()
            .map(|word| f32::from_bits(word).max(-120.0))
    }

    fn measured_dialogue_word(&self) -> Option<u32> {
        (self.measured_at_dialogue_floor || self.dialogue_word != NEGATIVE_ONE_TWENTY_WORD)
            .then_some(self.dialogue_word)
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
    identity: MeterFinalizationIdentity,
    true_peak: &TruePeakState,
    integrated: &IntegratedMeterState,
    ungated: &UngatedMeanState,
) -> Result<ProgrammeMeasurementProposal, ProgrammeMeasurementError> {
    let measurement =
        compute_programme_measurement_values(true_peak, integrated, ungated, false, None, None)?;
    let receipt =
        ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
            programme_id: identity.programme_id,
            attempt: identity.attempt,
            input_sha256: identity.input_sha256,
            config_sha256: identity.config_sha256,
            dialogue_word: measurement.dialogue_word,
            itu_word: measurement.itu_word,
            true_peak_word: measurement.true_peak_word,
            gating_kind: measurement.gating_kind.as_str(),
            selected_lufs_word: measurement.selected_word,
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

/// Computes the finalized scalar values without minting a programme receipt.
///
/// A receipt requires the enclosing standalone Meter root's actual canonical
/// terminal state identity and is therefore produced only by its owner stage.
pub(crate) fn compute_programme_measurement_values(
    true_peak: &TruePeakState,
    integrated: &IntegratedMeterState,
    ungated: &UngatedMeanState,
    use_ungated_loudness: bool,
    a_weighted_level: Option<f32>,
    speech: Option<(f64, Option<f64>, u32)>,
) -> Result<ProgrammeMeasurementValue, ProgrammeMeasurementError> {
    let true_peak_word = true_peak
        .final_true_peak_host_word()
        .map_err(map_peak_error)?;
    let sample_peak_word = true_peak
        .final_sample_peak_host_word()
        .map_err(map_peak_error)?;
    let itu_word =
        multiply_words(integrated.result_word(), HOST_SCALE_WORD).map_err(map_peak_error)?;
    let ungated_state_word = ungated.result_word();
    let ungated_word =
        multiply_words(ungated_state_word, HOST_SCALE_WORD).map_err(map_peak_error)?;
    let getter = di_off_getter(
        (!use_ungated_loudness && a_weighted_level.is_none()).then_some(itu_word),
        ungated_state_word,
        "1770-4",
        false,
        100.0_f32.to_bits(),
    )
    .map_err(map_getter_error)?;
    let mut gating_kind = match getter.public_gating_kind() {
        0 => ProgrammeGatingKind::Ungated,
        2 => ProgrammeGatingKind::Level,
        _ => return Err(ProgrammeMeasurementError::Gating),
    };
    let mut selected_word = a_weighted_level.map_or(getter.selected_host_word(), f32::to_bits);
    let mut speech_percentage_word = getter.speech_percentage_word();
    let mut dialogue =
        (getter.dialogue_word() != NEGATIVE_ONE_TWENTY_WORD).then_some(getter.dialogue_word());
    if let Some((percentage, level, threshold)) = speech {
        speech_percentage_word = (percentage as f32).to_bits();
        if let Some(level) = level {
            if !level.is_finite() {
                return Err(ProgrammeMeasurementError::Numeric);
            }
            let dialogue_word = (level as f32).max(-120.0).to_bits();
            dialogue = Some(dialogue_word);
            if percentage > f64::from(threshold) && level > -60.0 {
                selected_word = dialogue_word;
                gating_kind = ProgrammeGatingKind::Dialogue;
            }
        }
    }
    let dialnorm = dialnorm_from_selected_lufs(selected_word)?;
    let measurement = ProgrammeMeasurementValue::from_words(
        dialogue,
        speech_percentage_word,
        itu_word,
        ungated_word,
        true_peak_word,
        sample_peak_word,
        selected_word,
        gating_kind,
        dialnorm,
    )?;
    Ok(measurement)
}

#[allow(clippy::too_many_arguments)]
fn canonical_measurement_bytes(
    dialogue_word: u32,
    measured_at_dialogue_floor: bool,
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
    if measured_at_dialogue_floor {
        fields.insert("measured_at_dialogue_floor".to_owned(), Value::Bool(true));
    }
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
    fn dialogue_selection_uses_the_enabled_branch_and_a_valid_measurement() {
        for (programme, dialogue) in [(-18.2, -26.8), (-26.8, -18.2)] {
            for (speech, expected, selected_dialogue) in [
                (None, programme, false),
                (Some((80.0, Some(f64::from(dialogue)), 15)), dialogue, true),
                (Some((4.0, Some(f64::from(dialogue)), 15)), programme, false),
                (Some((0.0, None, 0)), programme, false),
            ] {
                let measured = compute_programme_measurement_values(
                    &TruePeakState::cold(),
                    &IntegratedMeterState::cold(),
                    &UngatedMeanState::cold(),
                    false,
                    Some(programme),
                    speech,
                )
                .unwrap();
                assert_eq!(f32::from_bits(measured.selected_word()), expected);
                assert_eq!(measured.gating_kind().is_dialogue(), selected_dialogue);
                assert_eq!(measured.dialnorm, (-expected).round() as u8);
            }
        }
    }

    #[test]
    fn dialogue_gates_compare_unrounded_percentage_and_loudness_strictly() {
        let programme = -21.92;
        for (accepted, percent_passes) in [(71, false), (72, false), (73, true)] {
            let percent = 100.0 * f64::from(accepted) / 480.0;
            for (level, loudness_passes) in [
                (-60.01, false),
                (-60.0, false),
                (-59.999_999_9, true),
                (-59.99, true),
            ] {
                let measurement = compute_programme_measurement_values(
                    &TruePeakState::cold(),
                    &IntegratedMeterState::cold(),
                    &UngatedMeanState::cold(),
                    false,
                    Some(programme),
                    Some((percent, Some(level), 15)),
                )
                .unwrap();
                let dialogue = percent_passes && loudness_passes;
                assert_eq!(
                    measurement.gating_kind().is_dialogue(),
                    dialogue,
                    "{percent}% {level}"
                );
                assert_eq!(measurement.dialogue_loudness(), Some(level as f32));
                assert_eq!(measurement.speech_percentage(), Some(percent as f32));
                assert_eq!(
                    measurement.selected_word(),
                    if dialogue { level as f32 } else { programme }.to_bits()
                );
            }
        }
    }

    #[test]
    fn finite_dialogue_at_the_report_floor_remains_distinct_from_absence() {
        for (input, expected) in [
            (None, None),
            (Some((0.0, None, 0)), None),
            (Some((5.0, None, 0)), None),
            (Some((5.0, Some(-160.0), 0)), Some(-120.0)),
            (Some((5.0, Some(-120.0), 0)), Some(-120.0)),
            (Some((5.0, Some(-119.0), 0)), Some(-119.0)),
        ] {
            let measurement = compute_programme_measurement_values(
                &TruePeakState::cold(),
                &IntegratedMeterState::cold(),
                &UngatedMeanState::cold(),
                false,
                Some(-21.92),
                input,
            )
            .unwrap();
            assert_eq!(measurement.dialogue_loudness(), expected, "{input:?}");
            assert_eq!(f32::from_bits(measurement.selected_word()), -21.92);
            let cached: ProgrammeMeasurementValue =
                serde_json::from_value(serde_json::to_value(&measurement).unwrap()).unwrap();
            assert_eq!(cached.dialogue_loudness(), expected);
            assert_eq!(cached.validate_cached(), Ok(()));
        }
    }

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
                programme_id: crate::transaction::test_programme_id("cold-finalize-test"),
                attempt: AttemptId::new(0),
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
            None,
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
