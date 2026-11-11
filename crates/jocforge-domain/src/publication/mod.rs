// pattern: Functional Core
//! Selected-profile keyless output proposals and durable ledger semantics.

mod flat7x_au_output;
mod flat7x_elementary_stream_stage;
mod output_owner;

#[cfg(test)]
mod tests;

pub use flat7x_au_output::{
    Flat7xAuAppendProposal, Flat7xAuAppendReceiptCandidate, Flat7xAuAttemptRetirement,
    Flat7xAuAttemptRetirementReason, Flat7xAuFinalizeProposal, Flat7xAuIntegrityFrameReceipt,
    Flat7xAuOutputAfterProposal, Flat7xAuOutputLedgerEntry, Flat7xAuOutputOwner,
    Flat7xAuOutputOwnerError, Flat7xAuSourceReceiptInput,
};
pub use flat7x_elementary_stream_stage::{
    Flat7xElementaryStreamStageProposal, Flat7xElementaryStreamStageProposalError,
};
pub use output_owner::{
    OutputAppendProposal, OutputConfiguration, OutputDurableAppendReceipt,
    OutputFinalExposureReceipt, OutputFinalizeProposal, OutputFinalizedProposal, OutputOwner,
    OutputOwnerCommitReceipt, OutputOwnerError, OutputOwnerStateReference, OutputProvider,
    OutputSourceAfterProposal, OutputSourceCommitReceipt, OutputSourceIdentity,
    OutputSourceProposal, OutputSuppressedSourceProposal,
};
