// pattern: Functional Core
//! Selected production-profile gate and validation-only profile types.

mod production;
mod validation;

pub use production::{
    AuthoredInputProfile, CodecRateProfile, CoreCarrierProfile, EmdfCarriageProfile,
    EncodingProfile, JocDownmixProfile, ProfileError, ResolvedEncodingProfile, SubstreamTopology,
    resolve_selected_profile,
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
