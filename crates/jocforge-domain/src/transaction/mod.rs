// pattern: Functional Core
//! Pure transaction identities, GTC owner references and capability boundaries.
//! The full ProgrammeState commit coordinator is not implemented.

use crate::timeline::FrontendCallbackIndex;
use sha2::{Digest, Sha256};

mod callback_only_root;
mod programme_state_availability;
mod source_transaction;

pub use callback_only_root::{
    CallbackOnlyCandidates, CallbackOnlyOwnerProjection, CallbackOnlyOwners, CallbackOnlyRootError,
    CallbackOnlyRootProposal, CallbackOnlyRootStatus, CallbackPhaseGeneration,
    CallbackProgrammeIdentity,
};
pub use programme_state_availability::{
    MeasurementReadyAvailabilityProposal, ProgrammeMainColdOwnerSnapshots,
    ProgrammeOwnerAvailability, ProgrammeOwnerConstructionReceipt, ProgrammeOwnerStateReference,
    ProgrammeOwnersAvailability, ProgrammeStateAvailabilityProjection, ProgrammeStateV3Error,
    ProgrammeStatus, ProgrammeTransactionIdentity,
};

#[allow(unused_imports)]
// Parent source stage is not yet called by the top-level callback root.
pub(crate) use source_transaction::{SourceTransaction, SourceTransactionStageError};

/// Opaque caller-supplied nonempty programme identity. It remains stable across restart attempts.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProgrammeId(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgrammeIdError {
    Empty,
}

impl ProgrammeId {
    pub fn new(value: &str) -> Result<Self, ProgrammeIdError> {
        if value.is_empty() {
            return Err(ProgrammeIdError::Empty);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Caller-owned restart attempt. Its owner increments it only after a complete cold reset.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AttemptId(u64);

impl AttemptId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// P1.3 callback receipt identifier, retained as a caller supplied string until the coordinator slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackTransactionId(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackTransactionIdError {
    Empty,
}

impl CallbackTransactionId {
    pub fn new(value: &str) -> Result<Self, CallbackTransactionIdError> {
        let value = value.to_owned();
        if value.is_empty() {
            return Err(CallbackTransactionIdError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Typed selected-chain join identity for one callback in one programme attempt.
///
/// This is only an internal equality/join value. It is not a GTC transaction ID or ABI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackLineageIdentity {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    callback_index: FrontendCallbackIndex,
}

impl CallbackLineageIdentity {
    pub const fn new(
        programme_id: ProgrammeId,
        attempt_id: AttemptId,
        callback_index: FrontendCallbackIndex,
    ) -> Self {
        Self {
            programme_id,
            attempt_id,
            callback_index,
        }
    }

    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }
}

/// Registered owners in the selected global transaction contract.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GtcOwnerId {
    ReaderEof,
    FrontendControl,
    SpatialControl,
    SpatialAudio,
    Joc,
    Meter,
    Drc,
    Analysis,
    Codec,
    Metadata,
    Integrity,
    Output,
}

impl GtcOwnerId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReaderEof => "reader_eof",
            Self::FrontendControl => "frontend_control",
            Self::SpatialControl => "spatial_control",
            Self::SpatialAudio => "spatial_audio",
            Self::Joc => "joc",
            Self::Meter => "meter",
            Self::Drc => "drc",
            Self::Analysis => "analysis",
            Self::Codec => "codec",
            Self::Metadata => "metadata",
            Self::Integrity => "integrity",
            Self::Output => "output",
        }
    }

    pub const fn state_ref(self) -> &'static str {
        match self {
            Self::ReaderEof => "owners.reader_eof",
            Self::FrontendControl => "owners.frontend_control",
            Self::SpatialControl => "owners.spatial_control",
            Self::SpatialAudio => "owners.spatial_audio",
            Self::Joc => "owners.joc",
            Self::Meter => "owners.meter",
            Self::Drc => "owners.drc",
            Self::Analysis => "owners.analysis",
            Self::Codec => "owners.codec",
            Self::Metadata => "owners.metadata",
            Self::Integrity => "owners.integrity",
            Self::Output => "owners.output",
        }
    }
}

/// Logical member reference derived only from a registered GTC owner ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnerStateRef {
    owner_id: GtcOwnerId,
}

impl OwnerStateRef {
    pub const fn from_owner(owner_id: GtcOwnerId) -> Self {
        Self { owner_id }
    }

    pub const fn owner_id(self) -> GtcOwnerId {
        self.owner_id
    }

    pub const fn as_str(self) -> &'static str {
        self.owner_id.state_ref()
    }
}

/// Canonical bytes and identities for one registered owner at one committed or staged point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GtcOwnerStateSnapshot<'a> {
    owner_id: GtcOwnerId,
    state_ref: OwnerStateRef,
    generation: u64,
    state_sha256: [u8; 32],
    cold_sha256: [u8; 32],
    canonical_bytes: &'a [u8],
}

impl<'a> GtcOwnerStateSnapshot<'a> {
    /// Recomputes the state digest from the complete canonical owner bytes.
    pub fn from_canonical_bytes(
        owner_id: GtcOwnerId,
        generation: u64,
        cold_sha256: [u8; 32],
        canonical_bytes: &'a [u8],
    ) -> Result<Self, GtcOwnerReceiptError> {
        if canonical_bytes.is_empty() {
            return Err(GtcOwnerReceiptError::EmptyCanonicalState);
        }
        Ok(Self {
            owner_id,
            state_ref: OwnerStateRef::from_owner(owner_id),
            generation,
            state_sha256: Sha256::digest(canonical_bytes).into(),
            cold_sha256,
            canonical_bytes,
        })
    }

    pub const fn owner_id(self) -> GtcOwnerId {
        self.owner_id
    }

    pub const fn state_ref(self) -> OwnerStateRef {
        self.state_ref
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn state_sha256(self) -> [u8; 32] {
        self.state_sha256
    }

    pub const fn cold_sha256(self) -> [u8; 32] {
        self.cold_sha256
    }

    pub const fn canonical_bytes(self) -> &'a [u8] {
        self.canonical_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GtcOwnerReceiptError {
    EmptyCanonicalState,
    OwnerIdentityMismatch,
    ColdRootMismatch,
    GenerationMismatch,
    GenerationOverflow,
}

/// One GTC owner transition receipt. It describes a stage; it does not commit a programme root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GtcOwnerReceipt {
    owner_id: GtcOwnerId,
    before_generation: u64,
    before_sha256: [u8; 32],
    after_generation: u64,
    after_sha256: [u8; 32],
    changed: bool,
    commit_count: u8,
    transaction_id: CallbackTransactionId,
}

impl GtcOwnerReceipt {
    pub fn from_transition(
        before: GtcOwnerStateSnapshot<'_>,
        after: GtcOwnerStateSnapshot<'_>,
        transaction_id: CallbackTransactionId,
    ) -> Result<Self, GtcOwnerReceiptError> {
        if before.owner_id != after.owner_id || before.state_ref != after.state_ref {
            return Err(GtcOwnerReceiptError::OwnerIdentityMismatch);
        }
        if before.cold_sha256 != after.cold_sha256 {
            return Err(GtcOwnerReceiptError::ColdRootMismatch);
        }
        let changed = before.state_sha256 != after.state_sha256
            || before.canonical_bytes != after.canonical_bytes;
        let expected_after = if changed {
            before
                .generation
                .checked_add(1)
                .ok_or(GtcOwnerReceiptError::GenerationOverflow)?
        } else {
            before.generation
        };
        if after.generation != expected_after {
            return Err(GtcOwnerReceiptError::GenerationMismatch);
        }
        Ok(Self {
            owner_id: before.owner_id,
            before_generation: before.generation,
            before_sha256: before.state_sha256,
            after_generation: after.generation,
            after_sha256: after.state_sha256,
            changed,
            commit_count: u8::from(changed),
            transaction_id,
        })
    }

    pub const fn owner_id(&self) -> GtcOwnerId {
        self.owner_id
    }

    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }

    pub const fn before_sha256(&self) -> [u8; 32] {
        self.before_sha256
    }

    pub const fn after_generation(&self) -> u64 {
        self.after_generation
    }

    pub const fn after_sha256(&self) -> [u8; 32] {
        self.after_sha256
    }

    pub const fn changed(&self) -> bool {
        self.changed
    }

    pub const fn commit_count(&self) -> u8 {
        self.commit_count
    }

    pub fn transaction_id(&self) -> &CallbackTransactionId {
        &self.transaction_id
    }
}

#[cfg(test)]
pub(crate) fn test_programme_id(label: &str) -> ProgrammeId {
    ProgrammeId::new(label).expect("test programme labels are nonempty")
}

#[cfg(test)]
mod owner_state_ref_tests {
    use super::{
        CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt, GtcOwnerReceiptError,
        GtcOwnerStateSnapshot, OwnerStateRef,
    };
    use sha2::{Digest, Sha256};

    #[test]
    fn each_registered_gtc_owner_derives_its_exact_state_ref() {
        let owners = [
            (GtcOwnerId::ReaderEof, "reader_eof", "owners.reader_eof"),
            (
                GtcOwnerId::FrontendControl,
                "frontend_control",
                "owners.frontend_control",
            ),
            (
                GtcOwnerId::SpatialControl,
                "spatial_control",
                "owners.spatial_control",
            ),
            (
                GtcOwnerId::SpatialAudio,
                "spatial_audio",
                "owners.spatial_audio",
            ),
            (GtcOwnerId::Joc, "joc", "owners.joc"),
            (GtcOwnerId::Meter, "meter", "owners.meter"),
            (GtcOwnerId::Drc, "drc", "owners.drc"),
            (GtcOwnerId::Analysis, "analysis", "owners.analysis"),
            (GtcOwnerId::Codec, "codec", "owners.codec"),
            (GtcOwnerId::Metadata, "metadata", "owners.metadata"),
            (GtcOwnerId::Integrity, "integrity", "owners.integrity"),
            (GtcOwnerId::Output, "output", "owners.output"),
        ];

        for (owner_id, expected_id, expected_state_ref) in owners {
            let state_ref = OwnerStateRef::from_owner(owner_id);
            assert_eq!(owner_id.as_str(), expected_id);
            assert_eq!(state_ref.owner_id(), owner_id);
            assert_eq!(state_ref.as_str(), expected_state_ref);
        }
    }

    #[test]
    fn owner_receipt_rehashes_complete_bytes_and_enforces_generation_commit_count() {
        let cold = [9_u8; 32];
        let before = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::SpatialControl,
            4,
            cold,
            b"complete-before-state",
        )
        .unwrap();
        let after = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::SpatialControl,
            5,
            cold,
            b"complete-after-state",
        )
        .unwrap();
        let transaction_id = CallbackTransactionId::new("callback-g4").unwrap();
        let receipt =
            GtcOwnerReceipt::from_transition(before, after, transaction_id.clone()).unwrap();
        assert_eq!(receipt.owner_id(), GtcOwnerId::SpatialControl);
        assert_eq!(receipt.before_generation(), 4);
        assert_eq!(receipt.after_generation(), 5);
        assert!(receipt.changed());
        assert_eq!(receipt.commit_count(), 1);
        assert_eq!(receipt.transaction_id(), &transaction_id);
        let before_sha256: [u8; 32] = Sha256::digest(b"complete-before-state").into();
        let after_sha256: [u8; 32] = Sha256::digest(b"complete-after-state").into();
        assert_eq!(before.state_sha256(), before_sha256);
        assert_eq!(after.state_sha256(), after_sha256);
    }

    #[test]
    fn unchanged_owner_receipt_keeps_generation_and_zero_commit_count() {
        let cold = [7_u8; 32];
        let before = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::Meter,
            3,
            cold,
            b"complete-meter-state",
        )
        .unwrap();
        let after = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::Meter,
            3,
            cold,
            b"complete-meter-state",
        )
        .unwrap();
        let receipt = GtcOwnerReceipt::from_transition(
            before,
            after,
            CallbackTransactionId::new("callback-g2").unwrap(),
        )
        .unwrap();
        assert!(!receipt.changed());
        assert_eq!(receipt.commit_count(), 0);
        assert_eq!(receipt.before_sha256(), receipt.after_sha256());
    }

    #[test]
    fn owner_receipt_rejects_cold_generation_and_empty_state_mismatches() {
        let bytes = b"state";
        let before = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::FrontendControl,
            0,
            [1; 32],
            bytes,
        )
        .unwrap();
        let after_cold = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::FrontendControl,
            1,
            [2; 32],
            b"next",
        )
        .unwrap();
        assert_eq!(
            GtcOwnerReceipt::from_transition(
                before,
                after_cold,
                CallbackTransactionId::new("tx").unwrap()
            )
            .unwrap_err(),
            GtcOwnerReceiptError::ColdRootMismatch
        );
        let after_skipped = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::FrontendControl,
            2,
            [1; 32],
            b"next",
        )
        .unwrap();
        assert_eq!(
            GtcOwnerReceipt::from_transition(
                before,
                after_skipped,
                CallbackTransactionId::new("tx").unwrap()
            )
            .unwrap_err(),
            GtcOwnerReceiptError::GenerationMismatch
        );
        assert_eq!(
            GtcOwnerStateSnapshot::from_canonical_bytes(GtcOwnerId::Codec, 0, [0; 32], b"")
                .unwrap_err(),
            GtcOwnerReceiptError::EmptyCanonicalState
        );
    }
}
