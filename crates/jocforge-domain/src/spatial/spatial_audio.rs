// pattern: Functional Core
//! Opaque post-window render packet boundary; the render producer remains deferred.

use super::spatial_control::{SpatialControlDigest, SpatialControlProviderLineage};
use crate::{
    profile::EncodingProfile,
    spatial::source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
    timeline::FrontendCallbackIndex,
    transaction::CallbackTransactionId,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PostWindowAudioBoundary {
    Installed48PostWindow48x3072,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AudioPayloadSha256([u8; 32]);

/// Opaque identity of the render proposal that supplied this post-window payload.
#[derive(Debug)]
pub(super) struct RenderReceiptIdentity;

/// Future render-producer-owned linear input to CurrentFeatures; no production constructor exists here.
#[must_use]
pub(super) struct PostWindowAudioPacket {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackTransactionId,
    callback_index: FrontendCallbackIndex,
    profile: EncodingProfile,
    provider_lineage: SpatialControlProviderLineage,
    before_generation: u64,
    before_digest: SpatialControlDigest,
    boundary: PostWindowAudioBoundary,
    payload_f32_bits: Box<[Box<[u32]>]>,
    payload_sha256: AudioPayloadSha256,
    render_receipt_identity: Arc<RenderReceiptIdentity>,
}

impl PostWindowAudioPacket {
    pub(super) fn programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.programme_id
    }

    pub(super) const fn attempt(&self) -> SourceFlagsAttempt {
        self.attempt
    }

    pub(super) const fn transaction_id(&self) -> &CallbackTransactionId {
        &self.transaction_id
    }

    pub(super) const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub(super) const fn profile(&self) -> EncodingProfile {
        self.profile
    }

    pub(super) const fn provider_lineage(&self) -> SpatialControlProviderLineage {
        self.provider_lineage
    }

    pub(super) const fn before_generation(&self) -> u64 {
        self.before_generation
    }

    pub(super) const fn before_digest(&self) -> SpatialControlDigest {
        self.before_digest
    }

    pub(super) const fn boundary(&self) -> PostWindowAudioBoundary {
        self.boundary
    }

    pub(super) fn source_count(&self) -> usize {
        self.payload_f32_bits.len()
    }

    pub(super) fn sample_count(&self, source: usize) -> Option<usize> {
        self.payload_f32_bits.get(source).map(|row| row.len())
    }

    pub(super) fn source_bits(&self, source: usize) -> Option<&[u32]> {
        self.payload_f32_bits.get(source).map(Box::as_ref)
    }

    pub(super) const fn payload_sha256(&self) -> AudioPayloadSha256 {
        self.payload_sha256
    }

    pub(super) fn render_receipt_identity(&self) -> Arc<RenderReceiptIdentity> {
        Arc::clone(&self.render_receipt_identity)
    }
}

#[cfg(test)]
pub(in crate::spatial) mod tests;
