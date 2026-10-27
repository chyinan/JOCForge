// pattern: Imperative Shell
//! F15P append composition for already prepared source proposals.

use jocforge_application::output_publication::{
    AttemptId, OutputConfiguration, OutputFinalizedProposal, OutputOwnerError,
    OutputRootBindingError, OutputSourceAfterProposal, OutputSourceProposal, OutputStagingOwnerId,
    ProgrammeId, ProgrammeOutputBinding, ProgrammeOutputFinalizeProposal,
    ProgrammeOutputSourceProposal,
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
