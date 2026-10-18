// pattern: Functional Core
//! One-time fanout capability for the selected installed48 post-window payload.

use super::spatial_control::SpatialControlRenderAuthority;
use super::{SpatialControlDigest, SpatialControlJoinMetadata, SpatialControlProviderLineage};
use crate::{
    numeric::{FiniteF32, FiniteF32Error},
    profile::EncodingProfile,
    spatial::source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
    timeline::FrontendCallbackIndex,
    transaction::CallbackTransactionId,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const SOURCE_COUNT: usize = 48;
const SAMPLES_PER_SOURCE: usize = 3072;
const PAYLOAD_SAMPLE_COUNT: usize = SOURCE_COUNT * SAMPLES_PER_SOURCE;
pub(super) const PAYLOAD_BYTE_COUNT: u64 =
    (PAYLOAD_SAMPLE_COUNT * std::mem::size_of::<u32>()) as u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackPhase {
    Normal,
    CachedDrain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostWindowBoundary {
    Installed48PostWindow48x3072,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioPayloadSha256([u8; 32]);

/// Opaque identity allocated once for a single spatial-control render stage.
#[derive(Debug)]
pub struct RenderStageIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PostWindowFanoutError {
    PayloadShape,
    NonFinitePayload,
}

#[derive(Clone)]
struct PostWindowLineage {
    programme_id: Arc<SourceFlagsProgrammeId>,
    attempt: SourceFlagsAttempt,
    phase: CallbackPhase,
    boundary: PostWindowBoundary,
    payload_sha256: AudioPayloadSha256,
    payload_byte_count: u64,
    render_stage_identity: Arc<RenderStageIdentity>,
    spatial_control_after_proposal_identity: Arc<SpatialControlJoinMetadata>,
}

/// A single-use proposal owned by the future spatial-control renderer.
///
/// Its renderer-only constructor has no public raw-input counterpart. The consumed proposal
/// yields exactly one view for each selected consumer.
///
/// ```compile_fail
/// use jocforge_domain::spatial::post_window_fanout::CurrentFeaturesAudioView;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<CurrentFeaturesAudioView>();
/// ```
/// ```compile_fail
/// use jocforge_domain::spatial::post_window_fanout::SpatialAudioPostWindowView;
/// fn requires_copy<T: Copy>() {}
/// requires_copy::<SpatialAudioPostWindowView>();
/// ```
/// ```compile_fail
/// use jocforge_domain::spatial::post_window_fanout::{
///     CurrentFeaturesAudioView, SpatialAudioPostWindowView,
/// };
/// fn current_features_only(_: CurrentFeaturesAudioView) {}
/// fn cannot_substitute_spatial_view(view: SpatialAudioPostWindowView) {
///     current_features_only(view);
/// }
/// ```
/// ```compile_fail
/// use jocforge_domain::spatial::post_window_fanout::PostWindowRenderProposal;
/// fn consume_twice(proposal: PostWindowRenderProposal) {
///     let _ = proposal.fanout();
///     let _ = proposal.fanout();
/// }
/// ```
/// ```compile_fail
/// use jocforge_domain::spatial::spatial_control::SpatialControlRenderAuthority;
/// let _authority = SpatialControlRenderAuthority::from_spatial_control_renderer();
/// ```
#[must_use]
pub struct PostWindowRenderProposal {
    payload: Arc<[FiniteF32]>,
    lineage: PostWindowLineage,
}

/// Linear read-only audio view for CurrentFeatures.
#[must_use]
pub struct CurrentFeaturesAudioView {
    payload: Arc<[FiniteF32]>,
    lineage: PostWindowLineage,
}

/// Linear read-only audio view reserved for the future spatial-audio input assembler.
#[allow(dead_code)]
#[must_use]
pub struct SpatialAudioPostWindowView {
    payload: Arc<[FiniteF32]>,
    lineage: PostWindowLineage,
}

impl PostWindowRenderProposal {
    /// Future spatial-control renderer entry. The producer supplies one complete source-major view.
    #[allow(dead_code)]
    pub(super) fn try_from_spatial_control_renderer(
        _authority: SpatialControlRenderAuthority,
        programme_id: SourceFlagsProgrammeId,
        attempt: SourceFlagsAttempt,
        spatial_control_after_proposal_identity: Arc<SpatialControlJoinMetadata>,
        phase: CallbackPhase,
        payload: Vec<f32>,
    ) -> Result<Self, PostWindowFanoutError> {
        if payload.len() != PAYLOAD_SAMPLE_COUNT {
            return Err(PostWindowFanoutError::PayloadShape);
        }
        let mut finite_payload = Vec::with_capacity(PAYLOAD_SAMPLE_COUNT);
        let mut hasher = Sha256::new();
        for sample in payload {
            let finite = FiniteF32::new(sample).map_err(map_finite_error)?;
            hasher.update(finite.to_bits().to_le_bytes());
            finite_payload.push(finite);
        }
        let digest = hasher.finalize();
        let mut payload_sha256 = [0; 32];
        payload_sha256.copy_from_slice(&digest);

        Ok(Self {
            payload: Arc::from(finite_payload.into_boxed_slice()),
            lineage: PostWindowLineage {
                programme_id: Arc::new(programme_id),
                attempt,
                phase,
                boundary: PostWindowBoundary::Installed48PostWindow48x3072,
                payload_sha256: AudioPayloadSha256(payload_sha256),
                payload_byte_count: PAYLOAD_BYTE_COUNT,
                render_stage_identity: Arc::new(RenderStageIdentity),
                spatial_control_after_proposal_identity,
            },
        })
    }

    /// Consumes the proposal and shares its single immutable payload allocation between two views.
    pub fn fanout(self) -> (CurrentFeaturesAudioView, SpatialAudioPostWindowView) {
        let PostWindowRenderProposal { payload, lineage } = self;
        let spatial_audio_payload = Arc::clone(&payload);
        let spatial_audio_lineage = lineage.clone();
        (
            CurrentFeaturesAudioView { payload, lineage },
            SpatialAudioPostWindowView {
                payload: spatial_audio_payload,
                lineage: spatial_audio_lineage,
            },
        )
    }
}

impl CurrentFeaturesAudioView {
    pub(super) fn programme_id(&self) -> &SourceFlagsProgrammeId {
        self.lineage.programme_id.as_ref()
    }

    pub(super) fn attempt(&self) -> SourceFlagsAttempt {
        self.lineage.attempt
    }

    pub(super) fn transaction_id(&self) -> &CallbackTransactionId {
        self.lineage
            .spatial_control_after_proposal_identity
            .transaction_id()
    }

    pub(super) fn callback_index(&self) -> FrontendCallbackIndex {
        self.lineage
            .spatial_control_after_proposal_identity
            .callback_index()
    }

    pub(super) fn callback_phase(&self) -> CallbackPhase {
        self.lineage.phase
    }

    pub(super) fn profile_identity(&self) -> EncodingProfile {
        self.lineage
            .spatial_control_after_proposal_identity
            .profile_identity()
    }

    pub(super) fn provider_lineage(&self) -> SpatialControlProviderLineage {
        self.lineage
            .spatial_control_after_proposal_identity
            .provider_lineage()
    }

    pub(super) fn before_generation(&self) -> u64 {
        self.lineage
            .spatial_control_after_proposal_identity
            .before_generation()
    }

    pub(super) fn before_digest(&self) -> SpatialControlDigest {
        self.lineage
            .spatial_control_after_proposal_identity
            .before_digest()
    }

    pub(super) const fn boundary(&self) -> PostWindowBoundary {
        self.lineage.boundary
    }

    pub(super) fn payload_sha256(&self) -> AudioPayloadSha256 {
        self.lineage.payload_sha256
    }

    pub(super) fn payload_byte_count(&self) -> u64 {
        self.lineage.payload_byte_count
    }

    pub(super) fn render_stage_identity(&self) -> &Arc<RenderStageIdentity> {
        &self.lineage.render_stage_identity
    }

    pub(super) fn spatial_control_after_proposal_identity(
        &self,
    ) -> &Arc<SpatialControlJoinMetadata> {
        &self.lineage.spatial_control_after_proposal_identity
    }

    pub(super) fn sample_count(&self) -> usize {
        self.payload.len()
    }

    pub(super) fn source_samples(&self, source: usize) -> Option<&[FiniteF32]> {
        source_sample_range(&self.payload, source)
    }
}

#[allow(dead_code)]
impl SpatialAudioPostWindowView {
    pub(super) fn programme_id(&self) -> &SourceFlagsProgrammeId {
        self.lineage.programme_id.as_ref()
    }

    pub(super) fn attempt(&self) -> SourceFlagsAttempt {
        self.lineage.attempt
    }

    pub(super) fn transaction_id(&self) -> &CallbackTransactionId {
        self.lineage
            .spatial_control_after_proposal_identity
            .transaction_id()
    }

    pub(super) fn callback_index(&self) -> FrontendCallbackIndex {
        self.lineage
            .spatial_control_after_proposal_identity
            .callback_index()
    }

    pub(super) fn callback_phase(&self) -> CallbackPhase {
        self.lineage.phase
    }

    pub(super) fn profile_identity(&self) -> EncodingProfile {
        self.lineage
            .spatial_control_after_proposal_identity
            .profile_identity()
    }

    pub(super) fn provider_lineage(&self) -> SpatialControlProviderLineage {
        self.lineage
            .spatial_control_after_proposal_identity
            .provider_lineage()
    }

    pub(super) fn before_generation(&self) -> u64 {
        self.lineage
            .spatial_control_after_proposal_identity
            .before_generation()
    }

    pub(super) fn before_digest(&self) -> SpatialControlDigest {
        self.lineage
            .spatial_control_after_proposal_identity
            .before_digest()
    }

    pub(super) fn after_digest(&self) -> SpatialControlDigest {
        self.lineage
            .spatial_control_after_proposal_identity
            .after_digest()
    }

    pub(super) const fn boundary(&self) -> PostWindowBoundary {
        self.lineage.boundary
    }

    pub(super) fn payload_sha256(&self) -> AudioPayloadSha256 {
        self.lineage.payload_sha256
    }

    pub(super) fn payload_byte_count(&self) -> u64 {
        self.lineage.payload_byte_count
    }

    pub(super) fn render_stage_identity(&self) -> &Arc<RenderStageIdentity> {
        &self.lineage.render_stage_identity
    }

    pub(super) fn spatial_control_after_proposal_identity(
        &self,
    ) -> &Arc<SpatialControlJoinMetadata> {
        &self.lineage.spatial_control_after_proposal_identity
    }

    pub(super) fn sample_count(&self) -> usize {
        self.payload.len()
    }

    pub(super) fn source_samples(&self, source: usize) -> Option<&[FiniteF32]> {
        source_sample_range(&self.payload, source)
    }
}

fn source_sample_range(payload: &[FiniteF32], source: usize) -> Option<&[FiniteF32]> {
    if source >= SOURCE_COUNT || payload.len() != PAYLOAD_SAMPLE_COUNT {
        return None;
    }
    let start = source * SAMPLES_PER_SOURCE;
    Some(&payload[start..start + SAMPLES_PER_SOURCE])
}

fn map_finite_error(_: FiniteF32Error) -> PostWindowFanoutError {
    PostWindowFanoutError::NonFinitePayload
}

#[cfg(test)]
pub(in crate::spatial) mod tests;
