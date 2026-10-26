// pattern: Functional Core
//! Spatial-owner storage boundary. CurrentFeatures stages a proposal; render and owner composition remain deferred.

pub mod bed_source_points;
pub mod candidate_selector;
pub mod component_parameters_provider;
#[allow(dead_code)]
pub mod current_features;
pub mod descriptor_mathematical_provider;
pub mod diffusion_filter;
pub mod diffusion_tail;
pub mod frontend_stages;
pub mod installed48_activity;
pub mod installed48_source_map;
pub mod outer_active_update;
pub mod outer_empty_matrix;
pub mod outer_energy_fallback;
pub mod outer_geometric_correction;
pub mod outer_postprocess;
pub mod point_response;
pub mod post_window_fanout;
pub mod prefix_membership;
pub mod production_callback;
pub mod qmf_analysis_synthesis;
pub mod qmf_fft64;
pub mod qmf_mathematical_fft_provider;
pub mod qmf_receiver_functional_provider;
pub mod rectangular_nnls;
pub mod selected_p11_geometry;
pub mod selected_p15_geometry;
pub mod selector_u04_handoff;
pub mod source_flags;
pub mod source_geometry_outer;
pub mod spatial_audio_state;
pub mod spatial_control;
pub mod target_context;
pub mod target_slot_install;
pub mod u04_projection;
pub mod u04_refinement;
pub mod virtual_prefix_geometry;
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
pub use production_callback::{
    NormalControlCallbackSource, SpatialProductionCallbackError, SpatialProductionCallbackInputs,
    SpatialProductionCallbackProposal, continue_spatial_production_callback,
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
    ComponentBeforeOverlapRows, ComponentStageError, MeterComponentOutputRows,
    MeterComponentStagePreview, MeterComponentStageProposal, SpatialAudioAfterProposal,
    SpatialAudioDigest, SpatialAudioError, SpatialAudioJoinMetadata, SpatialAudioMachine,
    SpatialAudioPrepared, SpatialAudioStaged, SpatialAudioState,
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
