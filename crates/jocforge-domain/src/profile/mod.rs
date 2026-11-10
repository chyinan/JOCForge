// pattern: Functional Core
//! Selected production-profile gate and validation-only profile types.

mod flat7x_access_unit_target;
mod production;
mod validation;

pub use flat7x_access_unit_target::{
    D0EmdfSkipFieldExtent, D0EmdfSkipFieldTarget, D0FrameExtent, DependentChannelMap, Eac3Acmod,
    Eac3CodedChannel, Eac3SubstreamId, Flat7xAuByteSequenceCandidate, Flat7xAuByteSequenceError,
    Flat7xAuCandidate, Flat7xAuPlanValidation, Flat7xAuPlanValidationError, Flat7xDownmixControls,
    Flat7xEmdfOverflowPolicy, Flat7xFrameTarget, Flat7xI0D0TargetPlan, Flat7xOpenGate,
    Flat7xSourceGeneration, Flat7xSubstreamRole, I0FrameExtent, ReconstructionCarrierPosition,
    compose_flat7x_au_byte_sequence, validate_flat7x_au_candidate,
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
