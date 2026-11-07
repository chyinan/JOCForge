// pattern: Imperative Shell
//! F15P append composition for already prepared source proposals.

use jocforge_application::{
    PreparedJobInput,
    output_publication::{
        AdmittedProductionPlan, AttemptId, OutputConfiguration, OutputFinalizedProposal,
        OutputOwnerError, OutputRootBindingError, OutputSourceAfterProposal, OutputSourceProposal,
        OutputStagingOwnerId, ProgrammeId, ProgrammeOutputBinding, ProgrammeOutputFinalizeProposal,
        ProgrammeOutputSourceProposal,
    },
    source_frame_bridge::{
        AnalysisNumericPlanStage, CodecOwnerProposal, CodecOwnerRoot, CodecOwnerStageError,
        MetadataSourceProposal, NoneDrcOwnerAfterProposal, SelectedSourceFrameCandidate,
        SourceFrameBridgeError, prepare_admitted_source_frame,
        stage_codec_owner_after_source_frame,
    },
};
use jocforge_io::atomic_output::{AtomicOutputStaging, OutputIoError};

#[cfg(test)]
mod tests;

/// IO staging capability created from one typed programme/output binding.
///
/// The opaque staging-owner identity is kept beside the existing F15 staging
/// configuration; it is not encoded in a filename or F15 state field.
pub struct ProgrammeOutputStaging {
    directory: std::path::PathBuf,
    inner: Option<AtomicOutputStaging>,
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    staging_owner_id: OutputStagingOwnerId,
    configuration: OutputConfiguration,
}

impl ProgrammeOutputStaging {
    /// Captures the typed binding without touching the filesystem.
    pub fn new(directory: &std::path::Path, binding: &ProgrammeOutputBinding) -> Self {
        let configuration = binding.output_configuration().clone();
        Self {
            directory: directory.to_path_buf(),
            inner: None,
            programme_id: binding.programme_id().clone(),
            attempt_id: binding.attempt_id(),
            staging_owner_id: binding.staging_owner_id().clone(),
            configuration,
        }
    }

    /// Creates the lazy IO staging adapter from the destination admitted by F15.
    /// No file opens until the first admitted append.
    pub fn from_admitted_plan(
        plan: &jocforge_application::output_publication::AdmittedProductionPlan,
    ) -> Result<Self, OutputPublicationError> {
        let final_path = std::path::Path::new(plan.final_destination());
        let parent = final_path
            .parent()
            .filter(|parent| parent.is_absolute())
            .ok_or(OutputPublicationError::Binding(
                OutputRootBindingError::DestinationMismatch,
            ))?;
        let product_name = final_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(OutputPublicationError::Binding(
                OutputRootBindingError::DestinationMismatch,
            ))?;
        let binding = plan.output_binding();
        if product_name != binding.output_configuration().product_name()
            || parent
                .join(binding.output_configuration().staging_name())
                .to_str()
                != Some(plan.staging_namespace())
        {
            return Err(OutputPublicationError::Binding(
                OutputRootBindingError::DestinationMismatch,
            ));
        }
        Ok(Self::new(parent, binding))
    }

    pub fn validate_binding(
        &self,
        binding: &ProgrammeOutputBinding,
    ) -> Result<(), OutputRootBindingError> {
        if &self.programme_id != binding.programme_id() {
            return Err(OutputRootBindingError::ProgrammeIdentityMismatch);
        }
        if self.attempt_id != binding.attempt_id() {
            return Err(OutputRootBindingError::AttemptIdentityMismatch);
        }
        if &self.staging_owner_id != binding.staging_owner_id() {
            return Err(OutputRootBindingError::StagingOwnerMismatch);
        }
        if &self.configuration != binding.output_configuration() {
            return Err(OutputRootBindingError::ProposalBindingMismatch);
        }
        Ok(())
    }

    fn ensure_open(
        &mut self,
        binding: &ProgrammeOutputBinding,
    ) -> Result<&mut AtomicOutputStaging, OutputPublicationError> {
        self.validate_binding(binding)
            .map_err(OutputPublicationError::Binding)?;
        if self.inner.is_none() {
            self.inner = Some(
                AtomicOutputStaging::create(&self.directory, &self.configuration)
                    .map_err(OutputPublicationError::Io)?,
            );
        }
        Ok(self.inner.as_mut().expect("staging was opened"))
    }

    pub fn staging_path(&self) -> std::path::PathBuf {
        self.directory.join(self.configuration.staging_name())
    }

    pub fn is_abandoned(&self) -> bool {
        self.inner
            .as_ref()
            .is_some_and(AtomicOutputStaging::is_abandoned)
    }

    /// Marks the whole attempt unusable after a fatal parent-stage failure.
    pub fn abandon_attempt(&mut self) {
        if let Some(staging) = self.inner.as_mut() {
            staging.abandon();
        }
    }
}

#[derive(Debug)]
pub enum OutputPublicationError {
    Io(OutputIoError),
    Owner(OutputOwnerError),
    Binding(OutputRootBindingError),
}

impl std::fmt::Display for OutputPublicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Owner(error) => write!(formatter, "{error}"),
            Self::Binding(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for OutputPublicationError {}

/// One source's durable F15 result and its matching noncommitting codec-owner proposal.
///
/// The parent still owns installation of both after-values with the other source roots.
#[must_use]
#[derive(Debug)]
pub struct StagedSourceFrameOutput {
    output_after: OutputSourceAfterProposal,
    codec_owner_proposal: CodecOwnerProposal,
}

impl StagedSourceFrameOutput {
    pub const fn output_after(&self) -> &OutputSourceAfterProposal {
        &self.output_after
    }

    pub const fn codec_owner_proposal(&self) -> &CodecOwnerProposal {
        &self.codec_owner_proposal
    }

    pub fn into_parts(self) -> (OutputSourceAfterProposal, CodecOwnerProposal) {
        (self.output_after, self.codec_owner_proposal)
    }
}

/// Errors while joining an accepted F12 source frame to durable F15 output and codec state.
#[derive(Debug)]
pub enum SourceFrameOutputStageError {
    CandidateSourceMismatch,
    CandidateFrameMismatch,
    CandidateAttemptMismatch,
    MissingDurableAppend,
    UnexpectedDurableAppend,
    Output(OutputPublicationError),
    CodecOwner(CodecOwnerStageError),
    SourceFrame(SourceFrameBridgeError),
    OutputPreparation(jocforge_application::output_publication::OutputPublicationPreparationError),
}

impl std::fmt::Display for SourceFrameOutputStageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CandidateSourceMismatch => formatter.write_str("SOURCE_FRAME_OUTPUT_SOURCE"),
            Self::CandidateFrameMismatch => formatter.write_str("SOURCE_FRAME_OUTPUT_BYTES"),
            Self::CandidateAttemptMismatch => formatter.write_str("SOURCE_FRAME_OUTPUT_ATTEMPT"),
            Self::MissingDurableAppend => formatter.write_str("SOURCE_FRAME_OUTPUT_APPEND_MISSING"),
            Self::UnexpectedDurableAppend => {
                formatter.write_str("SOURCE_FRAME_OUTPUT_APPEND_UNEXPECTED")
            }
            Self::Output(error) => write!(formatter, "{error}"),
            Self::CodecOwner(error) => write!(formatter, "{error:?}"),
            Self::SourceFrame(error) => write!(formatter, "{error:?}"),
            Self::OutputPreparation(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for SourceFrameOutputStageError {}

/// Durably stages the already validated F15 proposal for one accepted F12 candidate.
///
/// The caller prepares `output_proposal` with
/// `jocforge_application::output_publication::prepare_output_source_from_candidate`.
/// This function checks candidate source and bytes before touching the staging file, then
/// returns both owner after-values without installing either. The enclosing programme callback
/// remains responsible for replacing the output and codec roots together with its other owners.
pub fn stage_source_frame_output(
    staging: &mut ProgrammeOutputStaging,
    binding: &ProgrammeOutputBinding,
    codec_owner: &CodecOwnerRoot,
    candidate: &SelectedSourceFrameCandidate<'_>,
    output_proposal: ProgrammeOutputSourceProposal,
) -> Result<StagedSourceFrameOutput, SourceFrameOutputStageError> {
    let frame = candidate.frame();
    let proposal = output_proposal.proposal();
    if candidate.accepted_codec_frame().attempt_id() != binding.attempt_id()
        || output_proposal.attempt_id() != binding.attempt_id()
    {
        return Err(SourceFrameOutputStageError::CandidateAttemptMismatch);
    }
    let append_expected = match proposal {
        OutputSourceProposal::Suppressed(_) => None,
        OutputSourceProposal::Append(append) => Some(append.frame_bytes()),
    };
    let append_expected = validate_candidate_output_pair(
        &frame.source_index(),
        frame.frame_bytes(),
        &proposal.identity().source_index(),
        append_expected,
    )?;
    if codec_owner.generation().value().checked_add(1).is_none() {
        return Err(SourceFrameOutputStageError::CodecOwner(
            CodecOwnerStageError::GenerationOverflow,
        ));
    }

    let output_after = stage_output_source(staging, binding, output_proposal)
        .map_err(SourceFrameOutputStageError::Output)?;
    let codec_output_after = match (append_expected, output_after.durable_append_receipt()) {
        (true, Some(_)) => Some(&output_after),
        (false, None) => None,
        (true, None) => return Err(SourceFrameOutputStageError::MissingDurableAppend),
        (false, Some(_)) => return Err(SourceFrameOutputStageError::UnexpectedDurableAppend),
    };
    let codec_owner_proposal =
        stage_codec_owner_after_source_frame(codec_owner, candidate, codec_output_after)
            .map_err(SourceFrameOutputStageError::CodecOwner)?;

    Ok(StagedSourceFrameOutput {
        output_after,
        codec_owner_proposal,
    })
}

/// Builds one F12 candidate from same-source owner proposals and durably stages its F15 result.
///
/// The selected BAP value comes from `admitted_plan`. All audio-bearing proposals remain inputs
/// from the approved callback pipeline; the caller retains every owner replacement decision.
pub fn stage_admitted_source_frame_output(
    staging: &mut ProgrammeOutputStaging,
    binding: &ProgrammeOutputBinding,
    prepared: &PreparedJobInput,
    admitted_plan: &AdmittedProductionPlan,
    codec_owner: &CodecOwnerRoot,
    analysis_stage: &mut AnalysisNumericPlanStage,
    drc_after: &NoneDrcOwnerAfterProposal,
    metadata: &MetadataSourceProposal,
) -> Result<StagedSourceFrameOutput, SourceFrameOutputStageError> {
    if !admitted_plan.is_bound_to(prepared) {
        return Err(SourceFrameOutputStageError::SourceFrame(
            SourceFrameBridgeError::F15AdmissionRejected,
        ));
    }
    let candidate =
        prepare_admitted_source_frame(prepared, admitted_plan, analysis_stage, drc_after, metadata)
            .map_err(SourceFrameOutputStageError::SourceFrame)?;
    let output_proposal =
        jocforge_application::output_publication::prepare_output_source_from_candidate(
            prepared, binding, metadata, &candidate,
        )
        .map_err(SourceFrameOutputStageError::OutputPreparation)?;
    stage_source_frame_output(staging, binding, codec_owner, &candidate, output_proposal)
}

fn validate_candidate_output_pair<SourceIndex: Eq>(
    candidate_source: &SourceIndex,
    candidate_frame: &[u8],
    proposal_source: &SourceIndex,
    proposal_frame: Option<&[u8]>,
) -> Result<bool, SourceFrameOutputStageError> {
    if candidate_source != proposal_source {
        return Err(SourceFrameOutputStageError::CandidateSourceMismatch);
    }
    let Some(proposal_frame) = proposal_frame else {
        return Ok(false);
    };
    if candidate_frame != proposal_frame {
        return Err(SourceFrameOutputStageError::CandidateFrameMismatch);
    }
    Ok(true)
}

/// Applies the approved suppression/append branch. Append returns only after
/// exact readback and file fsync; the returned owner proposal remains uninstalled.
pub fn stage_output_source(
    staging: &mut ProgrammeOutputStaging,
    binding: &ProgrammeOutputBinding,
    proposal: ProgrammeOutputSourceProposal,
) -> Result<OutputSourceAfterProposal, OutputPublicationError> {
    staging
        .validate_binding(binding)
        .map_err(OutputPublicationError::Binding)?;
    binding
        .validate_source_proposal(&proposal)
        .map_err(OutputPublicationError::Binding)?;
    match proposal.into_proposal() {
        OutputSourceProposal::Suppressed(proposal) => Ok(proposal.into_after()),
        OutputSourceProposal::Append(proposal) => {
            let durable_receipt = staging
                .ensure_open(binding)?
                .append(&proposal)
                .map_err(OutputPublicationError::Io)?;
            proposal
                .complete(durable_receipt)
                .map_err(OutputPublicationError::Owner)
        }
    }
}

/// Finalizes only after the parent has prepared a terminal output proposal.
/// The returned after-owner remains a proposal for the enclosing root swap.
pub fn finalize_output(
    staging: &mut ProgrammeOutputStaging,
    binding: &ProgrammeOutputBinding,
    proposal: ProgrammeOutputFinalizeProposal,
) -> Result<OutputFinalizedProposal, OutputPublicationError> {
    staging
        .validate_binding(binding)
        .map_err(OutputPublicationError::Binding)?;
    binding
        .validate_finalize_proposal(&proposal)
        .map_err(OutputPublicationError::Binding)?;
    let proposal = proposal.into_proposal();
    let receipt = staging
        .ensure_open(binding)?
        .finalize(&proposal)
        .map_err(OutputPublicationError::Io)?;
    proposal
        .complete(receipt)
        .map_err(OutputPublicationError::Owner)
}
