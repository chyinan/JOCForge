// pattern: Functional Core
//! Pure transaction identities and capability boundaries. No coordinator state is implemented.

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

#[cfg(test)]
pub(crate) fn test_programme_id(label: &str) -> ProgrammeId {
    ProgrammeId::new(label).expect("test programme labels are nonempty")
}
