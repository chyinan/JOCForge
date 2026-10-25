// pattern: Functional Core
#![allow(dead_code)] // The whole-meter finalize coordinator remains a gated production caller.
//! Immutable whole-programme measurement receipt value.

use crate::transaction::{AttemptId, ProgrammeId};

const SELECTED_NORMAL_CALLBACKS: u64 = 4_247;
const SELECTED_ZERO_DRAINS: u64 = 3;
const SELECTED_ENERGY_WINDOWS: u64 = 6_794;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeasurementReceiptError {
    Identity,
    GatingKind,
    NonfiniteMeasurement,
    Counts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgrammeGatingKind {
    Ungated,
    Dialogue,
    Level,
}

impl ProgrammeGatingKind {
    pub fn parse_exact(value: &str) -> Result<Self, MeasurementReceiptError> {
        match value {
            "UNGATED" => Ok(Self::Ungated),
            "DIALOGUE" => Ok(Self::Dialogue),
            "LEVEL" => Ok(Self::Level),
            _ => Err(MeasurementReceiptError::GatingKind),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ungated => "UNGATED",
            Self::Dialogue => "DIALOGUE",
            Self::Level => "LEVEL",
        }
    }

    pub const fn public_code(self) -> u8 {
        match self {
            Self::Ungated => 0,
            Self::Dialogue => 1,
            Self::Level => 2,
        }
    }

    pub const fn is_dialogue(self) -> bool {
        matches!(self, Self::Dialogue)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgrammeMeasurementReceipt {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    input_sha256: [u8; 32],
    config_sha256: [u8; 32],
    dialogue_word: u32,
    itu_word: u32,
    true_peak_word: u32,
    gating_kind: ProgrammeGatingKind,
    selected_lufs_word: u32,
    meter_terminal_sha256: [u8; 32],
    normal_callbacks: u64,
    zero_drains: u64,
    energy_windows: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgrammeMeasurementReceiptInput<'a> {
    pub(crate) programme_id: ProgrammeId,
    pub(crate) attempt: AttemptId,
    pub(crate) input_sha256: [u8; 32],
    pub(crate) config_sha256: [u8; 32],
    pub(crate) dialogue_word: u32,
    pub(crate) itu_word: u32,
    pub(crate) true_peak_word: u32,
    pub(crate) gating_kind: &'a str,
    pub(crate) selected_lufs_word: u32,
    pub(crate) meter_terminal_sha256: [u8; 32],
    pub(crate) normal_callbacks: u64,
    pub(crate) zero_drains: u64,
    pub(crate) energy_windows: u64,
}

impl ProgrammeMeasurementReceipt {
    /// Admits values only after the enclosing meter prepass has verified its
    /// terminal ledger. This constructor performs structural validation; it
    /// does not execute or certify the meter prepass itself.
    pub(crate) fn from_verified_prepass(
        input: ProgrammeMeasurementReceiptInput<'_>,
    ) -> Result<Self, MeasurementReceiptError> {
        let gating_kind = ProgrammeGatingKind::parse_exact(input.gating_kind)?;
        if [
            input.dialogue_word,
            input.itu_word,
            input.true_peak_word,
            input.selected_lufs_word,
        ]
        .into_iter()
        .any(|word| !f32::from_bits(word).is_finite())
        {
            return Err(MeasurementReceiptError::NonfiniteMeasurement);
        }
        if input.normal_callbacks != SELECTED_NORMAL_CALLBACKS
            || input.zero_drains != SELECTED_ZERO_DRAINS
            || input.energy_windows != SELECTED_ENERGY_WINDOWS
        {
            return Err(MeasurementReceiptError::Counts);
        }
        Ok(Self {
            programme_id: input.programme_id,
            attempt: input.attempt,
            input_sha256: input.input_sha256,
            config_sha256: input.config_sha256,
            dialogue_word: input.dialogue_word,
            itu_word: input.itu_word,
            true_peak_word: input.true_peak_word,
            gating_kind,
            selected_lufs_word: input.selected_lufs_word,
            meter_terminal_sha256: input.meter_terminal_sha256,
            normal_callbacks: input.normal_callbacks,
            zero_drains: input.zero_drains,
            energy_windows: input.energy_windows,
        })
    }

    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> AttemptId {
        self.attempt
    }

    pub const fn input_sha256(&self) -> &[u8; 32] {
        &self.input_sha256
    }

    pub const fn config_sha256(&self) -> &[u8; 32] {
        &self.config_sha256
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

    pub const fn gating_kind(&self) -> ProgrammeGatingKind {
        self.gating_kind
    }

    pub const fn selected_lufs_word(&self) -> u32 {
        self.selected_lufs_word
    }

    pub const fn meter_terminal_sha256(&self) -> &[u8; 32] {
        &self.meter_terminal_sha256
    }

    pub const fn normal_callbacks(&self) -> u64 {
        self.normal_callbacks
    }

    pub const fn zero_drains(&self) -> u64 {
        self.zero_drains
    }

    pub const fn energy_windows(&self) -> u64 {
        self.energy_windows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(gating_kind: &str) -> ProgrammeMeasurementReceiptInput<'_> {
        ProgrammeMeasurementReceiptInput {
            programme_id: crate::transaction::test_programme_id("measurement-test-programme"),
            attempt: AttemptId::new(0),
            input_sha256: [0x11; 32],
            config_sha256: [0x22; 32],
            dialogue_word: (-120.0_f32).to_bits(),
            itu_word: (-24.0_f32).to_bits(),
            true_peak_word: (-1.0_f32).to_bits(),
            gating_kind,
            selected_lufs_word: (-24.0_f32).to_bits(),
            meter_terminal_sha256: [0x33; 32],
            normal_callbacks: 4_247,
            zero_drains: 3,
            energy_windows: 6_794,
        }
    }

    #[test]
    fn gating_kind_accepts_only_the_three_case_sensitive_public_meanings() {
        for (text, kind, code) in [
            ("UNGATED", ProgrammeGatingKind::Ungated, 0),
            ("DIALOGUE", ProgrammeGatingKind::Dialogue, 1),
            ("LEVEL", ProgrammeGatingKind::Level, 2),
        ] {
            let receipt = ProgrammeMeasurementReceipt::from_verified_prepass(input(text)).unwrap();
            assert_eq!(receipt.gating_kind(), kind);
            assert_eq!(receipt.gating_kind().as_str(), text);
            assert_eq!(receipt.gating_kind().public_code(), code);
        }
        for rejected in ["", "ungated", "dialogue", "level", "0", "1", "2"] {
            assert_eq!(
                ProgrammeMeasurementReceipt::from_verified_prepass(input(rejected)),
                Err(MeasurementReceiptError::GatingKind)
            );
        }
    }
}
