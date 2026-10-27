// pattern: Functional Core
//! Selected-profile keyless output proposals and durable ledger semantics.

mod output_owner;

#[cfg(test)]
mod tests;

pub use output_owner::{
    OutputAppendProposal, OutputConfiguration, OutputDurableAppendReceipt,
    OutputFinalExposureReceipt, OutputFinalizeProposal, OutputFinalizedProposal, OutputOwner,
    OutputOwnerCommitReceipt, OutputOwnerError, OutputOwnerStateReference, OutputProvider,
    OutputSourceAfterProposal, OutputSourceCommitReceipt, OutputSourceIdentity,
    OutputSourceProposal, OutputSuppressedSourceProposal,
};
