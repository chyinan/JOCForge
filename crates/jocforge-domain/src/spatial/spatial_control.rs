// pattern: Functional Core
//! Complete v5 persistent cold root and read-only staging capability boundary.
//!
//! No callback transition or packet producer is implemented here. CurrentFeatures owns its opaque
//! packet type; no public constructor, default packet, or local commit authority exists here.
//!
//! # Capability guarantees
//!
//! ```compile_fail
//! use jocforge_domain::spatial::SpatialControlMachine;
//! fn requires_clone<T: Clone>() {}
//! requires_clone::<SpatialControlMachine>();
//! ```
//! ```compile_fail
//! use jocforge_domain::spatial::SpatialControlPrepared;
//! fn requires_clone<T: Clone>() {}
//! requires_clone::<SpatialControlPrepared>();
//! ```
//! ```compile_fail
//! use jocforge_domain::spatial::SpatialControlStaged;
//! fn requires_clone<T: Clone>() {}
//! requires_clone::<SpatialControlStaged>();
//! ```
//! ```compile_fail
//! use jocforge_domain::spatial::{SpatialControlMachine, SpatialControlStaged};
//! fn try_local_commit(machine: &mut SpatialControlMachine, staged: SpatialControlStaged) {
//!     machine.commit(staged);
//! }
//! ```
//! ```compile_fail
//! use jocforge_domain::spatial::SpatialControlMachine;
//! fn try_mutate_committed(machine: &mut SpatialControlMachine) {
//!     let _ = machine.committed_mut();
//! }
//! ```
//! ```compile_fail
//! use jocforge_domain::spatial::SourceFlagsPacket;
//! let _packet = SourceFlagsPacket::new([[0_u8; 2]; 48]);
//! ```
//! ```compile_fail
//! use jocforge_domain::spatial::SpatialControlAfterProposal;
//! let _proposal = SpatialControlAfterProposal::new();
//! ```

use super::{current_features::CurrentFeaturesPacket, source_flags::SourceFlagsPacket};
use crate::frontend::{
    F01EProviderReceipt, F03TargetProviderReceipt, ProviderIdentity, ProviderSha256,
    ValidatedF01EColdProvider, ValidatedF03TargetColdProvider,
};
use crate::profile::{EncodingProfile, resolve_selected_profile};
use crate::timeline::{AdmSampleIndex, FrontendCallbackIndex, NormalCallbackCount};
use crate::transaction::CallbackTransactionId;
use sha2::{Digest, Sha256};
use std::fmt::Write as FmtWrite;

const NORMAL_CALLBACKS_COLD: u64 = 0;
const CACHED_DRAINS_COLD: u32 = 4;
const F01E_COLD_BYTES: usize = 4_402;
const F03_TARGET_PROJECTION_BYTES: u64 = 630;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialControlDigest([u8; 32]);

impl SpatialControlDigest {
    fn digest(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut value = [0; 32];
        value.copy_from_slice(&digest);
        Self(value)
    }

    pub fn to_hex(self) -> String {
        let mut value = String::with_capacity(64);
        for byte in self.0 {
            write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
        }
        value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialControlProviderLineage {
    f01e_identity: ProviderIdentity,
    f01e_manifest_sha256: ProviderSha256,
    f01e_provider_file_sha256: ProviderSha256,
    f01e_canonical_sha256: ProviderSha256,
    f01e_provider_generation: u64,
    f03_manifest_sha256: ProviderSha256,
    f03_provider_path: &'static str,
    f03_provider_bytes: u64,
    f03_provider_file_sha256: ProviderSha256,
    f03_target_projection_sha256: ProviderSha256,
    f03_target_projection_bytes: u64,
    f03_provider_generation: u64,
}

impl SpatialControlProviderLineage {
    fn from_receipts(f01e: &F01EProviderReceipt, f03: &F03TargetProviderReceipt) -> Self {
        let f03_lineage = f03.validated_manifest_lineage();
        Self {
            f01e_identity: f01e.provider_identity(),
            f01e_manifest_sha256: f01e.manifest_sha256(),
            f01e_provider_file_sha256: f01e.provider_file_sha256(),
            f01e_canonical_sha256: f01e.canonical_sha256(),
            f01e_provider_generation: f01e.provider_generation(),
            f03_manifest_sha256: f03_lineage.manifest_sha256(),
            f03_provider_path: f03_lineage.provider_path(),
            f03_provider_bytes: f03_lineage.provider_bytes(),
            f03_provider_file_sha256: f03_lineage.provider_sha256(),
            f03_target_projection_sha256: f03.target_projection_sha256(),
            f03_target_projection_bytes: f03.target_projection_bytes(),
            f03_provider_generation: f03.provider_generation(),
        }
    }

    pub const fn f01e_identity(self) -> ProviderIdentity {
        self.f01e_identity
    }

    pub const fn f01e_manifest_sha256(self) -> ProviderSha256 {
        self.f01e_manifest_sha256
    }

    pub const fn f01e_provider_file_sha256(self) -> ProviderSha256 {
        self.f01e_provider_file_sha256
    }

    pub const fn f01e_canonical_sha256(self) -> ProviderSha256 {
        self.f01e_canonical_sha256
    }

    pub const fn f01e_provider_generation(self) -> u64 {
        self.f01e_provider_generation
    }

    pub const fn f03_manifest_sha256(self) -> ProviderSha256 {
        self.f03_manifest_sha256
    }

    pub const fn f03_provider_path(self) -> &'static str {
        self.f03_provider_path
    }

    pub const fn f03_provider_bytes(self) -> u64 {
        self.f03_provider_bytes
    }

    pub const fn f03_provider_file_sha256(self) -> ProviderSha256 {
        self.f03_provider_file_sha256
    }

    pub const fn f03_target_projection_sha256(self) -> ProviderSha256 {
        self.f03_target_projection_sha256
    }

    pub const fn f03_target_projection_bytes(self) -> u64 {
        self.f03_target_projection_bytes
    }

    pub const fn f03_provider_generation(self) -> u64 {
        self.f03_provider_generation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketReadiness {
    NotReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialControlError {
    UnsupportedProfile,
    ProfileBindingMismatch,
    ProviderLineageMismatch,
    F01ECanonicalIdentityMismatch,
    F03TargetProjectionMismatch,
}

#[derive(Debug)]
struct ClockState {
    callback_index: FrontendCallbackIndex,
    source_cursor_samples: AdmSampleIndex,
    normal_callbacks: NormalCallbackCount,
    remaining_cached_drains: u32,
    retired: bool,
}

#[derive(Debug)]
struct SourceCacheState {
    normal_host38_cache_present: bool,
    control_snapshot_present: bool,
    phase_u8: u8,
}

#[derive(Debug)]
struct RenderAudioState {
    regular_raw_history_bits: Box<[u32]>,
    previous_pcm_bits: Box<[u32]>,
    previous_current_rows_bits: Box<[u32]>,
    diffusion_instances_present: bool,
    diffusion_previous_outputs_present: bool,
}

#[derive(Debug)]
struct FeatureHistoryState {
    source_vector_bits: Box<[u32]>,
    source_scalar_bits: Box<[u32]>,
}

#[derive(Debug)]
struct TargetState {
    current_xyz_u32: [u32; 45],
    current_id_u32: [u32; 15],
    current_attribute_u32: [u32; 15],
    previous_xyz_u32: [u32; 45],
    previous_id_u32: [u32; 15],
    previous_attribute_u32: [u32; 15],
    active_u8: [u8; 15],
    previous_marker_active_u8: [u8; 15],
}

#[derive(Debug)]
struct MatrixAssignmentState {
    assignment_i32: [i32; 15],
    markers_i32: [i32; 15],
    matrix_f32_bits: Box<[u32]>,
    writer_set_record_count: u32,
}

/// Immutable spatial-control state. This type is materialized only by the validated cold path.
#[derive(Debug)]
pub struct SpatialControlState {
    clocks: ClockState,
    source_cache: SourceCacheState,
    event_control_canonical_bytes: Box<[u8]>,
    render_audio: RenderAudioState,
    feature: FeatureHistoryState,
    targets: TargetState,
    matrix_assignment: MatrixAssignmentState,
    canonical_bytes: Box<[u8]>,
    digest: SpatialControlDigest,
}

impl SpatialControlState {
    fn from_cold(f01e_canonical_bytes: &[u8], f03: &ValidatedF03TargetColdProvider) -> Self {
        let mut state = Self {
            clocks: ClockState {
                callback_index: FrontendCallbackIndex::new(0),
                source_cursor_samples: AdmSampleIndex::new(0),
                normal_callbacks: NormalCallbackCount::new(NORMAL_CALLBACKS_COLD),
                remaining_cached_drains: CACHED_DRAINS_COLD,
                retired: false,
            },
            source_cache: SourceCacheState {
                normal_host38_cache_present: false,
                control_snapshot_present: false,
                phase_u8: 0,
            },
            event_control_canonical_bytes: f01e_canonical_bytes.to_vec().into_boxed_slice(),
            render_audio: RenderAudioState {
                regular_raw_history_bits: vec![0; 37 * 1536].into_boxed_slice(),
                previous_pcm_bits: vec![0; 38 * 1536].into_boxed_slice(),
                previous_current_rows_bits: vec![0; 48 * 1536].into_boxed_slice(),
                diffusion_instances_present: false,
                diffusion_previous_outputs_present: false,
            },
            feature: FeatureHistoryState {
                source_vector_bits: vec![0; 48 * 24].into_boxed_slice(),
                source_scalar_bits: vec![0; 48].into_boxed_slice(),
            },
            targets: TargetState {
                current_xyz_u32: *f03.current_xyz_u32(),
                current_id_u32: *f03.current_id_u32(),
                current_attribute_u32: *f03.current_attribute_u32(),
                previous_xyz_u32: *f03.previous_xyz_u32(),
                previous_id_u32: *f03.previous_id_u32(),
                previous_attribute_u32: *f03.previous_attribute_u32(),
                active_u8: *f03.active_u8(),
                previous_marker_active_u8: *f03.previous_marker_active_u8(),
            },
            matrix_assignment: MatrixAssignmentState {
                assignment_i32: [-1; 15],
                markers_i32: [0; 15],
                matrix_f32_bits: vec![0; 15 * 48].into_boxed_slice(),
                writer_set_record_count: 0,
            },
            canonical_bytes: Box::new([]),
            digest: SpatialControlDigest([0; 32]),
        };
        let canonical_bytes = state.serialize_v5();
        state.digest = SpatialControlDigest::digest(&canonical_bytes);
        state.canonical_bytes = canonical_bytes.into_boxed_slice();
        state
    }

    fn serialize_v5(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        push_u64(&mut bytes, self.clocks.callback_index.value());
        push_u64(&mut bytes, self.clocks.source_cursor_samples.value());
        push_u64(&mut bytes, self.clocks.normal_callbacks.value());
        push_u32(&mut bytes, self.clocks.remaining_cached_drains);
        bytes.push(u8::from(self.clocks.retired));

        bytes.push(u8::from(self.source_cache.normal_host38_cache_present));
        bytes.push(u8::from(self.source_cache.control_snapshot_present));
        bytes.push(self.source_cache.phase_u8);

        bytes.extend_from_slice(&self.event_control_canonical_bytes);

        push_u32_slice(&mut bytes, &self.render_audio.regular_raw_history_bits);
        push_u32_slice(&mut bytes, &self.render_audio.previous_pcm_bits);
        push_u32_slice(&mut bytes, &self.render_audio.previous_current_rows_bits);
        bytes.push(u8::from(self.render_audio.diffusion_instances_present));
        bytes.push(u8::from(
            self.render_audio.diffusion_previous_outputs_present,
        ));

        push_u32_slice(&mut bytes, &self.feature.source_vector_bits);
        push_u32_slice(&mut bytes, &self.feature.source_scalar_bits);

        push_u32_slice(&mut bytes, &self.targets.current_xyz_u32);
        push_u32_slice(&mut bytes, &self.targets.current_id_u32);
        push_u32_slice(&mut bytes, &self.targets.current_attribute_u32);
        push_u32_slice(&mut bytes, &self.targets.previous_xyz_u32);
        push_u32_slice(&mut bytes, &self.targets.previous_id_u32);
        push_u32_slice(&mut bytes, &self.targets.previous_attribute_u32);
        bytes.extend_from_slice(&self.targets.active_u8);
        bytes.extend_from_slice(&self.targets.previous_marker_active_u8);

        for value in self.matrix_assignment.assignment_i32 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in self.matrix_assignment.markers_i32 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        push_u32_slice(&mut bytes, &self.matrix_assignment.matrix_f32_bits);
        push_u32(&mut bytes, self.matrix_assignment.writer_set_record_count);

        bytes
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub const fn digest(&self) -> SpatialControlDigest {
        self.digest
    }

    pub fn event_control_canonical_bytes(&self) -> &[u8] {
        &self.event_control_canonical_bytes
    }

    pub fn source_vector_bits(&self) -> &[u32] {
        &self.feature.source_vector_bits
    }

    pub fn source_scalar_bits(&self) -> &[u32] {
        &self.feature.source_scalar_bits
    }

    pub const fn target_active_u8(&self) -> &[u8; 15] {
        &self.targets.active_u8
    }

    pub const fn previous_marker_active_u8(&self) -> &[u8; 15] {
        &self.targets.previous_marker_active_u8
    }

    pub const fn source_cursor_samples(&self) -> AdmSampleIndex {
        self.clocks.source_cursor_samples
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.clocks.callback_index
    }

    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        self.clocks.normal_callbacks
    }

    pub const fn remaining_cached_drains(&self) -> u32 {
        self.clocks.remaining_cached_drains
    }

    pub const fn retired(&self) -> bool {
        self.clocks.retired
    }

    pub const fn source_cache_is_cold(&self) -> bool {
        !self.source_cache.normal_host38_cache_present
            && !self.source_cache.control_snapshot_present
            && self.source_cache.phase_u8 == 0
    }

    pub fn regular_raw_history_bits(&self) -> &[u32] {
        &self.render_audio.regular_raw_history_bits
    }

    pub fn previous_pcm_bits(&self) -> &[u32] {
        &self.render_audio.previous_pcm_bits
    }

    pub fn previous_current_rows_bits(&self) -> &[u32] {
        &self.render_audio.previous_current_rows_bits
    }

    pub const fn diffusion_instances_present(&self) -> bool {
        self.render_audio.diffusion_instances_present
    }

    pub const fn diffusion_previous_outputs_present(&self) -> bool {
        self.render_audio.diffusion_previous_outputs_present
    }

    pub const fn target_current_xyz_u32(&self) -> &[u32; 45] {
        &self.targets.current_xyz_u32
    }

    pub const fn target_current_id_u32(&self) -> &[u32; 15] {
        &self.targets.current_id_u32
    }

    pub const fn target_current_attribute_u32(&self) -> &[u32; 15] {
        &self.targets.current_attribute_u32
    }

    pub const fn target_previous_xyz_u32(&self) -> &[u32; 45] {
        &self.targets.previous_xyz_u32
    }

    pub const fn target_previous_id_u32(&self) -> &[u32; 15] {
        &self.targets.previous_id_u32
    }

    pub const fn target_previous_attribute_u32(&self) -> &[u32; 15] {
        &self.targets.previous_attribute_u32
    }

    pub const fn assignment_i32(&self) -> &[i32; 15] {
        &self.matrix_assignment.assignment_i32
    }

    pub const fn markers_i32(&self) -> &[i32; 15] {
        &self.matrix_assignment.markers_i32
    }

    pub fn matrix_f32_bits(&self) -> &[u32] {
        &self.matrix_assignment.matrix_f32_bits
    }

    pub const fn writer_set_record_count(&self) -> u32 {
        self.matrix_assignment.writer_set_record_count
    }
}

/// Opaque complete after proposal supplied by future producer-owned capabilities.
#[allow(dead_code)]
#[must_use]
pub struct SpatialControlAfterProposal {
    _after_state: SpatialControlState,
    _join_metadata: SpatialControlJoinMetadata,
    _source_flags: SourceFlagsPacket,
    _current_features: CurrentFeaturesPacket,
}

/// Linear preparation capability. No public or test-only constructor exists in D1.
#[allow(dead_code)]
#[must_use]
pub struct SpatialControlPrepared {
    _proposal: SpatialControlAfterProposal,
    _before_generation: u64,
    _before_digest: SpatialControlDigest,
}

/// Linear staged-after capability. No local commit operation exists in D1.
#[allow(dead_code)]
#[must_use]
pub struct SpatialControlStaged {
    _proposal: SpatialControlAfterProposal,
    _after_digest: SpatialControlDigest,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SpatialControlJoinMetadata {
    transaction_id: CallbackTransactionId,
    callback_index: FrontendCallbackIndex,
    before_generation: u64,
    before_digest: SpatialControlDigest,
    after_digest: SpatialControlDigest,
    profile_identity: EncodingProfile,
    provider_lineage: SpatialControlProviderLineage,
}

impl SpatialControlJoinMetadata {
    pub fn transaction_id(&self) -> &CallbackTransactionId {
        &self.transaction_id
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }

    pub const fn before_digest(&self) -> SpatialControlDigest {
        self.before_digest
    }

    pub const fn after_digest(&self) -> SpatialControlDigest {
        self.after_digest
    }

    pub const fn profile_identity(&self) -> EncodingProfile {
        self.profile_identity
    }

    pub const fn provider_lineage(&self) -> SpatialControlProviderLineage {
        self.provider_lineage
    }

    #[cfg(test)]
    pub(crate) fn fixture(
        machine: &SpatialControlMachine,
        transaction_id: CallbackTransactionId,
        callback_index: FrontendCallbackIndex,
        before_generation: u64,
        before_digest: Option<[u8; 32]>,
        profile_identity: EncodingProfile,
    ) -> Self {
        Self {
            transaction_id,
            callback_index,
            before_generation,
            before_digest: before_digest.map_or_else(|| machine.digest(), SpatialControlDigest),
            after_digest: machine.digest(),
            profile_identity,
            provider_lineage: machine.provider_lineage(),
        }
    }
}

/// Single committed owner for the complete v5 persistent cold root.
#[must_use]
pub struct SpatialControlMachine {
    committed: SpatialControlState,
    generation: u64,
    profile_identity: EncodingProfile,
    provider_lineage: SpatialControlProviderLineage,
}

impl SpatialControlMachine {
    pub fn new_cold(
        profile_identity: EncodingProfile,
        f01e: &ValidatedF01EColdProvider,
        f03: &ValidatedF03TargetColdProvider,
    ) -> Result<Self, SpatialControlError> {
        resolve_selected_profile(&profile_identity)
            .map_err(|_| SpatialControlError::UnsupportedProfile)?;

        let f01e_receipt = f01e.receipt();
        let f03_receipt = f03.receipt();
        if f01e_receipt.provider_generation() != 0
            || f03_receipt.provider_generation() != 0
            || f01e_receipt.manifest_sha256()
                != f03_receipt.validated_manifest_lineage().manifest_sha256()
        {
            return Err(SpatialControlError::ProviderLineageMismatch);
        }
        if f01e.canonical_bytes().len() != F01E_COLD_BYTES
            || f01e.canonical_bytes().len() as u64 != f01e_receipt.canonical_bytes()
            || ProviderSha256::digest(f01e.canonical_bytes()) != f01e_receipt.canonical_sha256()
        {
            return Err(SpatialControlError::F01ECanonicalIdentityMismatch);
        }
        if f03.target_projection_bytes().len() as u64 != F03_TARGET_PROJECTION_BYTES
            || f03.target_projection_bytes().len() as u64 != f03_receipt.target_projection_bytes()
            || ProviderSha256::digest(f03.target_projection_bytes())
                != f03_receipt.target_projection_sha256()
        {
            return Err(SpatialControlError::F03TargetProjectionMismatch);
        }

        let provider_lineage =
            SpatialControlProviderLineage::from_receipts(f01e_receipt, f03_receipt);
        let committed = SpatialControlState::from_cold(f01e.canonical_bytes(), f03);
        Ok(Self {
            committed,
            generation: 0,
            profile_identity,
            provider_lineage,
        })
    }

    /// Installs literal before-feature bits only in isolated unit tests.
    #[cfg(test)]
    pub(super) fn set_current_features_test_history(
        &mut self,
        source_vector_bits: &[u32],
        source_scalar_bits: &[u32],
    ) {
        self.committed.feature.source_vector_bits = source_vector_bits.to_vec().into_boxed_slice();
        self.committed.feature.source_scalar_bits = source_scalar_bits.to_vec().into_boxed_slice();
        let canonical_bytes = self.committed.serialize_v5();
        self.committed.digest = SpatialControlDigest::digest(&canonical_bytes);
        self.committed.canonical_bytes = canonical_bytes.into_boxed_slice();
    }

    pub const fn committed(&self) -> &SpatialControlState {
        &self.committed
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn digest(&self) -> SpatialControlDigest {
        self.committed.digest()
    }

    pub const fn profile(&self) -> EncodingProfile {
        self.profile_identity
    }

    pub const fn provider_lineage(&self) -> SpatialControlProviderLineage {
        self.provider_lineage
    }

    pub const fn packet_readiness(&self) -> PacketReadiness {
        PacketReadiness::NotReady
    }

    /// Read-only guard for reusing a token under the machine's exact six-dimensional identity.
    pub fn validate_profile_identity(
        &self,
        candidate: &EncodingProfile,
    ) -> Result<(), SpatialControlError> {
        if *candidate == self.profile_identity {
            Ok(())
        } else {
            Err(SpatialControlError::ProfileBindingMismatch)
        }
    }

    /// Read-only guard for provider receipts before a future producer-generated proposal is staged.
    pub fn validate_provider_lineage(
        &self,
        f01e: &F01EProviderReceipt,
        f03: &F03TargetProviderReceipt,
    ) -> Result<(), SpatialControlError> {
        self.validate_provider_lineage_snapshot(SpatialControlProviderLineage::from_receipts(
            f01e, f03,
        ))
    }

    /// Read-only guard for a copied opaque lineage snapshot before a future stage join.
    pub fn validate_provider_lineage_snapshot(
        &self,
        candidate: SpatialControlProviderLineage,
    ) -> Result<(), SpatialControlError> {
        if candidate == self.provider_lineage {
            Ok(())
        } else {
            Err(SpatialControlError::ProviderLineageMismatch)
        }
    }
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32_slice(bytes: &mut Vec<u8>, values: &[u32]) {
    for value in values {
        push_u32(bytes, *value);
    }
}

#[cfg(test)]
mod tests {
    use super::{SpatialControlError, SpatialControlMachine};
    use crate::{
        frontend::{
            PackageManifestEvidence, ProviderFileEvidence, ProviderSha256,
            validate_f01e_package_manifest, validate_f01e_provider, validate_f03_package_manifest,
            validate_f03_target_cold_provider,
        },
        profile::EncodingProfile,
    };
    use std::sync::Arc;

    const MANIFEST_BYTES: &[u8] =
        include_bytes!("../../tests/fixtures/f01e_provider/MANIFEST.json");
    const F01E_PROVIDER_BYTES: &[u8] =
        include_bytes!("../../tests/fixtures/f01e_provider/f01e-selected-constructor-v1.json");
    const F03_PROVIDER_BYTES: &[u8] =
        include_bytes!("../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");

    fn selected_machine() -> SpatialControlMachine {
        let f01e_manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
        let f01e_manifest = validate_f01e_package_manifest(&f01e_manifest_evidence).unwrap();
        let f01e_evidence = ProviderFileEvidence::new(Arc::from(F01E_PROVIDER_BYTES), None);
        let f01e = validate_f01e_provider(&f01e_evidence, &f01e_manifest).unwrap();

        let f03_manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
        let f03_manifest = validate_f03_package_manifest(&f03_manifest_evidence).unwrap();
        let f03_evidence = ProviderFileEvidence::new(Arc::from(F03_PROVIDER_BYTES), None);
        let f03 = validate_f03_target_cold_provider(&f03_evidence, &f03_manifest).unwrap();
        SpatialControlMachine::new_cold(EncodingProfile::selected(), &f01e, &f03).unwrap()
    }

    #[test]
    fn distinct_provider_lineage_is_rejected_without_root_mutation() {
        let machine = selected_machine();
        let before_digest = machine.digest();
        let before_generation = machine.generation();
        let mut distinct = machine.provider_lineage();
        distinct.f03_target_projection_sha256 =
            ProviderSha256::digest(b"different provider lineage");

        assert_eq!(
            machine.validate_provider_lineage_snapshot(distinct),
            Err(SpatialControlError::ProviderLineageMismatch)
        );
        assert_eq!(machine.digest(), before_digest);
        assert_eq!(machine.generation(), before_generation);
    }
}
