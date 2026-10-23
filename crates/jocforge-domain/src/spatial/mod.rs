// pattern: Functional Core
//! Spatial-owner storage boundary. CurrentFeatures stages a proposal; render and owner composition remain deferred.

pub mod candidate_selector;
pub mod component_parameters_provider;
#[allow(dead_code)]
pub mod current_features;
pub mod descriptor_mathematical_provider;
pub mod outer_active_update;
pub mod outer_empty_matrix;
pub mod outer_energy_fallback;
pub mod outer_geometric_correction;
pub mod outer_postprocess;
pub mod post_window_fanout;
pub mod qmf_analysis_synthesis;
pub mod qmf_fft64;
pub mod qmf_mathematical_fft_provider;
pub mod qmf_receiver_functional_provider;
pub mod rectangular_nnls;
pub mod source_flags;
pub mod spatial_audio_state;
pub mod spatial_control;
pub mod target_slot_install;
pub mod u04_projection;
pub mod window_q15_provider;

pub use component_parameters_provider::{
    ComponentParameterProviderReceipt, ValidatedComponentParameterProvider,
    validate_component_parameter_provider,
};
pub use current_features::CurrentFeaturesPacket;
pub use descriptor_mathematical_provider::{
    DescriptorMathematicalProvider, DescriptorMathematicalProviderReceipt,
    validate_descriptor_mathematical_provider,
};
pub use qmf_fft64::{QmfFftError, QmfFftInput, QmfFftOutput, execute_qmf_fft64};
pub use qmf_mathematical_fft_provider::{
    QmfFftRootStage, QmfMathematicalFftProvider, QmfMathematicalFftProviderReceipt,
    validate_qmf_mathematical_fft_provider,
};
pub use qmf_receiver_functional_provider::{
    QmfFunctionalArraySourceClass, QmfReceiverFunctionalProvider,
    QmfReceiverFunctionalProviderReceipt, QmfReceiverFunctionalScalars,
    validate_qmf_receiver_functional_provider,
};
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
pub use window_q15_provider::{
    F04ParameterProviderError, F04ParameterProviderLineage, ValidatedF04ParameterPackageManifest,
    ValidatedWindowQ15Provider, WindowQ15ProviderReceipt, validate_f04_parameter_package_manifest,
    validate_window_q15_provider,
};
