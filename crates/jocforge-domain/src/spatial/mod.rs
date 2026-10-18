// pattern: Functional Core
//! Spatial-owner storage boundary. CurrentFeatures stages a proposal; render and owner composition remain deferred.

#[allow(dead_code)]
pub mod current_features;
pub mod source_flags;
#[allow(dead_code)]
pub(crate) mod spatial_audio;
pub mod spatial_audio_state;
pub mod spatial_control;

pub use current_features::CurrentFeaturesPacket;
pub use source_flags::SourceFlagsPacket;
pub use spatial_audio_state::{
    SpatialAudioAfterProposal, SpatialAudioDigest, SpatialAudioError, SpatialAudioJoinMetadata,
    SpatialAudioMachine, SpatialAudioPrepared, SpatialAudioStaged, SpatialAudioState,
};
pub use spatial_control::{
    PacketReadiness, SpatialControlAfterProposal, SpatialControlDigest, SpatialControlError,
    SpatialControlJoinMetadata, SpatialControlMachine, SpatialControlPrepared,
    SpatialControlProviderLineage, SpatialControlStaged, SpatialControlState,
};
