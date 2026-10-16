// pattern: Functional Core
//! Spatial-owner storage boundary. Packet producers and DSP algorithms are deferred.

pub mod spatial_control;

pub use spatial_control::{
    CurrentFeaturesPacket, PacketReadiness, SourceFlagsPacket, SpatialControlAfterProposal,
    SpatialControlDigest, SpatialControlError, SpatialControlJoinMetadata, SpatialControlMachine,
    SpatialControlPrepared, SpatialControlProviderLineage, SpatialControlStaged,
    SpatialControlState,
};
