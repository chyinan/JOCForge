// pattern: Functional Core

use std::fmt;

use crate::{
    BlockPartition, BlockPartitionStatus, EmdfProfile, JocCarriage, JocProfile, JocProfileSpec,
    RateBudget, ShortBlockPlan, classify_block_partition, profile_spec, resolve_short_block_plan,
};

/// Immutable encoding choices consumed by the JOC carriage writer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JocEncodingPlan {
    pub profile: JocProfile,
    pub spec: JocProfileSpec,
    pub carriage: JocCarriage,
    pub emdf_profile: EmdfProfile,
    pub rate: RateBudget,
    pub block_partition: ShortBlockPlan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
    InvalidBlockPartition {
        actual_blocks: usize,
    },
    UnsupportedBlockPartition {
        profile: JocProfile,
        carriage: JocCarriage,
        status: BlockPartitionStatus,
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
            Self::InvalidBlockPartition { actual_blocks } => write!(
                formatter,
                "invalid short-block partition: totals {actual_blocks} audio blocks; expected 6"
            ),
            Self::UnsupportedBlockPartition {
                profile,
                carriage,
                status,
            } => write!(
                formatter,
                "unsupported short-block partition ({status}) for profile {} and carriage {carriage:?}",
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
    resolve_encoding_plan_with_partition(
        profile,
        carriage,
        emdf_profile,
        bitrate_kbps,
        &[BlockPartition::Six],
    )
}

/// Resolves a complete plan with an explicit raw E-AC-3 block partition.
///
/// The default resolver above remains the stable six-block API. Short
/// partitions are admitted only for strict raw ETSI auxiliary-dependent
/// carriage; deployed skip-field, legacy-core, and other combinations remain
/// fail-closed until independently closed.
pub fn resolve_encoding_plan_with_partition(
    profile: JocProfile,
    carriage: JocCarriage,
    emdf_profile: EmdfProfile,
    bitrate_kbps: u32,
    partitions: &[BlockPartition],
) -> Result<JocEncodingPlan, JocPlanError> {
    let block_partition = resolve_short_block_plan(partitions).map_err(|error| {
        JocPlanError::InvalidBlockPartition {
            actual_blocks: match error {
                crate::ConformancePlanError::EmptyBlockPlan => 0,
                crate::ConformancePlanError::TotalBlocks { actual }
                | crate::ConformancePlanError::DependentCount { actual } => actual,
            },
        }
    })?;
    let status = classify_block_partition(partitions);
    if status != BlockPartitionStatus::PublicLegal
        || (!block_partition.is_default()
            && (carriage != JocCarriage::EtsiAuxDependent
                || emdf_profile != EmdfProfile::EtsiStrict))
    {
        return Err(JocPlanError::UnsupportedBlockPartition {
            profile,
            carriage,
            status,
        });
    }
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
        block_partition,
    })
}
