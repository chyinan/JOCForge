// pattern: Functional Core
//! Selected-profile keyless output proposals and durable ledger semantics.

mod flat7x_au_output;
mod output_owner;

#[cfg(test)]
mod tests;

pub use flat7x_au_output::{
    Flat7xAuAppendProposal, Flat7xAuAppendReceiptCandidate, Flat7xAuOutputAfterProposal,
    Flat7xAuOutputLedgerEntry, Flat7xAuOutputOwner, Flat7xAuOutputOwnerError,
};
pub use output_owner::{
    OutputAppendProposal, OutputConfiguration, OutputDurableAppendReceipt,
    OutputFinalExposureReceipt, OutputFinalizeProposal, OutputFinalizedProposal, OutputOwner,
    OutputOwnerCommitReceipt, OutputOwnerError, OutputOwnerStateReference, OutputProvider,
    OutputSourceAfterProposal, OutputSourceCommitReceipt, OutputSourceIdentity,
    OutputSourceProposal, OutputSuppressedSourceProposal,
};
