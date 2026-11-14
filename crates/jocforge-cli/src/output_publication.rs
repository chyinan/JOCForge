// pattern: Imperative Shell
//! F15P append composition for already prepared source proposals.

use jocforge_application::flat7x_pair_source::Flat7xPairI0FrameCandidate;
use jocforge_application::{
    PreparedJobInput,
    flat7x_pair_output::{
        Flat7xAuAppendProposal, Flat7xAuAttemptRetirement, Flat7xAuOutputAfterProposal,
        Flat7xAuOutputOwner, Flat7xAuSourceReceiptInput, Flat7xElementaryStreamStageProposal,
        Flat7xOpaqueD0FrameInput, Flat7xPairAppendProposalError, Flat7xSourceGeneration,
        stage_receipted_i0_with_opaque_d0,
    },
    meter_source_bridge::Flat7xI0D0TargetPlan,
    output_publication::{
        AdmittedProductionPlan, AttemptId, OutputConfiguration, OutputFinalizedProposal,
        OutputOwnerError, OutputRootBindingError, OutputSourceAfterProposal, OutputSourceProposal,
        OutputStagingOwnerId, ProgrammeId, ProgrammeOutputBinding, ProgrammeOutputFinalizeProposal,
        ProgrammeOutputSourceProposal,
    },
    source_frame_bridge::{
        AnalysisNumericPlanStage, CodecOwnerProposal, CodecOwnerRoot, CodecOwnerStageError,
        Flat7xD0OverlapHistory, MetadataSourceProposal, NoneDrcOwnerAfterProposal,
        SelectedSourceFrameCandidate, SourceFrameBridgeError, prepare_admitted_source_frame,
        stage_codec_owner_after_source_frame, stage_codec_owner_after_source_frame_with_d0_history,
    },
};
use jocforge_io::atomic_output::{AtomicOutputStaging, Flat7xAtomicAppendFailure, OutputIoError};

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
    flat7x_pair_owner: Option<Flat7xAuOutputOwner>,
}

#[derive(Debug)]
pub enum Flat7xOutputAppendRejection {
    OutputBinding(OutputRootBindingError),
    AttemptMismatch,
    ProgrammeMismatch,
    SourceIdentityMismatch,
    SourceTransactionMismatch,
    FrameExtentMismatch,
    AccessUnitExtentMismatch,
    OutputOffsetMismatch,
    FrameIntegrityMismatch,
}

#[derive(Debug)]
pub enum Flat7xOutputAppendFailure {
    Rejected {
        reason: Flat7xOutputAppendRejection,
        retirement: Flat7xAuAttemptRetirement,
    },
    StagingSetup {
        error: OutputPublicationError,
        retirement: Flat7xAuAttemptRetirement,
    },
    Append(Flat7xAtomicAppendFailure),
    #[cfg(test)]
    MemoryAppend(jocforge_io::flat7x_au_append::Flat7xAuAppendIoFailure),
}

#[derive(Debug)]
pub enum Flat7xPairOutputAppendFailure {
    PairProposal(Flat7xPairAppendProposalError),
    Output(Flat7xOutputAppendFailure),
    OutputOwner(OutputOwnerError),
}

impl Flat7xOutputAppendFailure {
    pub fn retirement(&self) -> &Flat7xAuAttemptRetirement {
        match self {
            Self::Rejected { retirement, .. } | Self::StagingSetup { retirement, .. } => retirement,
            Self::Append(failure) => failure.retirement(),
            #[cfg(test)]
            Self::MemoryAppend(failure) => failure.retirement(),
        }
    }
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
            flat7x_pair_owner: None,
        }
    }

    pub(crate) fn take_flat7x_pair_output_owner(
        &mut self,
        binding: &ProgrammeOutputBinding,
    ) -> Result<Flat7xAuOutputOwner, OutputPublicationError> {
        self.validate_binding(binding)
            .map_err(OutputPublicationError::Binding)?;
        if let Some(owner) = self.flat7x_pair_owner.take() {
            return Ok(owner);
        }
        Flat7xAuOutputOwner::cold(
            binding.output_configuration().attempt_id(),
            binding.output_configuration().expected_published_aus(),
        )
        .map_err(|error| OutputPublicationError::Flat7xPairOwner(format!("{error:?}")))
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

    pub fn candidate_path(&self) -> std::path::PathBuf {
        self.directory.join(self.configuration.product_name())
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

    /// Appends one domain-approved Flat7x I0+D0 proposal through the existing atomic adapter.
    ///
    /// The proposal type carries F15's validated paired source receipts. The Flat7x CLI source
    /// stage calls this after the same-source I0/D0 and Integrity proposals have been prepared.
    pub(crate) fn append_flat7x_au<'a>(
        &mut self,
        binding: &ProgrammeOutputBinding,
        proposal: Flat7xAuAppendProposal<'a>,
    ) -> Result<Flat7xAuOutputAfterProposal<'a>, Flat7xOutputAppendFailure> {
        if let Err(reason) = self.validate_flat7x_append(binding, &proposal) {
            return Err(Flat7xOutputAppendFailure::Rejected {
                reason,
                retirement: proposal.retire_staging_failure(),
            });
        }

        let staging = match self.ensure_open(binding) {
            Ok(staging) => staging,
            Err(error) => {
                return Err(Flat7xOutputAppendFailure::StagingSetup {
                    error,
                    retirement: proposal.retire_staging_failure(),
                });
            }
        };

        staging
            .append_flat7x_au(proposal)
            .map_err(Flat7xOutputAppendFailure::Append)
    }

    /// Joins the source-bound I0 candidate and caller-owned opaque D0 frame to F15 and the existing
    /// atomic staging adapter for callers that supply a D0 frame through the opaque-frame seam.
    #[allow(dead_code)] // No producer can currently supply the complete source-receipted pair.
    pub(crate) fn append_flat7x_pair<'a>(
        &mut self,
        binding: &ProgrammeOutputBinding,
        i0_frame: &Flat7xPairI0FrameCandidate,
        i0_source_generation: Flat7xSourceGeneration,
        d0_frame: Flat7xOpaqueD0FrameInput<'_>,
        output_owner: Flat7xAuOutputOwner,
        receipts: Flat7xAuSourceReceiptInput<'a>,
    ) -> Result<Flat7xAuOutputAfterProposal<'a>, Flat7xPairOutputAppendFailure> {
        let proposal = stage_receipted_i0_with_opaque_d0(
            i0_frame,
            i0_source_generation,
            d0_frame,
            output_owner,
            receipts,
        )
        .map_err(Flat7xPairOutputAppendFailure::PairProposal)?;
        self.append_flat7x_au(binding, proposal)
            .map_err(Flat7xPairOutputAppendFailure::Output)
    }

    #[allow(unsafe_code)]
    pub(crate) fn append_flat7x_pair_with_output_proposal<'a>(
        &mut self,
        binding: &ProgrammeOutputBinding,
        pair_proposal: Flat7xAuAppendProposal<'a>,
        output_proposal: ProgrammeOutputSourceProposal,
    ) -> Result<
        (Flat7xAuOutputAfterProposal<'a>, OutputSourceAfterProposal),
        Flat7xPairOutputAppendFailure,
    > {
        let pair_after = self
            .append_flat7x_au(binding, pair_proposal)
            .map_err(Flat7xPairOutputAppendFailure::Output)?;
        self.flat7x_pair_owner = Some(pair_after.after_owner().clone());

        match output_proposal.into_proposal() {
            OutputSourceProposal::Append(append) => {
                // SAFETY: pair_after came directly from append_flat7x_au, which runs the pair IO
                // writer, syncs the staging file, and verifies exact readback plus each frame CRC.
                let receipt = unsafe { append.record_flat7x_pair_io_success(pair_after.receipt()) }
                    .map_err(Flat7xPairOutputAppendFailure::OutputOwner)?;
                let output_after = append
                    .complete(receipt)
                    .map_err(Flat7xPairOutputAppendFailure::OutputOwner)?;
                Ok((pair_after, output_after))
            }
            OutputSourceProposal::Suppressed(_) => Err(Flat7xPairOutputAppendFailure::OutputOwner(
                OutputOwnerError::AuOrder,
            )),
        }
    }

    #[cfg(test)]
    fn append_flat7x_au_to_memory_for_test<'a>(
        &self,
        binding: &ProgrammeOutputBinding,
        sink: &mut std::io::Cursor<Vec<u8>>,
        proposal: Flat7xAuAppendProposal<'a>,
    ) -> Result<Flat7xAuOutputAfterProposal<'a>, Flat7xOutputAppendFailure> {
        if let Err(reason) = self.validate_flat7x_append(binding, &proposal) {
            return Err(Flat7xOutputAppendFailure::Rejected {
                reason,
                retirement: proposal.retire_staging_failure(),
            });
        }

        jocforge_io::flat7x_au_append::append_flat7x_au_to_memory_for_test(sink, proposal)
            .map_err(Flat7xOutputAppendFailure::MemoryAppend)
    }

    fn validate_flat7x_append(
        &self,
        binding: &ProgrammeOutputBinding,
        proposal: &Flat7xAuAppendProposal<'_>,
    ) -> Result<(), Flat7xOutputAppendRejection> {
        self.validate_binding(binding)
            .map_err(Flat7xOutputAppendRejection::OutputBinding)?;
        if proposal.attempt_id() != self.configuration.attempt_id() {
            return Err(Flat7xOutputAppendRejection::AttemptMismatch);
        }

        let (i0_identity, d0_identity) = proposal.source_identities();
        if i0_identity != d0_identity {
            return Err(Flat7xOutputAppendRejection::SourceIdentityMismatch);
        }
        if i0_identity.programme_id != self.programme_id.as_str() {
            return Err(Flat7xOutputAppendRejection::ProgrammeMismatch);
        }
        if i0_identity.attempt != self.attempt_id.value() {
            return Err(Flat7xOutputAppendRejection::AttemptMismatch);
        }
        if proposal
            .metadata_consumer_transaction_id()
            .is_none_or(|transaction| transaction.as_str() != i0_identity.transaction_id)
        {
            return Err(Flat7xOutputAppendRejection::SourceTransactionMismatch);
        }

        let ledger = proposal.ledger_entry();
        if ledger.metadata_identity()
            != (
                i0_identity.source_index,
                i0_identity.dispatch,
                ledger.au_index(),
            )
            || i0_identity.au_index != Some(ledger.au_index())
        {
            return Err(Flat7xOutputAppendRejection::SourceIdentityMismatch);
        }

        let target = Flat7xI0D0TargetPlan::approved_contract();
        if ledger.frame_lengths() != (target.i0().frame_bytes(), target.d0().frame_bytes()) {
            return Err(Flat7xOutputAppendRejection::FrameExtentMismatch);
        }
        let expected_access_unit_bytes = target.aggregate_frame_bytes();
        if ledger.length() != expected_access_unit_bytes
            || u32::try_from(proposal.bytes().len()).ok() != Some(expected_access_unit_bytes)
        {
            return Err(Flat7xOutputAppendRejection::AccessUnitExtentMismatch);
        }
        let expected_offset = ledger
            .au_index()
            .value()
            .checked_mul(u64::from(expected_access_unit_bytes))
            .ok_or(Flat7xOutputAppendRejection::OutputOffsetMismatch)?;
        if ledger.offset() != expected_offset {
            return Err(Flat7xOutputAppendRejection::OutputOffsetMismatch);
        }
        if ledger.frame_crc_residuals() != (0, 0) {
            return Err(Flat7xOutputAppendRejection::FrameIntegrityMismatch);
        }

        Ok(())
    }
}

/// Durable attempt-local Flat7x candidate artifact; this carries no production or DONE authority.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xExperimentalCandidateReceipt {
    attempt_id: String,
    candidate_path: std::path::PathBuf,
    bytes_written: u32,
    sha256: [u8; 32],
    file_sync_completed: bool,
    parent_sync_completed: bool,
}

impl Flat7xExperimentalCandidateReceipt {
    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn candidate_path(&self) -> &std::path::Path {
        &self.candidate_path
    }

    pub const fn bytes_written(&self) -> u32 {
        self.bytes_written
    }

    pub const fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }

    pub const fn file_sync_completed(&self) -> bool {
        self.file_sync_completed
    }

    pub const fn parent_sync_completed(&self) -> bool {
        self.parent_sync_completed
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    pub const fn final_exposure_allowed(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub enum OutputPublicationError {
    Io(OutputIoError),
    Owner(OutputOwnerError),
    Flat7xPairOwner(String),
    Flat7xPairFinalize(String),
    ExperimentalCandidate(String),
    ExperimentalCandidateRequired,
    Binding(OutputRootBindingError),
}

impl std::fmt::Display for OutputPublicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Owner(error) => write!(formatter, "{error}"),
            Self::Flat7xPairOwner(error) => write!(formatter, "{error}"),
            Self::Flat7xPairFinalize(error) => write!(formatter, "{error}"),
            Self::ExperimentalCandidate(error) => write!(formatter, "{error}"),
            Self::ExperimentalCandidateRequired => formatter.write_str(
                "Flat7x must use the experimental candidate capture path; generic final exposure is disabled",
            ),
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
    stage_source_frame_output_inner(
        staging,
        binding,
        codec_owner,
        candidate,
        output_proposal,
        None,
    )
}

pub fn stage_source_frame_output_with_d0_history(
    staging: &mut ProgrammeOutputStaging,
    binding: &ProgrammeOutputBinding,
    codec_owner: &CodecOwnerRoot,
    candidate: &SelectedSourceFrameCandidate<'_>,
    output_proposal: ProgrammeOutputSourceProposal,
    d0_history_after: &Flat7xD0OverlapHistory,
) -> Result<StagedSourceFrameOutput, SourceFrameOutputStageError> {
    stage_source_frame_output_inner(
        staging,
        binding,
        codec_owner,
        candidate,
        output_proposal,
        Some(d0_history_after),
    )
}

fn stage_source_frame_output_inner(
    staging: &mut ProgrammeOutputStaging,
    binding: &ProgrammeOutputBinding,
    codec_owner: &CodecOwnerRoot,
    candidate: &SelectedSourceFrameCandidate<'_>,
    output_proposal: ProgrammeOutputSourceProposal,
    d0_history_after: Option<&Flat7xD0OverlapHistory>,
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
    let codec_owner_proposal = match d0_history_after {
        Some(history) => stage_codec_owner_after_source_frame_with_d0_history(
            codec_owner,
            candidate,
            codec_output_after,
            history,
        ),
        None => stage_codec_owner_after_source_frame(codec_owner, candidate, codec_output_after),
    }
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
    if binding.output_configuration().frame_bytes()
        == Flat7xI0D0TargetPlan::approved_contract().aggregate_frame_bytes()
    {
        return Err(OutputPublicationError::ExperimentalCandidateRequired);
    }
    let proposal = proposal.into_proposal();
    let receipt = staging
        .ensure_open(binding)?
        .finalize(&proposal)
        .map_err(OutputPublicationError::Io)?;
    proposal
        .complete(receipt)
        .map_err(OutputPublicationError::Owner)
}

/// Writes one complete Flat7x stream to its attempt-local `.candidate.ec3` path.
///
/// The generic output finalize proposal and the complete F15 pair ledger are validated first.
/// The specialized stream stage performs exact extent/ledger/hash checks, file sync and readback;
/// the result is explicitly an experimental candidate and does not install a DONE output root.
pub fn capture_flat7x_experimental_candidate(
    staging: &mut ProgrammeOutputStaging,
    binding: &ProgrammeOutputBinding,
    output_proposal: ProgrammeOutputFinalizeProposal,
) -> Result<Flat7xExperimentalCandidateReceipt, OutputPublicationError> {
    staging
        .validate_binding(binding)
        .map_err(OutputPublicationError::Binding)?;
    binding
        .validate_finalize_proposal(&output_proposal)
        .map_err(OutputPublicationError::Binding)?;
    let candidate_path = staging.candidate_path();
    let expected_bytes = binding.output_configuration().expected_product_bytes();
    let attempt_id = binding.output_configuration().attempt_id().to_owned();
    let result = capture_flat7x_candidate_inner(staging, binding, output_proposal);
    match result {
        Ok(receipt) => {
            staging.abandon_attempt();
            Ok(receipt)
        }
        Err(error) => {
            staging.abandon_attempt();
            let observed_bytes = std::fs::metadata(&candidate_path)
                .ok()
                .map(|metadata| metadata.len());
            let failure_receipt = persist_flat7x_candidate_failure(
                &candidate_path,
                &attempt_id,
                expected_bytes,
                observed_bytes,
                &error.to_string(),
            );
            let failure_location = failure_receipt.map_or_else(
                |failure| format!("failure receipt could not be saved: {failure}"),
                |path| format!("failure receipt={}", path.display()),
            );
            Err(OutputPublicationError::ExperimentalCandidate(format!(
                "candidate capture failed: {error}; candidate_bytes={observed_bytes:?}/{expected_bytes}; {failure_location}"
            )))
        }
    }
}

fn capture_flat7x_candidate_inner(
    staging: &mut ProgrammeOutputStaging,
    binding: &ProgrammeOutputBinding,
    output_proposal: ProgrammeOutputFinalizeProposal,
) -> Result<Flat7xExperimentalCandidateReceipt, OutputPublicationError> {
    let target = Flat7xI0D0TargetPlan::approved_contract();
    if binding.output_configuration().frame_bytes() != target.aggregate_frame_bytes() {
        return Err(OutputPublicationError::ExperimentalCandidate(
            "candidate output binding is not the approved Flat7x aggregate".to_owned(),
        ));
    }

    let pair_finalize = staging
        .take_flat7x_pair_output_owner(binding)?
        .prepare_finalize()
        .map_err(|retirement| {
            OutputPublicationError::Flat7xPairFinalize(format!("{retirement:?}"))
        })?;
    let expected_length = output_proposal.proposal().expected_length();
    if pair_finalize.durable_length() != expected_length {
        return Err(OutputPublicationError::ExperimentalCandidate(
            "F15 pair ledger and OutputOwner lengths differ".to_owned(),
        ));
    }
    let stream_bytes = std::fs::read(staging.staging_path()).map_err(|error| {
        OutputPublicationError::ExperimentalCandidate(format!(
            "could not read complete F15 staging stream: {error}"
        ))
    })?;
    if u64::try_from(stream_bytes.len()).ok() != Some(expected_length) {
        return Err(OutputPublicationError::ExperimentalCandidate(
            "F15 staging stream length differs from the admitted OutputOwner extent".to_owned(),
        ));
    }
    let candidate_path = staging.candidate_path();
    let candidate_name = candidate_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            OutputPublicationError::ExperimentalCandidate(
                "candidate file name is not valid Unicode".to_owned(),
            )
        })?;
    if !candidate_name.ends_with(".candidate.ec3") || candidate_path.exists() {
        return Err(OutputPublicationError::ExperimentalCandidate(
            "Flat7x candidate path must be a new `.candidate.ec3` file".to_owned(),
        ));
    }
    let proposal = Flat7xElementaryStreamStageProposal::new(target, &stream_bytes, &pair_finalize)
        .map_err(|error| {
            OutputPublicationError::ExperimentalCandidate(format!(
                "Flat7x stream stage proposal: {error:?}"
            ))
        })?;
    let receipt = write_flat7x_candidate_stage_file(candidate_path, proposal)?;
    Ok(receipt)
}

fn write_flat7x_candidate_stage_file(
    candidate_path: std::path::PathBuf,
    proposal: Flat7xElementaryStreamStageProposal<'_, '_>,
) -> Result<Flat7xExperimentalCandidateReceipt, OutputPublicationError> {
    let candidate_name_is_valid = candidate_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".candidate.ec3"));
    if !candidate_name_is_valid || candidate_path.exists() {
        return Err(OutputPublicationError::ExperimentalCandidate(
            "candidate file name must end with `.candidate.ec3` and be new".to_owned(),
        ));
    }
    let candidate_parent = candidate_path.parent().ok_or_else(|| {
        OutputPublicationError::ExperimentalCandidate("candidate path has no parent".to_owned())
    })?;
    let attempt_id = proposal.finalize_proposal().attempt_id().to_owned();
    let mut candidate_file = std::fs::OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&candidate_path)
        .map_err(|error| {
            OutputPublicationError::ExperimentalCandidate(format!("create candidate file: {error}"))
        })?;
    let stage_after =
        match jocforge_io::flat7x_elementary_stream_stage::stage_flat7x_elementary_stream_to_file(
            &mut candidate_file,
            proposal,
        ) {
            Ok(after) => after,
            Err(error) => {
                let _ = candidate_file.sync_all();
                return Err(OutputPublicationError::ExperimentalCandidate(format!(
                    "write/readback candidate: {error}"
                )));
            }
        };
    if stage_after.production_allowed() || stage_after.final_exposure_allowed() {
        return Err(OutputPublicationError::ExperimentalCandidate(
            "Flat7x stream stage unexpectedly enabled production/final exposure".to_owned(),
        ));
    }
    let stage_receipt = stage_after.receipt();
    if !stage_receipt.file_sync_completed() {
        return Err(OutputPublicationError::ExperimentalCandidate(
            "candidate file sync receipt is absent".to_owned(),
        ));
    }
    let bytes_written = stage_receipt.bytes_written();
    let sha256 = *stage_receipt.readback_sha256();
    drop(candidate_file);
    jocforge_io::atomic_output::sync_experimental_candidate_directory(candidate_parent).map_err(
        |error| {
            OutputPublicationError::ExperimentalCandidate(format!(
                "candidate directory sync: {error}"
            ))
        },
    )?;
    Ok(Flat7xExperimentalCandidateReceipt {
        attempt_id,
        candidate_path,
        bytes_written,
        sha256,
        file_sync_completed: true,
        parent_sync_completed: true,
    })
}

fn persist_flat7x_candidate_failure(
    candidate_path: &std::path::Path,
    attempt_id: &str,
    expected_bytes: u64,
    observed_bytes: Option<u64>,
    error: &str,
) -> Result<std::path::PathBuf, String> {
    use std::io::Write as _;

    let candidate_name = candidate_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.ends_with(".candidate.ec3"))
        .ok_or_else(|| "candidate path has no `.candidate.ec3` name".to_owned())?;
    let parent = candidate_path
        .parent()
        .ok_or_else(|| "candidate path has no parent".to_owned())?;
    if !candidate_path.exists() {
        let mut empty_candidate = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(candidate_path)
            .map_err(|error| format!("create failed-candidate extent file: {error}"))?;
        empty_candidate
            .flush()
            .map_err(|error| format!("flush failed-candidate extent file: {error}"))?;
        empty_candidate
            .sync_all()
            .map_err(|error| format!("sync failed-candidate extent file: {error}"))?;
    }
    let observed_bytes = std::fs::metadata(candidate_path)
        .ok()
        .map(|metadata| metadata.len())
        .or(observed_bytes);
    let status = match observed_bytes {
        Some(size) if size == expected_bytes => "FAILED_CANDIDATE_FULL_EXTENT_UNVERIFIED",
        Some(size) if size > 0 => "FAILED_CANDIDATE_PARTIAL_EXTENT",
        _ => "FAILED_CANDIDATE_EMPTY_EXTENT",
    };
    let sidecar_path = candidate_path.with_file_name(format!("{candidate_name}.failure.txt"));
    let candidate_path_text = candidate_path.to_string_lossy().replace(['\r', '\n'], " ");
    let error_text = error.replace(['\r', '\n'], " ");
    let receipt_text = format!(
        "schema=jocforge.flat7x.experimental-candidate-failure.v1\nstatus={status}\nattempt_id={attempt_id}\ncandidate_path={candidate_path_text}\nexpected_bytes={expected_bytes}\nobserved_bytes={observed_bytes:?}\nproduction_allowed=false\nprogramme_done=false\nerror={error_text}\n"
    );
    let mut receipt_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&sidecar_path)
        .map_err(|error| format!("create candidate failure receipt: {error}"))?;
    receipt_file
        .write_all(receipt_text.as_bytes())
        .map_err(|error| format!("write candidate failure receipt: {error}"))?;
    receipt_file
        .flush()
        .map_err(|error| format!("flush candidate failure receipt: {error}"))?;
    receipt_file
        .sync_all()
        .map_err(|error| format!("sync candidate failure receipt: {error}"))?;
    drop(receipt_file);
    jocforge_io::atomic_output::sync_experimental_candidate_directory(parent)
        .map_err(|error| format!("sync candidate failure directory: {error}"))?;
    Ok(sidecar_path)
}
