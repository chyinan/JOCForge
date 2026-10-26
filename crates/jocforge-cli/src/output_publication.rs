// pattern: Imperative Shell
//! F15P append composition for already prepared source proposals.

use jocforge_application::output_publication::{
    OutputFinalizeProposal, OutputFinalizedProposal, OutputOwnerError, OutputSourceAfterProposal,
    OutputSourceProposal,
};
use jocforge_io::atomic_output::{AtomicOutputStaging, OutputIoError};

#[derive(Debug)]
pub enum OutputPublicationError {
    Io(OutputIoError),
    Owner(OutputOwnerError),
}

impl std::fmt::Display for OutputPublicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Owner(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for OutputPublicationError {}

/// Applies the approved suppression/append branch. Append returns only after
/// exact readback and file fsync; the returned owner proposal remains uninstalled.
pub fn stage_output_source(
    staging: &mut AtomicOutputStaging,
    proposal: OutputSourceProposal,
) -> Result<OutputSourceAfterProposal, OutputPublicationError> {
    match proposal {
        OutputSourceProposal::Suppressed(proposal) => Ok(proposal.into_after()),
        OutputSourceProposal::Append(proposal) => {
            let durable_receipt = staging
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
    staging: &mut AtomicOutputStaging,
    proposal: OutputFinalizeProposal,
) -> Result<OutputFinalizedProposal, OutputPublicationError> {
    let receipt = staging
        .finalize(&proposal)
        .map_err(OutputPublicationError::Io)?;
    proposal
        .complete(receipt)
        .map_err(OutputPublicationError::Owner)
}
