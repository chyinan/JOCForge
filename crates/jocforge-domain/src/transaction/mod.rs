// pattern: Functional Core
//! Pure transaction identities, GTC owner references and capability boundaries.
//! No coordinator state is implemented.

use crate::timeline::FrontendCallbackIndex;

mod source_transaction;

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

#[cfg(test)]
pub(crate) fn test_programme_id(label: &str) -> ProgrammeId {
    ProgrammeId::new(label).expect("test programme labels are nonempty")
}

#[cfg(test)]
mod owner_state_ref_tests {
    use super::{GtcOwnerId, OwnerStateRef};

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
}
