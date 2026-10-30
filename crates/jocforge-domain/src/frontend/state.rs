// pattern: Functional Core
//! Staging-only frontend-control root for the approved cold provider.

use super::ProviderSha256;
use super::normal_control_snapshot::{
    NormalControlAfterState, NormalControlAfterStateParts, NormalControlObjectState,
    NormalControlPreparedSnapshot, NormalControlSnapshotOutput, NormalControlSnapshotProposal,
};
use super::provider::{F01EProviderReceipt, NormalizedConfig, ValidatedF01EColdProvider};
use crate::timeline::{FrontendCallbackIndex, NormalCallbackCount};
use crate::transaction::{
    AttemptId, CallbackLineageIdentity, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt,
    GtcOwnerReceiptError, GtcOwnerStateSnapshot, ProgrammeId,
};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

const FRONTEND_CONTROL_OWNER_ID: &str = "frontend_control";
const FRONTEND_CONTROL_STATE_REF: &str = "owners.frontend_control";
const FRONTEND_CONTROL_AFTER_STATE_SCHEMA: &str = "FRONTEND_CONTROL_AFTER_STATE_V1";
const FRONTEND_CONTROL_ROWS: usize = 38;
const FRONTEND_CONTROL_REGULAR_ROWS: usize = 37;
const FRONTEND_CONTROL_WORDS_PER_ROW: usize = 13;
const FRONTEND_OBJECTS: usize = 28;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontendControlDigest(ProviderSha256);
impl FrontendControlDigest {
    pub fn to_hex(self) -> String {
        self.0.to_hex()
    }

    pub const fn to_bytes(self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontendControlPhase {
    BeforeFirstNormal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontendObjectState {
    object_ordinal: u32,
    event_cache: Option<Arc<[u8]>>,
    host_channel_index: Option<u32>,
    sampler_previous_present: bool,
    sampler_previous_xyz_u32: [u32; 3],
}
impl FrontendObjectState {
    pub const fn object_ordinal(&self) -> u32 {
        self.object_ordinal
    }
    pub const fn event_cache(&self) -> Option<&Arc<[u8]>> {
        self.event_cache.as_ref()
    }
    pub const fn host_channel_index(&self) -> Option<u32> {
        self.host_channel_index
    }
    pub const fn sampler_previous_present(&self) -> bool {
        self.sampler_previous_present
    }
    pub const fn sampler_previous_xyz_u32(&self) -> [u32; 3] {
        self.sampler_previous_xyz_u32
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontendNormalizedConfig {
    sample_rate_hz: u32,
    channels: u32,
    pcm_bits: u32,
    source_samples: u64,
    bit_rate_kbps: u32,
    bit_rate_enum: u32,
    layout_enum: u32,
    target_capacity: u32,
    frame_samples: u32,
    spatial_config_mode: u32,
    dispatch_mode: u32,
    virtual_prefix_count: u32,
    bed_count: u32,
    object_count: u32,
    event_start_samples: u64,
    line_drc: Arc<str>,
    rf_drc: Arc<str>,
    time_base: Arc<str>,
}
impl From<NormalizedConfig> for FrontendNormalizedConfig {
    fn from(value: NormalizedConfig) -> Self {
        Self {
            sample_rate_hz: value.sample_rate_hz,
            channels: value.channels,
            pcm_bits: value.pcm_bits,
            source_samples: value.source_samples,
            bit_rate_kbps: value.bit_rate_kbps,
            bit_rate_enum: value.bit_rate_enum,
            layout_enum: value.layout_enum,
            target_capacity: value.target_capacity,
            frame_samples: value.frame_samples,
            spatial_config_mode: value.spatial_config_mode,
            dispatch_mode: value.dispatch_mode,
            virtual_prefix_count: value.virtual_prefix_count,
            bed_count: value.bed_count,
            object_count: value.object_count,
            event_start_samples: value.event_start_samples,
            line_drc: value.line_drc.into(),
            rf_drc: value.rf_drc.into(),
            time_base: value.time_base.into(),
        }
    }
}
impl FrontendNormalizedConfig {
    pub const fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }
    pub const fn channels(&self) -> u32 {
        self.channels
    }
    pub const fn pcm_bits(&self) -> u32 {
        self.pcm_bits
    }
    pub const fn source_samples(&self) -> u64 {
        self.source_samples
    }
    pub const fn frame_samples(&self) -> u32 {
        self.frame_samples
    }
    pub const fn bit_rate_kbps(&self) -> u32 {
        self.bit_rate_kbps
    }
    pub const fn bit_rate_enum(&self) -> u32 {
        self.bit_rate_enum
    }
    pub const fn layout_enum(&self) -> u32 {
        self.layout_enum
    }
    pub const fn target_capacity(&self) -> u32 {
        self.target_capacity
    }
    pub const fn spatial_config_mode(&self) -> u32 {
        self.spatial_config_mode
    }
    pub const fn dispatch_mode(&self) -> u32 {
        self.dispatch_mode
    }
    pub const fn virtual_prefix_count(&self) -> u32 {
        self.virtual_prefix_count
    }
    pub const fn bed_count(&self) -> u32 {
        self.bed_count
    }
    pub const fn object_count(&self) -> u32 {
        self.object_count
    }
    pub const fn event_start_samples(&self) -> u64 {
        self.event_start_samples
    }
    pub fn line_drc(&self) -> &str {
        &self.line_drc
    }
    pub fn rf_drc(&self) -> &str {
        &self.rf_drc
    }
    pub fn time_base(&self) -> &str {
        &self.time_base
    }
}

#[derive(Clone, Debug)]
pub struct FrontendControlState {
    schema: &'static str,
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    owner_generation: u64,
    callback_index: FrontendCallbackIndex,
    normal_callbacks: NormalCallbackCount,
    remaining_cached_drains: u32,
    phase: FrontendControlPhase,
    objects: Arc<[FrontendObjectState]>,
    last_control38_snapshot: Option<Arc<[u32]>>,
    last_regular37_snapshot: Option<Arc<[u32]>>,
    dbmd_binding: Option<Arc<[u8]>>,
    physical_binding: Option<Arc<[u8]>>,
    normalized_config: Arc<FrontendNormalizedConfig>,
    prepared_identity: Option<Arc<NormalControlPreparedSnapshot>>,
    canonical: Arc<[u8]>,
    provider_receipt: F01EProviderReceipt,
}
impl FrontendControlState {
    fn from_provider(
        provider: ValidatedF01EColdProvider,
        programme_id: ProgrammeId,
        attempt_id: AttemptId,
    ) -> Result<Self, FrontendControlError> {
        let (cold, canonical, provider_receipt) = provider.into_parts();
        if ProviderSha256::digest(&canonical) != provider_receipt.canonical_sha256() {
            return Err(FrontendControlError::ProviderMismatch);
        }
        let objects = cold
            .objects
            .into_iter()
            .map(|object| FrontendObjectState {
                object_ordinal: object.ordinal,
                event_cache: None,
                host_channel_index: None,
                sampler_previous_present: false,
                sampler_previous_xyz_u32: [0; 3],
            })
            .collect::<Vec<_>>()
            .into();
        let callback_index = u32::try_from(cold.callback_index)
            .map(FrontendCallbackIndex::new)
            .map_err(|_| FrontendControlError::CallbackIndexOutOfRange)?;
        Ok(Self {
            schema: "F01E_SELECTED_COLD_V1",
            programme_id,
            attempt_id,
            owner_generation: 0,
            callback_index,
            normal_callbacks: NormalCallbackCount::new(cold.normal_callbacks),
            remaining_cached_drains: cold.remaining_cached_drains,
            phase: FrontendControlPhase::BeforeFirstNormal,
            objects,
            last_control38_snapshot: None,
            last_regular37_snapshot: None,
            dbmd_binding: None,
            physical_binding: None,
            normalized_config: Arc::new(cold.config.into()),
            prepared_identity: None,
            canonical,
            provider_receipt,
        })
    }

    fn from_normal_after_parts(
        before: &Self,
        parts: NormalControlAfterStateParts,
    ) -> Result<Self, FrontendControlError> {
        if !parts.prepared_identity.binds_frontend_state(before)
            || !parts.prepared_identity.binds_normal_objects(&parts.objects)
            || parts.control38_snapshot.len()
                != FRONTEND_CONTROL_ROWS * FRONTEND_CONTROL_WORDS_PER_ROW
            || parts.regular37_snapshot.len()
                != FRONTEND_CONTROL_REGULAR_ROWS * FRONTEND_CONTROL_WORDS_PER_ROW
            || parts.objects.len() != FRONTEND_OBJECTS
        {
            return Err(FrontendControlError::NormalAfterStateMismatch);
        }
        let objects: Arc<[FrontendObjectState]> = parts
            .objects
            .into_iter()
            .map(frontend_object_from_normal_control)
            .collect::<Vec<_>>()
            .into();
        let mut state = Self {
            schema: FRONTEND_CONTROL_AFTER_STATE_SCHEMA,
            programme_id: parts.programme_id,
            attempt_id: parts.attempt_id,
            owner_generation: parts.next_owner_generation,
            callback_index: parts.next_callback_index,
            normal_callbacks: parts.next_normal_callbacks,
            remaining_cached_drains: parts.remaining_cached_drains,
            phase: parts.phase,
            objects,
            last_control38_snapshot: Some(parts.control38_snapshot),
            last_regular37_snapshot: Some(parts.regular37_snapshot),
            dbmd_binding: parts.dbmd_binding_before,
            physical_binding: parts.physical_binding_before,
            normalized_config: Arc::clone(&before.normalized_config),
            prepared_identity: Some(parts.prepared_identity),
            canonical: Arc::from([]),
            provider_receipt: before.provider_receipt.clone(),
        };
        let canonical = encode_frontend_control_after_state(&state)?;
        state.canonical = Arc::from(canonical.into_boxed_slice());
        Ok(state)
    }

    pub const fn schema(&self) -> &'static str {
        self.schema
    }
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }
    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }
    pub const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }
    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        self.normal_callbacks
    }
    pub const fn remaining_cached_drains(&self) -> u32 {
        self.remaining_cached_drains
    }
    pub const fn phase(&self) -> FrontendControlPhase {
        self.phase
    }
    pub fn objects(&self) -> &[FrontendObjectState] {
        &self.objects
    }
    pub const fn last_control38_snapshot(&self) -> Option<&Arc<[u32]>> {
        self.last_control38_snapshot.as_ref()
    }
    pub const fn last_regular37_snapshot(&self) -> Option<&Arc<[u32]>> {
        self.last_regular37_snapshot.as_ref()
    }
    pub const fn dbmd_binding(&self) -> Option<&Arc<[u8]>> {
        self.dbmd_binding.as_ref()
    }
    pub const fn physical_binding(&self) -> Option<&Arc<[u8]>> {
        self.physical_binding.as_ref()
    }
    pub fn normalized_config(&self) -> &FrontendNormalizedConfig {
        &self.normalized_config
    }
    pub fn prepared_identity(&self) -> Option<&NormalControlPreparedSnapshot> {
        self.prepared_identity.as_deref()
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn digest(&self) -> FrontendControlDigest {
        FrontendControlDigest(ProviderSha256::digest(&self.canonical))
    }
}

/// Read-only state-reference view for the complete cold frontend-control owner.
///
/// The currently materialized machine has no commit API and its stage path only
/// clones this state. The view therefore reports the current cold owner bytes;
/// it does not claim a warmed normal-control producer or GTC receipt.
#[derive(Clone, Copy, Debug)]
pub struct FrontendControlOwnerStateReference<'a> {
    state: &'a FrontendControlState,
    generation: u64,
    state_sha256: FrontendControlDigest,
    cold_sha256: FrontendControlDigest,
}

impl FrontendControlOwnerStateReference<'_> {
    pub const fn owner_id(&self) -> &'static str {
        FRONTEND_CONTROL_OWNER_ID
    }

    pub const fn state_ref(&self) -> &'static str {
        FRONTEND_CONTROL_STATE_REF
    }

    pub const fn state(&self) -> &FrontendControlState {
        self.state
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn state_sha256(&self) -> FrontendControlDigest {
        self.state_sha256
    }

    pub const fn cold_sha256(&self) -> FrontendControlDigest {
        self.cold_sha256
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        self.state.canonical_bytes()
    }

    /// Adapts the complete owner reference to the registered GTC state member.
    pub fn gtc_snapshot(&self) -> Result<GtcOwnerStateSnapshot<'_>, GtcOwnerReceiptError> {
        GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::FrontendControl,
            self.generation,
            self.cold_sha256.to_bytes(),
            self.canonical_bytes(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrontendControlError {
    CallbackIdentityMismatch,
    CallbackIndexMismatch,
    CallbackIndexOutOfRange,
    GenerationMismatch,
    NormalAfterStateMismatch,
    ProviderMismatch,
    CanonicalEncoding,
    GtcReceipt(GtcOwnerReceiptError),
}

/// Linear prepare capability; intentionally not `Clone` or `Copy`.
///
/// ```compile_fail
/// # use jocforge_domain::frontend::FrontendControlPrepared;
/// fn duplicate(value: FrontendControlPrepared) { let _ = value.clone(); }
/// ```
#[derive(Debug)]
pub struct FrontendControlPrepared {
    transaction_id: CallbackLineageIdentity,
    before_generation: u64,
    before_digest: FrontendControlDigest,
    provider_manifest_sha256: ProviderSha256,
    provider_file_sha256: ProviderSha256,
}

/// Opaque linear staging capability for the future cross-root coordinator.
///
/// ```compile_fail
/// # use jocforge_domain::frontend::FrontendControlStaged;
/// fn duplicate(value: FrontendControlStaged) { let _ = value.clone(); }
/// ```
#[derive(Debug)]
pub struct FrontendControlStaged {
    join: FrontendControlJoinMetadata,
    after: FrontendControlState,
    after_digest: FrontendControlDigest,
    after_generation: u64,
}
impl FrontendControlStaged {
    pub const fn join_metadata(&self) -> &FrontendControlJoinMetadata {
        &self.join
    }
    pub const fn after_digest(&self) -> FrontendControlDigest {
        self.after_digest
    }
    pub fn staged_state_digest(&self) -> FrontendControlDigest {
        self.after.digest()
    }
    pub const fn after_generation(&self) -> u64 {
        self.after_generation
    }
    pub const fn after_state(&self) -> &FrontendControlState {
        &self.after
    }
}

/// Complete replacement value for the parent-owned frontend-control slot.
/// It carries no authority to mutate the committed machine that produced it.
#[must_use]
#[derive(Debug)]
pub struct FrontendControlReplacementCandidate {
    next_machine: FrontendControlMachine,
    receipt: GtcOwnerReceipt,
}

/// Linear parent handoff containing both the next frontend owner and the same-callback F03W view.
/// The projection shares the proposal's computed values; it does not resample objects.
#[must_use]
#[derive(Debug)]
pub struct FrontendControlNormalStage {
    staged: FrontendControlStaged,
    output: NormalControlSnapshotOutput,
}

impl FrontendControlNormalStage {
    pub const fn staged_owner(&self) -> &FrontendControlStaged {
        &self.staged
    }

    pub const fn callback_output(&self) -> &NormalControlSnapshotOutput {
        &self.output
    }

    pub fn into_parts(self) -> (FrontendControlStaged, NormalControlSnapshotOutput) {
        (self.staged, self.output)
    }
}

impl FrontendControlReplacementCandidate {
    pub const fn next_machine(&self) -> &FrontendControlMachine {
        &self.next_machine
    }

    pub const fn receipt(&self) -> &GtcOwnerReceipt {
        &self.receipt
    }

    pub fn into_parts(self) -> (FrontendControlMachine, GtcOwnerReceipt) {
        (self.next_machine, self.receipt)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontendControlJoinMetadata {
    transaction_id: CallbackLineageIdentity,
    before_generation: u64,
    before_digest: FrontendControlDigest,
    provider_manifest_sha256: ProviderSha256,
    provider_file_sha256: ProviderSha256,
}
impl FrontendControlJoinMetadata {
    pub const fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.transaction_id.callback_index()
    }
    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }
    pub const fn before_digest(&self) -> FrontendControlDigest {
        self.before_digest
    }
    pub const fn provider_manifest_sha256(&self) -> ProviderSha256 {
        self.provider_manifest_sha256
    }
    pub const fn provider_file_sha256(&self) -> ProviderSha256 {
        self.provider_file_sha256
    }
}

/// Single owner of committed frontend-control state. It has no standalone commit API.
///
/// ```compile_fail
/// # use jocforge_domain::frontend::FrontendControlMachine;
/// fn duplicate(value: FrontendControlMachine) { let _ = value.clone(); }
/// ```
///
/// ```compile_fail
/// # use jocforge_domain::frontend::{FrontendControlMachine, FrontendControlStaged};
/// fn standalone_commit(machine: &mut FrontendControlMachine, staged: FrontendControlStaged) {
///     machine.commit(staged);
/// }
/// ```
#[derive(Debug)]
pub struct FrontendControlMachine {
    committed: FrontendControlState,
    generation: u64,
    cold_digest: FrontendControlDigest,
}
impl FrontendControlMachine {
    pub fn from_provider(
        provider: ValidatedF01EColdProvider,
        programme_id: ProgrammeId,
        attempt_id: AttemptId,
    ) -> Result<Self, FrontendControlError> {
        let committed = FrontendControlState::from_provider(provider, programme_id, attempt_id)?;
        let cold_digest = committed.digest();
        Ok(Self {
            committed,
            generation: 0,
            cold_digest,
        })
    }
    pub const fn committed(&self) -> &FrontendControlState {
        &self.committed
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    /// Returns the complete current state reference without changing owner state.
    pub fn state_reference(&self) -> FrontendControlOwnerStateReference<'_> {
        FrontendControlOwnerStateReference {
            state: &self.committed,
            generation: self.generation,
            state_sha256: self.committed.digest(),
            cold_sha256: self.cold_digest,
        }
    }
    pub fn prepare(
        &self,
        transaction_id: CallbackLineageIdentity,
        before_generation: u64,
        provider_receipt: &F01EProviderReceipt,
    ) -> Result<FrontendControlPrepared, FrontendControlError> {
        if transaction_id.programme_id() != &self.committed.programme_id
            || transaction_id.attempt_id() != self.committed.attempt_id
        {
            return Err(FrontendControlError::CallbackIdentityMismatch);
        }
        let callback_index = transaction_id.callback_index();
        if callback_index != self.committed.callback_index {
            return Err(FrontendControlError::CallbackIndexMismatch);
        }
        if before_generation != self.generation {
            return Err(FrontendControlError::GenerationMismatch);
        }
        if provider_receipt.manifest_sha256() != self.committed.provider_receipt.manifest_sha256()
            || provider_receipt.provider_file_sha256()
                != self.committed.provider_receipt.provider_file_sha256()
            || provider_receipt.canonical_sha256()
                != self.committed.provider_receipt.canonical_sha256()
        {
            return Err(FrontendControlError::ProviderMismatch);
        }
        Ok(FrontendControlPrepared {
            transaction_id,
            before_generation,
            before_digest: self.committed.digest(),
            provider_manifest_sha256: provider_receipt.manifest_sha256(),
            provider_file_sha256: provider_receipt.provider_file_sha256(),
        })
    }
    // Consumes the one-shot prepared capability even though its identity fields are Copy.
    #[allow(clippy::needless_pass_by_value)]
    pub fn stage(
        &self,
        prepared: FrontendControlPrepared,
    ) -> Result<FrontendControlStaged, FrontendControlError> {
        if prepared.before_generation != self.generation
            || prepared.before_digest != self.committed.digest()
        {
            return Err(FrontendControlError::GenerationMismatch);
        }
        if prepared.transaction_id.programme_id() != &self.committed.programme_id
            || prepared.transaction_id.attempt_id() != self.committed.attempt_id
        {
            return Err(FrontendControlError::CallbackIdentityMismatch);
        }
        if prepared.transaction_id.callback_index() != self.committed.callback_index
            || prepared.provider_manifest_sha256
                != self.committed.provider_receipt.manifest_sha256()
            || prepared.provider_file_sha256
                != self.committed.provider_receipt.provider_file_sha256()
        {
            return Err(FrontendControlError::ProviderMismatch);
        }
        // This provider-identity stage is a no-op snapshot only. It is not an F01E
        // normal callback transition; use `stage_normal_snapshot` for callback after-state.
        let after = self.committed.clone();
        let after_digest = after.digest();
        Ok(FrontendControlStaged {
            join: FrontendControlJoinMetadata {
                transaction_id: prepared.transaction_id,
                before_generation: prepared.before_generation,
                before_digest: prepared.before_digest,
                provider_manifest_sha256: prepared.provider_manifest_sha256,
                provider_file_sha256: prepared.provider_file_sha256,
            },
            after,
            after_digest,
            after_generation: self.generation,
        })
    }

    /// Converts the complete typed normal-callback after-state into the frontend owner’s
    /// versioned local canonical representation. The current owner remains unchanged.
    fn stage_normal_after_state(
        &self,
        transaction_id: CallbackLineageIdentity,
        after_state: NormalControlAfterState,
    ) -> Result<FrontendControlStaged, FrontendControlError> {
        if transaction_id.programme_id() != &self.committed.programme_id
            || transaction_id.attempt_id() != self.committed.attempt_id
        {
            return Err(FrontendControlError::CallbackIdentityMismatch);
        }
        if transaction_id.callback_index() != self.committed.callback_index {
            return Err(FrontendControlError::CallbackIndexMismatch);
        }
        let parts = after_state.into_frontend_owner_parts();
        let expected_callback = self
            .committed
            .callback_index
            .value()
            .checked_add(1)
            .ok_or(FrontendControlError::CallbackIndexOutOfRange)?;
        let expected_generation = self
            .generation
            .checked_add(1)
            .ok_or(FrontendControlError::GenerationMismatch)?;
        if parts.programme_id != self.committed.programme_id
            || parts.attempt_id != self.committed.attempt_id
            || parts.next_callback_index.value() != expected_callback
            || parts.next_owner_generation != expected_generation
            || parts.phase != self.committed.phase
            || parts.remaining_cached_drains != self.committed.remaining_cached_drains
            || parts.physical_binding_before != self.committed.physical_binding
            || parts.dbmd_binding_before != self.committed.dbmd_binding
            || !parts
                .prepared_identity
                .binds_frontend_state(&self.committed)
            || !parts.prepared_identity.binds_normal_objects(&parts.objects)
            || parts.control38_snapshot.len()
                != FRONTEND_CONTROL_ROWS * FRONTEND_CONTROL_WORDS_PER_ROW
            || parts.regular37_snapshot.len()
                != FRONTEND_CONTROL_REGULAR_ROWS * FRONTEND_CONTROL_WORDS_PER_ROW
        {
            return Err(FrontendControlError::NormalAfterStateMismatch);
        }
        let before_generation = self.generation;
        let before_digest = self.committed.digest();
        let provider_manifest_sha256 = self.committed.provider_receipt.manifest_sha256();
        let provider_file_sha256 = self.committed.provider_receipt.provider_file_sha256();
        let after = FrontendControlState::from_normal_after_parts(&self.committed, parts)?;
        let after_digest = after.digest();
        Ok(FrontendControlStaged {
            join: FrontendControlJoinMetadata {
                transaction_id,
                before_generation,
                before_digest,
                provider_manifest_sha256,
                provider_file_sha256,
            },
            after,
            after_digest,
            after_generation: expected_generation,
        })
    }

    /// Consumes the exact normal proposal, transferring its after-state to the frontend owner
    /// while retaining the same computed source projection for F03W.
    pub fn stage_normal_snapshot(
        &self,
        proposal: NormalControlSnapshotProposal,
    ) -> Result<FrontendControlNormalStage, FrontendControlError> {
        let (after_state, output) = proposal.into_frontend_parent_parts();
        let transaction_id = output.transaction_id().clone();
        let staged = self.stage_normal_after_state(transaction_id, after_state)?;
        Ok(FrontendControlNormalStage { staged, output })
    }

    /// Returns a new owner value and its GTC transition receipt without mutating this machine.
    /// The application parent installs the candidate only with the other staged callback roots.
    pub fn replacement_candidate(
        &self,
        staged: FrontendControlStaged,
        transaction_id: CallbackTransactionId,
    ) -> Result<FrontendControlReplacementCandidate, FrontendControlError> {
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(FrontendControlError::GenerationMismatch)?;
        if staged.after_generation != next_generation
            || staged.after.schema != FRONTEND_CONTROL_AFTER_STATE_SCHEMA
            || staged.after.prepared_identity.is_none()
            || staged.join.before_generation != self.generation
            || staged.join.before_digest != self.committed.digest()
            || staged.join.transaction_id.programme_id() != &self.committed.programme_id
            || staged.join.transaction_id.attempt_id() != self.committed.attempt_id
            || staged.join.transaction_id.callback_index() != self.committed.callback_index
            || staged.join.provider_manifest_sha256
                != self.committed.provider_receipt.manifest_sha256()
            || staged.join.provider_file_sha256
                != self.committed.provider_receipt.provider_file_sha256()
            || staged.after.digest() != staged.after_digest
            || staged.after.owner_generation != staged.after_generation
        {
            return Err(FrontendControlError::GenerationMismatch);
        }
        let next_machine = Self {
            committed: staged.after,
            generation: staged.after_generation,
            cold_digest: self.cold_digest,
        };
        let before_reference = self.state_reference();
        let after_reference = next_machine.state_reference();
        let before = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::FrontendControl,
            before_reference.generation(),
            before_reference.cold_sha256().to_bytes(),
            before_reference.canonical_bytes(),
        )
        .map_err(FrontendControlError::GtcReceipt)?;
        let after = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::FrontendControl,
            after_reference.generation(),
            after_reference.cold_sha256().to_bytes(),
            after_reference.canonical_bytes(),
        )
        .map_err(FrontendControlError::GtcReceipt)?;
        let receipt = GtcOwnerReceipt::from_transition(before, after, transaction_id)
            .map_err(FrontendControlError::GtcReceipt)?;
        Ok(FrontendControlReplacementCandidate {
            next_machine,
            receipt,
        })
    }
}

fn frontend_object_from_normal_control(value: NormalControlObjectState) -> FrontendObjectState {
    let parts = value.into_frontend_parts();
    FrontendObjectState {
        object_ordinal: parts.object_ordinal,
        event_cache: parts.event_cache,
        host_channel_index: parts.host_channel_index,
        sampler_previous_present: parts.sampler_previous_present,
        sampler_previous_xyz_u32: parts.sampler_previous_xyz_u32,
    }
}

fn encode_frontend_control_after_state(
    state: &FrontendControlState,
) -> Result<Vec<u8>, FrontendControlError> {
    let prepared = state
        .prepared_identity
        .as_ref()
        .ok_or(FrontendControlError::NormalAfterStateMismatch)?;
    let control38 = snapshot_rows_json(
        state
            .last_control38_snapshot
            .as_ref()
            .ok_or(FrontendControlError::NormalAfterStateMismatch)?,
        FRONTEND_CONTROL_ROWS,
    )?;
    let regular37 = snapshot_rows_json(
        state
            .last_regular37_snapshot
            .as_ref()
            .ok_or(FrontendControlError::NormalAfterStateMismatch)?,
        FRONTEND_CONTROL_REGULAR_ROWS,
    )?;
    let objects = state
        .objects
        .iter()
        .map(|object| {
            json!({
                "object_ordinal": object.object_ordinal,
                "event_cache_bytes": object.event_cache.as_deref(),
                "host_channel_index": object.host_channel_index,
                "sampler_previous_present": object.sampler_previous_present,
                "sampler_previous_xyz_u32": object.sampler_previous_xyz_u32,
            })
        })
        .collect::<Vec<_>>();
    let config = &state.normalized_config;
    let provider = &state.provider_receipt;
    let value = json!({
        "schema": state.schema,
        "owner_id": FRONTEND_CONTROL_OWNER_ID,
        "programme_id": state.programme_id.as_str(),
        "attempt_id": state.attempt_id.value(),
        "owner_generation": state.owner_generation,
        "callback_index": state.callback_index.value(),
        "normal_callbacks": state.normal_callbacks.value(),
        "remaining_cached_drains": state.remaining_cached_drains,
        "phase": frontend_phase_name(state.phase),
        "objects": objects,
        "last_control38_snapshot": control38,
        "last_regular37_snapshot": regular37,
        "physical_binding_bytes": state.physical_binding.as_deref(),
        "dbmd_binding_bytes": state.dbmd_binding.as_deref(),
        "normalized_config": {
            "sample_rate_hz": config.sample_rate_hz(),
            "channels": config.channels(),
            "pcm_bits": config.pcm_bits(),
            "source_samples": config.source_samples(),
            "bit_rate_kbps": config.bit_rate_kbps(),
            "bit_rate_enum": config.bit_rate_enum(),
            "layout_enum": config.layout_enum(),
            "target_capacity": config.target_capacity(),
            "frame_samples": config.frame_samples(),
            "spatial_config_mode": config.spatial_config_mode(),
            "dispatch_mode": config.dispatch_mode(),
            "virtual_prefix_count": config.virtual_prefix_count(),
            "bed_count": config.bed_count(),
            "object_count": config.object_count(),
            "event_start_samples": config.event_start_samples(),
            "line_drc": config.line_drc(),
            "rf_drc": config.rf_drc(),
            "time_base": config.time_base(),
        },
        "provider_identity": {
            "manifest_sha256": provider.manifest_sha256().to_hex(),
            "provider_file_sha256": provider.provider_file_sha256().to_hex(),
            "provider_file_bytes": provider.provider_file_bytes(),
            "canonical_sha256": provider.canonical_sha256().to_hex(),
            "canonical_bytes": provider.canonical_bytes(),
            "provider_generation": provider.provider_generation(),
        },
        "prepared_identity": prepared
            .canonical_json_value()
            .map_err(|_| FrontendControlError::CanonicalEncoding)?,
    });
    let canonical = canonicalize_json(value);
    serde_json::to_vec(&canonical).map_err(|_| FrontendControlError::CanonicalEncoding)
}

fn snapshot_rows_json(words: &[u32], row_count: usize) -> Result<Value, FrontendControlError> {
    let expected_words = row_count
        .checked_mul(FRONTEND_CONTROL_WORDS_PER_ROW)
        .ok_or(FrontendControlError::CanonicalEncoding)?;
    if words.len() != expected_words {
        return Err(FrontendControlError::NormalAfterStateMismatch);
    }
    let rows = words
        .chunks_exact(FRONTEND_CONTROL_WORDS_PER_ROW)
        .map(|row| Value::Array(row.iter().copied().map(Value::from).collect()))
        .collect::<Vec<_>>();
    Ok(Value::Array(rows))
}

fn frontend_phase_name(phase: FrontendControlPhase) -> &'static str {
    match phase {
        FrontendControlPhase::BeforeFirstNormal => "BEFORE_FIRST_NORMAL",
    }
}

fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted: BTreeMap<_, _> = object
                .into_iter()
                .map(|(key, value)| (key, canonicalize_json(value)))
                .collect();
            Value::Object(sorted.into_iter().collect::<Map<_, _>>())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_json).collect()),
        scalar => scalar,
    }
}
