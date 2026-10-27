// pattern: Functional Core
//! Staging-only frontend-control root for the approved cold provider.

use super::ProviderSha256;
use super::provider::{F01EProviderReceipt, NormalizedConfig, ValidatedF01EColdProvider};
use crate::timeline::{FrontendCallbackIndex, NormalCallbackCount};
use crate::transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId};
use std::sync::Arc;

const FRONTEND_CONTROL_OWNER_ID: &str = "frontend_control";
const FRONTEND_CONTROL_STATE_REF: &str = "owners.frontend_control";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontendControlDigest(ProviderSha256);
impl FrontendControlDigest {
    pub fn to_hex(self) -> String {
        self.0.to_hex()
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
            canonical,
            provider_receipt,
        })
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrontendControlError {
    CallbackIdentityMismatch,
    CallbackIndexMismatch,
    CallbackIndexOutOfRange,
    GenerationMismatch,
    ProviderMismatch,
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
        })
    }
}
