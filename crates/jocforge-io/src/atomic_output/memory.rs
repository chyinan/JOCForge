// pattern: Imperative Shell
//! Attempt-scoped staging that keeps the product inside the calling isolate.
//!
//! A browser isolate has no directory to stage into. This adapter runs the same append protocol
//! as the atomic file adapter - proposal identity, exact length accounting, exact readback, frame
//! digest and CRC residual, then the guarded receipt issuer - against one owned byte buffer, so
//! the frames and the receipts the F15 owner roots receive are the ones the file adapter would
//! have produced. There is no separate device to flush and no namespace to rename: the buffer is
//! the publication target, and the terminal hand-off moves the complete buffer to the caller in
//! one step, so no partial product is ever visible.

use jocforge_domain::{
    integrity::crc16::eac3_crc16,
    publication::{
        OutputAppendProposal, OutputConfiguration, OutputDurableAppendReceipt,
        OutputFinalExposureReceipt, OutputFinalizeProposal,
    },
};

use super::{
    OutputIoError, OutputIoFailure, OutputIoPhase, SessionState, is_single_component, sha256,
};

/// Attempt-scoped product assembly without a filesystem.
#[derive(Debug)]
pub struct MemoryOutputStaging {
    staging_name: String,
    product_name: String,
    attempt_id: String,
    bytes: Vec<u8>,
    state: SessionState,
}

impl MemoryOutputStaging {
    /// Accepts one attempt-scoped configuration without opening anything.
    pub fn create(configuration: &OutputConfiguration) -> Result<Self, OutputIoError> {
        if !is_single_component(configuration.staging_name())
            || !is_single_component(configuration.intent_name())
            || !is_single_component(configuration.product_name())
        {
            return Err(OutputIoError::new(
                OutputIoPhase::Setup,
                OutputIoFailure::InvalidName,
            ));
        }
        Ok(Self {
            staging_name: configuration.staging_name().to_owned(),
            product_name: configuration.product_name().to_owned(),
            attempt_id: configuration.attempt_id().to_owned(),
            bytes: Vec::new(),
            state: SessionState::Open,
        })
    }

    /// Appends one exact source proposal and returns the same receipt the file adapter mints.
    pub fn append(
        &mut self,
        proposal: &OutputAppendProposal,
    ) -> Result<OutputDurableAppendReceipt, OutputIoError> {
        match self.state {
            SessionState::Abandoned => {
                return Err(OutputIoError::new(
                    OutputIoPhase::AppendPreflight,
                    OutputIoFailure::SessionAbandoned,
                ));
            }
            SessionState::Finalized => {
                return Err(OutputIoError::new(
                    OutputIoPhase::AppendPreflight,
                    OutputIoFailure::SessionFinalized,
                ));
            }
            SessionState::ExposureUncertain | SessionState::Open => {}
        }
        let result = self.append_inner(proposal);
        if result.is_err() {
            self.state = SessionState::Abandoned;
        }
        result
    }

    #[allow(unsafe_code)]
    fn append_inner(
        &mut self,
        proposal: &OutputAppendProposal,
    ) -> Result<OutputDurableAppendReceipt, OutputIoError> {
        if proposal.attempt_id() != self.attempt_id || proposal.staging_name() != self.staging_name
        {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendPreflight,
                OutputIoFailure::ProposalIdentity,
            ));
        }
        let initial_length = self.bytes.len();
        if u64::try_from(initial_length).ok() != Some(proposal.expected_offset()) {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendPreflight,
                OutputIoFailure::StagingLength,
            ));
        }
        let frame_bytes = proposal.frame_bytes();
        let frame_length = u64::try_from(frame_bytes.len()).map_err(|_| {
            OutputIoError::new(OutputIoPhase::AppendWrite, OutputIoFailure::WriteOverrun)
        })?;
        if frame_length == 0 {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendWrite,
                OutputIoFailure::ZeroProgress,
            ));
        }
        self.bytes.extend_from_slice(frame_bytes);
        if self.bytes.len() != initial_length + frame_bytes.len() {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendReadback,
                OutputIoFailure::StagingLength,
            ));
        }

        let readback = &self.bytes[initial_length..];
        if readback != frame_bytes || sha256(readback) != *proposal.frame_sha256() {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendReadback,
                OutputIoFailure::ReadbackMismatch,
            ));
        }
        let crc_residual = eac3_crc16(&readback[2..]);
        if crc_residual != 0 {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendReadback,
                OutputIoFailure::CrcMismatch,
            ));
        }

        // SAFETY: this exact proposal was written at its declared offset into the owned buffer;
        // the resulting length, exact readback, frame hash and CRC residual were all checked on
        // that same buffer. The buffer never leaves this isolate, so the write is complete before
        // the receipt is minted.
        unsafe { proposal.record_durable_io_success(vec![frame_length], sha256(readback), 0) }
            .map_err(|_| {
                OutputIoError::new(
                    OutputIoPhase::AppendReadback,
                    OutputIoFailure::DurableReceipt,
                )
            })
    }

    /// The selected browser profile has no Flat7x pair owner.
    ///
    /// A pair proposal consumes the attempt either way; the browser target never reaches this
    /// branch because seven-carrier layouts are rejected before staging is created, so the
    /// attempt is retired rather than staged.
    pub fn append_flat7x_au<'a>(
        &mut self,
        proposal: jocforge_domain::publication::Flat7xAuAppendProposal<'a>,
    ) -> Result<
        jocforge_domain::publication::Flat7xAuOutputAfterProposal<'a>,
        super::Flat7xAtomicAppendFailure,
    > {
        self.state = SessionState::Abandoned;
        Err(super::Flat7xAtomicAppendFailure::SessionAbandoned(
            proposal.retire_staging_failure(),
        ))
    }
    /// Accepts the terminal F15 proposal and returns the terminal receipt for the buffer.
    pub fn finalize(
        &mut self,
        proposal: &OutputFinalizeProposal,
    ) -> Result<OutputFinalExposureReceipt, OutputIoError> {
        match self.state {
            SessionState::Abandoned => {
                return Err(OutputIoError::new(
                    OutputIoPhase::FinalizePreflight,
                    OutputIoFailure::SessionAbandoned,
                ));
            }
            SessionState::Finalized => {
                return Err(OutputIoError::new(
                    OutputIoPhase::FinalizePreflight,
                    OutputIoFailure::SessionFinalized,
                ));
            }
            SessionState::ExposureUncertain | SessionState::Open => {}
        }
        if proposal.attempt_id() != self.attempt_id
            || proposal.staging_name() != self.staging_name
            || proposal.product_name() != self.product_name
        {
            self.state = SessionState::Abandoned;
            return Err(OutputIoError::new(
                OutputIoPhase::FinalizePreflight,
                OutputIoFailure::FinalizeProposalIdentity,
            ));
        }
        let product_length = u64::try_from(self.bytes.len()).map_err(|_| {
            OutputIoError::new(
                OutputIoPhase::FinalizePreflight,
                OutputIoFailure::FinalizeLength,
            )
        })?;
        if product_length != proposal.expected_length() {
            self.state = SessionState::Abandoned;
            return Err(OutputIoError::new(
                OutputIoPhase::FinalizePreflight,
                OutputIoFailure::FinalizeLength,
            ));
        }
        let receipt = proposal
            .record_durable_io_success(sha256(&self.bytes), product_length)
            .map_err(|_| {
                self.state = SessionState::Abandoned;
                OutputIoError::new(
                    OutputIoPhase::FinalizePreflight,
                    OutputIoFailure::DurableReceipt,
                )
            })?;
        self.state = SessionState::Finalized;
        Ok(receipt)
    }

    /// The product bytes assembled by every accepted append.
    pub fn product_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Consumes the staging area and returns the assembled product.
    pub fn into_product_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Attempt-scoped staging name reported by the file adapter as a sibling path.
    pub fn staging_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.staging_name)
    }

    /// Product name reported by the file adapter as a sibling path.
    pub fn product_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.product_name)
    }

    pub fn is_abandoned(&self) -> bool {
        matches!(self.state, SessionState::Abandoned)
    }

    pub fn is_exposure_uncertain(&self) -> bool {
        matches!(self.state, SessionState::ExposureUncertain)
    }

    pub fn abandon(&mut self) {
        self.state = SessionState::Abandoned;
    }
}
