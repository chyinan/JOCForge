// pattern: Functional Core
//! Complete v6 persistent spatial-control root and read-only staging capability boundary.
//!
//! A selected normal callback can now produce a complete pending F03W/F05R56 after-root from the
//! same callback's typed producer outputs. Packets and receipts remain linear, no local commit
//! authority exists, and the enclosing programme transaction remains the sole committer.
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

use super::{
    current_features::{CurrentFeaturesPacket, CurrentFeaturesProduced},
    diffusion_tail::DiffusionTailState,
    production_callback::SpatialControlCallbackProjection,
    source_flags::SourceFlagsPacket,
};
use crate::frontend::normal_control_snapshot::NormalControlRow;
use crate::frontend::{
    F01EProviderReceipt, F03TargetProviderReceipt, ProviderIdentity, ProviderSha256,
    ValidatedF01EColdProvider, ValidatedF03TargetColdProvider,
};
use crate::joc::oamd::reader_target::{
    ReaderTargetControlRows, ReaderTargetState, ReaderTargetUpdateReceipt,
};
use crate::profile::{EncodingProfile, resolve_selected_profile};
use crate::timeline::{AdmSampleIndex, FrontendCallbackIndex, NormalCallbackCount};
use crate::transaction::CallbackLineageIdentity;
use sha2::{Digest, Sha256};
use std::fmt::Write as FmtWrite;
use std::sync::Arc;

const NORMAL_CALLBACKS_COLD: u64 = 0;
const CACHED_DRAINS_COLD: u32 = 4;
const F01E_COLD_BYTES: usize = 4_402;
const F03_TARGET_PROJECTION_BYTES: u64 = 630;
const SPATIAL_CONTROL_OWNER_ID: &str = "spatial_control";
const SPATIAL_CONTROL_STATE_REF: &str = "owners.spatial_control";
const PHYSICAL_HOST_COUNT: usize = 38;
const REGULAR_TAIL_COUNT: usize = 37;
const CONTROL_WORDS_PER_TRACK: usize = 13;

#[derive(Clone, Debug)]
enum PhysicalControlSnapshotRow {
    Bed6([u32; 6]),
    Object13([u32; CONTROL_WORDS_PER_TRACK]),
}

impl PhysicalControlSnapshotRow {
    fn from_normal_control(row: &NormalControlRow) -> Self {
        match row {
            NormalControlRow::Bed6 { words, .. } => Self::Bed6(*words),
            NormalControlRow::Object13 { control, .. } => Self::Object13(control.words()),
        }
    }
}

/// Producer capability reserved for the future F03W direct-LFE ingress path.
#[derive(Debug)]
pub(super) struct SpatialControlLfeIngressAuthority {
    _private: (),
}

impl SpatialControlLfeIngressAuthority {
    #[allow(dead_code)]
    fn for_f03w_lfe_ingress() -> Self {
        Self { _private: () }
    }

    #[cfg(test)]
    pub(super) fn fixture() -> Self {
        Self { _private: () }
    }
}

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

/// Read-only GTC owner-state view over the complete persistent SpatialControl root.
///
/// `state_ref` is a logical member key. The borrowed bytes are the complete v5
/// canonical owner preimage; this view contains no receipt or parent commit state.
#[derive(Clone, Copy, Debug)]
pub struct SpatialControlOwnerStateReference<'a> {
    state: &'a SpatialControlState,
    generation: u64,
    state_sha256: SpatialControlDigest,
    cold_sha256: SpatialControlDigest,
}

impl<'a> SpatialControlOwnerStateReference<'a> {
    pub const fn owner_id(&self) -> &'static str {
        SPATIAL_CONTROL_OWNER_ID
    }

    pub const fn state_ref(&self) -> &'static str {
        SPATIAL_CONTROL_STATE_REF
    }

    pub const fn state(&self) -> &'a SpatialControlState {
        self.state
    }

    pub fn canonical_bytes(&self) -> &'a [u8] {
        self.state.canonical_bytes()
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn state_sha256(&self) -> SpatialControlDigest {
        self.state_sha256
    }

    pub const fn cold_sha256(&self) -> SpatialControlDigest {
        self.cold_sha256
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

/// Before-state identity for a render callback; it does not claim a parent after-state digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SpatialControlRenderIdentity {
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    before_generation: u64,
    before_digest: SpatialControlDigest,
    profile_identity: EncodingProfile,
    provider_lineage: SpatialControlProviderLineage,
}

impl SpatialControlRenderIdentity {
    pub(super) fn transaction_id(&self) -> &CallbackLineageIdentity {
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

    pub const fn profile_identity(&self) -> EncodingProfile {
        self.profile_identity
    }

    pub const fn provider_lineage(&self) -> SpatialControlProviderLineage {
        self.provider_lineage
    }
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

    #[cfg(test)]
    pub(crate) fn fixture_mismatch_for_type10(mut lineage: Self) -> Self {
        lineage.f03_provider_generation += 1;
        lineage
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
    RenderCallbackMismatch,
    ProviderLineageMismatch,
    F01ECanonicalIdentityMismatch,
    F03TargetProjectionMismatch,
    CallbackAfterStateInvalid,
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
    last_normal_host38_bits: Option<Arc<[u32]>>,
    last_control_snapshot: Option<Box<[PhysicalControlSnapshotRow; PHYSICAL_HOST_COUNT]>>,
    phase_u8: u8,
}

#[derive(Debug)]
struct RenderAudioState {
    regular_raw_history_bits: Box<[u32]>,
    previous_pcm_bits: Arc<[u32]>,
    previous_current_rows_bits: Box<[u32]>,
    diffusion_tails: [Option<super::diffusion_tail::DiffusionTailState>; REGULAR_TAIL_COUNT],
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
}

#[derive(Debug)]
struct MatrixAssignmentState {
    assignment_i32: [i32; 15],
    markers_i32: [i32; 15],
    matrix_f32_bits: Box<[u32]>,
    writer_set: Box<[u8]>,
}

pub(super) struct SpatialControlAfterStateParts {
    pub(super) callback_index: FrontendCallbackIndex,
    pub(super) source_cursor_samples: AdmSampleIndex,
    pub(super) normal_callbacks: NormalCallbackCount,
    pub(super) source_pcm_rows: Vec<Vec<u32>>,
    pub(super) control38_rows: [NormalControlRow; PHYSICAL_HOST_COUNT],
    pub(super) regular_raw_history_bits: Box<[u32]>,
    pub(super) previous_current_rows_bits: Box<[u32]>,
    pub(super) diffusion_tails: [Option<DiffusionTailState>; REGULAR_TAIL_COUNT],
    pub(super) reader_target_state: ReaderTargetState,
    pub(super) projection: SpatialControlCallbackProjection,
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
    reader_targets: ReaderTargetState,
    matrix_assignment: MatrixAssignmentState,
    canonical_bytes: Box<[u8]>,
    digest: SpatialControlDigest,
}

impl SpatialControlState {
    fn from_cold(
        f01e_canonical_bytes: &[u8],
        f03: &ValidatedF03TargetColdProvider,
        reader_targets: ReaderTargetState,
    ) -> Self {
        let cold_pcm: Arc<[u32]> = vec![0; PHYSICAL_HOST_COUNT * 1_536].into();
        let mut state = Self {
            clocks: ClockState {
                callback_index: FrontendCallbackIndex::new(0),
                source_cursor_samples: AdmSampleIndex::new(0),
                normal_callbacks: NormalCallbackCount::new(NORMAL_CALLBACKS_COLD),
                remaining_cached_drains: CACHED_DRAINS_COLD,
                retired: false,
            },
            source_cache: SourceCacheState {
                last_normal_host38_bits: None,
                last_control_snapshot: None,
                phase_u8: 0,
            },
            event_control_canonical_bytes: f01e_canonical_bytes.to_vec().into_boxed_slice(),
            render_audio: RenderAudioState {
                regular_raw_history_bits: vec![0; REGULAR_TAIL_COUNT * 1_536].into_boxed_slice(),
                previous_pcm_bits: cold_pcm,
                previous_current_rows_bits: vec![0; 48 * 1536].into_boxed_slice(),
                diffusion_tails: std::array::from_fn(|_| None),
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
            },
            reader_targets,
            matrix_assignment: MatrixAssignmentState {
                assignment_i32: [-1; 15],
                markers_i32: [0; 15],
                matrix_f32_bits: vec![0; 15 * 48].into_boxed_slice(),
                writer_set: Box::new([]),
            },
            canonical_bytes: Box::new([]),
            digest: SpatialControlDigest([0; 32]),
        };
        let canonical_bytes = state.serialize_v6();
        state.digest = SpatialControlDigest::digest(&canonical_bytes);
        state.canonical_bytes = canonical_bytes.into_boxed_slice();
        state
    }

    fn serialize_v6(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"JOCSCS6\0");

        push_u64(&mut bytes, u64::from(self.clocks.callback_index.value()));
        push_u64(&mut bytes, self.clocks.source_cursor_samples.value());
        push_u64(&mut bytes, self.clocks.normal_callbacks.value());
        push_u32(&mut bytes, self.clocks.remaining_cached_drains);
        bytes.push(u8::from(self.clocks.retired));

        push_optional_u32_slice(
            &mut bytes,
            self.source_cache.last_normal_host38_bits.as_deref(),
        );
        match &self.source_cache.last_control_snapshot {
            None => bytes.push(0),
            Some(rows) => {
                bytes.push(1);
                push_u32(&mut bytes, PHYSICAL_HOST_COUNT as u32);
                for row in rows.iter() {
                    match row {
                        PhysicalControlSnapshotRow::Bed6(words) => {
                            bytes.push(6);
                            push_u32_slice(&mut bytes, words);
                        }
                        PhysicalControlSnapshotRow::Object13(words) => {
                            bytes.push(13);
                            push_u32_slice(&mut bytes, words);
                        }
                    }
                }
            }
        }
        bytes.push(self.source_cache.phase_u8);

        bytes.extend_from_slice(&self.event_control_canonical_bytes);

        push_u32_slice(&mut bytes, &self.render_audio.regular_raw_history_bits);
        push_u32_slice(&mut bytes, &self.render_audio.previous_pcm_bits);
        push_u32_slice(&mut bytes, &self.render_audio.previous_current_rows_bits);
        push_u32(&mut bytes, REGULAR_TAIL_COUNT as u32);
        for tail in &self.render_audio.diffusion_tails {
            match tail {
                None => bytes.push(0),
                Some(tail) => {
                    bytes.push(1);
                    let tail_bytes = tail.canonical_bytes();
                    push_u32(&mut bytes, tail_bytes.len() as u32);
                    bytes.extend_from_slice(&tail_bytes);
                }
            }
        }

        push_u32_slice(&mut bytes, &self.feature.source_vector_bits);
        push_u32_slice(&mut bytes, &self.feature.source_scalar_bits);

        push_u32_slice(&mut bytes, &self.targets.current_xyz_u32);
        push_u32_slice(&mut bytes, &self.targets.current_id_u32);
        push_u32_slice(&mut bytes, &self.targets.current_attribute_u32);
        push_u32_slice(&mut bytes, &self.targets.previous_xyz_u32);
        push_u32_slice(&mut bytes, &self.targets.previous_id_u32);
        push_u32_slice(&mut bytes, &self.targets.previous_attribute_u32);
        bytes.extend_from_slice(&self.targets.active_u8);

        let reader_target_bytes = self.reader_targets.canonical_bytes();
        push_u32(&mut bytes, reader_target_bytes.len() as u32);
        bytes.extend_from_slice(&reader_target_bytes);

        for value in self.matrix_assignment.assignment_i32 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in self.matrix_assignment.markers_i32 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        push_u32_slice(&mut bytes, &self.matrix_assignment.matrix_f32_bits);
        push_u32(&mut bytes, self.matrix_assignment.writer_set.len() as u32);
        bytes.extend_from_slice(&self.matrix_assignment.writer_set);

        bytes
    }

    fn from_callback_parts(
        before: &Self,
        parts: SpatialControlAfterStateParts,
        source_vector_bits: Box<[u32]>,
        source_scalar_bits: Box<[u32]>,
    ) -> Result<Self, SpatialControlError> {
        let next_callback = before
            .clocks
            .callback_index
            .value()
            .checked_add(1)
            .ok_or(SpatialControlError::CallbackAfterStateInvalid)?;
        let next_normal_callbacks = before
            .clocks
            .normal_callbacks
            .value()
            .checked_add(1)
            .ok_or(SpatialControlError::CallbackAfterStateInvalid)?;
        if parts.callback_index != before.clocks.callback_index
            || parts.normal_callbacks.value() != next_normal_callbacks
            || parts.source_cursor_samples.value() < before.clocks.source_cursor_samples.value()
            || parts.source_pcm_rows.len() != PHYSICAL_HOST_COUNT
            || parts.source_pcm_rows.iter().any(|row| row.len() != 1_536)
            || parts.regular_raw_history_bits.len() != REGULAR_TAIL_COUNT * 1_536
            || parts.previous_current_rows_bits.len() != 48 * 1_536
            || source_vector_bits.len() != 48 * 24
            || source_scalar_bits.len() != 48
            || parts
                .control38_rows
                .iter()
                .enumerate()
                .any(|(host, row)| usize::from(row.physical_host()) != host)
        {
            return Err(SpatialControlError::CallbackAfterStateInvalid);
        }

        let pcm_bits: Arc<[u32]> = parts
            .source_pcm_rows
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .into();
        if pcm_bits.len() != PHYSICAL_HOST_COUNT * 1_536
            || parts
                .previous_current_rows_bits
                .iter()
                .chain(&parts.regular_raw_history_bits)
                .chain(&source_vector_bits)
                .chain(&source_scalar_bits)
                .any(|word| !f32::from_bits(*word).is_finite())
        {
            return Err(SpatialControlError::CallbackAfterStateInvalid);
        }

        let control_rows = parts
            .control38_rows
            .map(|row| PhysicalControlSnapshotRow::from_normal_control(&row));
        let projection = parts.projection;
        let target = projection.target_state();
        let previous_marker_active_u8 = parts
            .reader_target_state
            .previous_active_u8()
            .map_err(|_| SpatialControlError::CallbackAfterStateInvalid)?;
        if target.active_flags.iter().any(|flag| *flag > 1)
            || target
                .xyz_words
                .iter()
                .flatten()
                .chain(target.previous_xyz_words.iter().flatten())
                .any(|word| !f32::from_bits(*word).is_finite())
            || previous_marker_active_u8.iter().any(|flag| *flag > 1)
            || projection
                .markers_i32()
                .iter()
                .any(|marker| !(-1..=1).contains(marker))
            || projection
                .assignment_i32()
                .iter()
                .any(|assignment| !(-1..15).contains(assignment))
            || projection.matrix_f32_bits().len() != 15 * 48
            || projection
                .matrix_f32_bits()
                .iter()
                .any(|word| !f32::from_bits(*word).is_finite())
            || projection.writer_set().len() > 15
            || projection.writer_set().iter().any(|slot| *slot >= 15)
            || projection
                .writer_set()
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(SpatialControlError::CallbackAfterStateInvalid);
        }

        let mut state = Self {
            clocks: ClockState {
                callback_index: FrontendCallbackIndex::new(next_callback),
                source_cursor_samples: parts.source_cursor_samples,
                normal_callbacks: parts.normal_callbacks,
                remaining_cached_drains: before.clocks.remaining_cached_drains,
                retired: false,
            },
            source_cache: SourceCacheState {
                last_normal_host38_bits: Some(Arc::clone(&pcm_bits)),
                last_control_snapshot: Some(Box::new(control_rows)),
                phase_u8: 1,
            },
            event_control_canonical_bytes: before.event_control_canonical_bytes.clone(),
            render_audio: RenderAudioState {
                regular_raw_history_bits: parts.regular_raw_history_bits,
                previous_pcm_bits: pcm_bits,
                previous_current_rows_bits: parts.previous_current_rows_bits,
                diffusion_tails: parts.diffusion_tails,
            },
            feature: FeatureHistoryState {
                source_vector_bits,
                source_scalar_bits,
            },
            targets: TargetState {
                current_xyz_u32: target
                    .xyz_words
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .try_into()
                    .map_err(|_| SpatialControlError::CallbackAfterStateInvalid)?,
                current_id_u32: target.ids_u32,
                current_attribute_u32: target.attributes_u32,
                previous_xyz_u32: target
                    .previous_xyz_words
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .try_into()
                    .map_err(|_| SpatialControlError::CallbackAfterStateInvalid)?,
                previous_id_u32: target.previous_ids_u32,
                previous_attribute_u32: target.previous_attributes_u32,
                active_u8: target.active_flags,
            },
            reader_targets: parts.reader_target_state,
            matrix_assignment: MatrixAssignmentState {
                assignment_i32: *projection.assignment_i32(),
                markers_i32: *projection.markers_i32(),
                matrix_f32_bits: projection.matrix_f32_bits().to_vec().into_boxed_slice(),
                writer_set: projection.writer_set().to_vec().into_boxed_slice(),
            },
            canonical_bytes: Box::new([]),
            digest: SpatialControlDigest([0; 32]),
        };
        let canonical_bytes = state.serialize_v6();
        state.digest = SpatialControlDigest::digest(&canonical_bytes);
        state.canonical_bytes = canonical_bytes.into_boxed_slice();
        Ok(state)
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

    pub fn previous_marker_active_u8(&self) -> Result<[u8; 15], SpatialControlError> {
        self.reader_targets
            .previous_active_u8()
            .map_err(|_| SpatialControlError::CallbackAfterStateInvalid)
    }

    pub const fn reader_targets_callback_index(&self) -> FrontendCallbackIndex {
        self.reader_targets.callback_index()
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
        self.source_cache.last_normal_host38_bits.is_none()
            && self.source_cache.last_control_snapshot.is_none()
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

    pub fn diffusion_instances_present(&self) -> bool {
        self.render_audio
            .diffusion_tails
            .iter()
            .any(Option::is_some)
    }

    pub fn diffusion_previous_outputs_present(&self) -> bool {
        self.render_audio.diffusion_tails.iter().any(|tail| {
            tail.as_ref()
                .is_some_and(|tail| tail.previous_filter_outputs().is_some())
        })
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

    pub fn writer_set_record_count(&self) -> u32 {
        self.matrix_assignment.writer_set.len() as u32
    }
}

/// Opaque complete v6 after-root staged from producer-owned callback capabilities.
#[allow(dead_code)]
#[must_use]
pub struct SpatialControlAfterProposal {
    after_state: SpatialControlState,
    join_metadata: SpatialControlJoinMetadata,
    source_flags: SourceFlagsPacket,
    current_features: CurrentFeaturesPacket,
    reader_target_receipt: Option<ReaderTargetUpdateReceipt>,
    reader_target_controls: ReaderTargetControlRows,
}

impl SpatialControlAfterProposal {
    pub const fn after_state(&self) -> &SpatialControlState {
        &self.after_state
    }

    pub const fn join_metadata(&self) -> &SpatialControlJoinMetadata {
        &self.join_metadata
    }

    pub const fn callback_packets(&self) -> (&SourceFlagsPacket, &CurrentFeaturesPacket) {
        (&self.source_flags, &self.current_features)
    }

    pub const fn reader_target_controls(&self) -> &ReaderTargetControlRows {
        &self.reader_target_controls
    }

    pub const fn reader_target_receipt(&self) -> Option<&ReaderTargetUpdateReceipt> {
        self.reader_target_receipt.as_ref()
    }
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

/// Opaque constructor authority for the future spatial-control render stage.
pub(super) struct SpatialControlRenderAuthority {
    _private: (),
}

impl SpatialControlRenderAuthority {
    /// Only descendants of spatial_control can mint production render authority.
    #[allow(dead_code)]
    fn from_spatial_control_renderer() -> Self {
        Self { _private: () }
    }

    #[cfg(test)]
    pub(in crate::spatial) fn fixture() -> Self {
        Self { _private: () }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SpatialControlJoinMetadata {
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    before_generation: u64,
    before_digest: SpatialControlDigest,
    after_digest: SpatialControlDigest,
    profile_identity: EncodingProfile,
    provider_lineage: SpatialControlProviderLineage,
}

impl SpatialControlJoinMetadata {
    /// Callback-local staging identity from the committed before root. The enclosing F03W parent
    /// supplies the complete after root and admission only after all callback stages succeed.
    pub(super) fn pending_callback(
        owner: &SpatialControlMachine,
        transaction_id: CallbackLineageIdentity,
    ) -> Result<Self, SpatialControlError> {
        let callback_index = owner.committed().callback_index();
        if transaction_id.callback_index() != callback_index || owner.committed().retired() {
            return Err(SpatialControlError::RenderCallbackMismatch);
        }
        Ok(Self {
            transaction_id,
            callback_index,
            before_generation: owner.generation(),
            before_digest: owner.digest(),
            after_digest: owner.digest(),
            profile_identity: owner.profile(),
            provider_lineage: owner.provider_lineage(),
        })
    }

    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
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

    pub(super) fn render_identity(&self) -> SpatialControlRenderIdentity {
        SpatialControlRenderIdentity {
            transaction_id: self.transaction_id.clone(),
            callback_index: self.callback_index,
            before_generation: self.before_generation,
            before_digest: self.before_digest,
            profile_identity: self.profile_identity,
            provider_lineage: self.provider_lineage,
        }
    }

    #[cfg(test)]
    pub(crate) fn fixture(
        machine: &SpatialControlMachine,
        transaction_id: CallbackLineageIdentity,
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

/// Single committed owner for the complete v6 persistent cold root.
#[must_use]
pub struct SpatialControlMachine {
    committed: SpatialControlState,
    generation: u64,
    cold_digest: SpatialControlDigest,
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
        let reader_targets = ReaderTargetState::cold();
        if reader_targets
            .previous_active_u8()
            .map_err(|_| SpatialControlError::CallbackAfterStateInvalid)?
            != *f03.previous_marker_active_u8()
        {
            return Err(SpatialControlError::CallbackAfterStateInvalid);
        }
        let committed = SpatialControlState::from_cold(f01e.canonical_bytes(), f03, reader_targets);
        let cold_digest = committed.digest();
        Ok(Self {
            committed,
            generation: 0,
            cold_digest,
            profile_identity,
            provider_lineage,
        })
    }

    pub(super) fn propose_callback_after(
        &self,
        transaction_id: CallbackLineageIdentity,
        parts: SpatialControlAfterStateParts,
        current_features: CurrentFeaturesProduced,
        reader_target_receipt: Option<ReaderTargetUpdateReceipt>,
        reader_target_controls: ReaderTargetControlRows,
    ) -> Result<SpatialControlAfterProposal, SpatialControlError> {
        let CurrentFeaturesProduced {
            feature_state_proposal,
            packet,
            source_flags,
            lineage,
        } = current_features;
        if transaction_id.callback_index() != self.committed.callback_index()
            || lineage.callback_index != self.committed.callback_index()
            || lineage.transaction_id != transaction_id
            || lineage.before_generation != self.generation
            || lineage.before_digest != self.digest()
            || feature_state_proposal.before_generation != self.generation
            || feature_state_proposal.before_digest != self.digest()
            || lineage.profile != self.profile_identity
            || lineage.provider_lineage != self.provider_lineage
            || source_flags.lineage().transaction_id() != &transaction_id
            || source_flags.lineage().callback_index() != self.committed.callback_index()
            || source_flags.validate_payload().is_err()
        {
            return Err(SpatialControlError::CallbackAfterStateInvalid);
        }

        let after_state = SpatialControlState::from_callback_parts(
            &self.committed,
            parts,
            feature_state_proposal.source_vector_bits,
            feature_state_proposal.source_scalar_bits,
        )?;
        let mut join_metadata = SpatialControlJoinMetadata::pending_callback(self, transaction_id)?;
        join_metadata.after_digest = after_state.digest();
        Ok(SpatialControlAfterProposal {
            after_state,
            join_metadata,
            source_flags,
            current_features: packet,
            reader_target_receipt,
            reader_target_controls,
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
        let canonical_bytes = self.committed.serialize_v6();
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

    /// Returns the complete current root through the registered logical GTC state_ref.
    pub fn state_reference(&self) -> SpatialControlOwnerStateReference<'_> {
        SpatialControlOwnerStateReference {
            state: &self.committed,
            generation: self.generation,
            state_sha256: self.committed.digest(),
            cold_sha256: self.cold_digest,
        }
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

    /// Binds a render payload to its committed callback-before identity without requiring the
    /// complete after-state that is produced only after feature/target/matrix stages finish.
    pub(super) fn render_identity(
        &self,
        transaction_id: CallbackLineageIdentity,
    ) -> Result<SpatialControlRenderIdentity, SpatialControlError> {
        let callback_index = self.committed.callback_index();
        if transaction_id.callback_index() != callback_index || self.committed.retired() {
            return Err(SpatialControlError::RenderCallbackMismatch);
        }
        if resolve_selected_profile(&self.profile_identity).is_err() {
            return Err(SpatialControlError::UnsupportedProfile);
        }
        Ok(SpatialControlRenderIdentity {
            transaction_id,
            callback_index,
            before_generation: self.generation,
            before_digest: self.digest(),
            profile_identity: self.profile_identity,
            provider_lineage: self.provider_lineage,
        })
    }

    /// Keeps render authority construction inside the selected spatial-control owner.
    pub(super) fn render_authority() -> SpatialControlRenderAuthority {
        SpatialControlRenderAuthority::from_spatial_control_renderer()
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

fn push_optional_u32_slice(bytes: &mut Vec<u8>, values: Option<&[u32]>) {
    match values {
        None => bytes.push(0),
        Some(values) => {
            bytes.push(1);
            push_u32(
                bytes,
                u32::try_from(values.len()).expect("optional U32 slice length fits U32"),
            );
            push_u32_slice(bytes, values);
        }
    }
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
    fn owner_state_reference_contains_the_complete_v6_cold_root() {
        let machine = selected_machine();
        let reference = machine.state_reference();

        assert_eq!(reference.owner_id(), "spatial_control");
        assert_eq!(reference.state_ref(), "owners.spatial_control");
        assert_eq!(reference.generation(), 0);
        assert!(std::ptr::eq(reference.state(), machine.committed()));
        assert_eq!(
            reference.canonical_bytes(),
            machine.committed().canonical_bytes()
        );
        assert_eq!(reference.canonical_bytes().len(), 777_665);
        assert_eq!(reference.state_sha256(), machine.digest());
        assert_eq!(reference.cold_sha256(), machine.digest());
        assert_eq!(
            reference.state_sha256().to_hex(),
            "d68ef7af3e4d2fd0575926e3e44e1bdeda80db3e9c21a1fd1a8daf95010fbb50"
        );
    }

    #[test]
    fn owner_state_reference_keeps_the_cold_digest_separate_from_current_state() {
        let mut machine = selected_machine();
        let cold_sha256 = machine.state_reference().cold_sha256();
        let mut source_vector_bits = vec![0; 48 * 24];
        source_vector_bits[0] = 0x3f80_0000;

        machine.set_current_features_test_history(&source_vector_bits, &[0; 48]);

        let updated = machine.state_reference();
        assert_ne!(updated.state_sha256(), cold_sha256);
        assert_eq!(updated.cold_sha256(), cold_sha256);
        assert_eq!(updated.generation(), 0);
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
