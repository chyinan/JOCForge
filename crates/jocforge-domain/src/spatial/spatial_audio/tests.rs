// pattern: Imperative Shell
use super::{
    AudioPayloadSha256, PostWindowAudioBoundary, PostWindowAudioPacket, RenderReceiptIdentity,
};
use crate::{
    profile::EncodingProfile,
    spatial::source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
    spatial::spatial_control::{SpatialControlDigest, SpatialControlProviderLineage},
    timeline::FrontendCallbackIndex,
    transaction::CallbackTransactionId,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Test-only raw input adapter for consumer boundary vectors.
#[allow(clippy::too_many_arguments)]
pub(in crate::spatial) fn fixture(
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackTransactionId,
    callback_index: FrontendCallbackIndex,
    profile: EncodingProfile,
    provider_lineage: &SpatialControlProviderLineage,
    before_generation: u64,
    before_digest: SpatialControlDigest,
    payload_f32_bits: Vec<Vec<u32>>,
) -> PostWindowAudioPacket {
    let mut hasher = Sha256::new();
    for row in &payload_f32_bits {
        for word in row {
            hasher.update(word.to_le_bytes());
        }
    }
    let digest = hasher.finalize();
    let mut digest_bytes = [0; 32];
    digest_bytes.copy_from_slice(&digest);
    PostWindowAudioPacket {
        programme_id,
        attempt,
        transaction_id,
        callback_index,
        profile,
        provider_lineage: *provider_lineage,
        before_generation,
        before_digest,
        boundary: PostWindowAudioBoundary::Installed48PostWindow48x3072,
        payload_f32_bits: payload_f32_bits
            .into_iter()
            .map(Vec::into_boxed_slice)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        payload_sha256: AudioPayloadSha256(digest_bytes),
        render_receipt_identity: Arc::new(RenderReceiptIdentity),
    }
}
