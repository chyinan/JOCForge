// pattern: Functional Core

use std::fmt;

use crate::{EmdfProfile, JocCarriage, JocProfile, JocProfileSpec, RateBudget, profile_spec};

/// Immutable encoding choices consumed by the JOC carriage writer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JocEncodingPlan {
    pub profile: JocProfile,
    pub spec: JocProfileSpec,
    pub carriage: JocCarriage,
    pub emdf_profile: EmdfProfile,
    pub rate: RateBudget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocPlanError {
    UnsupportedCarriage {
        profile: JocProfile,
        carriage: JocCarriage,
    },
    UnsupportedBitrate {
        profile: JocProfile,
        carriage: JocCarriage,
        bitrate_kbps: u32,
    },
}

impl fmt::Display for JocPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedCarriage { profile, carriage } => write!(
                formatter,
                "unsupported carriage {carriage:?} for profile {}",
                profile.semantic_name()
            ),
            Self::UnsupportedBitrate {
                profile,
                carriage,
                bitrate_kbps,
            } => write!(
                formatter,
                "unsupported total bitrate {bitrate_kbps} kbps for profile {} and carriage {carriage:?}",
                profile.semantic_name()
            ),
        }
    }
}

impl std::error::Error for JocPlanError {}

/// Resolves the complete public profile/carriage/signaling/rate contract.
///
/// Unsupported combinations fail here before ADM I/O, FFmpeg invocation, or
/// metadata assembly. The resolver deliberately keeps signaling independent
/// from carriage while still rejecting carriage families without a closed
/// profile-specific topology.
pub fn resolve_encoding_plan(
    profile: JocProfile,
    carriage: JocCarriage,
    emdf_profile: EmdfProfile,
    bitrate_kbps: u32,
) -> Result<JocEncodingPlan, JocPlanError> {
    if carriage == JocCarriage::I0SkipField && profile != JocProfile::Idx0 {
        return Err(JocPlanError::UnsupportedCarriage { profile, carriage });
    }
    let rate = crate::rate_budget_for_carriage(carriage, bitrate_kbps).map_err(|_| {
        JocPlanError::UnsupportedBitrate {
            profile,
            carriage,
            bitrate_kbps,
        }
    })?;
    Ok(JocEncodingPlan {
        profile,
        spec: profile_spec(profile),
        carriage,
        emdf_profile,
        rate,
    })
}
