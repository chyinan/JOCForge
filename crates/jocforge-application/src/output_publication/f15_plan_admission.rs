// pattern: Functional Core
//! Private F15 plan and destination admission boundary.
//!
//! This module retains actual typed provider values in owner-role handles and
//! checks their shared parent/input/profile binding before any output work. The
//! product path remains unwired until each owner supplies its runtime validator
//! handle and the parent supplies a destination capability from its trusted
//! pre-I/O boundary.

use super::{OutputRootBindingError, OutputStagingOwnerId, SelectedOutputProviderProjection};
use crate::PreparedJobInput;
use jocforge_domain::{
    meter::prepared_job_identity::PreparedJobInputIdentity,
    profile::{
        EmdfCarriageProfile, EncodingProfile, ProfileError, ResolvedEncodingProfile,
        resolve_selected_profile,
    },
    timeline::{AdmSampleCount, ClockError, ProgrammeCounts, programme_counts},
    transaction::{AttemptId, ProgrammeId},
};
use std::marker::PhantomData;
use std::sync::Arc;

const SELECTED_F15_DEPENDENCY_MANIFEST_SHA256: &str =
    "a2cf06839d3da81d14240ccd6526eebc0b878721d48a03fd0d0813f26961f6ce";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct F15DependencyManifestIdentity(&'static str);

impl F15DependencyManifestIdentity {
    const fn selected() -> Self {
        Self(SELECTED_F15_DEPENDENCY_MANIFEST_SHA256)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum F15ProviderRole {
    CodecF12,
    JocF05Jl,
    F13Metadata,
    F14Integrity,
}

struct CodecF12Role;
struct JocF05JlRole;
struct F13MetadataRole;
struct F14IntegrityRole;

/// Private marker implemented only for a value type returned by the matching
/// owner validator. Production implementations are intentionally absent until
/// each role loader exposes its selected, validated value type.
trait F15RoleValidatedValue<Role> {}

#[derive(Clone)]
struct F15PlanParentContext {
    programme_id: ProgrammeId,
    prepared_input_identity: Arc<PreparedJobInputIdentity>,
    profile_dimensions: EncodingProfile,
    resolved_profile: ResolvedEncodingProfile,
    dependency_manifest_identity: F15DependencyManifestIdentity,
    source_samples: u64,
}

impl F15PlanParentContext {
    fn from_prepared_input(input: &PreparedJobInput) -> Result<Self, F15PlanAdmissionError> {
        let resolved_profile = input.configuration().selected_profile();
        let profile_dimensions =
            EncodingProfile::selected_with_carriage(resolved_profile.carriage());
        let re_resolved = resolve_selected_profile(&profile_dimensions)
            .map_err(F15PlanAdmissionError::Profile)?;
        if re_resolved != resolved_profile {
            return Err(F15PlanAdmissionError::ResolvedProfileMismatch);
        }
        Ok(Self {
            programme_id: input.programme_id().clone(),
            prepared_input_identity: input.domain_identity_capability(),
            profile_dimensions,
            resolved_profile,
            dependency_manifest_identity: F15DependencyManifestIdentity::selected(),
            source_samples: input.configuration().source_samples(),
        })
    }

    #[cfg(test)]
    fn for_test(
        programme_id: ProgrammeId,
        prepared_input_identity: Arc<PreparedJobInputIdentity>,
        profile_dimensions: EncodingProfile,
        source_samples: u64,
    ) -> Result<Self, F15PlanAdmissionError> {
        let resolved_profile = resolve_selected_profile(&profile_dimensions)
            .map_err(F15PlanAdmissionError::Profile)?;
        Ok(Self {
            programme_id,
            prepared_input_identity,
            profile_dimensions,
            resolved_profile,
            dependency_manifest_identity: F15DependencyManifestIdentity::selected(),
            source_samples,
        })
    }

    fn admission_binding(&self) -> OwnerAdmissionBinding {
        OwnerAdmissionBinding {
            programme_id: self.programme_id.clone(),
            prepared_input_identity: Arc::clone(&self.prepared_input_identity),
            profile_dimensions: self.profile_dimensions,
            resolved_profile: self.resolved_profile,
            carriage: self.resolved_profile.carriage(),
            dependency_manifest_identity: self.dependency_manifest_identity,
        }
    }
}

#[derive(Clone)]
struct OwnerAdmissionBinding {
    programme_id: ProgrammeId,
    prepared_input_identity: Arc<PreparedJobInputIdentity>,
    profile_dimensions: EncodingProfile,
    resolved_profile: ResolvedEncodingProfile,
    carriage: EmdfCarriageProfile,
    dependency_manifest_identity: F15DependencyManifestIdentity,
}

/// A role-specific handle retains the actual immutable value returned by its
/// validator. It is not a manifest/hash-only or PASS-boolean receipt.
struct OwnerIssuedProviderAdmission<Role, Value: F15RoleValidatedValue<Role>> {
    value: Arc<Value>,
    binding: OwnerAdmissionBinding,
    role: PhantomData<fn() -> Role>,
}

impl<Role, Value: F15RoleValidatedValue<Role>> OwnerIssuedProviderAdmission<Role, Value> {
    fn from_owner_validated_value(value: Arc<Value>, binding: OwnerAdmissionBinding) -> Self {
        Self {
            value,
            binding,
            role: PhantomData,
        }
    }
}

struct F15ProviderAdmissionCandidates<
    CodecValue: F15RoleValidatedValue<CodecF12Role>,
    JocValue: F15RoleValidatedValue<JocF05JlRole>,
    MetadataValue: F15RoleValidatedValue<F13MetadataRole>,
    IntegrityValue: F15RoleValidatedValue<F14IntegrityRole>,
> {
    codec_f12: Option<OwnerIssuedProviderAdmission<CodecF12Role, CodecValue>>,
    joc_f05jl: Option<OwnerIssuedProviderAdmission<JocF05JlRole, JocValue>>,
    f13_metadata: Option<OwnerIssuedProviderAdmission<F13MetadataRole, MetadataValue>>,
    f14_integrity: Option<OwnerIssuedProviderAdmission<F14IntegrityRole, IntegrityValue>>,
}

struct ValidatedF15ProviderAdmissions<
    CodecValue: F15RoleValidatedValue<CodecF12Role>,
    JocValue: F15RoleValidatedValue<JocF05JlRole>,
    MetadataValue: F15RoleValidatedValue<F13MetadataRole>,
    IntegrityValue: F15RoleValidatedValue<F14IntegrityRole>,
> {
    codec_f12: OwnerIssuedProviderAdmission<CodecF12Role, CodecValue>,
    joc_f05jl: OwnerIssuedProviderAdmission<JocF05JlRole, JocValue>,
    f13_metadata: OwnerIssuedProviderAdmission<F13MetadataRole, MetadataValue>,
    f14_integrity: OwnerIssuedProviderAdmission<F14IntegrityRole, IntegrityValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NormalizedFinalDestinationIdentity(String);

impl NormalizedFinalDestinationIdentity {
    /// Wraps the exact normalized identity returned by the trusted parent/IO
    /// admission boundary; this constructor performs no filesystem access.
    fn from_parent_validated(value: &str) -> Result<Self, F15PlanAdmissionError> {
        if value.is_empty() || value.contains('\0') {
            return Err(F15PlanAdmissionError::InvalidFinalDestinationIdentity);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParentReservedStagingNamespace(String);

impl ParentReservedStagingNamespace {
    fn from_parent_reservation(value: &str) -> Result<Self, F15PlanAdmissionError> {
        if value.is_empty() || value.contains('\0') {
            return Err(F15PlanAdmissionError::InvalidStagingNamespace);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FinalDestinationPolicy {
    NoOverwrite,
}

/// Opaque parent-issued pre-I/O capability. Durable write/exposure receipts
/// remain owned by the IO shell and are not pre-minted here.
struct ParentValidatedDestinationCapability {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    staging_owner_id: OutputStagingOwnerId,
    final_destination: NormalizedFinalDestinationIdentity,
    staging_namespace: ParentReservedStagingNamespace,
    policy: FinalDestinationPolicy,
}

impl ParentValidatedDestinationCapability {
    fn from_parent_validation(
        programme_id: ProgrammeId,
        attempt_id: AttemptId,
        staging_owner_id: OutputStagingOwnerId,
        final_destination: NormalizedFinalDestinationIdentity,
        staging_namespace: ParentReservedStagingNamespace,
    ) -> Result<Self, F15PlanAdmissionError> {
        if final_destination.0 == staging_namespace.0 {
            return Err(F15PlanAdmissionError::DestinationAndStagingAlias);
        }
        Ok(Self {
            programme_id,
            attempt_id,
            staging_owner_id,
            final_destination,
            staging_namespace,
            policy: FinalDestinationPolicy::NoOverwrite,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct F15DynamicExtents {
    programme_counts: ProgrammeCounts,
    published_output_bytes: u64,
}

/// Private, parent-owned F15 aggregate. It carries four actual typed values and
/// a pre-I/O destination capability, but does not construct OutputOwner or write files.
#[allow(dead_code)]
struct ValidatedEncodingPlan<
    CodecValue: F15RoleValidatedValue<CodecF12Role>,
    JocValue: F15RoleValidatedValue<JocF05JlRole>,
    MetadataValue: F15RoleValidatedValue<F13MetadataRole>,
    IntegrityValue: F15RoleValidatedValue<F14IntegrityRole>,
> {
    programme_id: ProgrammeId,
    prepared_input_identity: Arc<PreparedJobInputIdentity>,
    profile_dimensions: EncodingProfile,
    resolved_profile: ResolvedEncodingProfile,
    dependency_manifest_identity: F15DependencyManifestIdentity,
    providers: ValidatedF15ProviderAdmissions<CodecValue, JocValue, MetadataValue, IntegrityValue>,
    destination: ParentValidatedDestinationCapability,
    extents: F15DynamicExtents,
    output_provider_projection: SelectedOutputProviderProjection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum F15PlanAdmissionError {
    Profile(ProfileError),
    ResolvedProfileMismatch,
    MissingOwnerAdmission(F15ProviderRole),
    ProgrammeIdentityMismatch(F15ProviderRole),
    PreparedInputIdentityMismatch(F15ProviderRole),
    ProfileDimensionMismatch(F15ProviderRole),
    ResolvedProfileMismatchForRole(F15ProviderRole),
    CarriageMismatch(F15ProviderRole),
    DependencyManifestMismatch(F15ProviderRole),
    ApprovedDependencyManifestMismatch,
    DestinationCapabilityMissing,
    DestinationProgrammeMismatch,
    DestinationAttemptMismatch,
    DestinationStagingOwnerMismatch,
    InvalidFinalDestinationIdentity,
    InvalidStagingNamespace,
    DestinationAndStagingAlias,
    ZeroSampleCount,
    ArithmeticOverflow,
    OutputProviderProjection(OutputRootBindingError),
}

fn validate_owner_binding<Role, Value: F15RoleValidatedValue<Role>>(
    role: F15ProviderRole,
    handle: &OwnerIssuedProviderAdmission<Role, Value>,
    context: &F15PlanParentContext,
) -> Result<(), F15PlanAdmissionError> {
    if handle.binding.programme_id != context.programme_id {
        return Err(F15PlanAdmissionError::ProgrammeIdentityMismatch(role));
    }
    if !Arc::ptr_eq(
        &handle.binding.prepared_input_identity,
        &context.prepared_input_identity,
    ) {
        return Err(F15PlanAdmissionError::PreparedInputIdentityMismatch(role));
    }
    if handle.binding.profile_dimensions != context.profile_dimensions {
        return Err(F15PlanAdmissionError::ProfileDimensionMismatch(role));
    }
    if handle.binding.resolved_profile != context.resolved_profile {
        return Err(F15PlanAdmissionError::ResolvedProfileMismatchForRole(role));
    }
    if handle.binding.carriage != context.resolved_profile.carriage() {
        return Err(F15PlanAdmissionError::CarriageMismatch(role));
    }
    if handle.binding.dependency_manifest_identity != context.dependency_manifest_identity {
        return Err(F15PlanAdmissionError::DependencyManifestMismatch(role));
    }
    Ok(())
}

/// Parent-only admission gate. No runtime receipt is constructed here; every
/// present role must already carry the owning validator's typed loaded value.
#[allow(dead_code)]
fn admit_validated_encoding_plan<
    CodecValue: F15RoleValidatedValue<CodecF12Role>,
    JocValue: F15RoleValidatedValue<JocF05JlRole>,
    MetadataValue: F15RoleValidatedValue<F13MetadataRole>,
    IntegrityValue: F15RoleValidatedValue<F14IntegrityRole>,
>(
    context: F15PlanParentContext,
    current_attempt: AttemptId,
    current_staging_owner: &OutputStagingOwnerId,
    admissions: F15ProviderAdmissionCandidates<CodecValue, JocValue, MetadataValue, IntegrityValue>,
    destination: Option<ParentValidatedDestinationCapability>,
) -> Result<
    ValidatedEncodingPlan<CodecValue, JocValue, MetadataValue, IntegrityValue>,
    F15PlanAdmissionError,
> {
    if context.dependency_manifest_identity != F15DependencyManifestIdentity::selected() {
        return Err(F15PlanAdmissionError::ApprovedDependencyManifestMismatch);
    }

    let codec_f12 = admissions
        .codec_f12
        .ok_or(F15PlanAdmissionError::MissingOwnerAdmission(
            F15ProviderRole::CodecF12,
        ))?;
    let joc_f05jl = admissions
        .joc_f05jl
        .ok_or(F15PlanAdmissionError::MissingOwnerAdmission(
            F15ProviderRole::JocF05Jl,
        ))?;
    let f13_metadata =
        admissions
            .f13_metadata
            .ok_or(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::F13Metadata,
            ))?;
    let f14_integrity =
        admissions
            .f14_integrity
            .ok_or(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::F14Integrity,
            ))?;

    validate_owner_binding(F15ProviderRole::CodecF12, &codec_f12, &context)?;
    validate_owner_binding(F15ProviderRole::JocF05Jl, &joc_f05jl, &context)?;
    validate_owner_binding(F15ProviderRole::F13Metadata, &f13_metadata, &context)?;
    validate_owner_binding(F15ProviderRole::F14Integrity, &f14_integrity, &context)?;

    let destination = destination.ok_or(F15PlanAdmissionError::DestinationCapabilityMissing)?;
    if destination.programme_id != context.programme_id {
        return Err(F15PlanAdmissionError::DestinationProgrammeMismatch);
    }
    if destination.attempt_id != current_attempt {
        return Err(F15PlanAdmissionError::DestinationAttemptMismatch);
    }
    if destination.staging_owner_id != *current_staging_owner {
        return Err(F15PlanAdmissionError::DestinationStagingOwnerMismatch);
    }
    if destination.final_destination.0.is_empty()
        || destination.staging_namespace.0.is_empty()
        || destination.final_destination.0 == destination.staging_namespace.0
        || destination.policy != FinalDestinationPolicy::NoOverwrite
    {
        return Err(F15PlanAdmissionError::InvalidFinalDestinationIdentity);
    }

    let sample_count =
        AdmSampleCount::new(context.source_samples).map_err(|error| match error {
            ClockError::ZeroSampleCount => F15PlanAdmissionError::ZeroSampleCount,
            ClockError::ArithmeticOverflow => F15PlanAdmissionError::ArithmeticOverflow,
        })?;
    let counts =
        programme_counts(sample_count).map_err(|_| F15PlanAdmissionError::ArithmeticOverflow)?;
    let projection = SelectedOutputProviderProjection::project_selected_f15_output_provider(
        context.resolved_profile,
    )
    .map_err(F15PlanAdmissionError::OutputProviderProjection)?;
    let published_output_bytes = counts
        .as_tuple()
        .4
        .checked_mul(u64::from(projection.output_provider().frame_bytes))
        .ok_or(F15PlanAdmissionError::ArithmeticOverflow)?;

    Ok(ValidatedEncodingPlan {
        programme_id: context.programme_id,
        prepared_input_identity: context.prepared_input_identity,
        profile_dimensions: context.profile_dimensions,
        resolved_profile: context.resolved_profile,
        dependency_manifest_identity: context.dependency_manifest_identity,
        providers: ValidatedF15ProviderAdmissions {
            codec_f12,
            joc_f05jl,
            f13_metadata,
            f14_integrity,
        },
        destination,
        extents: F15DynamicExtents {
            programme_counts: counts,
            published_output_bytes,
        },
        output_provider_projection: projection,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jocforge_domain::{
        frontend::job_configuration::SelectedInputKind,
        meter::prepared_job_identity::{
            NormalizedJobConfigurationIdentity, PreparedAdmChunkSpan, PreparedAdmGraphProvenance,
            PreparedInputResourceIdentity,
        },
        profile::{EmdfCarriageProfile, EncodingProfile},
    };

    struct TestProviderValue(u8);

    impl F15RoleValidatedValue<CodecF12Role> for TestProviderValue {}
    impl F15RoleValidatedValue<JocF05JlRole> for TestProviderValue {}
    impl F15RoleValidatedValue<F13MetadataRole> for TestProviderValue {}
    impl F15RoleValidatedValue<F14IntegrityRole> for TestProviderValue {}

    fn identity(resource_byte: u8) -> Arc<PreparedJobInputIdentity> {
        let resource = PreparedInputResourceIdentity::new([resource_byte; 32], 1_000);
        let graph = PreparedAdmGraphProvenance::new(
            resource,
            PreparedAdmChunkSpan::new(*b"axml", 100, 50, [0x41; 32]),
            PreparedAdmChunkSpan::new(*b"chna", 200, 30, [0x42; 32]),
            PreparedAdmChunkSpan::new(*b"dbmd", 300, 40, [0x43; 32]),
        );
        Arc::new(
            PreparedJobInputIdentity::new(
                resource,
                NormalizedJobConfigurationIdentity::new(
                    [0x22; 32],
                    [0x33; 32],
                    SelectedInputKind::Adm,
                ),
                graph,
            )
            .expect("test identity has valid resource-bound graph spans"),
        )
    }

    fn parent_context(source_samples: u64, resource_byte: u8) -> F15PlanParentContext {
        let dimensions = EncodingProfile::selected();
        F15PlanParentContext::for_test(
            ProgrammeId::new("plan-test-programme").expect("nonempty test programme"),
            identity(resource_byte),
            dimensions,
            source_samples,
        )
        .expect("selected test profile resolves")
    }

    fn role_handle<Role>(
        context: &F15PlanParentContext,
        value: u8,
    ) -> OwnerIssuedProviderAdmission<Role, TestProviderValue>
    where
        TestProviderValue: F15RoleValidatedValue<Role>,
    {
        OwnerIssuedProviderAdmission::from_owner_validated_value(
            Arc::new(TestProviderValue(value)),
            context.admission_binding(),
        )
    }

    fn complete_admissions(
        context: &F15PlanParentContext,
    ) -> F15ProviderAdmissionCandidates<
        TestProviderValue,
        TestProviderValue,
        TestProviderValue,
        TestProviderValue,
    > {
        F15ProviderAdmissionCandidates {
            codec_f12: Some(role_handle(context, 1)),
            joc_f05jl: Some(role_handle(context, 2)),
            f13_metadata: Some(role_handle(context, 3)),
            f14_integrity: Some(role_handle(context, 4)),
        }
    }

    fn destination(
        context: &F15PlanParentContext,
        attempt: AttemptId,
        staging_owner_id: OutputStagingOwnerId,
        final_path: &str,
        staging_namespace: &str,
    ) -> ParentValidatedDestinationCapability {
        ParentValidatedDestinationCapability::from_parent_validation(
            context.programme_id.clone(),
            attempt,
            staging_owner_id,
            NormalizedFinalDestinationIdentity::from_parent_validated(final_path)
                .expect("test identity is nonempty"),
            ParentReservedStagingNamespace::from_parent_reservation(staging_namespace)
                .expect("test staging namespace is nonempty"),
        )
        .expect("final and staging identities are distinct")
    }

    fn admit_for_test(
        context: F15PlanParentContext,
        attempt: AttemptId,
        staging_owner_id: &OutputStagingOwnerId,
        admissions: F15ProviderAdmissionCandidates<
            TestProviderValue,
            TestProviderValue,
            TestProviderValue,
            TestProviderValue,
        >,
        destination: Option<ParentValidatedDestinationCapability>,
    ) -> Result<
        ValidatedEncodingPlan<
            TestProviderValue,
            TestProviderValue,
            TestProviderValue,
            TestProviderValue,
        >,
        F15PlanAdmissionError,
    > {
        admit_validated_encoding_plan(context, attempt, staging_owner_id, admissions, destination)
    }

    #[test]
    fn plan_requires_all_four_typed_owner_values_before_destination_admission() {
        let context = parent_context(6_522_906, 0x11);
        let attempt = AttemptId::new(4);
        let staging_owner = OutputStagingOwnerId::new("stage-4").unwrap();
        let mut admissions = complete_admissions(&context);
        admissions.codec_f12 = None;
        assert!(matches!(
            admit_for_test(context.clone(), attempt, &staging_owner, admissions, None),
            Err(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::CodecF12
            ))
        ));

        let mut admissions = complete_admissions(&context);
        admissions.joc_f05jl = None;
        assert!(matches!(
            admit_for_test(context.clone(), attempt, &staging_owner, admissions, None),
            Err(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::JocF05Jl
            ))
        ));

        let mut admissions = complete_admissions(&context);
        admissions.f13_metadata = None;
        assert!(matches!(
            admit_for_test(context.clone(), attempt, &staging_owner, admissions, None),
            Err(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::F13Metadata
            ))
        ));

        let mut admissions = complete_admissions(&context);
        admissions.f14_integrity = None;
        assert!(matches!(
            admit_for_test(context, attempt, &staging_owner, admissions, None),
            Err(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::F14Integrity
            ))
        ));
    }

    #[test]
    fn plan_rejects_role_input_attempt_staging_and_destination_mismatches() {
        let context = parent_context(6_522_906, 0x11);
        let attempt = AttemptId::new(4);
        let staging_owner = OutputStagingOwnerId::new("stage-4").unwrap();
        let cap = destination(
            &context,
            attempt,
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-4",
        );

        let foreign_context = parent_context(6_522_906, 0x12);
        let mut admissions = complete_admissions(&context);
        admissions.codec_f12 = Some(role_handle(&foreign_context, 1));
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                admissions,
                Some(cap)
            ),
            Err(F15PlanAdmissionError::PreparedInputIdentityMismatch(
                F15ProviderRole::CodecF12
            ))
        ));

        let cap = destination(
            &context,
            AttemptId::new(3),
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-3",
        );
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                complete_admissions(&context),
                Some(cap)
            ),
            Err(F15PlanAdmissionError::DestinationAttemptMismatch)
        ));

        let wrong_owner = OutputStagingOwnerId::new("stage-other").unwrap();
        let cap = destination(
            &context,
            attempt,
            wrong_owner,
            "normalized://out/product.ec3",
            "staging://owner-other/attempt-4",
        );
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                complete_admissions(&context),
                Some(cap)
            ),
            Err(F15PlanAdmissionError::DestinationStagingOwnerMismatch)
        ));
    }

    #[test]
    fn plan_rejects_profile_carriage_and_dependency_mismatches() {
        let context = parent_context(6_522_906, 0x11);
        let attempt = AttemptId::new(4);
        let staging_owner = OutputStagingOwnerId::new("stage-4").unwrap();
        let cap = destination(
            &context,
            attempt,
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-4",
        );

        let alternate_dimensions =
            EncodingProfile::selected_with_carriage(EmdfCarriageProfile::EtsiEmdf);
        let alternate_context = F15PlanParentContext::for_test(
            context.programme_id.clone(),
            Arc::clone(&context.prepared_input_identity),
            alternate_dimensions,
            context.source_samples,
        )
        .expect("selected ETSI profile resolves");
        let mut admissions = complete_admissions(&context);
        admissions.codec_f12 = Some(role_handle(&alternate_context, 1));
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                admissions,
                Some(cap)
            ),
            Err(F15PlanAdmissionError::ProfileDimensionMismatch(
                F15ProviderRole::CodecF12
            ))
        ));

        let cap = destination(
            &context,
            attempt,
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-4",
        );
        let mut admissions = complete_admissions(&context);
        let codec = admissions.codec_f12.as_mut().unwrap();
        codec.binding.carriage = EmdfCarriageProfile::EtsiEmdf;
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                admissions,
                Some(cap)
            ),
            Err(F15PlanAdmissionError::CarriageMismatch(
                F15ProviderRole::CodecF12
            ))
        ));

        let cap = destination(
            &context,
            attempt,
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-4",
        );
        let mut admissions = complete_admissions(&context);
        let metadata = admissions.f13_metadata.as_mut().unwrap();
        metadata.binding.dependency_manifest_identity =
            F15DependencyManifestIdentity("unapproved-manifest");
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                admissions,
                Some(cap)
            ),
            Err(F15PlanAdmissionError::DependencyManifestMismatch(
                F15ProviderRole::F13Metadata
            ))
        ));
    }

    #[test]
    fn plan_rejects_missing_destination_and_checked_count_overflow() {
        let context = parent_context(6_522_906, 0x11);
        let attempt = AttemptId::new(4);
        let staging_owner = OutputStagingOwnerId::new("stage-4").unwrap();
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                complete_admissions(&context),
                None
            ),
            Err(F15PlanAdmissionError::DestinationCapabilityMissing)
        ));

        let overflow_context = parent_context(u64::MAX, 0x11);
        let overflow_owner = OutputStagingOwnerId::new("stage-overflow").unwrap();
        let overflow_destination = destination(
            &overflow_context,
            attempt,
            overflow_owner.clone(),
            "normalized://out/overflow.ec3",
            "staging://owner-overflow/attempt-4",
        );
        assert!(matches!(
            admit_for_test(
                overflow_context.clone(),
                attempt,
                &overflow_owner,
                complete_admissions(&overflow_context),
                Some(overflow_destination)
            ),
            Err(F15PlanAdmissionError::ArithmeticOverflow)
        ));

        let alias = ParentValidatedDestinationCapability::from_parent_validation(
            overflow_context.programme_id.clone(),
            attempt,
            overflow_owner,
            NormalizedFinalDestinationIdentity::from_parent_validated("same-identity").unwrap(),
            ParentReservedStagingNamespace::from_parent_reservation("same-identity").unwrap(),
        );
        assert!(matches!(
            alias,
            Err(F15PlanAdmissionError::DestinationAndStagingAlias)
        ));
    }

    #[test]
    fn private_plan_derives_checked_counts_and_output_projection_without_io() {
        let context = parent_context(6_522_906, 0x11);
        let attempt = AttemptId::new(4);
        let staging_owner = OutputStagingOwnerId::new("stage-4").unwrap();
        let cap = destination(
            &context,
            attempt,
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-4",
        );
        let admissions = complete_admissions(&context);
        let plan = admit_for_test(
            context.clone(),
            attempt,
            &staging_owner,
            admissions,
            Some(cap),
        )
        .expect("all typed test values and pre-I/O capability bind to one parent input");
        assert_eq!(
            plan.extents.programme_counts.as_tuple(),
            (4_247, 486, 4_251, 4_249, 4_247, 1_050)
        );
        assert_eq!(plan.extents.published_output_bytes, 17_395_712);
        assert_eq!(
            plan.output_provider_projection
                .output_provider()
                .frame_bytes,
            4_096
        );
        assert_eq!(
            plan.output_provider_projection
                .output_provider()
                .suppressed_sources,
            2
        );
        assert_eq!(plan.profile_dimensions, EncodingProfile::selected());
        assert_eq!(
            plan.resolved_profile.carriage(),
            EmdfCarriageProfile::DeployedCompat
        );
        assert_eq!(
            plan.dependency_manifest_identity,
            F15DependencyManifestIdentity::selected()
        );
        assert_eq!(plan.destination.policy, FinalDestinationPolicy::NoOverwrite);
        assert_eq!(plan.providers.codec_f12.value.0, 1);
        assert_eq!(plan.providers.joc_f05jl.value.0, 2);
        assert_eq!(plan.providers.f13_metadata.value.0, 3);
        assert_eq!(plan.providers.f14_integrity.value.0, 4);
        assert_eq!(plan.programme_id.as_str(), "plan-test-programme");
        assert!(Arc::ptr_eq(
            &plan.prepared_input_identity,
            &plan.providers.codec_f12.binding.prepared_input_identity
        ));
    }
}
