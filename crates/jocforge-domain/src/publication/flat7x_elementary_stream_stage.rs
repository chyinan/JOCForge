// pattern: Functional Core
//! Immutable Flat7x elementary-stream staging proposal; final exposure is outside this contract.

use super::Flat7xAuFinalizeProposal;
use crate::{
    codec::elementary_stream_composer::{
        Flat7xElementaryStreamExtent, measure_flat7x_elementary_stream,
    },
    profile::Flat7xI0D0TargetPlan,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xElementaryStreamStageProposalError {
    UnsupportedTargetContext,
    FinalizeProductionCapabilityUnexpected,
    ExtentMismatch,
    FinalizeLedgerMismatch { au_index: u32 },
    AccessUnitLedgerHashMismatch { au_index: u32 },
    StageReceiptAttemptMismatch,
    StageReceiptOwnerGenerationMismatch,
    StageReceiptExtentMismatch,
    StageReceiptDurabilityMissing,
    StageReceiptReadbackHashMismatch,
}

#[must_use]
pub struct Flat7xElementaryStreamStageProposal<'bytes, 'finalize> {
    target: Flat7xI0D0TargetPlan,
    stream_bytes: &'bytes [u8],
    extent: Flat7xElementaryStreamExtent,
    finalize_proposal: &'finalize Flat7xAuFinalizeProposal,
}

impl std::fmt::Debug for Flat7xElementaryStreamStageProposal<'_, '_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Flat7xElementaryStreamStageProposal")
            .field("extent", &self.extent)
            .field("stream_byte_length", &self.stream_bytes.len())
            .field("production_allowed", &false)
            .finish()
    }
}

impl<'bytes, 'finalize> Flat7xElementaryStreamStageProposal<'bytes, 'finalize> {
    pub fn new(
        target: Flat7xI0D0TargetPlan,
        stream_bytes: &'bytes [u8],
        finalize_proposal: &'finalize Flat7xAuFinalizeProposal,
    ) -> Result<Self, Flat7xElementaryStreamStageProposalError> {
        if target != Flat7xI0D0TargetPlan::approved_contract() {
            return Err(Flat7xElementaryStreamStageProposalError::UnsupportedTargetContext);
        }
        if finalize_proposal.production_allowed() {
            return Err(
                Flat7xElementaryStreamStageProposalError::FinalizeProductionCapabilityUnexpected,
            );
        }
        let access_unit_count = usize::try_from(finalize_proposal.expected_access_units())
            .map_err(|_| Flat7xElementaryStreamStageProposalError::ExtentMismatch)?;
        let extent = measure_flat7x_elementary_stream(target, access_unit_count)
            .map_err(|_| Flat7xElementaryStreamStageProposalError::ExtentMismatch)?;
        if finalize_proposal.durable_length() != u64::from(extent.byte_length())
            || u64::try_from(finalize_proposal.ledger().len()).ok()
                != Some(finalize_proposal.expected_access_units())
            || stream_bytes.len()
                != usize::try_from(extent.byte_length())
                    .map_err(|_| Flat7xElementaryStreamStageProposalError::ExtentMismatch)?
        {
            return Err(Flat7xElementaryStreamStageProposalError::ExtentMismatch);
        }

        let i0_frame_bytes = usize::from(target.i0().frame_bytes());
        let d0_frame_bytes = usize::from(target.d0().frame_bytes());
        let au_frame_bytes = usize::try_from(target.aggregate_frame_bytes())
            .map_err(|_| Flat7xElementaryStreamStageProposalError::ExtentMismatch)?;
        for (index, entry) in finalize_proposal.ledger().iter().enumerate() {
            let au_index = u32::try_from(index)
                .map_err(|_| Flat7xElementaryStreamStageProposalError::ExtentMismatch)?;
            let au_index_u64 = u64::from(au_index);
            let expected_offset = au_index_u64
                .checked_mul(u64::from(target.aggregate_frame_bytes()))
                .ok_or(Flat7xElementaryStreamStageProposalError::ExtentMismatch)?;
            let expected_au_index = entry.au_index();
            let expected_frames = (target.i0().frame_bytes(), target.d0().frame_bytes());
            if expected_au_index.value() != au_index_u64
                || entry.offset() != expected_offset
                || entry.length() != target.aggregate_frame_bytes()
                || entry.frame_lengths() != expected_frames
                || entry.metadata_identity().2 != expected_au_index
                || entry.frame_crc_residuals() != (0, 0)
            {
                return Err(
                    Flat7xElementaryStreamStageProposalError::FinalizeLedgerMismatch { au_index },
                );
            }

            let start = usize::try_from(expected_offset)
                .map_err(|_| Flat7xElementaryStreamStageProposalError::ExtentMismatch)?;
            let end = start
                .checked_add(au_frame_bytes)
                .ok_or(Flat7xElementaryStreamStageProposalError::ExtentMismatch)?;
            let Some(au_bytes) = stream_bytes.get(start..end) else {
                return Err(
                    Flat7xElementaryStreamStageProposalError::FinalizeLedgerMismatch { au_index },
                );
            };
            let i0_end = i0_frame_bytes;
            let Some(i0_bytes) = au_bytes.get(..i0_end) else {
                return Err(
                    Flat7xElementaryStreamStageProposalError::FinalizeLedgerMismatch { au_index },
                );
            };
            let Some(d0_bytes) = au_bytes.get(i0_end..i0_end + d0_frame_bytes) else {
                return Err(
                    Flat7xElementaryStreamStageProposalError::FinalizeLedgerMismatch { au_index },
                );
            };
            let (independent_ledger_hash, dependent_ledger_hash) = entry.frame_sha256();
            let i0_sha256: [u8; 32] = Sha256::digest(i0_bytes).into();
            let d0_sha256: [u8; 32] = Sha256::digest(d0_bytes).into();
            let au_sha256: [u8; 32] = Sha256::digest(au_bytes).into();
            if &i0_sha256 != independent_ledger_hash
                || &d0_sha256 != dependent_ledger_hash
                || &au_sha256 != entry.au_sha256()
            {
                return Err(
                    Flat7xElementaryStreamStageProposalError::AccessUnitLedgerHashMismatch {
                        au_index,
                    },
                );
            }
        }

        Ok(Self {
            target,
            stream_bytes,
            extent,
            finalize_proposal,
        })
    }

    pub const fn target(&self) -> Flat7xI0D0TargetPlan {
        self.target
    }

    pub const fn extent(&self) -> Flat7xElementaryStreamExtent {
        self.extent
    }

    pub const fn stream_bytes(&self) -> &'bytes [u8] {
        self.stream_bytes
    }

    pub fn finalize_proposal(&self) -> &'finalize Flat7xAuFinalizeProposal {
        self.finalize_proposal
    }

    pub fn attempt_id(&self) -> &str {
        self.finalize_proposal.attempt_id()
    }

    pub const fn owner_generation(&self) -> u64 {
        self.finalize_proposal.owner_generation()
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    pub fn complete(
        self,
        receipt: Flat7xElementaryStreamStageReceiptCandidate,
    ) -> Result<
        Flat7xElementaryStreamStageAfterProposal<'finalize>,
        Flat7xElementaryStreamStageProposalError,
    > {
        if receipt.attempt_id != self.finalize_proposal.attempt_id() {
            return Err(Flat7xElementaryStreamStageProposalError::StageReceiptAttemptMismatch);
        }
        if receipt.owner_generation != self.finalize_proposal.owner_generation() {
            return Err(
                Flat7xElementaryStreamStageProposalError::StageReceiptOwnerGenerationMismatch,
            );
        }
        if receipt.extent != self.extent
            || receipt.bytes_written != self.extent.byte_length()
            || receipt.write_call_count == 0
        {
            return Err(Flat7xElementaryStreamStageProposalError::StageReceiptExtentMismatch);
        }
        if !receipt.file_flush_completed || !receipt.file_sync_completed {
            return Err(Flat7xElementaryStreamStageProposalError::StageReceiptDurabilityMissing);
        }
        let expected_sha256: [u8; 32] = Sha256::digest(self.stream_bytes).into();
        if receipt.readback_sha256 != expected_sha256 {
            return Err(Flat7xElementaryStreamStageProposalError::StageReceiptReadbackHashMismatch);
        }

        Ok(Flat7xElementaryStreamStageAfterProposal {
            finalize_proposal: self.finalize_proposal,
            receipt,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xElementaryStreamStageReceiptCandidate {
    attempt_id: String,
    owner_generation: u64,
    extent: Flat7xElementaryStreamExtent,
    bytes_written: u32,
    write_call_count: u32,
    readback_sha256: [u8; 32],
    file_flush_completed: bool,
    file_sync_completed: bool,
}

impl Flat7xElementaryStreamStageReceiptCandidate {
    pub fn new(
        proposal: &Flat7xElementaryStreamStageProposal<'_, '_>,
        bytes_written: u32,
        write_call_count: u32,
        readback_sha256: [u8; 32],
        file_flush_completed: bool,
        file_sync_completed: bool,
    ) -> Self {
        Self {
            attempt_id: proposal.attempt_id().to_owned(),
            owner_generation: proposal.owner_generation(),
            extent: proposal.extent(),
            bytes_written,
            write_call_count,
            readback_sha256,
            file_flush_completed,
            file_sync_completed,
        }
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub const fn owner_generation(&self) -> u64 {
        self.owner_generation
    }

    pub const fn extent(&self) -> Flat7xElementaryStreamExtent {
        self.extent
    }

    pub const fn bytes_written(&self) -> u32 {
        self.bytes_written
    }

    pub const fn write_call_count(&self) -> u32 {
        self.write_call_count
    }

    pub const fn readback_sha256(&self) -> &[u8; 32] {
        &self.readback_sha256
    }

    pub const fn file_flush_completed(&self) -> bool {
        self.file_flush_completed
    }

    pub const fn file_sync_completed(&self) -> bool {
        self.file_sync_completed
    }
}

#[must_use]
pub struct Flat7xElementaryStreamStageAfterProposal<'finalize> {
    finalize_proposal: &'finalize Flat7xAuFinalizeProposal,
    receipt: Flat7xElementaryStreamStageReceiptCandidate,
}

impl std::fmt::Debug for Flat7xElementaryStreamStageAfterProposal<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Flat7xElementaryStreamStageAfterProposal")
            .field("attempt_id", &self.finalize_proposal.attempt_id())
            .field(
                "owner_generation",
                &self.finalize_proposal.owner_generation(),
            )
            .field("extent", &self.receipt.extent())
            .field("production_allowed", &false)
            .finish()
    }
}

impl Flat7xElementaryStreamStageAfterProposal<'_> {
    pub fn finalize_proposal(&self) -> &Flat7xAuFinalizeProposal {
        self.finalize_proposal
    }

    pub const fn receipt(&self) -> &Flat7xElementaryStreamStageReceiptCandidate {
        &self.receipt
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    pub const fn final_exposure_allowed(&self) -> bool {
        false
    }
}
