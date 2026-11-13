// pattern: Functional Core
//! Selected production-profile gate and validation-only profile types.

mod flat7x_access_unit_target;
mod flat7x_source_role_map;
mod production;
mod validation;

pub use flat7x_access_unit_target::{
    D0EmdfSkipFieldExtent, D0EmdfSkipFieldTarget, D0FrameExtent, DependentChannelMap, DownmixIndex,
    Eac3Acmod, Eac3CodedChannel, Eac3SubstreamId, Flat7xAuByteSequenceCandidate,
    Flat7xAuByteSequenceError, Flat7xAuCandidate, Flat7xAuPlanValidation,
    Flat7xAuPlanValidationError, Flat7xDownmixControls, Flat7xEmdfOverflowPolicy, Flat7xFrameCodec,
    Flat7xFrameTarget, Flat7xI0D0TargetPlan, Flat7xOpenGate, Flat7xSourceGeneration,
    Flat7xSubstreamRole, I0FrameExtent, JocDownmixAlgorithmControls, JocSpatialIndex,
    ReconstructionCarrierPosition, compose_flat7x_au_byte_sequence, validate_flat7x_au_candidate,
};
pub use flat7x_source_role_map::{
    AdmPcmRow, Flat7xD0CarrierPcmRows, Flat7xD0CarrierProjectionError, Flat7xD0CarrierSource,
    Flat7xD0ChannelMapLocation, Flat7xD0ChannelSource, Flat7xD0PcmRows, Flat7xD0RowBinding,
    Flat7xD0SourceRole, Flat7xI0CarrierRole, Flat7xI0PcmRows, Flat7xI0RowBinding,
    Flat7xProvisionalSourcePcmView, Flat7xProvisionalSourceRoleMap, Flat7xSourceRoleMapError,
    PROVISIONAL_D0_CARRIER_SOURCES, build_flat7x_provisional_source_role_map,
    project_flat7x_d0_carrier_pcm_rows, project_flat7x_source_pcm_rows,
};

pub use production::{
    AuthoredInputProfile, CodecRateProfile, CoreCarrierProfile, EmdfCarriageProfile,
    EncodingProfile, JocDownmixProfile, JocReconstructionLayout, ProfileError,
    ResolvedEncodingProfile, SelectedDataRate, SubstreamTopology, resolve_selected_profile,
};
pub use validation::{RenderTarget, ValidationProfile};

/// Validation profiles cannot enter production resolution.
///
/// ```compile_fail
/// use jocforge_domain::profile::{resolve_selected_profile, RenderTarget, ValidationProfile};
/// let validation = ValidationProfile::new(RenderTarget::OpenJoc714);
/// resolve_selected_profile(&validation);
/// ```
pub struct ProductionValidationTypeSeparation;
