// pattern: Functional Core
//! Noncommitting four-owner projection for the selected g=0/g=1 callback-only boundary.
//!
//! This value is not a complete `ProgrammeState` and cannot commit itself. A future programme
//! parent may install its four after-roots only as one member of the complete twelve-owner root
//! replacement. Construction after Meter requires the real typed `ProgrammeMeasurementReceipt`.

use crate::{
    frontend::{FrontendControlMachine, FrontendControlReplacementCandidate},
    meter::measurement_receipt::ProgrammeMeasurementReceipt,
    spatial::{
        SpatialAudioMachine, SpatialAudioReplacementCandidate, SpatialControlMachine,
        SpatialControlReplacementCandidate,
    },
    state::{ReaderEofMachine, ReaderEofReplacementCandidate},
    timeline::FrontendCallbackIndex,
};
use sha2::{Digest, Sha256};

use super::{
    AttemptId, CallbackLineageIdentity, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt,
    ProgrammeId,
};

/// The only root phases represented by this bounded callback-only projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackOnlyRootStatus {
    Ready,
    Active,
}

/// Parent-owned phase generation. It advances once for each enclosing root replacement.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CallbackPhaseGeneration(u64);

impl CallbackPhaseGeneration {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Parent-supplied identity projection for the exact prepared input admitted to this attempt.
/// Construction belongs to the application parent that holds `PreparedJobInput`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackProgrammeIdentity {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    input_sha256: [u8; 32],
    config_sha256: [u8; 32],
}

impl CallbackProgrammeIdentity {
    pub const fn new(
        programme_id: ProgrammeId,
        attempt: AttemptId,
        input_sha256: [u8; 32],
        config_sha256: [u8; 32],
    ) -> Self {
        Self {
            programme_id,
            attempt,
            input_sha256,
            config_sha256,
        }
    }

    pub const fn programme_id(&self) -> &ProgrammeId {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackOnlyRootError {
    MeterReceiptIdentityMismatch,
    InitialOwnerRootMismatch,
    CallbackLineageMismatch,
    CallbackStatusMismatch,
    CallbackClockOverflow,
    PhaseGenerationOverflow,
    OwnerReceiptMismatch(GtcOwnerId),
    AfterOwnerMismatch(GtcOwnerId),
}

/// Four main roots that participate in the approved g=0/g=1 callback-only replacement.
///
/// The type deliberately excludes the other eight GTC owners; it is an input projection for a
/// later full programme coordinator, never a substitute for that root.
#[must_use]
pub struct CallbackOnlyOwners {
    reader_eof: ReaderEofMachine,
    frontend_control: FrontendControlMachine,
    spatial_control: SpatialControlMachine,
    spatial_audio: SpatialAudioMachine,
}

impl CallbackOnlyOwners {
    pub const fn new(
        reader_eof: ReaderEofMachine,
        frontend_control: FrontendControlMachine,
        spatial_control: SpatialControlMachine,
        spatial_audio: SpatialAudioMachine,
    ) -> Self {
        Self {
            reader_eof,
            frontend_control,
            spatial_control,
            spatial_audio,
        }
    }

    pub const fn reader_eof(&self) -> &ReaderEofMachine {
        &self.reader_eof
    }

    pub const fn frontend_control(&self) -> &FrontendControlMachine {
        &self.frontend_control
    }

    pub const fn spatial_control(&self) -> &SpatialControlMachine {
        &self.spatial_control
    }

    pub const fn spatial_audio(&self) -> &SpatialAudioMachine {
        &self.spatial_audio
    }

    pub fn into_parts(
        self,
    ) -> (
        ReaderEofMachine,
        FrontendControlMachine,
        SpatialControlMachine,
        SpatialAudioMachine,
    ) {
        (
            self.reader_eof,
            self.frontend_control,
            self.spatial_control,
            self.spatial_audio,
        )
    }
}

/// The exact four owner candidates produced by one callback's F01-F04 prepare sequence.
#[must_use]
pub struct CallbackOnlyCandidates {
    reader_eof: ReaderEofReplacementCandidate,
    frontend_control: FrontendControlReplacementCandidate,
    spatial_control: SpatialControlReplacementCandidate,
    spatial_audio: SpatialAudioReplacementCandidate,
}

impl CallbackOnlyCandidates {
    pub const fn new(
        reader_eof: ReaderEofReplacementCandidate,
        frontend_control: FrontendControlReplacementCandidate,
        spatial_control: SpatialControlReplacementCandidate,
        spatial_audio: SpatialAudioReplacementCandidate,
    ) -> Self {
        Self {
            reader_eof,
            frontend_control,
            spatial_control,
            spatial_audio,
        }
    }
}

/// A validated partial owner projection after Meter's immutable ready receipt exists.
///
/// This type accepts only callback indices zero and one. It has no `commit`, mutation, or
/// publication method. Its after-value is still only the four-owner subset described above.
#[must_use]
pub struct CallbackOnlyOwnerProjection {
    identity: CallbackProgrammeIdentity,
    measurement_receipt: ProgrammeMeasurementReceipt,
    status: CallbackOnlyRootStatus,
    callback_clock: FrontendCallbackIndex,
    phase_generation: CallbackPhaseGeneration,
    owners: CallbackOnlyOwners,
}

impl CallbackOnlyOwnerProjection {
    /// Builds the selected main-root projection only from a Meter-issued receipt and cold roots.
    pub fn from_meter_ready(
        identity: CallbackProgrammeIdentity,
        measurement_receipt: ProgrammeMeasurementReceipt,
        phase_generation: CallbackPhaseGeneration,
        owners: CallbackOnlyOwners,
    ) -> Result<Self, CallbackOnlyRootError> {
        if measurement_receipt.programme_id() != identity.programme_id()
            || measurement_receipt.attempt() != identity.attempt()
            || measurement_receipt.input_sha256() != identity.input_sha256()
            || measurement_receipt.config_sha256() != identity.config_sha256()
        {
            return Err(CallbackOnlyRootError::MeterReceiptIdentityMismatch);
        }
        if owners.reader_eof.generation() != 0
            || owners.reader_eof.committed().callback_index().value() != 0
            || owners.frontend_control.generation() != 0
            || owners.frontend_control.committed().programme_id() != identity.programme_id()
            || owners.frontend_control.committed().attempt_id() != identity.attempt()
            || owners.frontend_control.committed().callback_index().value() != 0
            || owners.spatial_control.generation() != 0
            || owners.spatial_control.committed().callback_index().value() != 0
            || owners.spatial_audio.generation() != 0
            || owners.spatial_audio.committed().callback_index().value() != 0
        {
            return Err(CallbackOnlyRootError::InitialOwnerRootMismatch);
        }

        Ok(Self {
            identity,
            measurement_receipt,
            status: CallbackOnlyRootStatus::Ready,
            callback_clock: FrontendCallbackIndex::new(0),
            phase_generation,
            owners,
        })
    }

    pub const fn programme_id(&self) -> &ProgrammeId {
        self.identity.programme_id()
    }

    pub const fn attempt(&self) -> AttemptId {
        self.identity.attempt()
    }

    pub const fn status(&self) -> CallbackOnlyRootStatus {
        self.status
    }

    pub const fn callback_clock(&self) -> FrontendCallbackIndex {
        self.callback_clock
    }

    pub const fn phase_generation(&self) -> CallbackPhaseGeneration {
        self.phase_generation
    }

    pub const fn measurement_receipt(&self) -> &ProgrammeMeasurementReceipt {
        &self.measurement_receipt
    }

    pub const fn owners(&self) -> &CallbackOnlyOwners {
        &self.owners
    }

    /// Validates the complete callback-only receipt set and returns, without installing it, the
    /// replacement projection for g=0 or g=1. The GTC transaction ID and callback lineage are
    /// separate required inputs and must both match the receipt/root identities.
    pub fn prepare_callback_only(
        &self,
        callback_lineage: &CallbackLineageIdentity,
        gtc_transaction_id: CallbackTransactionId,
        candidates: CallbackOnlyCandidates,
    ) -> Result<CallbackOnlyRootProposal, CallbackOnlyRootError> {
        let callback = self.callback_clock.value();
        let callback_index = callback_lineage.callback_index().value();
        if callback_lineage.programme_id() != self.identity.programme_id()
            || callback_lineage.attempt_id() != self.identity.attempt()
            || callback_index != callback
        {
            return Err(CallbackOnlyRootError::CallbackLineageMismatch);
        }

        match (self.status, callback) {
            (CallbackOnlyRootStatus::Ready, 0) | (CallbackOnlyRootStatus::Active, 1) => {}
            _ => return Err(CallbackOnlyRootError::CallbackStatusMismatch),
        }

        let next_callback = callback
            .checked_add(1)
            .ok_or(CallbackOnlyRootError::CallbackClockOverflow)?;
        let next_phase_generation = self
            .phase_generation
            .0
            .checked_add(1)
            .map(CallbackPhaseGeneration)
            .ok_or(CallbackOnlyRootError::PhaseGenerationOverflow)?;

        validate_owner_transition(
            GtcOwnerId::ReaderEof,
            &gtc_transaction_id,
            reader_snapshot(&self.owners.reader_eof),
            reader_snapshot(candidates.reader_eof.next_machine()),
            candidates.reader_eof.gtc_receipt(),
        )?;
        validate_owner_transition(
            GtcOwnerId::FrontendControl,
            &gtc_transaction_id,
            frontend_snapshot(&self.owners.frontend_control),
            frontend_snapshot(candidates.frontend_control.next_machine()),
            candidates.frontend_control.receipt(),
        )?;
        validate_owner_transition(
            GtcOwnerId::SpatialControl,
            &gtc_transaction_id,
            spatial_control_snapshot(&self.owners.spatial_control),
            spatial_control_snapshot(candidates.spatial_control.next_machine()),
            candidates.spatial_control.receipt(),
        )?;
        validate_owner_transition(
            GtcOwnerId::SpatialAudio,
            &gtc_transaction_id,
            spatial_audio_snapshot(&self.owners.spatial_audio),
            spatial_audio_snapshot(candidates.spatial_audio.next_machine()),
            candidates.spatial_audio.receipt(),
        )?;

        if candidates
            .reader_eof
            .next_machine()
            .committed()
            .callback_index()
            .value()
            != next_callback
        {
            return Err(CallbackOnlyRootError::AfterOwnerMismatch(
                GtcOwnerId::ReaderEof,
            ));
        }
        if candidates
            .frontend_control
            .next_machine()
            .committed()
            .callback_index()
            .value()
            != next_callback
            || candidates
                .frontend_control
                .next_machine()
                .committed()
                .programme_id()
                != self.identity.programme_id()
            || candidates
                .frontend_control
                .next_machine()
                .committed()
                .attempt_id()
                != self.identity.attempt()
        {
            return Err(CallbackOnlyRootError::AfterOwnerMismatch(
                GtcOwnerId::FrontendControl,
            ));
        }
        if candidates
            .spatial_control
            .next_machine()
            .committed()
            .callback_index()
            .value()
            != next_callback
        {
            return Err(CallbackOnlyRootError::AfterOwnerMismatch(
                GtcOwnerId::SpatialControl,
            ));
        }
        if candidates
            .spatial_audio
            .next_machine()
            .committed()
            .callback_index()
            .value()
            != next_callback
        {
            return Err(CallbackOnlyRootError::AfterOwnerMismatch(
                GtcOwnerId::SpatialAudio,
            ));
        }

        let CallbackOnlyCandidates {
            reader_eof,
            frontend_control,
            spatial_control,
            spatial_audio,
        } = candidates;
        let (reader_eof, _, reader_receipt) = reader_eof.into_parts();
        let (frontend_control, frontend_receipt) = frontend_control.into_parts();
        let (spatial_control, spatial_receipt, _) = spatial_control.into_parts();
        let (spatial_audio, spatial_audio_receipt) = spatial_audio.into_parts();
        let receipts = [
            reader_receipt,
            frontend_receipt,
            spatial_receipt,
            spatial_audio_receipt,
        ];
        let after_root = Self {
            identity: self.identity.clone(),
            measurement_receipt: self.measurement_receipt.clone(),
            status: CallbackOnlyRootStatus::Active,
            callback_clock: FrontendCallbackIndex::new(next_callback),
            phase_generation: next_phase_generation,
            owners: CallbackOnlyOwners::new(
                reader_eof,
                frontend_control,
                spatial_control,
                spatial_audio,
            ),
        };
        Ok(CallbackOnlyRootProposal {
            after_root,
            gtc_transaction_id,
            callback_index: FrontendCallbackIndex::new(callback),
            receipts,
        })
    }
}

/// Fully validated four-owner after-root proposal. The enclosing twelve-owner parent must
/// perform the eventual single swap; this value does not claim that swap has happened.
#[must_use]
pub struct CallbackOnlyRootProposal {
    after_root: CallbackOnlyOwnerProjection,
    gtc_transaction_id: CallbackTransactionId,
    callback_index: FrontendCallbackIndex,
    receipts: [GtcOwnerReceipt; 4],
}

impl CallbackOnlyRootProposal {
    pub const fn after_root(&self) -> &CallbackOnlyOwnerProjection {
        &self.after_root
    }

    pub const fn gtc_transaction_id(&self) -> &CallbackTransactionId {
        &self.gtc_transaction_id
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn receipts(&self) -> &[GtcOwnerReceipt; 4] {
        &self.receipts
    }

    pub fn into_parts(
        self,
    ) -> (
        CallbackOnlyOwnerProjection,
        CallbackTransactionId,
        FrontendCallbackIndex,
        [GtcOwnerReceipt; 4],
    ) {
        (
            self.after_root,
            self.gtc_transaction_id,
            self.callback_index,
            self.receipts,
        )
    }
}

#[derive(Clone, Copy)]
struct OwnerSnapshot {
    generation: u64,
    sha256: [u8; 32],
    callback_index: u64,
}

fn validate_owner_transition(
    expected_owner: GtcOwnerId,
    expected_transaction_id: &CallbackTransactionId,
    before: OwnerSnapshot,
    after: OwnerSnapshot,
    receipt: &GtcOwnerReceipt,
) -> Result<(), CallbackOnlyRootError> {
    let expected_after_generation = before
        .generation
        .checked_add(1)
        .ok_or(CallbackOnlyRootError::OwnerReceiptMismatch(expected_owner))?;
    let expected_after_callback = before
        .callback_index
        .checked_add(1)
        .ok_or(CallbackOnlyRootError::OwnerReceiptMismatch(expected_owner))?;
    if receipt.owner_id() != expected_owner
        || receipt.transaction_id() != expected_transaction_id
        || receipt.before_generation() != before.generation
        || receipt.before_sha256() != before.sha256
        || receipt.after_generation() != expected_after_generation
        || receipt.after_sha256() != after.sha256
        || !receipt.changed()
        || receipt.commit_count() != 1
        || after.generation != expected_after_generation
        || after.callback_index != expected_after_callback
    {
        return Err(CallbackOnlyRootError::OwnerReceiptMismatch(expected_owner));
    }
    Ok(())
}

fn canonical_sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn reader_snapshot(machine: &ReaderEofMachine) -> OwnerSnapshot {
    let state = machine.state_reference();
    OwnerSnapshot {
        generation: state.generation(),
        sha256: canonical_sha256(state.canonical_bytes()),
        callback_index: u64::from(machine.committed().callback_index().value()),
    }
}

fn frontend_snapshot(machine: &FrontendControlMachine) -> OwnerSnapshot {
    let state = machine.state_reference();
    OwnerSnapshot {
        generation: state.generation(),
        sha256: canonical_sha256(state.canonical_bytes()),
        callback_index: u64::from(machine.committed().callback_index().value()),
    }
}

fn spatial_control_snapshot(machine: &SpatialControlMachine) -> OwnerSnapshot {
    let state = machine.state_reference();
    OwnerSnapshot {
        generation: state.generation(),
        sha256: canonical_sha256(state.canonical_bytes()),
        callback_index: u64::from(machine.committed().callback_index().value()),
    }
}

fn spatial_audio_snapshot(machine: &SpatialAudioMachine) -> OwnerSnapshot {
    let state = machine.state_reference();
    OwnerSnapshot {
        generation: state.generation(),
        sha256: canonical_sha256(state.canonical_bytes()),
        callback_index: u64::from(machine.committed().callback_index().value()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{GtcOwnerReceiptError, GtcOwnerStateSnapshot};

    fn owner_receipt(
        owner_id: GtcOwnerId,
        transaction_id: CallbackTransactionId,
    ) -> GtcOwnerReceipt {
        let before_bytes = [owner_id.as_str().as_bytes(), b"-before"].concat();
        let after_bytes = [owner_id.as_str().as_bytes(), b"-after"].concat();
        let before =
            GtcOwnerStateSnapshot::from_canonical_bytes(owner_id, 0, [0x11; 32], &before_bytes)
                .unwrap();
        let after =
            GtcOwnerStateSnapshot::from_canonical_bytes(owner_id, 1, [0x11; 32], &after_bytes)
                .unwrap();
        GtcOwnerReceipt::from_transition(before, after, transaction_id).unwrap()
    }

    #[test]
    fn transition_guard_accepts_each_registered_callback_only_owner() {
        let transaction_id = CallbackTransactionId::new("local-callback-g0").unwrap();
        for owner_id in [
            GtcOwnerId::ReaderEof,
            GtcOwnerId::FrontendControl,
            GtcOwnerId::SpatialControl,
            GtcOwnerId::SpatialAudio,
        ] {
            let receipt = owner_receipt(owner_id, transaction_id.clone());
            validate_owner_transition(
                owner_id,
                &transaction_id,
                OwnerSnapshot {
                    generation: 0,
                    sha256: canonical_sha256(&[owner_id.as_str().as_bytes(), b"-before"].concat()),
                    callback_index: 0,
                },
                OwnerSnapshot {
                    generation: 1,
                    sha256: canonical_sha256(&[owner_id.as_str().as_bytes(), b"-after"].concat()),
                    callback_index: 1,
                },
                &receipt,
            )
            .unwrap();
        }
    }

    #[test]
    fn transition_guard_rejects_foreign_owner_transaction_and_roots() {
        let transaction_id = CallbackTransactionId::new("local-callback-g0").unwrap();
        let receipt = owner_receipt(GtcOwnerId::ReaderEof, transaction_id.clone());
        assert_eq!(
            validate_owner_transition(
                GtcOwnerId::FrontendControl,
                &transaction_id,
                OwnerSnapshot {
                    generation: 0,
                    sha256: receipt.before_sha256(),
                    callback_index: 0,
                },
                OwnerSnapshot {
                    generation: 1,
                    sha256: receipt.after_sha256(),
                    callback_index: 1,
                },
                &receipt,
            ),
            Err(CallbackOnlyRootError::OwnerReceiptMismatch(
                GtcOwnerId::FrontendControl
            ))
        );
        let other_transaction = CallbackTransactionId::new("local-callback-g1").unwrap();
        assert_eq!(
            validate_owner_transition(
                GtcOwnerId::ReaderEof,
                &other_transaction,
                OwnerSnapshot {
                    generation: 0,
                    sha256: receipt.before_sha256(),
                    callback_index: 0,
                },
                OwnerSnapshot {
                    generation: 1,
                    sha256: receipt.after_sha256(),
                    callback_index: 1,
                },
                &receipt,
            ),
            Err(CallbackOnlyRootError::OwnerReceiptMismatch(
                GtcOwnerId::ReaderEof
            ))
        );
        assert_eq!(
            validate_owner_transition(
                GtcOwnerId::ReaderEof,
                &transaction_id,
                OwnerSnapshot {
                    generation: 0,
                    sha256: [0xff; 32],
                    callback_index: 0,
                },
                OwnerSnapshot {
                    generation: 1,
                    sha256: receipt.after_sha256(),
                    callback_index: 1,
                },
                &receipt,
            ),
            Err(CallbackOnlyRootError::OwnerReceiptMismatch(
                GtcOwnerId::ReaderEof
            ))
        );
    }

    #[test]
    fn gtc_owner_receipt_helper_rejects_nontransitioning_states() {
        let transaction_id = CallbackTransactionId::new("local-callback-g0").unwrap();
        let bytes = b"same-state";
        let before = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::ReaderEof,
            0,
            [0x11; 32],
            bytes,
        )
        .unwrap();
        let bad_after = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::ReaderEof,
            1,
            [0x11; 32],
            bytes,
        )
        .unwrap();
        assert_eq!(
            GtcOwnerReceipt::from_transition(before, bad_after, transaction_id),
            Err(GtcOwnerReceiptError::GenerationMismatch)
        );
    }
}
