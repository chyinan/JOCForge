// pattern: Functional Core
//! Builds one independent QMF history/window lane for each selected ADM JOC carrier role.

use crate::{
    analysis::source_frame::AnalysisOwnerGeneration,
    profile::{Flat7xJocCarrierRole, Flat7xJocCarrierRowBinding, SevenCarrierPcmInput},
    spatial::{
        QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
        qmf_analysis_synthesis::{
            QMF_ANALYSIS_SAMPLES_PER_SLOT, QMF_ANALYSIS_SAMPLES_PER_SOURCE,
            QMF_ANALYSIS_SLOTS_PER_SOURCE, QMF_ANALYSIS_SPECTRUM_WORDS, QmfAnalysisHistory,
            QmfAnalysisSynthesisError, analyze_qmf_row,
        },
    },
    timeline::{CodecSourceIndex, source_for_callback},
    transaction::{CallbackLineageIdentity, CallbackTransactionId},
};

const CARRIER_COUNT: usize = 7;
const WINDOW_SLOT_COUNT: usize = 48;

type QmfSpectrum = [u32; QMF_ANALYSIS_SPECTRUM_WORDS];
type QmfWindowsByCarrier = [Box<[QmfSpectrum]>; CARRIER_COUNT];

/// Source-stage identity for a seven-carrier QMF proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SevenCarrierQmfSourceIdentity {
    callback_lineage: CallbackLineageIdentity,
    source_transaction_id: CallbackTransactionId,
    source_index: CodecSourceIndex,
    analysis_generation: AnalysisOwnerGeneration,
}

impl SevenCarrierQmfSourceIdentity {
    pub fn new(
        callback_lineage: CallbackLineageIdentity,
        source_transaction_id: CallbackTransactionId,
        source_index: CodecSourceIndex,
        analysis_generation: AnalysisOwnerGeneration,
    ) -> Result<Self, SevenCarrierWindowError> {
        if source_for_callback(callback_lineage.callback_index()) != Some(source_index) {
            return Err(SevenCarrierWindowError::SourceClockMismatch);
        }
        Ok(Self {
            callback_lineage,
            source_transaction_id,
            source_index,
            analysis_generation,
        })
    }

    pub const fn callback_lineage(&self) -> &CallbackLineageIdentity {
        &self.callback_lineage
    }

    pub const fn source_transaction_id(&self) -> &CallbackTransactionId {
        &self.source_transaction_id
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn analysis_generation(&self) -> AnalysisOwnerGeneration {
        self.analysis_generation
    }
}

/// Seven independent QMF analysis histories and previous 24-slot windows.
///
/// This value is staged by the caller and returned as an after-state proposal. It has no commit
/// authority of its own.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SevenCarrierQmfState {
    analysis_history_by_carrier: [QmfAnalysisHistory; CARRIER_COUNT],
    previous_slots_by_carrier: [Box<[QmfSpectrum; QMF_ANALYSIS_SLOTS_PER_SOURCE]>; CARRIER_COUNT],
    last_source_identity: Option<SevenCarrierQmfSourceIdentity>,
    role_bindings: Option<[Flat7xJocCarrierRowBinding; CARRIER_COUNT]>,
}

impl SevenCarrierQmfState {
    pub fn cold() -> Self {
        Self {
            analysis_history_by_carrier: std::array::from_fn(|_| QmfAnalysisHistory::cold()),
            previous_slots_by_carrier: std::array::from_fn(|_| {
                Box::new([[0; QMF_ANALYSIS_SPECTRUM_WORDS]; QMF_ANALYSIS_SLOTS_PER_SOURCE])
            }),
            last_source_identity: None,
            role_bindings: None,
        }
    }

    pub const fn analysis_history_by_carrier(&self) -> &[QmfAnalysisHistory; CARRIER_COUNT] {
        &self.analysis_history_by_carrier
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, SevenCarrierWindowError> {
        let identity = self
            .last_source_identity
            .as_ref()
            .ok_or(SevenCarrierWindowError::CanonicalStateUnavailable)?;
        let role_bindings = self
            .role_bindings
            .as_ref()
            .ok_or(SevenCarrierWindowError::CanonicalStateUnavailable)?;
        let programme_id = identity.callback_lineage.programme_id().as_str();
        let source_transaction_id = identity.source_transaction_id.as_str();
        let mut bytes = Vec::with_capacity(
            96 + programme_id.len()
                + source_transaction_id.len()
                + CARRIER_COUNT
                    * (9 * QMF_ANALYSIS_SAMPLES_PER_SLOT * 4
                        + QMF_ANALYSIS_SLOTS_PER_SOURCE * QMF_ANALYSIS_SPECTRUM_WORDS * 4),
        );
        bytes.extend_from_slice(b"SEVQMF01");
        bytes.push(7);
        for binding in role_bindings {
            bytes.push(carrier_role_code(binding.role()));
            bytes.push(binding.row().physical_host().value());
        }
        append_canonical_string(&mut bytes, programme_id)?;
        bytes.extend_from_slice(&identity.callback_lineage.attempt_id().value().to_le_bytes());
        bytes.extend_from_slice(
            &identity
                .callback_lineage
                .callback_index()
                .value()
                .to_le_bytes(),
        );
        append_canonical_string(&mut bytes, source_transaction_id)?;
        bytes.extend_from_slice(&identity.source_index.value().to_le_bytes());
        bytes.extend_from_slice(&identity.analysis_generation.value().to_le_bytes());
        for history in &self.analysis_history_by_carrier {
            for word in history.rows().iter().flatten() {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        for slots in &self.previous_slots_by_carrier {
            for word in slots.iter().flatten() {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        Ok(bytes)
    }

    pub fn matches_dispatch(&self, dispatch: crate::timeline::JocDispatchIndex) -> bool {
        self.last_source_identity.as_ref().is_some_and(|identity| {
            identity.source_index.value().checked_add(1) == Some(dispatch.value())
                && self.role_bindings.is_some()
        })
    }
}

fn append_canonical_string(
    bytes: &mut Vec<u8>,
    value: &str,
) -> Result<(), SevenCarrierWindowError> {
    let length = u32::try_from(value.len())
        .map_err(|_| SevenCarrierWindowError::CanonicalStateUnavailable)?;
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

const fn carrier_role_code(role: Flat7xJocCarrierRole) -> u8 {
    match role {
        Flat7xJocCarrierRole::Left => 0,
        Flat7xJocCarrierRole::Right => 1,
        Flat7xJocCarrierRole::Center => 2,
        Flat7xJocCarrierRole::LeftSurround => 3,
        Flat7xJocCarrierRole::RightSurround => 4,
        Flat7xJocCarrierRole::LeftRearSurround => 5,
        Flat7xJocCarrierRole::RightRearSurround => 6,
    }
}

/// Seven 48-slot raw-QMF windows in `[L,R,C,Ls,Rs,Lrs,Rrs]` order.
///
/// Each window contains the preceding source's 24 QMF slots followed by this source's 24 slots.
/// It records ADM row bindings separately from I0/D0 coded-row order and does not apply the
/// existing five-carrier mode-0 delay/FIR stage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SevenCarrierWindowSet {
    source_identity: SevenCarrierQmfSourceIdentity,
    role_bindings: [Flat7xJocCarrierRowBinding; CARRIER_COUNT],
    windows_by_carrier: QmfWindowsByCarrier,
}

impl SevenCarrierWindowSet {
    pub const fn source_identity(&self) -> &SevenCarrierQmfSourceIdentity {
        &self.source_identity
    }

    pub const fn role_bindings(&self) -> &[Flat7xJocCarrierRowBinding; CARRIER_COUNT] {
        &self.role_bindings
    }

    pub fn windows_for_role(&self, role: Flat7xJocCarrierRole) -> Option<&[QmfSpectrum]> {
        let index = self
            .role_bindings
            .iter()
            .position(|binding| binding.role() == role)?;
        Some(self.windows_by_carrier[index].as_ref())
    }

    pub fn words_in_target_order(&self) -> Vec<u32> {
        let mut words =
            Vec::with_capacity(CARRIER_COUNT * WINDOW_SLOT_COUNT * QMF_ANALYSIS_SPECTRUM_WORDS);
        for carrier_windows in &self.windows_by_carrier {
            for spectrum in carrier_windows {
                words.extend_from_slice(spectrum);
            }
        }
        words
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// QMF after-state and window proposal for one admitted seven-carrier PCM source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SevenCarrierWindowProposal {
    after_state: SevenCarrierQmfState,
    windows: SevenCarrierWindowSet,
}

impl SevenCarrierWindowProposal {
    pub const fn after_state(&self) -> &SevenCarrierQmfState {
        &self.after_state
    }

    pub const fn windows(&self) -> &SevenCarrierWindowSet {
        &self.windows
    }

    pub fn into_parts(self) -> (SevenCarrierQmfState, SevenCarrierWindowSet) {
        (self.after_state, self.windows)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SevenCarrierWindowError {
    SourceClockMismatch,
    SourceSequenceMismatch,
    RoleBindingMismatch,
    CanonicalStateUnavailable,
    PcmRowLengthMismatch {
        role: Flat7xJocCarrierRole,
        actual_samples: usize,
    },
    QmfAnalysis(QmfAnalysisSynthesisError),
}

/// Analyzes the seven selected PCM roles using the shared single-row QMF analyzer.
///
/// The function consumes the seven semantic PCM rows directly. It maintains independent QMF
/// histories and previous-window slots for each role; its result remains a noncommitting proposal.
pub fn stage_seven_carrier_windows(
    input: SevenCarrierPcmInput<'_>,
    source_identity: SevenCarrierQmfSourceIdentity,
    before: &SevenCarrierQmfState,
    functional_provider: &QmfReceiverFunctionalProvider,
    fft_provider: &QmfMathematicalFftProvider,
) -> Result<SevenCarrierWindowProposal, SevenCarrierWindowError> {
    let rows = input.rows_in_target_order();
    let role_bindings = *input.role_bindings();
    for (index, row) in rows.iter().enumerate() {
        if row.len() != QMF_ANALYSIS_SAMPLES_PER_SOURCE {
            return Err(SevenCarrierWindowError::PcmRowLengthMismatch {
                role: role_bindings[index].role(),
                actual_samples: row.len(),
            });
        }
    }
    validate_source_sequence(before, &source_identity, &role_bindings)?;

    let mut histories_after = std::array::from_fn(|_| QmfAnalysisHistory::cold());
    let mut previous_slots_after = std::array::from_fn(|_| {
        Box::new([[0; QMF_ANALYSIS_SPECTRUM_WORDS]; QMF_ANALYSIS_SLOTS_PER_SOURCE])
    });
    let mut windows_by_carrier: QmfWindowsByCarrier = std::array::from_fn(|_| {
        vec![[0; QMF_ANALYSIS_SPECTRUM_WORDS]; WINDOW_SLOT_COUNT].into_boxed_slice()
    });

    let analyzed = crate::dsp_workers::map_rows(CARRIER_COUNT, |carrier| {
        let source_samples = rows[carrier]
            .iter()
            .map(|word| f32::from_bits(*word))
            .collect::<Vec<_>>();
        analyze_qmf_row(
            &source_samples,
            &before.analysis_history_by_carrier[carrier],
            functional_provider,
            fft_provider,
        )
        .map_err(SevenCarrierWindowError::QmfAnalysis)
    })?;
    for (carrier, row) in analyzed.into_iter().enumerate() {
        let (current_slots, history_after) = row.into_parts();
        windows_by_carrier[carrier][..QMF_ANALYSIS_SLOTS_PER_SOURCE]
            .copy_from_slice(&before.previous_slots_by_carrier[carrier][..]);
        windows_by_carrier[carrier][QMF_ANALYSIS_SLOTS_PER_SOURCE..]
            .copy_from_slice(current_slots.as_ref());
        previous_slots_after[carrier] = current_slots;
        histories_after[carrier] = history_after;
    }

    let after_state = SevenCarrierQmfState {
        analysis_history_by_carrier: histories_after,
        previous_slots_by_carrier: previous_slots_after,
        last_source_identity: Some(source_identity.clone()),
        role_bindings: Some(role_bindings),
    };
    let windows = SevenCarrierWindowSet {
        source_identity,
        role_bindings,
        windows_by_carrier,
    };
    Ok(SevenCarrierWindowProposal {
        after_state,
        windows,
    })
}

fn validate_source_sequence(
    before: &SevenCarrierQmfState,
    current: &SevenCarrierQmfSourceIdentity,
    current_roles: &[Flat7xJocCarrierRowBinding; CARRIER_COUNT],
) -> Result<(), SevenCarrierWindowError> {
    match (&before.last_source_identity, &before.role_bindings) {
        (None, None) if current.source_index.value() == 0 => Ok(()),
        (Some(previous), Some(previous_roles)) => {
            let source_after = previous.source_index.value().checked_add(1);
            let callback_after = previous
                .callback_lineage
                .callback_index()
                .value()
                .checked_add(1);
            let analysis_after = previous.analysis_generation.value().checked_add(1);
            if source_after == Some(current.source_index.value())
                && callback_after == Some(current.callback_lineage.callback_index().value())
                && analysis_after == Some(current.analysis_generation.value())
                && previous.callback_lineage.programme_id()
                    == current.callback_lineage.programme_id()
                && previous.callback_lineage.attempt_id() == current.callback_lineage.attempt_id()
            {
                if previous_roles == current_roles {
                    Ok(())
                } else {
                    Err(SevenCarrierWindowError::RoleBindingMismatch)
                }
            } else {
                Err(SevenCarrierWindowError::SourceSequenceMismatch)
            }
        }
        _ => Err(SevenCarrierWindowError::SourceSequenceMismatch),
    }
}

#[cfg(test)]
mod tests;
