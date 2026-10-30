// pattern: Imperative Shell
//! Joins the current metadata and complete-frame proposals into one output-owner proposal.

#[cfg(any(test, feature = "test-support"))]
use jocforge_domain::profile::EmdfCarriageProfile;
use jocforge_domain::profile::ResolvedEncodingProfile;
use jocforge_domain::publication::OutputProvider;
use jocforge_domain::{
    codec::{
        owner::AcceptedCodecFrame,
        published_emdf_binding::emdf_frame_input_for_source,
        selected_main_q_search::{SelectedMainQOutcome, SelectedMainQSearchResult},
        source_frame_writer::SelectedSourceFrameCandidate,
    },
    joc::emdf::metadata_owner::{MetadataEofProposal, MetadataSourceProposal},
    publication::OutputSourceIdentity,
    timeline::{
        ClockError, CodecSourceIndex, PublicationDisposition, PublishedAuIndex,
        publication_for_source,
    },
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub use jocforge_domain::publication::OutputConfiguration;
pub use jocforge_domain::transaction::{AttemptId, ProgrammeId};

pub use jocforge_domain::publication::{
    OutputFinalizeProposal, OutputFinalizedProposal, OutputOwner, OutputOwnerError,
    OutputSourceAfterProposal, OutputSourceProposal,
};

/// Exact output-policy provider projection for one resolved selected carriage.
///
/// This value is not a validated encoding plan or publication authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedOutputProviderProjection {
    profile: ResolvedEncodingProfile,
    provider: OutputProvider,
}

impl SelectedOutputProviderProjection {
    #[cfg(any(test, feature = "test-support"))]
    fn project_selected_f15_output_provider(
        profile: ResolvedEncodingProfile,
    ) -> Result<Self, OutputRootBindingError> {
        let canonical_utf8 = match profile.carriage() {
            EmdfCarriageProfile::DeployedCompat => DEPLOYED_COMPAT_PROVIDER_CANONICAL_UTF8,
            EmdfCarriageProfile::EtsiEmdf => ETSI_EMDF_PROVIDER_CANONICAL_UTF8,
            EmdfCarriageProfile::Unsupported => {
                return Err(OutputRootBindingError::UnsupportedProfile);
            }
        };
        let provider_sha256 = Sha256::digest(canonical_utf8.as_bytes()).into();
        Ok(Self {
            profile,
            provider: OutputProvider {
                profile_id: "selected-keyless-output-policy.v1".to_owned(),
                provider_sha256,
                frame_bytes: 4_096,
                suppressed_sources: 2,
                staging_name_template: "attempt-{attempt_id}.ec3.partial".to_owned(),
                intent_name_template: "attempt-{attempt_id}.finalize-intent.json".to_owned(),
            },
        })
    }

    pub const fn profile(&self) -> ResolvedEncodingProfile {
        self.profile
    }

    pub const fn output_provider(&self) -> &OutputProvider {
        &self.provider
    }
}

#[cfg(any(test, feature = "test-support"))]
const DEPLOYED_COMPAT_PROVIDER_CANONICAL_UTF8: &str = "{\"emdf_carriage_mode\":\"deployed_compat\",\"encoding_profile_id\":\"eac3-joc-48k-1024k-acmod7-uncoupled-nonspx\",\"f15_dependency_manifest_sha256\":\"a2cf06839d3da81d14240ccd6526eebc0b878721d48a03fd0d0813f26961f6ce\",\"frame_bytes\":4096,\"intent_name_template\":\"attempt-{attempt_id}.finalize-intent.json\",\"profile_id\":\"selected-keyless-output-policy.v1\",\"schema\":\"output-owner-provider.v1\",\"staging_name_template\":\"attempt-{attempt_id}.ec3.partial\",\"suppressed_sources\":2}";

#[cfg(any(test, feature = "test-support"))]
const ETSI_EMDF_PROVIDER_CANONICAL_UTF8: &str = "{\"emdf_carriage_mode\":\"etsi_emdf\",\"encoding_profile_id\":\"eac3-joc-48k-1024k-acmod7-uncoupled-nonspx\",\"f15_dependency_manifest_sha256\":\"a2cf06839d3da81d14240ccd6526eebc0b878721d48a03fd0d0813f26961f6ce\",\"frame_bytes\":4096,\"intent_name_template\":\"attempt-{attempt_id}.finalize-intent.json\",\"profile_id\":\"selected-keyless-output-policy.v1\",\"schema\":\"output-owner-provider.v1\",\"staging_name_template\":\"attempt-{attempt_id}.ec3.partial\",\"suppressed_sources\":2}";

use super::PreparedJobInput;

#[cfg(test)]
mod tests;

/// Constructors exposed only to unit/integration tests for isolated IO checks.
///
/// This module is absent from default product builds. It is not an encoding-plan
/// validator and cannot be used as production authority.
#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    use super::{
        OutputRootBindingError, OutputStagingOwnerId, ProgrammeOutputBinding,
        SelectedOutputProviderProjection,
    };
    use crate::PreparedJobInput;
    use jocforge_domain::transaction::AttemptId;

    /// Builds a synthetic binding for isolated staging tests only.
    pub fn bind_selected_output_for_test(
        input: &PreparedJobInput,
        attempt_id: AttemptId,
        staging_owner_id: OutputStagingOwnerId,
    ) -> Result<ProgrammeOutputBinding, OutputRootBindingError> {
        let provider = SelectedOutputProviderProjection::project_selected_f15_output_provider(
            input.configuration().selected_profile(),
        )?
        .output_provider()
        .clone();
        ProgrammeOutputBinding::for_test_from_prepared_input(
            input,
            attempt_id,
            staging_owner_id,
            provider,
        )
    }
}

/// Parent-owned opaque identity for one programme's output staging namespace.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct OutputStagingOwnerId(String);

impl OutputStagingOwnerId {
    /// Wraps a unique opaque identity supplied by the programme parent.
    ///
    /// The value is not derived from the attempt label, product name, or path.
    pub fn new(value: &str) -> Result<Self, OutputRootBindingError> {
        if value.is_empty() {
            return Err(OutputRootBindingError::EmptyStagingOwnerId);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Programme identity carried beside the existing F15 leaf state.
///
/// This is an output binding inside the programme parent, not a complete GTC
/// root or a commit coordinator. Its identifiers do not change F15 state bytes.
pub struct ProgrammeOutputBinding {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    staging_owner_id: OutputStagingOwnerId,
    output_owner: OutputOwner,
}

impl ProgrammeOutputBinding {
    /// Constructs a synthetic output binding from prepared input for tests only.
    ///
    /// This does not validate a complete encoding plan. Production construction
    /// remains unavailable until the trusted parent plan exists.
    #[cfg(any(test, feature = "test-support"))]
    fn for_test_from_prepared_input(
        input: &PreparedJobInput,
        attempt_id: AttemptId,
        staging_owner_id: OutputStagingOwnerId,
        provider: OutputProvider,
    ) -> Result<Self, OutputRootBindingError> {
        Self::for_test_values(
            input.programme_id().clone(),
            attempt_id,
            staging_owner_id,
            provider,
            input.configuration().output_reference().local_path(),
            input.configuration().source_samples(),
        )
    }

    #[cfg(any(test, feature = "test-support"))]
    fn for_test_values(
        programme_id: ProgrammeId,
        attempt_id: AttemptId,
        staging_owner_id: OutputStagingOwnerId,
        provider: OutputProvider,
        product_name: &str,
        source_samples: u64,
    ) -> Result<Self, OutputRootBindingError> {
        let attempt_label = canonical_attempt_label(attempt_id);
        let configuration =
            OutputConfiguration::new(provider, &attempt_label, product_name, source_samples)
                .map_err(OutputRootBindingError::OutputOwner)?;
        let output_owner = OutputOwner::cold(configuration);
        Self::bind_test_owner(programme_id, attempt_id, staging_owner_id, output_owner)
    }

    #[cfg(any(test, feature = "test-support"))]
    fn bind_test_owner(
        programme_id: ProgrammeId,
        attempt_id: AttemptId,
        staging_owner_id: OutputStagingOwnerId,
        output_owner: OutputOwner,
    ) -> Result<Self, OutputRootBindingError> {
        if output_owner.configuration().attempt_id() != canonical_attempt_label(attempt_id) {
            return Err(OutputRootBindingError::AttemptBindingMismatch);
        }
        Ok(Self {
            programme_id,
            attempt_id,
            staging_owner_id,
            output_owner,
        })
    }

    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }

    pub fn staging_owner_id(&self) -> &OutputStagingOwnerId {
        &self.staging_owner_id
    }

    pub(super) fn output_owner(&self) -> &OutputOwner {
        &self.output_owner
    }

    pub fn output_configuration(&self) -> &OutputConfiguration {
        self.output_owner.configuration()
    }

    pub fn validate_source_proposal(
        &self,
        bound: &ProgrammeOutputSourceProposal,
    ) -> Result<(), OutputRootBindingError> {
        self.validate_identity(
            bound.programme_id(),
            bound.attempt_id(),
            bound.staging_owner_id(),
        )?;
        let proposal = bound.proposal();
        let current_digest = self
            .output_owner
            .state_digest()
            .map_err(OutputRootBindingError::OutputOwner)?;
        let next_generation = self
            .output_owner
            .generation()
            .checked_add(1)
            .ok_or(OutputRootBindingError::GenerationOverflow)?;
        let identity = proposal.identity();
        let expected_dispatch = identity.source_index().value().checked_add(1);
        let expected_au = au_for_source(identity.source_index())
            .map_err(|_| OutputRootBindingError::ProposalBindingMismatch)?;
        if proposal.attempt_id() != canonical_attempt_label(self.attempt_id())
            || proposal.staging_name() != self.output_configuration().staging_name()
            || proposal.product_name() != self.output_configuration().product_name()
            || proposal.before_state_sha256() != &current_digest
            || proposal.proposed_generation() != next_generation
            || identity.source_index() != self.output_owner.next_source_index()
            || identity.dispatch().value() != expected_dispatch.unwrap_or(u64::MAX)
            || identity.au_index() != expected_au
        {
            return Err(OutputRootBindingError::ProposalBindingMismatch);
        }
        Ok(())
    }

    pub fn validate_finalize_proposal(
        &self,
        bound: &ProgrammeOutputFinalizeProposal,
    ) -> Result<(), OutputRootBindingError> {
        self.validate_identity(
            bound.programme_id(),
            bound.attempt_id(),
            bound.staging_owner_id(),
        )?;
        let proposal = bound.proposal();
        let configuration = self.output_configuration();
        let current_digest = self
            .output_owner
            .state_digest()
            .map_err(OutputRootBindingError::OutputOwner)?;
        if proposal.attempt_id() != canonical_attempt_label(self.attempt_id())
            || proposal.staging_name() != configuration.staging_name()
            || proposal.product_name() != configuration.product_name()
            || proposal.state_sha256() != &current_digest
            || proposal.before_state_sha256() != &current_digest
            || proposal.before_generation() != self.output_owner.generation()
            || proposal.expected_length() != configuration.expected_product_bytes()
        {
            return Err(OutputRootBindingError::ProposalBindingMismatch);
        }
        Ok(())
    }

    fn validate_identity(
        &self,
        programme_id: &ProgrammeId,
        attempt_id: AttemptId,
        staging_owner_id: &OutputStagingOwnerId,
    ) -> Result<(), OutputRootBindingError> {
        if programme_id != self.programme_id() {
            return Err(OutputRootBindingError::ProgrammeIdentityMismatch);
        }
        if attempt_id != self.attempt_id() {
            return Err(OutputRootBindingError::AttemptIdentityMismatch);
        }
        if staging_owner_id != self.staging_owner_id() {
            return Err(OutputRootBindingError::StagingOwnerMismatch);
        }
        if self.output_configuration().attempt_id() != canonical_attempt_label(self.attempt_id()) {
            return Err(OutputRootBindingError::AttemptBindingMismatch);
        }
        Ok(())
    }

    fn bind_source_proposal(
        &self,
        proposal: OutputSourceProposal,
    ) -> Result<ProgrammeOutputSourceProposal, OutputRootBindingError> {
        let bound = ProgrammeOutputSourceProposal {
            programme_id: self.programme_id.clone(),
            attempt_id: self.attempt_id,
            staging_owner_id: self.staging_owner_id.clone(),
            proposal,
        };
        self.validate_source_proposal(&bound)?;
        Ok(bound)
    }

    fn bind_finalize_proposal(
        &self,
        proposal: OutputFinalizeProposal,
    ) -> Result<ProgrammeOutputFinalizeProposal, OutputRootBindingError> {
        let bound = ProgrammeOutputFinalizeProposal {
            programme_id: self.programme_id.clone(),
            attempt_id: self.attempt_id,
            staging_owner_id: self.staging_owner_id.clone(),
            proposal,
        };
        self.validate_finalize_proposal(&bound)?;
        Ok(bound)
    }

    fn validate_lineage(
        &self,
        programme_id: &ProgrammeId,
        attempt_id: AttemptId,
    ) -> Result<(), OutputPublicationPreparationError> {
        if programme_id != self.programme_id() {
            return Err(OutputPublicationPreparationError::ProgrammeIdentityMismatch);
        }
        if attempt_id != self.attempt_id() {
            return Err(OutputPublicationPreparationError::AttemptIdentityMismatch);
        }
        self.validate_output_attempt()
    }

    fn validate_output_attempt(&self) -> Result<(), OutputPublicationPreparationError> {
        if self.output_owner.configuration().attempt_id()
            != canonical_attempt_label(self.attempt_id())
        {
            return Err(OutputPublicationPreparationError::OutputAttemptBindingMismatch);
        }
        Ok(())
    }
}

/// F15 source proposal paired with its typed programme/output binding.
#[must_use]
#[derive(Debug)]
pub struct ProgrammeOutputSourceProposal {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    staging_owner_id: OutputStagingOwnerId,
    proposal: OutputSourceProposal,
}

impl ProgrammeOutputSourceProposal {
    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }

    pub fn staging_owner_id(&self) -> &OutputStagingOwnerId {
        &self.staging_owner_id
    }

    pub fn proposal(&self) -> &OutputSourceProposal {
        &self.proposal
    }

    pub fn into_proposal(self) -> OutputSourceProposal {
        self.proposal
    }
}

/// F15 finalize proposal paired with its typed programme/output binding.
#[must_use]
#[derive(Debug)]
pub struct ProgrammeOutputFinalizeProposal {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    staging_owner_id: OutputStagingOwnerId,
    proposal: OutputFinalizeProposal,
}

impl ProgrammeOutputFinalizeProposal {
    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }

    pub fn staging_owner_id(&self) -> &OutputStagingOwnerId {
        &self.staging_owner_id
    }

    pub fn proposal(&self) -> &OutputFinalizeProposal {
        &self.proposal
    }

    pub fn into_proposal(self) -> OutputFinalizeProposal {
        self.proposal
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputRootBindingError {
    UnsupportedProfile,
    EmptyStagingOwnerId,
    AttemptBindingMismatch,
    ProgrammeIdentityMismatch,
    AttemptIdentityMismatch,
    StagingOwnerMismatch,
    ProposalBindingMismatch,
    GenerationOverflow,
    OutputOwner(OutputOwnerError),
}

impl std::fmt::Display for OutputRootBindingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedProfile => formatter.write_str("OUTPUT_PROFILE_UNSUPPORTED"),
            Self::EmptyStagingOwnerId => formatter.write_str("OUTPUT_STAGING_OWNER_ID"),
            Self::AttemptBindingMismatch => formatter.write_str("OUTPUT_ATTEMPT_BINDING"),
            Self::ProgrammeIdentityMismatch => {
                formatter.write_str("OUTPUT_PROGRAMME_IDENTITY_MISMATCH")
            }
            Self::AttemptIdentityMismatch => {
                formatter.write_str("OUTPUT_ATTEMPT_IDENTITY_MISMATCH")
            }
            Self::StagingOwnerMismatch => formatter.write_str("OUTPUT_STAGING_OWNER_MISMATCH"),
            Self::ProposalBindingMismatch => {
                formatter.write_str("OUTPUT_PROPOSAL_BINDING_MISMATCH")
            }
            Self::GenerationOverflow => formatter.write_str("OUTPUT_GENERATION_OVERFLOW"),
            Self::OutputOwner(error) => write!(formatter, "{}", error.code()),
        }
    }
}

impl std::error::Error for OutputRootBindingError {}

fn canonical_attempt_label(attempt_id: AttemptId) -> String {
    attempt_id.value().to_string()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputPublicationPreparationError {
    NoBestFrame,
    FrameSourceMismatch,
    ProgrammeIdentityMismatch,
    AttemptIdentityMismatch,
    OutputAttemptBindingMismatch,
    OutputRootBinding(OutputRootBindingError),
    MetadataSourceMismatch,
    MetadataDispatchMismatch,
    MetadataAuMismatch,
    MetadataContainerDisposition,
    PreparedInputMismatch,
    CodecCandidateIntegrity,
    MetadataEofDisposition,
    OutputOwner(OutputOwnerError),
}

impl std::fmt::Display for OutputPublicationPreparationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoBestFrame => formatter.write_str("OUTPUT_NO_BEST_FRAME"),
            Self::FrameSourceMismatch => formatter.write_str("OUTPUT_FRAME_SOURCE_MISMATCH"),
            Self::ProgrammeIdentityMismatch => {
                formatter.write_str("OUTPUT_PROGRAMME_IDENTITY_MISMATCH")
            }
            Self::AttemptIdentityMismatch => {
                formatter.write_str("OUTPUT_ATTEMPT_IDENTITY_MISMATCH")
            }
            Self::OutputAttemptBindingMismatch => {
                formatter.write_str("OUTPUT_ATTEMPT_BINDING_MISMATCH")
            }
            Self::OutputRootBinding(error) => write!(formatter, "{error}"),
            Self::MetadataSourceMismatch => formatter.write_str("OUTPUT_METADATA_SOURCE_MISMATCH"),
            Self::MetadataDispatchMismatch => {
                formatter.write_str("OUTPUT_METADATA_DISPATCH_MISMATCH")
            }
            Self::MetadataAuMismatch => formatter.write_str("OUTPUT_METADATA_AU_MISMATCH"),
            Self::MetadataContainerDisposition => {
                formatter.write_str("OUTPUT_METADATA_CONTAINER_DISPOSITION")
            }
            Self::PreparedInputMismatch => formatter.write_str("OUTPUT_PREPARED_INPUT_MISMATCH"),
            Self::CodecCandidateIntegrity => {
                formatter.write_str("OUTPUT_CODEC_CANDIDATE_INTEGRITY")
            }
            Self::MetadataEofDisposition => formatter.write_str("OUTPUT_METADATA_EOF_DISPOSITION"),
            Self::OutputOwner(error) => write!(formatter, "{}", error.code()),
        }
    }
}

impl std::error::Error for OutputPublicationPreparationError {}

/// Prepares the F15P source proposal from the already staged F13 owner result
/// and selected F12 q-search winner. It does not perform IO or install state.
pub fn prepare_output_source(
    output_root: &ProgrammeOutputBinding,
    metadata: &MetadataSourceProposal,
    q_search: &SelectedMainQSearchResult,
) -> Result<ProgrammeOutputSourceProposal, OutputPublicationPreparationError> {
    let SelectedMainQOutcome::Best(candidate) = q_search.outcome() else {
        return Err(OutputPublicationPreparationError::NoBestFrame);
    };
    prepare_output_source_for_frame(
        output_root,
        metadata,
        candidate.source_index(),
        candidate.frame_bytes(),
    )
}

/// Prepares an F15 source from the actual accepted F12 candidate and its retained same-source
/// receipts. It never reconstructs a q-search result or reruns F12.
pub fn prepare_output_source_from_candidate(
    prepared: &PreparedJobInput,
    output_root: &ProgrammeOutputBinding,
    metadata: &MetadataSourceProposal,
    candidate: &SelectedSourceFrameCandidate<'_>,
) -> Result<ProgrammeOutputSourceProposal, OutputPublicationPreparationError> {
    let numeric_plan = candidate.numeric_plan_receipt();
    let source_identity = numeric_plan.identity();
    let source_key = source_identity.source_stage_key();
    let frame = candidate.frame();
    let prepared_identity = prepared.domain_identity_capability();
    let candidate_prepared_identity = source_key
        .prepared_job_input_identity()
        .ok_or(OutputPublicationPreparationError::PreparedInputMismatch)?;
    if !Arc::ptr_eq(candidate_prepared_identity, &prepared_identity)
        || source_identity != frame.source_generation_identity()
        || candidate.bsi_input().source_identity() != source_identity
        || source_identity.source_index() != frame.source_index()
        || source_key.attempt_id() != metadata.bodies_lineage().attempt()
        || source_key.source_index() != metadata.bodies_lineage().source_index()
        || Some(source_key.transaction_id()) != metadata.consumer_transaction_id()
        || prepared.programme_id() != output_root.programme_id()
        || prepared.programme_id() != metadata.bodies_lineage().programme_id()
        || prepared.configuration().source_samples()
            != output_root.output_configuration().source_samples()
    {
        return Err(OutputPublicationPreparationError::PreparedInputMismatch);
    }

    let expected_emdf = emdf_frame_input_for_source(
        metadata.parsed_container(),
        numeric_plan.shared_analysis(),
        prepared.programme_id(),
        &prepared_identity,
    )
    .map_err(|_| OutputPublicationPreparationError::MetadataContainerDisposition)?;
    if &expected_emdf != candidate.emdf_input() {
        return Err(OutputPublicationPreparationError::MetadataContainerDisposition);
    }

    let accepted_frame = AcceptedCodecFrame::from_candidate(frame)
        .map_err(|_| OutputPublicationPreparationError::CodecCandidateIntegrity)?;
    if accepted_frame != candidate.accepted_codec_frame()
        || !candidate.attempted_qs().contains(&frame.q())
    {
        return Err(OutputPublicationPreparationError::CodecCandidateIntegrity);
    }

    prepare_output_source_for_frame(
        output_root,
        metadata,
        frame.source_index(),
        frame.frame_bytes(),
    )
}

fn prepare_output_source_for_frame(
    output_root: &ProgrammeOutputBinding,
    metadata: &MetadataSourceProposal,
    source_index: CodecSourceIndex,
    frame: &[u8],
) -> Result<ProgrammeOutputSourceProposal, OutputPublicationPreparationError> {
    let lineage = metadata.bodies_lineage();
    output_root.validate_lineage(lineage.programme_id(), lineage.attempt())?;
    if source_index != lineage.source_index() {
        return Err(OutputPublicationPreparationError::FrameSourceMismatch);
    }
    let expected_dispatch = source_index
        .dispatch()
        .map_err(|_| OutputPublicationPreparationError::MetadataDispatchMismatch)?;
    if metadata.receipt().programme_id() != output_root.programme_id()
        || metadata.receipt().attempt() != output_root.attempt_id()
        || metadata.receipt().programme_id() != lineage.programme_id()
        || metadata.receipt().attempt() != lineage.attempt()
        || metadata.receipt().transaction_id() != lineage.transaction_id()
        || metadata.receipt().source_index() != source_index
    {
        return Err(OutputPublicationPreparationError::MetadataSourceMismatch);
    }
    if lineage.dispatch() != expected_dispatch || metadata.receipt().dispatch() != expected_dispatch
    {
        return Err(OutputPublicationPreparationError::MetadataDispatchMismatch);
    }

    let expected_au = au_for_source(source_index)
        .map_err(|_| OutputPublicationPreparationError::MetadataAuMismatch)?;
    if metadata.receipt().au_index() != expected_au {
        return Err(OutputPublicationPreparationError::MetadataAuMismatch);
    }
    if metadata.emdf().is_some() != expected_au.is_some() {
        return Err(OutputPublicationPreparationError::MetadataContainerDisposition);
    }

    let declared_frame_sha256: [u8; 32] = Sha256::digest(frame).into();
    let proposal = output_root
        .output_owner()
        .prepare_source(
            OutputSourceIdentity::new(source_index, expected_dispatch, expected_au),
            frame,
            &declared_frame_sha256,
        )
        .map_err(OutputPublicationPreparationError::OutputOwner)?;
    output_root
        .bind_source_proposal(proposal)
        .map_err(OutputPublicationPreparationError::OutputRootBinding)
}

fn au_for_source(source_index: CodecSourceIndex) -> Result<Option<PublishedAuIndex>, ClockError> {
    match publication_for_source(source_index)? {
        PublicationDisposition::Suppressed { .. } => Ok(None),
        PublicationDisposition::Published(consumption) => Ok(Some(consumption.au)),
    }
}

/// Binds metadata EOF's exact pending-writer disposition to the runtime output
/// plan before creating a terminal F15P proposal. No queue is popped or flushed.
pub fn prepare_output_finalize(
    output_root: &ProgrammeOutputBinding,
    metadata_eof: &MetadataEofProposal,
) -> Result<ProgrammeOutputFinalizeProposal, OutputPublicationPreparationError> {
    output_root.validate_lineage(metadata_eof.programme_id(), metadata_eof.attempt())?;
    let output_owner = output_root.output_owner();
    let expected_sources = output_owner.configuration().expected_sources();
    let ordinary_tail = expected_sources
        .checked_sub(1)
        .ok_or(OutputPublicationPreparationError::MetadataEofDisposition)?;
    let au_count = expected_sources
        .checked_sub(2)
        .ok_or(OutputPublicationPreparationError::MetadataEofDisposition)?;
    let mut expected_pending = Vec::with_capacity(7);
    for queue_id in [1_u8, 2, 11] {
        expected_pending.push((queue_id, CodecSourceIndex::new(au_count), "DROP_AT_DONE"));
        expected_pending.push((
            queue_id,
            CodecSourceIndex::new(ordinary_tail),
            "DROP_AT_DONE",
        ));
    }
    expected_pending.push((14, CodecSourceIndex::new(ordinary_tail), "DROP_AT_DONE"));
    let mut actual_pending = metadata_eof.pending_writer_dispositions();
    expected_pending
        .sort_by_key(|(queue_id, writer, disposition)| (*queue_id, writer.value(), *disposition));
    actual_pending
        .sort_by_key(|(queue_id, writer, disposition)| (*queue_id, writer.value(), *disposition));
    if metadata_eof.expected_sources() != output_owner.configuration().expected_sources()
        || !metadata_eof.after_is_finalized()
        || !metadata_eof.queues_are_empty()
        || !metadata_eof.pending_writers_are_drop_at_done()
        || actual_pending != expected_pending
    {
        return Err(OutputPublicationPreparationError::MetadataEofDisposition);
    }
    let proposal = output_owner
        .prepare_finalize()
        .map_err(OutputPublicationPreparationError::OutputOwner)?;
    output_root
        .bind_finalize_proposal(proposal)
        .map_err(OutputPublicationPreparationError::OutputRootBinding)
}
