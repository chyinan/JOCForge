// pattern: Functional Core
//! Reader/EOF single-root lifecycle contracts.

use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderEofDigest([u8; 32]);

impl ReaderEofDigest {
    pub fn from_canonical_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut value = [0_u8; 32];
        value.copy_from_slice(&digest);
        Self(value)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Host38Frame(Arc<[f32]>);
impl Host38Frame {
    pub fn new(values: Vec<f32>) -> Result<Self, ReaderEofError> {
        if values.len() != 38 * 1536 {
            return Err(ReaderEofError::HostShape);
        }
        Ok(Self(values.into()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Control38Snapshot(Arc<[u32]>);
impl Control38Snapshot {
    pub fn new(values: Vec<u32>) -> Result<Self, ReaderEofError> {
        if values.len() != 38 * 13 {
            return Err(ReaderEofError::ControlShape);
        }
        Ok(Self(values.into()))
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderEofPhase {
    Unset = 0,
    Normal = 1,
    CachedDrain = 2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReaderEofState {
    callback_index: crate::timeline::FrontendCallbackIndex,
    source_cursor_samples: u64,
    normal_callbacks: crate::timeline::NormalCallbackCount,
    remaining_cached_drains: u32,
    last_normal_host38: Option<Host38Frame>,
    last_control_snapshot: Option<Control38Snapshot>,
    phase: ReaderEofPhase,
}

impl ReaderEofState {
    pub const fn cold() -> Self {
        Self {
            callback_index: crate::timeline::FrontendCallbackIndex::new(0),
            source_cursor_samples: 0,
            normal_callbacks: crate::timeline::NormalCallbackCount::new(0),
            remaining_cached_drains: 4,
            last_normal_host38: None,
            last_control_snapshot: None,
            phase: ReaderEofPhase::Unset,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        callback_index: crate::timeline::FrontendCallbackIndex,
        source_cursor_samples: u64,
        normal_callbacks: crate::timeline::NormalCallbackCount,
        remaining_cached_drains: u32,
        last_normal_host38: Option<Host38Frame>,
        last_control_snapshot: Option<Control38Snapshot>,
        phase: ReaderEofPhase,
    ) -> Result<Self, ReaderEofError> {
        if remaining_cached_drains > 4 {
            return Err(ReaderEofError::DrainRange);
        }
        Ok(Self {
            callback_index,
            source_cursor_samples,
            normal_callbacks,
            remaining_cached_drains,
            last_normal_host38,
            last_control_snapshot,
            phase,
        })
    }
    pub const fn callback_index(&self) -> crate::timeline::FrontendCallbackIndex {
        self.callback_index
    }
    pub const fn source_cursor_samples(&self) -> u64 {
        self.source_cursor_samples
    }
    pub const fn normal_callbacks(&self) -> crate::timeline::NormalCallbackCount {
        self.normal_callbacks
    }
    pub const fn remaining_cached_drains(&self) -> u32 {
        self.remaining_cached_drains
    }
    pub const fn last_normal_host38(&self) -> Option<&Host38Frame> {
        self.last_normal_host38.as_ref()
    }
    pub const fn last_control_snapshot(&self) -> Option<&Control38Snapshot> {
        self.last_control_snapshot.as_ref()
    }
    pub const fn phase(&self) -> ReaderEofPhase {
        self.phase
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        // The canonical state schema fixes this clock field at U64LE even
        // though FrontendCallbackIndex uses a compact U32 representation.
        out.extend_from_slice(&u64::from(self.callback_index.value()).to_le_bytes());
        out.extend_from_slice(&self.source_cursor_samples.to_le_bytes());
        out.extend_from_slice(&self.normal_callbacks.value().to_le_bytes());
        out.extend_from_slice(&self.remaining_cached_drains.to_le_bytes());
        match &self.last_normal_host38 {
            None => out.push(0),
            Some(frame) => {
                out.push(1);
                for value in frame.0.iter() {
                    out.extend_from_slice(&value.to_bits().to_le_bytes());
                }
            }
        }
        match &self.last_control_snapshot {
            None => out.push(0),
            Some(control) => {
                out.push(1);
                for value in control.0.iter() {
                    out.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        out.push(self.phase as u8);
        out
    }
    pub fn digest(&self) -> ReaderEofDigest {
        ReaderEofDigest::from_canonical_bytes(&self.canonical_bytes())
    }
}

/// Read-only GTC owner-state view over the complete canonical ReaderEof root.
///
/// `state_ref` names the logical member in the enclosing programme snapshot; it
/// does not replace the complete canonical bytes or grant parent commit authority.
#[derive(Debug, Eq, PartialEq)]
pub struct ReaderEofOwnerStateReference {
    generation: u64,
    state_sha256: ReaderEofDigest,
    cold_sha256: ReaderEofDigest,
    canonical_bytes: Vec<u8>,
}

impl ReaderEofOwnerStateReference {
    fn from_state(state: &ReaderEofState, generation: u64) -> Self {
        let canonical_bytes = state.canonical_bytes();
        let state_sha256 = ReaderEofDigest::from_canonical_bytes(&canonical_bytes);
        let cold_sha256 = ReaderEofState::cold().digest();
        Self {
            generation,
            state_sha256,
            cold_sha256,
            canonical_bytes,
        }
    }

    pub const fn owner_id(&self) -> &'static str {
        "reader_eof"
    }

    pub const fn state_ref(&self) -> &'static str {
        "owners.reader_eof"
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn state_sha256(&self) -> ReaderEofDigest {
        self.state_sha256
    }

    pub const fn cold_sha256(&self) -> ReaderEofDigest {
        self.cold_sha256
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReaderEofError {
    HostShape,
    ControlShape,
    DrainRange,
    EmptyTransactionId,
    GenerationMismatch,
    Retired,
}

/// Prepared capabilities are linear and cannot be cloned.
///
/// ```compile_fail
/// use jocforge_domain::state::{ReaderEofMachine, ReaderEofState};
/// let machine = ReaderEofMachine::cold();
/// let prepared = machine.prepare("tx", ReaderEofState::cold()).unwrap();
/// let _copy = prepared.clone();
/// ```
#[derive(Debug, PartialEq)]
pub struct ReaderEofPrepared {
    transaction_id: String,
    before_generation: u64,
    after: ReaderEofState,
}
/// Staged capabilities are linear and cannot be cloned.
///
/// ```compile_fail
/// use jocforge_domain::state::{ReaderEofMachine, ReaderEofState};
/// let machine = ReaderEofMachine::cold();
/// let prepared = machine.prepare("tx", ReaderEofState::cold()).unwrap();
/// let staged = machine.stage(prepared).unwrap();
/// let _copy = staged.clone();
/// ```
#[derive(Debug)]
pub struct ReaderEofStaged {
    transaction_id: String,
    before_generation: u64,
    before_digest: ReaderEofDigest,
    after: ReaderEofState,
    after_digest: ReaderEofDigest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderEofOwnerId {
    ReaderEof,
}
impl ReaderEofOwnerId {
    pub const fn as_str(self) -> &'static str {
        "reader_eof"
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderEofReceipt {
    owner_id: ReaderEofOwnerId,
    transaction_id: String,
    before_generation: u64,
    before_digest: ReaderEofDigest,
    after_generation: u64,
    after_digest: ReaderEofDigest,
    changed: bool,
    commit_count: u8,
}
impl ReaderEofReceipt {
    pub const fn owner_id(&self) -> ReaderEofOwnerId {
        self.owner_id
    }
    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }
    pub const fn before_digest(&self) -> ReaderEofDigest {
        self.before_digest
    }
    pub const fn after_generation(&self) -> u64 {
        self.after_generation
    }
    pub const fn after_digest(&self) -> ReaderEofDigest {
        self.after_digest
    }
    pub const fn changed(&self) -> bool {
        self.changed
    }
    pub const fn commit_count(&self) -> u8 {
        self.commit_count
    }
    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }
}

/// The committed owner is linear and cannot be cloned.
///
/// ```compile_fail
/// use jocforge_domain::state::ReaderEofMachine;
/// let machine = ReaderEofMachine::cold();
/// let _second_owner = machine.clone();
/// ```
#[derive(Debug)]
pub struct ReaderEofMachine {
    committed: ReaderEofState,
    generation: u64,
    retired: bool,
}
impl ReaderEofMachine {
    pub const fn cold() -> Self {
        Self {
            committed: ReaderEofState::cold(),
            generation: 0,
            retired: false,
        }
    }
    pub const fn committed(&self) -> &ReaderEofState {
        &self.committed
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn is_retired(&self) -> bool {
        self.retired
    }
    /// Returns the full committed owner state as a read-only GTC reference.
    pub fn state_reference(&self) -> ReaderEofOwnerStateReference {
        ReaderEofOwnerStateReference::from_state(&self.committed, self.generation)
    }
    pub fn prepare(
        &self,
        transaction_id: &str,
        after: ReaderEofState,
    ) -> Result<ReaderEofPrepared, ReaderEofError> {
        if self.retired {
            return Err(ReaderEofError::Retired);
        }
        if transaction_id.is_empty() {
            return Err(ReaderEofError::EmptyTransactionId);
        }
        let expected = self
            .committed
            .callback_index
            .value()
            .checked_add(1)
            .ok_or(ReaderEofError::GenerationMismatch)?;
        if after.callback_index.value() != expected {
            return Err(ReaderEofError::GenerationMismatch);
        }
        Ok(ReaderEofPrepared {
            transaction_id: transaction_id.to_owned(),
            before_generation: self.generation,
            after,
        })
    }
    pub fn stage(&self, prepared: ReaderEofPrepared) -> Result<ReaderEofStaged, ReaderEofError> {
        if self.retired {
            return Err(ReaderEofError::Retired);
        }
        if prepared.before_generation != self.generation {
            return Err(ReaderEofError::GenerationMismatch);
        }
        let before_digest = self.committed.digest();
        let after_digest = prepared.after.digest();
        Ok(ReaderEofStaged {
            transaction_id: prepared.transaction_id,
            before_generation: prepared.before_generation,
            before_digest,
            after: prepared.after,
            after_digest,
        })
    }
    fn validate_staged(&self, staged: &ReaderEofStaged) -> Result<(), ReaderEofError> {
        if self.retired {
            return Err(ReaderEofError::Retired);
        }
        if staged.before_generation != self.generation
            || staged.before_digest != self.committed.digest()
        {
            return Err(ReaderEofError::GenerationMismatch);
        }
        Ok(())
    }
    pub fn abort(&self, staged: ReaderEofStaged) -> Result<(), ReaderEofError> {
        let result = self.validate_staged(&staged);
        drop(staged);
        result
    }
    pub fn commit(&mut self, staged: ReaderEofStaged) -> Result<ReaderEofReceipt, ReaderEofError> {
        self.validate_staged(&staged)?;
        let after_generation = self
            .generation
            .checked_add(1)
            .ok_or(ReaderEofError::GenerationMismatch)?;
        self.committed = staged.after;
        self.generation = after_generation;
        Ok(ReaderEofReceipt {
            owner_id: ReaderEofOwnerId::ReaderEof,
            transaction_id: staged.transaction_id,
            before_generation: staged.before_generation,
            before_digest: staged.before_digest,
            after_generation,
            after_digest: staged.after_digest,
            changed: staged.before_digest != staged.after_digest,
            commit_count: 1,
        })
    }
    pub fn fatal(&mut self, staged: ReaderEofStaged) -> Result<(), ReaderEofError> {
        self.validate_staged(&staged)?;
        drop(staged);
        self.retired = true;
        Ok(())
    }
    #[must_use]
    pub fn full_restart(self) -> Self {
        drop(self);
        Self::cold()
    }
}
