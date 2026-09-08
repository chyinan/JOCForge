// pattern: Functional Core

use std::fmt;

use crate::{
    BlockPartition, BlockPartitionStatus, CapabilityVerdict, DependentLfePolicy, DependentPlan,
    EmdfProfile, JocCarriage, JocProfile, JocProfileSpec, OwnershipPlan, RateBudget,
    ShortBlockPlan, canonical_dependent_channel_maps, classify_block_partition, profile_spec,
    rate_budget_for_carriage, rate_budget_for_dependents, resolve_short_block_plan,
    semantic_maximum, validate_ownership_plan,
};

/// Outer form selected for a JOC composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocContainer {
    Raw,
    Cmaf,
}

impl fmt::Display for JocContainer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Raw => "raw",
            Self::Cmaf => "cmaf",
        })
    }
}

/// Programme-level LFE ownership requested by a composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LfeOwnership {
    Absent,
    Independent,
    DependentSupplement,
    DependentReplacement,
}

/// Complete public composition request. This is pure data; no ADM or process
/// I/O is performed while resolving it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionRequest {
    pub profile: JocProfile,
    pub container: JocContainer,
    pub carriage: JocCarriage,
    pub emdf_profile: EmdfProfile,
    pub bitrate_kbps: u32,
    pub partitions: Vec<BlockPartition>,
    pub dependent_count: usize,
    pub lfe_ownership: LfeOwnership,
}

/// A semantic classification with an actionable, stable reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityClassification {
    pub verdict: CapabilityVerdict,
    pub reason: String,
}

/// One row of the cheap, non-encoding composition capability lattice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionCapabilityRow {
    pub profile: JocProfile,
    pub container: JocContainer,
    pub carriage: JocCarriage,
    pub emdf_profile: EmdfProfile,
    pub partitions: Vec<BlockPartition>,
    pub dependent_count: usize,
    pub lfe_ownership: LfeOwnership,
    pub bitrate_kbps: u32,
    pub classification: CapabilityClassification,
}

/// Error returned when a composition does not resolve to an encoding plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionError {
    pub classification: CapabilityClassification,
}

impl fmt::Display for CompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {}",
            self.classification.verdict, self.classification.reason
        )
    }
}

impl std::error::Error for CompositionError {}

/// Immutable encoding choices consumed by the JOC carriage writer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JocEncodingPlan {
    pub profile: JocProfile,
    pub spec: JocProfileSpec,
    pub container: JocContainer,
    pub carriage: JocCarriage,
    pub emdf_profile: EmdfProfile,
    pub rate: RateBudget,
    pub block_partition: ShortBlockPlan,
    pub dependent_count: usize,
    pub lfe_ownership: LfeOwnership,
    pub ownership: Option<OwnershipPlan>,
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

/// Pure composition resolver shared by plan, writer, CLI and matrix callers.
pub struct CompositionResolver;

impl CompositionResolver {
    /// Resolves every public composition axis into one immutable plan.
    pub fn resolve(request: &CompositionRequest) -> Result<JocEncodingPlan, CompositionError> {
        resolve_composition_inner(request)
    }

    /// Returns the semantic result without collapsing a rejection into a
    /// generic unsupported error.
    #[must_use]
    pub fn classify(request: &CompositionRequest) -> CapabilityClassification {
        classify_composition(request)
    }
}

fn classified(verdict: CapabilityVerdict, reason: impl Into<String>) -> CompositionError {
    CompositionError {
        classification: CapabilityClassification {
            verdict,
            reason: reason.into(),
        },
    }
}

fn canonical_ownership(
    profile: JocProfile,
    dependent_count: usize,
    lfe_ownership: LfeOwnership,
) -> Result<OwnershipPlan, CompositionError> {
    let mut maps = canonical_dependent_channel_maps(profile, dependent_count)
        .map_err(|reason| classified(CapabilityVerdict::SemanticCeiling, reason))?;
    let independent_lfe = matches!(
        lfe_ownership,
        LfeOwnership::Independent | LfeOwnership::DependentReplacement
    );
    // Replacement intentionally carries the predecessor in I0 and the same
    // public location in D0; decoding replaces the I0 contribution, so the
    // programme still has one final LFE owner (the dependent).
    if matches!(
        lfe_ownership,
        LfeOwnership::DependentSupplement | LfeOwnership::DependentReplacement
    ) {
        let first = maps.first_mut().ok_or_else(|| {
            classified(
                CapabilityVerdict::Invalid,
                "dependent LFE ownership requires at least one dependent",
            )
        })?;
        *first |= 0x0001;
    }
    let dependents = maps
        .into_iter()
        .enumerate()
        .map(|(index, channel_map)| {
            let lfe_policy = match (index, lfe_ownership) {
                (0, LfeOwnership::DependentSupplement) => DependentLfePolicy::Supplement,
                (0, LfeOwnership::DependentReplacement) => DependentLfePolicy::ReplaceIndependent,
                _ => DependentLfePolicy::Absent,
            };
            DependentPlan::new(u8::try_from(index).unwrap_or(7), channel_map, lfe_policy)
        })
        .collect::<Vec<_>>();
    let plan = OwnershipPlan {
        profile,
        independent_lfe,
        final_joc_owner: u8::try_from(dependents.len().saturating_sub(1)).unwrap_or(7),
        dependents,
    };
    validate_ownership_plan(&plan).map_err(|error| {
        classified(
            CapabilityVerdict::Invalid,
            format!("invalid canonical ownership plan: {error}"),
        )
    })?;
    Ok(plan)
}

fn resolve_partition_shape(
    request: &CompositionRequest,
) -> Result<(ShortBlockPlan, BlockPartitionStatus), CompositionError> {
    let plan = resolve_short_block_plan(&request.partitions).map_err(|error| {
        classified(
            CapabilityVerdict::Invalid,
            format!("invalid block partition: {error}"),
        )
    })?;
    Ok((plan, classify_block_partition(&request.partitions)))
}

fn resolve_composition_inner(
    request: &CompositionRequest,
) -> Result<JocEncodingPlan, CompositionError> {
    let (block_partition, partition_status) = resolve_partition_shape(request)?;
    if request.dependent_count > 8 {
        return Err(classified(
            CapabilityVerdict::Invalid,
            "dependent count is outside the public D0..D7 range",
        ));
    }
    if matches!(
        request.lfe_ownership,
        LfeOwnership::DependentSupplement | LfeOwnership::DependentReplacement
    ) && request.dependent_count == 0
    {
        return Err(classified(
            CapabilityVerdict::Invalid,
            "dependent LFE ownership requires at least one dependent",
        ));
    }

    let short = !block_partition.is_default();
    match request.container {
        JocContainer::Cmaf => {
            if request.carriage != JocCarriage::EtsiAuxDependent
                || request.emdf_profile != EmdfProfile::EtsiStrict
                || short
                || partition_status != BlockPartitionStatus::PublicLegal
                || request.dependent_count != 1
                || request.lfe_ownership != LfeOwnership::Absent
            {
                return Err(classified(
                    CapabilityVerdict::NotApplicable,
                    "CMAF is closed only for strict six-block I0+D0 without LFE",
                ));
            }
        }
        JocContainer::Raw => match request.carriage {
            JocCarriage::I0SkipField => {
                if request.dependent_count != 0 {
                    return Err(classified(
                        CapabilityVerdict::NotApplicable,
                        "I0 skip-field carriage has no dependent topology",
                    ));
                }
                if matches!(
                    request.profile,
                    JocProfile::Idx1 | JocProfile::Idx2 | JocProfile::Idx4
                ) {
                    return Err(classified(
                        CapabilityVerdict::NotApplicable,
                        "I0-only skip-field carriage cannot supply the seven-channel profile inputs",
                    ));
                }
                if request.profile == JocProfile::Idx3
                    && request.emdf_profile == EmdfProfile::DeployedCompat
                {
                    return Err(classified(
                        CapabilityVerdict::NotApplicable,
                        "deployed compatibility remains limited to the idx0/1024 preset",
                    ));
                }
            }
            JocCarriage::LegacyAc3Dependent => {
                if request.dependent_count != 1 {
                    return Err(classified(
                        CapabilityVerdict::NotApplicable,
                        "legacy AC-3 carriage is closed only for one E-AC-3 D0",
                    ));
                }
                if short {
                    return Err(classified(
                        CapabilityVerdict::NotApplicable,
                        "fixed legacy AC-3 I0 does not participate in the E-AC-3 short-block axis",
                    ));
                }
                if request.bitrate_kbps != 1280
                    && !(request.profile == JocProfile::Idx1
                        && matches!(request.bitrate_kbps, 1024 | 1088 | 1152 | 1216))
                {
                    return Err(classified(
                        CapabilityVerdict::RejectedCapacity,
                        "validated legacy-core capacity is idx0..idx4 at 1280 kbps or idx1 at 1024..1216 kbps",
                    ));
                }
            }
            JocCarriage::EtsiAuxDependent => {
                if request.dependent_count == 0 {
                    return Err(classified(
                        CapabilityVerdict::NotApplicable,
                        "raw JOC carriage requires at least one dependent",
                    ));
                }
                if request.dependent_count > semantic_maximum(request.profile) {
                    return Err(classified(
                        CapabilityVerdict::SemanticCeiling,
                        format!(
                            "{} canonical ownership supports at most D0..D{}",
                            request.profile.semantic_name(),
                            semantic_maximum(request.profile).saturating_sub(1)
                        ),
                    ));
                }
            }
        },
    }

    if partition_status == BlockPartitionStatus::PublicAmbiguous {
        return Err(classified(
            CapabilityVerdict::PublicAmbiguous,
            "mixed short-frame ownership is not publicly closed; use 6, 3+3, 2+2+2, or 1x6",
        ));
    }

    let rate = if request.container == JocContainer::Raw
        && request.carriage == JocCarriage::EtsiAuxDependent
        && request.dependent_count > 1
    {
        rate_budget_for_dependents(request.bitrate_kbps, request.dependent_count)
    } else {
        rate_budget_for_carriage(request.carriage, request.bitrate_kbps)
    }
    .map_err(|error| {
        classified(
            CapabilityVerdict::RejectedCapacity,
            format!("composition exceeds the selected bitrate capacity: {error}"),
        )
    })?;
    let ownership = if request.dependent_count == 0 {
        None
    } else {
        Some(canonical_ownership(
            request.profile,
            request.dependent_count,
            request.lfe_ownership,
        )?)
    };
    Ok(JocEncodingPlan {
        profile: request.profile,
        spec: profile_spec(request.profile),
        container: request.container,
        carriage: request.carriage,
        emdf_profile: request.emdf_profile,
        rate,
        block_partition,
        dependent_count: request.dependent_count,
        lfe_ownership: request.lfe_ownership,
        ownership,
    })
}

/// Resolves the requested composition through the central semantic model.
pub fn resolve_composition(
    request: &CompositionRequest,
) -> Result<JocEncodingPlan, CompositionError> {
    resolve_composition_inner(request)
}

/// Classifies a request even when it is rejected, preserving the exact reason.
#[must_use]
pub fn classify_composition(request: &CompositionRequest) -> CapabilityClassification {
    match resolve_composition_inner(request) {
        Ok(_) => CapabilityClassification {
            verdict: CapabilityVerdict::SupportedPass,
            reason: "public composition, ownership, and bitrate plan resolved".to_owned(),
        },
        Err(error) => error.classification,
    }
}

/// Enumerates a bounded capability lattice using only the pure resolver. It
/// intentionally classifies cells without invoking FFmpeg, OpenJOC, or PCM
/// generation; heavyweight validation is reserved for representatives.
#[must_use]
pub fn composition_capability_matrix() -> Vec<CompositionCapabilityRow> {
    let profiles = [
        JocProfile::Idx0,
        JocProfile::Idx1,
        JocProfile::Idx2,
        JocProfile::Idx3,
        JocProfile::Idx4,
    ];
    let containers = [JocContainer::Raw, JocContainer::Cmaf];
    let carriages = [
        JocCarriage::EtsiAuxDependent,
        JocCarriage::I0SkipField,
        JocCarriage::LegacyAc3Dependent,
    ];
    let emdf_profiles = [EmdfProfile::EtsiStrict, EmdfProfile::DeployedCompat];
    let partitions = [
        vec![BlockPartition::Six],
        vec![BlockPartition::Three, BlockPartition::Three],
        vec![BlockPartition::Two; 3],
        vec![BlockPartition::One; 6],
        vec![
            BlockPartition::One,
            BlockPartition::Two,
            BlockPartition::Three,
        ],
    ];
    let lfe_ownership = [
        LfeOwnership::Absent,
        LfeOwnership::Independent,
        LfeOwnership::DependentSupplement,
        LfeOwnership::DependentReplacement,
    ];
    let bitrates = [
        1024, 1088, 1152, 1216, 1280, 1408, 1536, 1664, 2304, 2944, 3584, 4224, 4864,
    ];
    let mut rows = Vec::new();
    for profile in profiles {
        for container in containers {
            for carriage in carriages {
                for emdf_profile in emdf_profiles {
                    for partition in &partitions {
                        for dependent_count in 0..=8 {
                            for lfe in lfe_ownership {
                                for bitrate_kbps in bitrates {
                                    let request = CompositionRequest {
                                        profile,
                                        container,
                                        carriage,
                                        emdf_profile,
                                        bitrate_kbps,
                                        partitions: partition.clone(),
                                        dependent_count,
                                        lfe_ownership: lfe,
                                    };
                                    rows.push(CompositionCapabilityRow {
                                        profile,
                                        container,
                                        carriage,
                                        emdf_profile,
                                        partitions: partition.clone(),
                                        dependent_count,
                                        lfe_ownership: lfe,
                                        bitrate_kbps,
                                        classification: classify_composition(&request),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    rows
}

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
/// partitions are admitted for raw ETSI auxiliary-dependent carriage when the
/// selected strict/deployed signaling and ownership path is closed; skip-field
/// and legacy-core short combinations remain fail-closed.
pub fn resolve_encoding_plan_with_partition(
    profile: JocProfile,
    carriage: JocCarriage,
    emdf_profile: EmdfProfile,
    bitrate_kbps: u32,
    partitions: &[BlockPartition],
) -> Result<JocEncodingPlan, JocPlanError> {
    let request = CompositionRequest {
        profile,
        container: JocContainer::Raw,
        carriage,
        emdf_profile,
        bitrate_kbps,
        partitions: partitions.to_vec(),
        dependent_count: usize::from(carriage != JocCarriage::I0SkipField),
        lfe_ownership: crate::LfeOwnership::Absent,
    };
    resolve_composition(&request).map_err(|error| match error.classification.verdict {
        CapabilityVerdict::RejectedCapacity => JocPlanError::UnsupportedBitrate {
            profile,
            carriage,
            bitrate_kbps,
        },
        CapabilityVerdict::SpecLegalNotImplemented if carriage == JocCarriage::I0SkipField => {
            if partitions == [BlockPartition::Six] {
                JocPlanError::UnsupportedCarriage { profile, carriage }
            } else {
                JocPlanError::UnsupportedBlockPartition {
                    profile,
                    carriage,
                    status: classify_block_partition(partitions),
                }
            }
        }
        _ => {
            let status = classify_block_partition(partitions);
            if status == BlockPartitionStatus::Invalid {
                let actual_blocks = partitions
                    .iter()
                    .map(|partition| partition.audio_blocks())
                    .sum();
                JocPlanError::InvalidBlockPartition { actual_blocks }
            } else {
                JocPlanError::UnsupportedBlockPartition {
                    profile,
                    carriage,
                    status,
                }
            }
        }
    })
}
